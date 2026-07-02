#!/usr/bin/env python3
"""Generate the static docs site for the tbzl.dev bzlmod registry.

Reads this registry's modules/ tree + metadata.json files and emits a static
site (landing + a filterable module index + a page per module) under --out.
Stdlib only — no deps, so the Pages workflow needs no pip install. Published to
GitHub Pages at https://tbzl.dev/; Bazel consumers use https://registry.tbzl.dev/.
"""
import argparse
import html
import json
import os
from pathlib import Path

REGISTRY_URL = "https://registry.tbzl.dev"
REPO_URL = "https://github.com/tomato-bazel/bazel-registry"

CSS = """
:root { --fg:#1a1a1a; --muted:#666; --line:#e5e5e5; --accent:#c0392b; --bg:#fff; }
* { box-sizing:border-box; }
body { font:15px/1.5 -apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;
       color:var(--fg); background:var(--bg); margin:0; }
header { display:flex; align-items:center; gap:.6rem; padding:1rem 1.5rem; border-bottom:1px solid var(--line); }
.brand { font-weight:700; font-size:1.15rem; color:var(--accent); text-decoration:none; }
.tag { color:var(--muted); font-size:.8rem; }
main { max-width:960px; margin:0 auto; padding:1.5rem; }
footer { max-width:960px; margin:2rem auto; padding:1rem 1.5rem; border-top:1px solid var(--line);
         color:var(--muted); font-size:.85rem; }
h1 { font-size:1.6rem; margin:.2rem 0 1rem; }
h2 { font-size:1.1rem; margin:1.5rem 0 .5rem; }
pre { background:#f6f6f6; border:1px solid var(--line); border-radius:6px; padding:.7rem .9rem;
      overflow:auto; font-size:.9rem; }
code { font-family:ui-monospace,SFMono-Regular,Menlo,monospace; }
a { color:var(--accent); }
input#q { width:100%; padding:.6rem .8rem; font-size:1rem; border:1px solid var(--line);
          border-radius:6px; margin:1rem 0; }
table { width:100%; border-collapse:collapse; }
th,td { text-align:left; padding:.5rem .6rem; border-bottom:1px solid var(--line); }
th { color:var(--muted); font-weight:600; font-size:.8rem; text-transform:uppercase; letter-spacing:.03em; }
td:nth-child(2),td:nth-child(3),th:nth-child(2),th:nth-child(3) { color:var(--muted); }
.versions { list-style:none; padding:0; }
.versions li { padding:.35rem 0; border-bottom:1px solid var(--line); }
.yanked { color:#fff; background:#999; border-radius:4px; font-size:.7rem; padding:.05rem .35rem; }
.meta { color:var(--muted); }
"""


def esc(s):
    return html.escape(str(s))


def repo_link(repo):
    # "github:org/repo" -> https://github.com/org/repo
    if repo.startswith("github:"):
        return "https://github.com/" + repo[len("github:"):]
    return repo


def read_modules(root):
    mods = []
    mroot = root / "modules"
    for name in sorted(os.listdir(mroot)):
        meta_path = mroot / name / "metadata.json"
        if not meta_path.is_file():
            continue
        meta = json.loads(meta_path.read_text())
        versions = meta.get("versions", []) or []
        mods.append({
            "name": name,
            "homepage": meta.get("homepage", "") or "",
            "repository": (meta.get("repository") or [""])[0],
            "maintainers": meta.get("maintainers", []) or [],
            "versions": versions,
            "yanked": meta.get("yanked_versions", {}) or {},
            "latest": versions[-1] if versions else "",
        })
    return mods


