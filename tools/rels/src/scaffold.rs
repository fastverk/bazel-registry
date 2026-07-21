//! `rels scaffold` — bootstrap a new fastverk rules_* repo with
//! the standard baseline: MODULE.bazel @ 0.0.1, MIT license,
//! README, CHANGELOG, .gitignore, .bazelrc with registry pin,
//! .github/workflows/ci.yml, BUILD.bazel exporting MODULE.bazel.
//!
//! The new tree starts `rels audit`-clean on day one.
//!
//! With `--create-gh`, also runs `gh repo create
//! fastverk/<name> --public --description=<desc>` and pushes
//! the initial commit upstream.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Args as ClapArgs;

use crate::common::Env;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// New repo name. Must start with `rules_`.
    #[arg(long)]
    pub name: String,

    /// One-line description used in README.md, MODULE.bazel
    /// comment, and the gh repo (with --create-gh).
    #[arg(long)]
    pub description: String,

    /// Output directory. Defaults to
    /// `<workspaces_root>/<name>`.
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Skip `git init` + initial commit.
    #[arg(long)]
    pub no_git: bool,

    /// Run `gh repo create fastverk/<name>` after the initial
    /// commit and push `origin main`. Requires `gh` authenticated.
    #[arg(long)]
    pub create_gh: bool,

    /// GitHub org under which to create the repo. Default
    /// `fastverk`.
    #[arg(long, default_value = "fastverk")]
    pub gh_org: String,
}

pub fn run(env: &Env, args: Args) -> Result<()> {
    if !args.name.starts_with("rules_") {
        bail!(
            "rels scaffold: --name must start with `rules_` (got {:?})",
            args.name,
        );
    }

    let dest = args
        .path
        .clone()
        .unwrap_or_else(|| env.workspaces_root.join(&args.name));
    if dest.exists() {
        bail!("rels scaffold: target {} already exists", dest.display());
    }
    fs::create_dir_all(&dest)
        .with_context(|| format!("mkdir {}", dest.display()))?;

    write_templates(&dest, &args)?;

    eprintln!("rels scaffold: wrote {} files under {}", FILE_COUNT, dest.display());

    if !args.no_git {
        git_init(&dest)?;
    }

    if args.create_gh {
        if args.no_git {
            bail!("--create-gh requires git init (drop --no-git)");
        }
        gh_create_and_push(&dest, &args)?;
    }

    eprintln!(
        "rels scaffold: done. Next steps:\n  cd {}\n  bazel test //... (no targets yet — add some!)\n  edit README.md, MODULE.bazel, etc.{}",
        dest.display(),
        if args.create_gh {
            ""
        } else {
            "\n  gh repo create fastverk/<name> --public --source=. --push  # when ready"
        },
    );
    Ok(())
}

const FILE_COUNT: usize = 8;

fn write_templates(dest: &Path, args: &Args) -> Result<()> {
    write(dest, ".gitignore", GITIGNORE)?;
    write(dest, ".bazelrc", BAZELRC)?;
    write(dest, "BUILD.bazel", BUILD_BAZEL)?;
    write(dest, "MODULE.bazel", &module_bazel(args))?;
    write(dest, "LICENSE", &license(2026, "Matt Marshall"))?;
    write(dest, "README.md", &readme(args))?;
    write(dest, "CHANGELOG.md", &changelog(args))?;
    fs::create_dir_all(dest.join(".github").join("workflows"))?;
    write(dest, ".github/workflows/ci.yml", CI_YAML)?;
    Ok(())
}

fn write(dest: &Path, rel: &str, content: &str) -> Result<()> {
    let path = dest.join(rel);
    fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn git_init(dest: &Path) -> Result<()> {
    run_in("git", &["init", "-b", "main"], dest)?;
    run_in("git", &["add", "-A"], dest)?;
    // Author identity matches every other fastverk commit. Don't
    // mutate the user's global git config — pass via -c flags.
    let status = Command::new("git")
        .args([
            "-c", "user.name=Matt Marshall",
            "-c", "user.email=mateomm@gmail.com",
            "commit", "-m", "Initial scaffold via `rels scaffold`",
        ])
        .current_dir(dest)
        .status()
        .with_context(|| format!("git commit in {}", dest.display()))?;
    if !status.success() {
        bail!("git commit failed in {}", dest.display());
    }
    Ok(())
}

fn gh_create_and_push(dest: &Path, args: &Args) -> Result<()> {
    eprintln!(
        "rels scaffold: creating gh repo {}/{}",
        args.gh_org, args.name,
    );
    let status = Command::new("gh")
        .args([
            "repo", "create",
            &format!("{}/{}", args.gh_org, args.name),
            "--public",
            "--description", &args.description,
            "--source", ".",
            "--push",
            "--remote", "origin",
        ])
        .current_dir(dest)
        .status()
        .with_context(|| "running `gh repo create`")?;
    if !status.success() {
        bail!(
            "gh repo create failed (is `gh` authenticated? `gh auth status`)",
        );
    }
    Ok(())
}

fn run_in(prog: &str, args: &[&str], dir: &Path) -> Result<()> {
    let status = Command::new(prog)
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("{} {} in {}", prog, args.join(" "), dir.display()))?;
    if !status.success() {
        bail!(
            "{} {} failed (exit {})",
            prog,
            args.join(" "),
            status,
        );
    }
    Ok(())
}

