#!/usr/bin/env python3
"""Generate README.md module table from local registry metadata files."""

from __future__ import annotations

import argparse
import json
from collections import OrderedDict
from pathlib import Path
from typing import Iterable


def parse_version(version: str) -> tuple:
    parts = version.split(".")
    parsed = []
    for part in parts:
        if part.isdigit():
            parsed.append((0, int(part)))
        else:
            parsed.append((1, part))
    return tuple(parsed)


def pick_latest(versions: Iterable[str]) -> str:
    vals = [v for v in versions if v]
    if not vals:
        return "0.0.0"
    return sorted(vals, key=parse_version)[-1]


def repo_from_metadata(data: dict, module_name: str) -> tuple[str, str]:
    """Return (display, url) for a module's source repository.

    Handles both `repository` spellings the registry actually contains:

      * `"github:owner/name"` — the BCR convention, and what most entries use.
      * a full `https://…` URL — what a module hosted somewhere other than
        GitHub carries (rules_aion lives on gitlab.savvifi.com).

    Previously only the first form was understood and everything else fell back
    to `fastverk/<module>` rendered under `https://github.com/…`, i.e. a link to
    a repository that does not exist. That was invisible while every entry was a
    GitHub one; it stops being invisible the moment a GitLab-hosted module is
    registered, and a fabricated URL in the published table is worse than an
    absent one because it looks authoritative.
    """
    repos = data.get("repository", [])
    entry = repos[0] if repos and isinstance(repos, list) else None

    if isinstance(entry, str):
        if entry.startswith("github:"):
            slug = entry[len("github:") :]
            return slug, f"https://github.com/{slug}"
        if entry.startswith(("https://", "http://")):
            # Display the host-qualified path so the table shows WHERE it lives,
            # not just a slug that looks like every GitHub one.
            without_scheme = entry.split("://", 1)[1].rstrip("/")
            return without_scheme, entry

    # No usable metadata. Keep the historical guess, but point it at the homepage
    # when there is one rather than inventing a GitHub URL.
    homepage = data.get("homepage")
    slug = f"fastverk/{module_name}"
    return slug, homepage or f"https://github.com/{slug}"


def load_modules(metadata_files: list[str]) -> list[tuple[str, str, str, str]]:
    rows = []
    for mf in metadata_files:
        path = Path(mf)
        module_name = path.parent.name
        data = json.loads(path.read_text())
        latest = pick_latest(data.get("versions", []))
        repo, url = repo_from_metadata(data, module_name)
        rows.append((module_name, latest, repo, url))
    rows.sort(key=lambda t: t[0])
    return rows


def categorize(
    rows: list[tuple[str, str, str, str]],
    rules: OrderedDict[str, dict],
) -> OrderedDict[str, list[tuple[str, str, str, str]]]:
    categorized = OrderedDict((name, []) for name in rules.keys())
    categorized["Uncategorized"] = []

    for row in rows:
        module_name = row[0]
        placed = False
        for category, rule in rules.items():
            exact = set(rule.get("exact", []))
            prefixes = rule.get("prefix", [])
            if module_name in exact or any(
                module_name.startswith(prefix)
                for prefix in prefixes
            ):
                categorized[category].append(row)
                placed = True
                break
        if not placed:
            categorized["Uncategorized"].append(row)

    return categorized


def render_rows(rows: list[tuple[str, str, str, str]]) -> str:
    out = []
    for module_name, latest, repo, url in rows:
        out.append(f"| [`{module_name}`]({url}) | {latest} | `{repo}` |")
    return "\n".join(out)


def render_sections(categorized: OrderedDict[str, list[tuple[str, str, str, str]]]) -> str:
    sections = []
    for category, rows in categorized.items():
        if not rows:
            continue
        sections.append(f"### {category}")
        sections.append("")
        sections.append("| Module | Latest | Repository |")
        sections.append("|---|---|---|")
        sections.append(render_rows(rows))
        sections.append("")
    return "\n".join(sections).rstrip() + "\n"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--template", required=True)
    parser.add_argument("--rules", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("metadata", nargs="+")
    args = parser.parse_args()

    rows = load_modules(args.metadata)
    rules = json.loads(Path(args.rules).read_text(), object_pairs_hook=OrderedDict)
    categorized = categorize(rows, rules)
    sections = render_sections(categorized)

    template = Path(args.template).read_text()
    content = template.replace("{{CATEGORIZED_MODULE_TABLES}}", sections)
    Path(args.out).write_text(content)


if __name__ == "__main__":
    main()
