#!/usr/bin/env python3
"""Re-derive specs/RUST_DIALECT.json from its own markers and diff it.

The manifest says `generated_by: marker scan, 8 Rust constructs`, and nothing
in the repository re-runs that scan. It is used as a ceiling -- specs listed
here are treated as out of scope for emitter work (#2688) -- so when it drifts,
work is assigned to the wrong side of the line.

Two ways it was wrong when this was written:

  - **14 specs the markers hit are not listed.** Ten of them for `array_semi`.
  - **11 listed entries no longer stand.** Two rest on a marker that appears
    only inside a `//` comment, which the original scan did not skip; nine rest
    on `array_semi` alone, and the emitter now handles that form. Two of those
    eleven pass `zig ast-check` today, so the manifest was exempting specs that
    work.

This script reports the diff. It does not rewrite the manifest: which markers
still mean "out of scope" is a judgement about the language, not about text.
"""
import collections
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

# One pattern per marker the manifest uses. Comment lines are skipped, which
# the original scan did not do.
PATTERNS = {
    "let": re.compile(r"^\s*let\s+(mut\s+)?[A-Za-z_]"),
    "match": re.compile(r"^\s*(?:\w+\s*=\s*)?match\s+\S+\s*\{"),
    "impl": re.compile(r"^\s*impl\b"),
    "for_in": re.compile(r"^\s*for\s+[A-Za-z_]\w*\s+in\s"),
    "self_ref": re.compile(r"&(mut\s+)?self\b"),
    "tuple_ret": re.compile(r"->\s*\([^)]*,[^)]*\)"),
    "array_semi": re.compile(r"\[[^\[\]]+;\s*[A-Za-z0-9_]+\s*\]"),
    "vec": re.compile(r"\bVec\s*<"),
}

# Markers the emitter now handles, so a spec resting only on these is no longer
# out of scope. Move a marker here when the construct is supported, and the
# diff below will surface the entries it makes stale.
SUPPORTED = {"array_semi"}


def markers(path: pathlib.Path) -> set:
    hit = set()
    for line in path.read_text(errors="replace").splitlines():
        if line.strip().startswith("//"):
            continue
        for name, rx in PATTERNS.items():
            if rx.search(line):
                hit.add(name)
    return hit


def main() -> int:
    manifest = json.load(open(ROOT / "specs/RUST_DIALECT.json"))
    listed = {e["path"] for e in manifest["files"]}

    hits = {}
    for p in sorted(list((ROOT / "specs").rglob("*.t27"))
                    + list((ROOT / "specs").rglob("*.vibee"))):
        found = markers(p)
        if found:
            hits[str(p.relative_to(ROOT))] = found

    unlisted = sorted(set(hits) - listed)
    stale = sorted(f for f in listed
                   if (ROOT / f).exists() and hits.get(f, set()) <= SUPPORTED)

    print(f"  specs the markers hit:      {len(hits)}")
    print(f"  specs the manifest lists:   {len(listed)}")
    print(f"  hit but NOT listed:         {len(unlisted)}")
    print(f"  listed but no longer standing: {len(stale)}")

    if unlisted:
        by_marker = collections.Counter(m for f in unlisted for m in hits[f])
        print("\n  unlisted, by marker:")
        for m, n in by_marker.most_common():
            print(f"    {n:3d}  {m}")
        for f in unlisted[:10]:
            print(f"      {f}   {sorted(hits[f])}")

    if stale:
        print(f"\n  listed on {' / '.join(sorted(SUPPORTED))} alone, or on a comment:")
        for f in stale[:12]:
            print(f"      {f}   {sorted(hits.get(f, []))}")

    if len(sys.argv) > 1:
        scan = json.load(open(sys.argv[1]))
        passing = [f for f in stale if scan.get(f, {}).get("first") == "VALID"]
        print(f"\n  of the stale entries, ast-check VALID today: {len(passing)}")
        for f in passing:
            print(f"      {f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
