#!/usr/bin/env python3
"""Catalog integrity: every source= resolves, and the five phi-family neighbours
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

    # 2. The five neighbours that share a prefix must all be present.
    ids = set(re.findall(r"id=(\S+)", text))
    # Four families on two axes -- phi-derived against theorem-derived, binary
    # against ternary -- plus the 2-bit alphabet that shares their prefix. Each has
    # been lost at least once to a glob or to being mistaken for a rename of
    # another, so each is named here explicitly.
    for want, spec, why in [
        ("gf16", "gf16.t27", "phi axis, binary"),
        ("gft16", "gft16.t27", "phi axis, ternary"),
        ("bnf16", "bnf16.t27", "theorem axis, binary -- the control for TNF"),
        ("tnf16", "tnf16.t27", "theorem axis, ternary"),
        ("gfternary", "gfternary.t27", "2-bit {-phi, 0, +phi} alphabet, not a float"),
    ]:
        if want not in ids:
            problems.append(f"MISSING   id={want} from the catalog ({why})")
        # The catalog row is not enough: the spec file itself must be on disk.
        # A glob deleted gfternary.t27 while its row stayed, and nothing noticed.
        if not (ROOT / "specs/numeric" / spec).exists():
            problems.append(f"MISSING   specs/numeric/{spec} on disk ({why})")

    # 3. They must be distinct families, not aliases of one another.
    fam = {name: {i for i in ids if re.fullmatch(pat, i)}
           for name, pat in [("gf", r"gf\d+"), ("gft", r"gft\d+"),
                             ("bnf", r"bnf\d+"), ("tnf", r"tnf\d+")]}
    for name, members in fam.items():
        if not members:
            problems.append(f"COLLAPSED  family {name} has no rungs left")
    # T68: "these are the same format under two names" -- the thing the check
    # below is for, and it has happened. It used to be tested as a family
    # INTERSECTION, which cannot fire: `gf\d+`, `gft\d+`, `bnf\d+` and
    # `tnf\d+` are pairwise disjoint under fullmatch, so `fam[a] & fam[b]` is
    # empty for every input. Brute-forced every string up to six characters
    # over the relevant alphabet: zero match two families.
    #
    # Aliasing is visible where it actually happens -- two catalog rows naming
    # the SAME spec file. Today 43 rows carry a spec path and they resolve to
    # 43 distinct specs, one to one. (`source=` is NOT the field to check: it
    # is a citation, "Alam 2021" and the like, and 30 of 109 rows legitimately
    # share one.)
    by_spec = {}
    for row in rows:
        ident = re.search(r"id=(\S+)", row)
        ident = ident.group(1) if ident else "?"
        for path in re.findall(r'(specs/numeric/[A-Za-z0-9_]+\.t27)', row):
            by_spec.setdefault(path, []).append(ident)
    for path, owners in sorted(by_spec.items()):
        if len(owners) > 1:
            problems.append(
                f"ALIAS      {path} is claimed by {len(owners)} rows: {sorted(owners)}"
            )

    # 4. The former name must stay searchable. Not for citation reasons -- the
    # ladder has never been published under either name -- but because research
    # notes, prior branches and the author's own profile still use it, and every
    # measurement against takum/tekum/posit was recorded under the old label.
    # The phi families must keep their rule visible: it is what distinguishes them
    # from the theorem-derived pair, and it was ad hoc once already.
    if "round((N-1)/phi^2)" not in text:
        problems.append("LOST      the golden-section rule from the GF-T block")

    if problems:
        for p in problems:
            print(p)
        print(f"FAIL: {len(problems)} problem(s)")
        return 1
    fam_sizes = " + ".join(f"{len(v)} {k.upper()}" for k, v in fam.items())
    print(f"OK: {len(rows)} catalog rows, every source= resolves, "
          f"{fam_sizes} + gfternary, four families present and distinct")
    return 0


if __name__ == "__main__":
    sys.exit(main())
