#!/usr/bin/env python3
"""Catalog integrity: every source= resolves, and the three phi-family neighbours
are all present and distinct.

Written after a glob of `gft*` deleted specs/numeric/gfternary.t27 -- a 2-bit
{-phi, 0, +phi} alphabet, an entirely different object from the TEF ladder that
merely shares a prefix. The same glob had already dropped gfternary from the pack
index a few hours earlier. Twice is a pattern, so it gets a gate.

Exits non-zero on any failure.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SSOT = ROOT / "specs/numeric/formats_catalog.t27"


def main():
    text = SSOT.read_text(encoding="utf-8")
    rows = re.findall(r"// CATALOG: (.+)", text)
    problems = []

    # 1. Every source= that names a spec file must resolve.
    for row in rows:
        for path in re.findall(r'(specs/numeric/[A-Za-z0-9_]+\.t27)', row):
            if not (ROOT / path).exists():
                ident = re.search(r"id=(\S+)", row)
                problems.append(f"DANGLING  {ident.group(1) if ident else '?'} -> {path}")

    # 2. The three neighbours that share a prefix must all be present.
    ids = set(re.findall(r"id=(\S+)", text))
    for want, spec, why in [
        ("gfternary", "gfternary.t27", "2-bit {-phi, 0, +phi} alphabet"),
        ("gf16", "gf16.t27", "binary GoldenFloat ladder"),
        ("tnf16", "tnf16.t27", "ternary network float ladder"),
    ]:
        if want not in ids:
            problems.append(f"MISSING   id={want} from the catalog ({why})")
        # The catalog row is not enough: the spec file itself must be on disk.
        # A glob deleted gfternary.t27 while its row stayed, and nothing noticed.
        if not (ROOT / "specs/numeric" / spec).exists():
            problems.append(f"MISSING   specs/numeric/{spec} on disk ({why})")

    # 3. They must be distinct families, not aliases of one another.
    tnf = {i for i in ids if re.fullmatch(r"tnf\d+", i)}
    gf = {i for i in ids if re.fullmatch(r"gf\d+", i)}
    if not tnf or not gf or tnf & gf:
        problems.append(f"COLLAPSED tnf={len(tnf)} gf={len(gf)} overlap={sorted(tnf & gf)}")

    # 4. The former name must stay searchable. Not for citation reasons -- the
    # ladder has never been published under either name -- but because research
    # notes, prior branches and the author's own profile still use it, and every
    # measurement against takum/tekum/posit was recorded under the old label.
    if 'former_name="GF-T16"' not in text:
        problems.append('LOST      former_name="GF-T16" (internal continuity)')

    if problems:
        for p in problems:
            print(p)
        print(f"FAIL: {len(problems)} problem(s)")
        return 1
    print(f"OK: {len(rows)} catalog rows, every source= resolves, "
          f"{len(gf)} GF + {len(tnf)} TNF + gfternary all present and distinct")
    return 0


if __name__ == "__main__":
    sys.exit(main())
