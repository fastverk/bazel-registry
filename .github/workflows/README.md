# constellation automation — the fastverk App control plane

> ### Status (2026-07-26): the three cron workflows below are not running.
>
> Every scheduled run of `ratchet.yml`, `report.yml` and `conformance.yml` since
> at least 2026-07-06 has failed at step 2, *Mint App token — tomato-bazel*, with
> `[@octokit/auth-app] appId option is required`, and every later step skipped.
> `FASTVERK_APP_ID` and `FASTVERK_APP_PRIVATE_KEY` do not exist as repo secrets
> or as org secrets — both endpoints return an empty list. The App installed on
> `tomato-bazel` is `fastverk-ci-bot` (installation `143784054`), so **step 2 of
> "Installing the fastverk App" below was never completed.**
>
> Consequences worth stating plainly, because each has been read as something
> else: no bump PR has ever been opened, which is the entire reason C1 sits at
> 28 rather than the dependency ratchet having driven it down; the Pages
> dashboard is stale; and `conformance.yml`'s `continue-on-error` is moot,
> because the job has never reached the step it guards. Only `gate-ratchet.yml`
> works, and it works precisely because it needs no credentials.

These workflows let the **fastverk GitHub App** reproduce, on a schedule, the
consistency work that was first done by hand — via **encapsulated commands +
config**, nothing bespoke. They span **both orgs** — `tomato-bazel` (the Bazel
distro) and `fastverk` (products + services, incl. `fastverk/agent`) — minting one
installation token per org, so the graph, the ratchet, and the conformance gate
are all cross-org.

| Concern | Command (encapsulated) | Config (source of truth) | Workflow |
|---|---|---|---|
| Dependency drift → bumps | `rels deps [--write]` (`tools/rels`) | `rules_tomato//bom/versions.json` | `ratchet.yml` |
| Consolidated graph + report (visuals) | `tools/graph/graph.py` | both orgs' repos | `report.yml` |
| Convention conformance | SHACL (`pyshacl`; rules_jena in-Bazel) | `rules_tomato//conventions/*.shacl.ttl` | `conformance.yml` |
| Publish a module | `rels release` | `modules/*/source.json` | *(manual / release)* |
| **Registry admission (PR-time)** | **`gate-ratchet`** (`tomato-bazel/gate`) | the pinned `GATE_REF` + `//gates/*.rq` | **`gate-ratchet.yml`** |

## What each does
- **`gate-ratchet.yml`** — on every `pull_request`, and **the only workflow here
  that gates a merge.** Projects the PR and its base to RDF with the *same
  pinned* `tomato-bazel/gate` revision, runs one SPARQL query per invariant
  against both, and fails the PR if any **per-gate** count went up. Note
  per-gate, not the total: a PR that clears three D2 rows and adds one D3 row
  fails.

  A ratchet rather than a gate, because the corpus is not green — this stops it
  getting worse instead of demanding zero first. Each invariant graduates to a
  real `sparql_query_test` in gate's `//gates` the moment its count reaches zero
  (S4 already has).

  It runs on **every** PR with no `paths:` filter, deliberately. A path-filtered
  workflow that does not match is *skipped*, and a skipped workflow never creates
  its check run — so a required check would sit on "Expected" for ever and block
  the PR. The filter is a step inside the job instead: a PR touching neither
  `modules/**` nor `bazel_registry.json` reports green in ~7s rather than
  ~12m50s. Measured, both numbers.

  Needs **no credentials**, because both repos are public. That is why it works
  while the three cron workflows below do not.
- **`ratchet.yml`** — weekly (or on demand): audits every repo's `MODULE.bazel`
  across both orgs against the BOM and opens a forward-only **bump PR** per
  behind-drift repo, in that repo's own org. Merges only after each repo's CI is
  green. (The same `rels deps` audit can be a required PR check so drift can't land.)
- **`report.yml`** — weekly: regenerates the dependency-graph SVG + drift/convention
  report for the whole constellation and publishes it to **GitHub Pages** (the
  health dashboard).
- **`conformance.yml`** — weekly: projects the constellation to RDF and validates it
  against the SHACL convention contract. Carries `continue-on-error: true`.

  Its comment says to flip that off "once conventions are backfilled". Do not,
  yet, and not for that reason. The flag is currently **unreachable** — see the
  status note above — so flipping it changes nothing observable. And when the
  credentials do exist, a global boolean over ~90 repos in two orgs is the wrong
  instrument: this repo would go red for a missing `CHANGELOG.md` in a `fastverk`
  service it cannot fix, which is how a gate gets trained into background noise.
  The pattern that works is already in-tree — `ENFORCED_ON_REGISTRY` in gate's
  `//gates`, promoting one invariant at a time the moment its count hits zero,
  with a ratchet holding the line meanwhile. Apply that shape to SHACL: fail on
  an *increase* against a committed baseline, then drop `continue-on-error` per
  tier as each reaches zero.

## Required checks (branch protection)

`main` is protected on **both** `bazel-registry` and `gate`, **including
administrators**, with "require branches to be up to date" on. A direct
`git push origin main` is refused; everything goes through a PR.

The required contexts are the job `name:` strings, **verbatim** — a check-run
name is the job's `name:`, or its id when there is none:

| Repo | Required context | Declared at |
|---|---|---|
| `bazel-registry` | `gate conformance ratchet` | `gate-ratchet.yml`, job `ratchet` |
| `gate` | `bazel test //... (ubuntu-latest)` | `ci.yml`, job `test` (matrix) |
| `gate` | `bazel test //... (macos-latest)` | `ci.yml`, job `test` (matrix) |
| `gate` | `buildifier lint` | `ci.yml`, job `buildifier` |
| `gate` | `vendored snapshot is current` | `ci.yml`, job `snapshot` |

Four rules follow from this, and three of them are ways to brick the branch:

1. **Un-filter before you require.** A required check must be able to report on
   *every* PR. Add the requirement only after a version of the workflow that
   always reports is on `main`.
2. **Never rename a required job under live protection.** The old context stays
   required and never reports again. Update the protection payload first, or do
   both while protection is off.
3. **Never add `required_pull_request_reviews` here.** This is effectively a
   single-maintainer repo; a review requirement combined with admin enforcement
   makes `main` permanently unmergeable. It is set to `null` on purpose.
4. `gate`'s `build` and `deploy` checks come from its `report.yml` (Pages,
   schedule-only) and must **not** be required — they never run on a PR.

`strict: true` is deliberate despite costing an "Update branch" + 12m50s when
main moves. The gates are *global graph invariants*: two PRs can each be
non-regressing against a shared base and still interact to raise C1, with
neither ever having failed. `strict` forces the second to re-project. Auto-merge
and branch auto-deletion are on to make that tolerable.

## Installing the fastverk App (one-time, owner action) — on BOTH orgs
1. Install the **fastverk** GitHub App on `tomato-bazel` **and** `fastverk`, granting
   repo **contents** + **pull-requests** (and **Pages** on this repo for the dashboard).
2. Add two **org secrets** (on the org that runs these workflows):
   - `FASTVERK_APP_ID` — the App's numeric id.
   - `FASTVERK_APP_PRIVATE_KEY` — the App's PEM private key.
3. Enable **GitHub Pages** (source: GitHub Actions) on this repo for `report.yml`.

The workflows mint a short-lived, org-scoped installation token via
`actions/create-github-app-token` — no long-lived PATs. The App is *auth*; `rels`
+ `graph.py` + the BOM/SHACL are the *logic*. Every step here is runnable locally
the same way (`rels deps --bom …`, `python3 tools/graph/graph.py …`).