def page(title, body, depth=0):
    root = "../" * depth
    return (
        f"<!doctype html><html lang=en><head>"
        f'<meta charset=utf-8><meta name=viewport content="width=device-width,initial-scale=1">'
        f"<title>{esc(title)}</title><link rel=stylesheet href=\"{root}style.css\"></head>"
        f'<body><header><a class=brand href="{root}index.html">tbzl.dev</a>'
        f"<span class=tag>tomato-bazel registry</span></header><main>{body}</main>"
        f'<footer>bzlmod registry &middot; <code>--registry={REGISTRY_URL}/</code>'
        f' &middot; <a href="{REPO_URL}">source</a></footer></body></html>'
    )


def gen_index(mods):
    rows = []
    for m in mods:
        hp = f'<a href="{esc(m["homepage"])}">home</a>' if m["homepage"] else ""
        rows.append(
            f'<tr data-name="{esc(m["name"])}">'
            f'<td><a href="modules/{esc(m["name"])}/index.html">{esc(m["name"])}</a></td>'
            f'<td>{esc(m["latest"])}</td><td>{len(m["versions"])}</td><td>{hp}</td></tr>'
        )
    body = (
        "<h1>The tomato-bazel Bazel registry</h1>"
        '<p>A <a href="https://bazel.build/external/registry">bzlmod</a> registry. '
        "Point Bazel at it (in <code>.bazelrc</code>):</p>"
        f"<pre>common --registry={REGISTRY_URL}/\n"
        "common --registry=https://bcr.bazel.build/</pre>"
        f"<p>{len(mods)} modules.</p>"
        '<input id=q placeholder="filter modules…" autofocus>'
        "<table><thead><tr><th>module</th><th>latest</th><th>versions</th><th></th></tr></thead>"
        f'<tbody id=rows>{"".join(rows)}</tbody></table>'
        "<script>"
        "const q=document.getElementById('q'),rows=[...document.querySelectorAll('#rows tr')];"
        "q.oninput=()=>{const v=q.value.toLowerCase();"
        "rows.forEach(r=>r.style.display=r.dataset.name.includes(v)?'':'none')};"
        "</script>"
    )
    return page("tbzl.dev — Bazel registry", body)


def gen_module(m):
    vers = []
    for v in reversed(m["versions"]):
        yanked = m["yanked"].get(v)
        badge = f' <span class=yanked title="{esc(yanked)}">yanked</span>' if yanked else ""
        vdir = f"{REPO_URL}/tree/main/modules/{m['name']}/{v}"
        vers.append(f'<li><a href="{vdir}">{esc(v)}</a>{badge}</li>')
    maint = ", ".join(esc(x.get("name", "")) for x in m["maintainers"] if x.get("name"))
    repo = repo_link(m["repository"])
    meta_bits = []
    if m["homepage"]:
        meta_bits.append(f'<a href="{esc(m["homepage"])}">{esc(m["homepage"])}</a>')
    if repo:
        meta_bits.append(f'repo: <a href="{esc(repo)}">{esc(m["repository"])}</a>')
    if maint:
        meta_bits.append(f"maintainers: {maint}")
    body = (
        f"<h1>{esc(m['name'])}</h1>"
        f'<p class=meta>{" &middot; ".join(meta_bits)}</p>'
        f'<pre>bazel_dep(name = "{esc(m["name"])}", version = "{esc(m["latest"])}")</pre>'
        f'<h2>versions</h2><ul class=versions>{"".join(vers)}</ul>'
    )
    return page(f"{m['name']} — tbzl.dev", body, depth=2)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="_site")
    ap.add_argument("--root", default=".")
    args = ap.parse_args()
    root, out = Path(args.root), Path(args.out)
    mods = read_modules(root)
    out.mkdir(parents=True, exist_ok=True)
    (out / "style.css").write_text(CSS)
    (out / "index.html").write_text(gen_index(mods))
    for m in mods:
        d = out / "modules" / m["name"]
        d.mkdir(parents=True, exist_ok=True)
        (d / "index.html").write_text(gen_module(m))
    print(f"generated {len(mods)} module pages -> {out}")


if __name__ == "__main__":
    main()
