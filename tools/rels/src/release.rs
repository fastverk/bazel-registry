//! `rels release` — port of the legacy `tools/add_module/add_module.py`.
//!
//! Given a `--repo fastverk/rules_X --version 0.Y.Z` pair, this:
//!
//!   1. Resolves the GitHub-auto-generated tarball URL
//!      (`<repo>/archive/refs/tags/v<version>.tar.gz`).
//!   2. Downloads it; computes the SRI integrity hash
//!      (`sha256-<base64>`).
//!   3. Extracts `<strip_prefix>/MODULE.bazel` from the tarball.
//!   4. Writes
//!      `modules/<name>/<version>/{source.json, MODULE.bazel}` and
//!      upserts `modules/<name>/metadata.json` (creating it if
//!      absent).
//!
//! Outputs the integrity to stderr; stdout is reserved for future
//! structured-output options.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use clap::Args as ClapArgs;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;

use crate::common::{Env, Maintainer, ModuleMetadata, SourceJson};

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// GitHub repo as `owner/name` (e.g. `fastverk/rules_uv`).
    /// Required unless both `--url` and `--name` are supplied.
    #[arg(long)]
    repo: Option<String>,

    /// Module version (e.g. `0.1.0`).
    #[arg(long)]
    version: String,

    /// Module name. Defaults to the repo basename.
    #[arg(long)]
    name: Option<String>,

    /// Override tarball URL (skips GitHub convention).
    #[arg(long)]
    url: Option<String>,

    /// Override `strip_prefix` (default `<repo-basename>-<version>`).
    #[arg(long)]
    strip_prefix: Option<String>,

    /// Tag prefix prepended to `--version`. Default `v`.
    #[arg(long, default_value = "v")]
    tag_prefix: String,

    /// Overwrite an existing version entry.
    #[arg(long)]
    force: bool,
}

pub fn run(env: &Env, args: Args) -> Result<()> {
    if args.repo.is_none() && (args.url.is_none() || args.name.is_none()) {
        bail!("either --repo, or both --url and --name, is required");
    }

    // Provisional only — used for messages until the tarball tells us the real
    // name. See the note on `declared_module_name` below.
    let provisional = match (&args.name, &args.repo) {
        (Some(n), _) => n.clone(),
        (None, Some(r)) => repo_basename(r)?,
        (None, None) => unreachable!(),
    };

    let tag = format!("{}{}", args.tag_prefix, args.version);

    let (url, strip_prefix) = match &args.url {
        Some(u) => (
            u.clone(),
            args.strip_prefix.clone().unwrap_or_default(),
        ),
        None => {
            let repo = args.repo.as_ref().unwrap();
            let basename = repo_basename(repo)?;
            (
                format!("https://github.com/{}/archive/refs/tags/{}.tar.gz", repo, tag),
                args.strip_prefix
                    .clone()
                    .unwrap_or_else(|| format!("{}-{}", basename, args.version)),
            )
        }
    };

    eprintln!("fetching {} ...", url);
    let tarball = fetch(&url)?;
    let integrity = sri_integrity(&tarball);
    let module_bazel = extract_module_bazel(&tarball, &strip_prefix)?;

    // THE DIRECTORY MUST BE THE MODULE NAME, NOT THE REPO NAME.
    //
    // Bazel resolves a dependency by looking up `modules/<module name>/<version>/`,
    // where the module name is what MODULE.bazel declares. The repo basename is
    // usually the same and so this went unnoticed, but it is not the same whenever
    // a repo is hyphenated: meridian-ux/meridian-schemas declares
    // `module(name = "meridian_schemas")`. Defaulting to the basename wrote
    // `modules/meridian-schemas/`, a tree Bazel never looks in — no error, no
    // warning, just a version that stays unresolvable. meridian_schemas 0.20.0
    // through 0.24.0 sat tagged and unpublished for exactly this reason.
    //
    // The tarball is already in hand and already parsed, so take the name from
    // the source of truth rather than guessing it from the URL.
    let name = match (&args.name, declared_module_name(&module_bazel)) {
        (Some(explicit), Some(declared)) if explicit != &declared => bail!(
            "--name {:?} disagrees with the module's own declaration {:?}; \
             the registry directory must be the DECLARED name or Bazel will \
             never find it",
            explicit,
            declared,
        ),
        (Some(explicit), _) => explicit.clone(),
        (None, Some(declared)) => declared,
        (None, None) => bail!(
            "could not read `module(name = ...)` out of {}'s MODULE.bazel; \
             pass --name explicitly",
            provisional,
        ),
    };

    let version_dir = env.modules_dir().join(&name).join(&args.version);
    if version_dir.exists() && !args.force {
        bail!(
            "{} already exists; use --force to overwrite",
            version_dir
                .strip_prefix(&env.registry_root)
                .unwrap_or(&version_dir)
                .display(),
        );
    }

    fs::create_dir_all(&version_dir)
        .with_context(|| format!("mkdir {}", version_dir.display()))?;
    fs::write(version_dir.join("MODULE.bazel"), module_bazel)
        .with_context(|| format!("write {}/MODULE.bazel", version_dir.display()))?;

    let source = SourceJson {
        integrity: integrity.clone(),
        strip_prefix: strip_prefix.clone(),
        url: url.clone(),
        archive_type: None,
    };
    source.write(&version_dir.join("source.json"))?;

    if let Some(repo) = &args.repo {
        upsert_metadata(&env.modules_dir().join(&name), repo, &args.version)?;
    }

    eprintln!(
        "wrote {}/",
        version_dir
            .strip_prefix(&env.registry_root)
            .unwrap_or(&version_dir)
            .display(),
    );
    eprintln!("  integrity: {}", integrity);
    Ok(())
}

