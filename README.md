# fastverk bazel-registry

Bzlmod registry for fastverk Bazel modules.

The registry repo itself is public. It describes where to fetch each
module source tarball. Some module entries point at private GitHub
repos and require auth (covered below).

## Modules

This section is generated from local registry metadata and category
rules in tools/readme/categorization_rules.json.

<!-- BOTNOC:MODULES_TABLE -->
### Core

| Module | Latest | Repository |
|---|---|---|
| [`rules_agentic_ide`](https://github.com/tomato-bazel/rules_agentic_ide) | 0.0.4 | `tomato-bazel/rules_agentic_ide` |
| [`rules_ci_ir`](https://github.com/tomato-bazel/rules_ci_ir) | 0.0.1 | `tomato-bazel/rules_ci_ir` |
| [`rules_lang`](https://github.com/tomato-bazel/rules_lang) | 0.4.5 | `tomato-bazel/rules_lang` |
| [`rules_lean`](https://github.com/tomato-bazel/rules_lean) | 0.5.5 | `tomato-bazel/rules_lean` |
| [`rules_spec`](https://github.com/fastverk/rules_spec) | 0.5.1 | `fastverk/rules_spec` |

### UI and Rendering

| Module | Latest | Repository |
|---|---|---|
| [`rules_meridian`](https://github.com/mattmarshall/meridian) | 0.2.1 | `mattmarshall/meridian` |
| [`rules_storybook`](https://github.com/tomato-bazel/rules_storybook) | 0.2.0 | `tomato-bazel/rules_storybook` |
| [`rules_vite`](https://github.com/tomato-bazel/rules_vite) | 0.1.1 | `tomato-bazel/rules_vite` |
| [`rules_walkthrough`](https://github.com/fastverk/rules_walkthrough) | 0.1.0 | `fastverk/rules_walkthrough` |
| [`rules_web`](https://github.com/tomato-bazel/rules_web) | 0.0.1 | `tomato-bazel/rules_web` |

### Cloud and Infrastructure

| Module | Latest | Repository |
|---|---|---|
| [`rules_autoconf`](https://github.com/tomato-bazel/rules_autoconf) | 0.1.0 | `tomato-bazel/rules_autoconf` |
| [`rules_cloudformation`](https://github.com/tomato-bazel/rules_cloudformation) | 0.8.0 | `tomato-bazel/rules_cloudformation` |
| [`rules_docker`](https://github.com/fastverk/rules_docker_compose) | 0.2.6 | `fastverk/rules_docker_compose` |
| [`rules_github`](https://github.com/tomato-bazel/rules_github) | 0.1.2 | `tomato-bazel/rules_github` |
| [`rules_gitlab`](https://github.com/tomato-bazel/rules_gitlab) | 0.3.4 | `tomato-bazel/rules_gitlab` |
| [`rules_nextjs`](https://github.com/tomato-bazel/rules_nextjs) | 0.3.0 | `tomato-bazel/rules_nextjs` |
| [`rules_openapi`](https://github.com/tomato-bazel/rules_openapi) | 0.4.0 | `tomato-bazel/rules_openapi` |
| [`rules_postgres`](https://github.com/tomato-bazel/rules_postgres) | 0.8.3 | `tomato-bazel/rules_postgres` |
| [`rules_runpod`](https://github.com/fastverk/rules_runpod) | 0.0.11 | `fastverk/rules_runpod` |
| [`rules_tectonic`](https://github.com/tomato-bazel/rules_tectonic) | 0.2.0 | `tomato-bazel/rules_tectonic` |

### Data and Knowledge

| Module | Latest | Repository |
|---|---|---|
| [`rules_beam`](https://github.com/tomato-bazel/rules_beam) | 0.0.2 | `tomato-bazel/rules_beam` |
| [`rules_huggingface`](https://github.com/tomato-bazel/rules_huggingface) | 0.0.3 | `tomato-bazel/rules_huggingface` |
| [`rules_jena`](https://github.com/tomato-bazel/rules_jena) | 0.3.2 | `tomato-bazel/rules_jena` |
| [`rules_jsonschema`](https://github.com/tomato-bazel/rules_jsonschema) | 0.3.0 | `tomato-bazel/rules_jsonschema` |
| [`rules_puml`](https://github.com/tomato-bazel/rules_puml) | 0.0.2 | `tomato-bazel/rules_puml` |
| [`rules_rdf`](https://github.com/tomato-bazel/rules_rdf) | 0.4.0 | `tomato-bazel/rules_rdf` |
| [`rules_schema_org`](https://github.com/tomato-bazel/rules_schema_org) | 0.0.1 | `tomato-bazel/rules_schema_org` |

### Language and Build Tooling

| Module | Latest | Repository |
|---|---|---|
| [`rules_bibtex`](https://github.com/tomato-bazel/rules_bibtex) | 0.0.6 | `tomato-bazel/rules_bibtex` |
| [`rules_bun`](https://github.com/tomato-bazel/rules_bun) | 0.4.0 | `tomato-bazel/rules_bun` |
| [`rules_cc_cross`](https://github.com/tomato-bazel/rules_cc_cross) | 0.1.0 | `tomato-bazel/rules_cc_cross` |
| [`rules_chrome`](https://github.com/tomato-bazel/rules_chrome) | 0.1.1 | `tomato-bazel/rules_chrome` |
| [`rules_lora`](https://github.com/tomato-bazel/rules_lora) | 0.1.3 | `tomato-bazel/rules_lora` |
| [`rules_mdbook`](https://github.com/tomato-bazel/rules_mdbook) | 0.3.1 | `tomato-bazel/rules_mdbook` |
| [`rules_meson`](https://github.com/tomato-bazel/rules_meson) | 0.0.1 | `tomato-bazel/rules_meson` |
| [`rules_ssh_tui`](https://github.com/tomato-bazel/rules_ssh_tui) | 0.0.5 | `tomato-bazel/rules_ssh_tui` |
| [`rules_uv`](https://github.com/tomato-bazel/rules_uv) | 0.7.4 | `tomato-bazel/rules_uv` |

### Uncategorized

| Module | Latest | Repository |
|---|---|---|
| [`botnoc`](https://github.com/fastverk/botnoc) | 0.1.0 | `fastverk/botnoc` |
| [`brand`](https://github.com/fastverk/brand) | 0.3.1 | `fastverk/brand` |
| [`brando`](https://github.com/mattmarshall/brando) | 0.1.1 | `mattmarshall/brando` |
| [`buildbarn`](https://github.com/fastverk/buildbarn) | 0.0.2 | `fastverk/buildbarn` |
| [`decomposer`](https://github.com/fastverk/decomposer) | 0.0.1 | `fastverk/decomposer` |
| [`fastverk-app`](https://github.com/fastverk/fastverk-app) | 0.0.2 | `fastverk/fastverk-app` |
| [`forge`](https://github.com/fastverk/forge) | 0.0.1 | `fastverk/forge` |
| [`fvkit`](https://github.com/fastverk/fvkit) | 0.0.9 | `fastverk/fvkit` |
| [`gate`](https://github.com/tomato-bazel/gate) | 0.0.1 | `tomato-bazel/gate` |
| [`meridian`](https://github.com/mattmarshall/meridian) | 0.2.6 | `mattmarshall/meridian` |
| [`meridian_chat`](https://github.com/meridian-ux/meridian-chat) | 0.1.0 | `meridian-ux/meridian-chat` |
| [`meridian_schemas`](https://github.com/meridian-ux/meridian-schemas) | 0.15.0 | `meridian-ux/meridian-schemas` |
| [`meridian_tui`](https://github.com/meridian-ux/meridian-tui) | 0.4.0 | `meridian-ux/meridian-tui` |
| [`meridian_uiview_core`](https://github.com/meridian-ux/meridian-uiview-core) | 0.4.0 | `meridian-ux/meridian-uiview-core` |
| [`meridian_web`](https://github.com/meridian-ux/meridian-web) | 0.5.0 | `meridian-ux/meridian-web` |
| [`pinax`](https://github.com/mattmarshall/pinax) | 0.1.0 | `mattmarshall/pinax` |
| [`rules_aip`](https://github.com/tomato-bazel/rules_aip) | 0.2.2 | `tomato-bazel/rules_aip` |
| [`rules_astro`](https://github.com/tomato-bazel/rules_astro) | 0.0.1 | `tomato-bazel/rules_astro` |
| [`rules_cc_host`](https://github.com/tomato-bazel/rules_cc_host) | 0.1.0 | `tomato-bazel/rules_cc_host` |
| [`rules_ci`](https://github.com/tomato-bazel/rules_ci) | 0.2.0 | `tomato-bazel/rules_ci` |
| [`rules_docker_compose`](https://github.com/tomato-bazel/rules_docker_compose) | 0.2.6 | `tomato-bazel/rules_docker_compose` |
| [`rules_eslint`](https://github.com/tomato-bazel/rules_eslint) | 0.1.0 | `tomato-bazel/rules_eslint` |
| [`rules_fastverk`](https://github.com/tomato-bazel/rules_fastverk) | 0.0.3 | `tomato-bazel/rules_fastverk` |
| [`rules_fastverk_plugin`](https://github.com/tomato-bazel/rules_fastverk_plugin) | 0.0.1 | `tomato-bazel/rules_fastverk_plugin` |
| [`rules_graphviz`](https://github.com/tomato-bazel/rules_graphviz) | 0.2.0 | `tomato-bazel/rules_graphviz` |
| [`rules_helm`](https://github.com/tomato-bazel/rules_helm) | 0.1.0 | `tomato-bazel/rules_helm` |
| [`rules_k8s`](https://github.com/tomato-bazel/rules_k8s) | 0.0.3 | `tomato-bazel/rules_k8s` |
| [`rules_macvm`](https://github.com/tomato-bazel/rules_macvm) | 0.0.1 | `tomato-bazel/rules_macvm` |
| [`rules_markdown`](https://github.com/tomato-bazel/rules_markdown) | 0.0.3 | `tomato-bazel/rules_markdown` |
| [`rules_podman`](https://github.com/tomato-bazel/rules_podman) | 0.0.2 | `tomato-bazel/rules_podman` |
| [`rules_readme`](https://github.com/tomato-bazel/rules_readme) | 0.0.3 | `tomato-bazel/rules_readme` |
| [`rules_systemd`](https://github.com/tomato-bazel/rules_systemd) | 0.0.1 | `tomato-bazel/rules_systemd` |
| [`rules_tap`](https://github.com/tomato-bazel/rules_tap) | 0.0.3 | `tomato-bazel/rules_tap` |
| [`rules_tla`](https://github.com/tomato-bazel/rules_tla) | 0.1.0 | `tomato-bazel/rules_tla` |
| [`rules_tomato`](https://github.com/tomato-bazel/rules_tomato) | 0.1.2 | `tomato-bazel/rules_tomato` |
| [`rules_vscode`](https://github.com/tomato-bazel/rules_vscode) | 0.0.2 | `tomato-bazel/rules_vscode` |
| [`rules_xsd`](https://github.com/tomato-bazel/rules_xsd) | 0.0.1 | `tomato-bazel/rules_xsd` |
| [`spec`](https://github.com/fastverk/spec) | 0.6.0 | `fastverk/spec` |
| [`tomato_brand`](https://github.com/tomato-bazel/brand) | 0.0.1 | `tomato-bazel/brand` |
| [`vpn`](https://github.com/fastverk/vpn) | 0.0.1 | `fastverk/vpn` |
| [`wave`](https://github.com/fastverk/wave) | 0.0.1 | `fastverk/wave` |

<!-- /BOTNOC:MODULES_TABLE -->

## Quick start

Add the registry chain to your consumer's .bazelrc:

```
common --registry=https://registry.tbzl.dev/
common --registry=https://bcr.bazel.build/
```

Put this registry before BCR so its entries win for overlapping module
names.

Then declare the dep in MODULE.bazel:

```python
bazel_dep(name = "rules_lora", version = "0.0.1")
```

That is it: no local_path_override needed. The registry resolves the
module's MODULE.bazel + source.json and Bazel fetches the tarball URL.

## Auth

For module entries that point at private GitHub repos, Bazel needs to
forward GitHub credentials when fetching. Two equivalent approaches:

### Option A: Bazel credential helper (recommended)

Project-local and does not pollute global state. Drop a small shell
script into your repo and reference it from .bazelrc. The canonical
implementation lives in
https://github.com/fastverk/rules_postgres/blob/main/tools/credhelper/gh-cred-helper.sh
and forwards gh auth token (or GITHUB_TOKEN / GH_TOKEN) as a bearer
header.

Wire it in .bazelrc:

```
common --credential_helper=*.github.com=%workspace%/tools/credhelper/gh-cred-helper.sh
common --credential_helper=github.com=%workspace%/tools/credhelper/gh-cred-helper.sh
common --credential_helper=raw.githubusercontent.com=%workspace%/tools/credhelper/gh-cred-helper.sh
common --credential_helper=codeload.github.com=%workspace%/tools/credhelper/gh-cred-helper.sh
```

### Option B: ~/.netrc

```
machine api.github.com
  login <your-github-username>
  password <gh-personal-access-token-with-repo-scope>

machine codeload.github.com
  login <your-github-username>
  password <gh-personal-access-token-with-repo-scope>

machine raw.githubusercontent.com
  login <your-github-username>
  password <gh-personal-access-token-with-repo-scope>
```

Run chmod 600 ~/.netrc. Bazel reads this automatically.

## Maintenance

Adding a new version or onboarding a new module goes through rels:
https://github.com/fastverk/bazel-registry/tree/main/tools/rels

```sh
cd ~/Workspace/rules_lora
git tag v0.0.2 -m "rules_lora v0.0.2"
git push origin v0.0.2

cd ~/Workspace/bazel-registry
GH_TOKEN=$(gh auth token) \
  tools/rels/target/release/rels release \
    --repo fastverk/rules_lora \
    --version 0.0.2 \
    --registry-root ~/Workspace/bazel-registry

git add modules/rules_lora
git commit -m "Register rules_lora v0.0.2"
git push origin main
```

## When to use this registry

Use this registry for fastverk-owned modules.

## Module-table maintenance

Regenerate README from local metadata with:

```sh
bazel run //:readme.write
```

The companion test target //:readme.write_test detects drift in CI.