// -----------------------------------------------------------------------------
// File templates
// -----------------------------------------------------------------------------

const GITIGNORE: &str = "\
bazel-*
.bazelrc.user
.DS_Store
target/
.claude/
MODULE.bazel.lock
";

// registry.tbzl.dev, not registry.fastverk.com: the fleet is consolidating onto
// the former (fastverk/images#10 makes it entry #1 in the baked
// //ci:system_bazelrc, with per-repo PRs following across both orgs). Until this
// template changed, every repo `rels scaffold` minted was born on the legacy URL
// and immediately needed the migration PR — tomato-bazel/gate was exactly that.
// All three URLs are CloudFront fronts of the same content, so this is a rename.
const BAZELRC: &str = "\
common --enable_bzlmod

common --registry=https://registry.tbzl.dev/
common --registry=https://bcr.bazel.build/

test --test_output=errors
test --test_summary=terse
";

const BUILD_BAZEL: &str = "\
package(default_visibility = [\"//visibility:public\"])

exports_files([\"MODULE.bazel\"])
";

const CI_YAML: &str = r#"name: ci

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

# Cancel in-flight runs when a new commit lands on the same ref.
concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  test:
    name: bazel test //...
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: bazel-contrib/setup-bazel@0.10.0
        with:
          bazelisk-cache: true
          disk-cache: ${{ github.workflow }}-${{ matrix.os }}
          repository-cache: true
      - run: bazel test //... --test_output=errors

  buildifier:
    name: buildifier lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: install buildifier
        run: |
          curl -fsSL -o /usr/local/bin/buildifier \
            https://github.com/bazelbuild/buildtools/releases/download/v7.3.1/buildifier-linux-amd64
          chmod +x /usr/local/bin/buildifier
      - run: buildifier --mode=check --lint=warn -r .
"#;

fn module_bazel(args: &Args) -> String {
    format!(
        "# {}\n\
         module(\n\
         \x20\x20\x20\x20name = \"{}\",\n\
         \x20\x20\x20\x20version = \"0.0.1\",\n\
         )\n\
         \n\
         bazel_dep(name = \"platforms\", version = \"1.0.0\")\n\
         bazel_dep(name = \"bazel_skylib\", version = \"1.8.2\")\n\
         \n\
         # Dev-only.\n\
         bazel_dep(name = \"rules_shell\", version = \"0.6.1\", dev_dependency = True)\n\
         bazel_dep(name = \"stardoc\", version = \"0.7.2\", dev_dependency = True)\n",
        args.description,
        args.name,
    )
}

fn license(year: i32, holder: &str) -> String {
    format!(
        "MIT License\n\
         \n\
         Copyright (c) {year} {holder}\n\
         \n\
         Permission is hereby granted, free of charge, to any person obtaining a copy\n\
         of this software and associated documentation files (the \"Software\"), to deal\n\
         in the Software without restriction, including without limitation the rights\n\
         to use, copy, modify, merge, publish, distribute, sublicense, and/or sell\n\
         copies of the Software, and to permit persons to whom the Software is\n\
         furnished to do so, subject to the following conditions:\n\
         \n\
         The above copyright notice and this permission notice shall be included in all\n\
         copies or substantial portions of the Software.\n\
         \n\
         THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR\n\
         IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,\n\
         FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE\n\
         AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER\n\
         LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,\n\
         OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE\n\
         SOFTWARE.\n",
    )
}

fn readme(args: &Args) -> String {
    format!(
        "# {name}\n\
         \n\
         {desc}\n\
         \n\
         ## Status: v0.0.1 — scaffold\n\
         \n\
         No public surface yet. See `CHANGELOG.md` for what has shipped.\n\
         \n\
         ## Install\n\
         \n\
         `.bazelrc`:\n\
         \n\
         ```\n\
         common --registry=https://registry.tbzl.dev/\n\
         common --registry=https://bcr.bazel.build/\n\
         ```\n\
         \n\
         `MODULE.bazel`:\n\
         \n\
         ```python\n\
         bazel_dep(name = \"{name}\", version = \"0.0.1\")\n\
         ```\n",
        name = args.name,
        desc = args.description,
    )
}

fn changelog(args: &Args) -> String {
    format!(
        "# Changelog\n\
         \n\
         All notable changes to {name}. The format is loosely\n\
         [Keep a Changelog](https://keepachangelog.com/) — version headers\n\
         mirror the published bazel-registry entries.\n\
         \n\
         ## 0.0.1 — scaffold\n\
         \n\
         - Initial scaffold via `rels scaffold`. No public API yet.\n",
        name = args.name,
    )
}