/// The `name` a MODULE.bazel declares, which is what Bazel resolves against.
///
/// Deliberately line-oriented rather than a full parse: it skips comments (a
/// module docstring routinely contains the word "module(") and reads the first
/// `name = "..."` inside the `module(...)` call.
fn declared_module_name(module_bazel: &str) -> Option<String> {
    let mut in_call = false;
    for line in module_bazel.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        if !in_call {
            if t.starts_with("module(") {
                in_call = true;
            } else {
                continue;
            }
        }
        if let Some(i) = t.find("name") {
            let after = &t[i..];
            if let Some(q1) = after.find('"') {
                if let Some(q2) = after[q1 + 1..].find('"') {
                    return Some(after[q1 + 1..q1 + 1 + q2].to_string());
                }
            }
        }
        if t.contains(')') {
            break;
        }
    }
    None
}

fn repo_basename(repo: &str) -> Result<String> {
    repo.rsplit('/')
        .next()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("invalid --repo {:?}: missing owner/name shape", repo))
}

fn fetch(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        // GitHub redirects archive tarball requests through codeload;
        // reqwest must follow them and forward the auth header.
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .context("build reqwest client")?;

    let mut req = client.get(url);

    // Forward a GitHub token if one is set in the environment so
    // private-repo tarballs (e.g., fastverk/rules_lang) resolve.
    // Tries GITHUB_TOKEN first (CI conventional), then GH_TOKEN
    // (gh-cli conventional), then falls back to `gh auth token`
    // shell-out so interactive users with gh installed but no env
    // var still work.
    let host_is_github = url.starts_with("https://github.com/")
        || url.starts_with("https://api.github.com/")
        || url.starts_with("https://codeload.github.com/");
    if host_is_github {
        if let Some(tok) = github_token() {
            req = req.bearer_auth(tok);
            // GitHub's archive endpoint accepts the standard
            // Accept: application/vnd.github+json header but doesn't
            // require it for tarball downloads. Leave default Accept.
            req = req.header("User-Agent", "fastverk-rels/0.1");
        }
    }

    let resp = req
        .send()
        .with_context(|| format!("GET {}", url))?
        .error_for_status()
        .with_context(|| format!("HTTP {}", url))?;
    let bytes = resp
        .bytes()
        .with_context(|| format!("read body from {}", url))?;
    Ok(bytes.to_vec())
}

fn github_token() -> Option<String> {
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    if let Ok(t) = std::env::var("GH_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    // Last-resort: shell out to `gh auth token`. This is only used by
    // interactive users; CI should set GITHUB_TOKEN explicitly.
    if let Ok(out) = std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

fn sri_integrity(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    format!("sha256-{}", BASE64.encode(digest))
}

fn extract_module_bazel(tarball: &[u8], strip_prefix: &str) -> Result<String> {
    let cursor = Cursor::new(tarball);
    let gz = GzDecoder::new(cursor);
    let mut tar = Archive::new(gz);
    let candidate: PathBuf = if strip_prefix.is_empty() {
        PathBuf::from("MODULE.bazel")
    } else {
        Path::new(strip_prefix).join("MODULE.bazel")
    };
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.as_ref() == candidate {
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut entry, &mut buf)?;
            return Ok(buf);
        }
    }
    bail!(
        "MODULE.bazel not found at '{}/MODULE.bazel' inside tarball. \
         Pass --strip-prefix if the tarball uses a non-standard top-level directory.",
        strip_prefix,
    )
}

fn upsert_metadata(module_dir: &Path, repo: &str, version: &str) -> Result<()> {
    let meta_path = module_dir.join("metadata.json");
    let mut meta = if meta_path.exists() {
        ModuleMetadata::read(&meta_path)?
    } else {
        ModuleMetadata {
            homepage: format!("https://github.com/{}", repo),
            maintainers: vec![Maintainer {
                name: "Matt Marshall".to_string(),
                github: Some("mattmarshall".to_string()),
                email: None,
            }],
            repository: vec![format!("github:{}", repo)],
            versions: Vec::new(),
            yanked_versions: serde_json::Map::new(),
        }
    };
    meta.upsert_version(version);
    meta.write(&meta_path)
}
