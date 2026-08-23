#!/usr/bin/env python3
"""Catalog integrity: every source= resolves, and the five phi-family neighbours
are all present and distinct.

Written after a glob of `gft*` deleted specs/numeric/gfternary.t27 -- a 2-bit
{-phi, 0, +phi} alphabet, an entirely different object from the TEF ladder that
merely shares a prefix. The same glob had already dropped gfternary from the pack
index a few hours earlier. Twice is a pattern, so it gets a gate.

Exits non-zero on any failure.

  tools/check_catalog_integrity.py                gate
  tools/check_catalog_integrity.py --self-check   negative control
"""
import os
import pathlib
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _prereq import broken, plant  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
SSOT_REL = "specs/numeric/formats_catalog.t27"


# `root` is REQUIRED, deliberately not `root=ROOT`. A default is bound at import
# time, so a control that planted a tree elsewhere would still scan the repo --
# and a run from a directory where `__file__` is relative would resolve ROOT to
# `/`, scan nothing, and report zero problems for the wrong reason. Every caller
# says which tree it means.
def check(root):
    """Every rule, over the catalog under `root`. Returns (problems, rows, fam)."""
    text = (root / SSOT_REL).read_text(encoding="utf-8")
    rows = re.findall(r"// CATALOG: (.+)", text)
    problems = []

    # 1. Every source= that names a spec file must resolve.
    for row in rows:
        for path in re.findall(r'(specs/numeric/[A-Za-z0-9_]+\.t27)', row):
            if not (root / path).exists():
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
        if not (root / "specs/numeric" / spec).exists():
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
    #
    # T82: this comment described a check that a later commit DELETED, replacing
    # it with the golden-section test below and leaving the four lines above it
    # in place. Measured: stripping all nine `former_name=` fields from the SSOT
    # passed BOTH catalog gates green, and a repo-wide grep finds `former_name`
    # nowhere outside specs/ and two docs -- so nothing else would have noticed
    # either. Every tnf rung must still carry the name its measurements were
    # recorded under.
    for m in re.finditer(r"id=(tnf\d+)\b", text):
        rung = m.group(1)
        row = text[m.start():text.find("\n", m.start())]
        want = 'former_name="GF-T%s"' % rung[3:]
        if want not in row:
            problems.append(f"LOST      {rung} no longer carries {want}")

    # 5. The phi families must keep their rule visible: it is what distinguishes
    # them from the theorem-derived pair, and it was ad hoc once already.
    if "round((N-1)/phi^2)" not in text:
        problems.append("LOST      the golden-section rule from the GF-T block")

    return problems, rows, fam


# --- negative control -------------------------------------------------------
#
# T85: this gate ran green from the day it was written and had never once been
# shown to go red. `tri gates sweep` named it as one of four with no control at
# all -- while docs/NOW.md claimed it had been "verified red on each of those
# failures individually". That claim was not reproducible; this is.
#
# Every case below calls check() -- the SAME function main() calls -- against a
# planted tree handed over as an argument. No comparison is re-implemented here,
# and no subprocess is spawned, so module-level ROOT is never consulted and
# cannot silently resolve to `/`.

_SPECS = ["gf16.t27", "gft16.t27", "bnf16.t27", "tnf16.t27", "gfternary.t27"]
_ROWS = [
    'id=gf16 name="GF16" bits=16 source="specs/numeric/gf16.t27"',
    'id=gft16 name="GF-T16" bits=16 source="specs/numeric/gft16.t27"',
    'id=bnf16 name="BNF16" bits=16 source="specs/numeric/bnf16.t27"',
    'id=tnf16 name="TNF16" former_name="GF-T16" bits=16 source="specs/numeric/tnf16.t27"',
    'id=gfternary name="GFTernary" bits=2 source="specs/numeric/gfternary.t27"',
]
_RULE = "  // Closed-form normative rule: e = round((N-1)/phi^2); m = N-1-e;"
# Prose that QUOTES the field rule 4 looks for, in a comment rather than a row --
# the real catalog's T82 note does exactly this. It is load-bearing for the
# control: rule 4 must search the offending ROW, and with the string present
# somewhere else in the file a `row`->`text` slip stops the check from ever
# firing. Without this line the two spellings are indistinguishable and that
# mutant survives. Must contain no `id=`, or it would join `ids`.
_PROSE = '  // T82: every tnf rung carries former_name="GF-T16" and its siblings.'


def _plant(td, rows=None, specs=None, rule=True):
    """Write a minimal but rule-complete catalog tree under `td`; return its root."""
    root = pathlib.Path(td)
    (root / "specs/numeric").mkdir(parents=True, exist_ok=True)
    for s in (_SPECS if specs is None else specs):
        (root / "specs/numeric" / s).write_text("module x;\n", encoding="utf-8")
    body = ["module formats_catalog;", _PROSE]
    if rule:
        body.append(_RULE)
    body += ["  // CATALOG: " + r for r in (_ROWS if rows is None else rows)]
    (root / SSOT_REL).write_text("\n".join(body) + "\n", encoding="utf-8")
    return root


def self_check():
    import shutil
    import subprocess
    import tempfile

    ok = True

    def case(label, expect, absent=(), **kw):
        """Plant one fault, run the real check, demand exactly `expect` back.

        `expect` is the full problem list, so every OTHER branch staying silent
        is asserted too -- a different fault reaching the right exit code
        through the wrong branch cannot pass. `absent` additionally names
        markers by text, for the branches that share a prefix.
        """
        nonlocal ok
        with tempfile.TemporaryDirectory() as td:
            problems, _, _ = check(_plant(td, **kw))
        leaked = [a for a in absent if any(a in p for p in problems)]
        good = problems == expect and not leaked
        print("  %-26s %s" % (label, "RED, correct branch" if good else "CONTROL FAILED"))
        if not good:
            ok = False
            print("       expected %r" % (expect,))
            print("       got      %r" % (problems,))
            if leaked:
                print("       neighbouring marker leaked: %r" % (leaked,))

    # The clean planted tree must be silent, or every case below passes for free.
    with tempfile.TemporaryDirectory() as td:
        problems, rows, fam = check(_plant(td))
    sizes = {k: len(v) for k, v in fam.items()}
    want_sizes = {"gf": 1, "gft": 1, "bnf": 1, "tnf": 1}
    clean_ok = problems == [] and len(rows) == 5 and sizes == want_sizes
    print("  %-26s %s" % ("clean planted tree", "silent" if clean_ok else "CONTROL FAILED"))
    if not clean_ok:
        ok = False
        print("       problems %r rows %d families %r" % (problems, len(rows), sizes))
    # Family sizes are asserted, not just emptiness: widening a family pattern
    # (`gf\d+` -> `gf\w+`) would swallow gfternary into the gf family, and every
    # non-empty check would stay green while the families stopped being distinct.

    def drop(sub):
        return [r for r in _ROWS if sub not in r]

    # 1. source= naming a spec that is not on disk.
    case("1 dangling source",
         ["DANGLING  posit16 -> specs/numeric/posit16.t27"],
         absent=("MISSING", "COLLAPSED", "ALIAS", "LOST"),
         rows=_ROWS + ['id=posit16 name="Posit16" source="specs/numeric/posit16.t27"'])

    # 2. A named neighbour gone from the catalog while its spec file survives --
    #    the row half of the failure that motivated the gate.
    case("2 neighbour row gone",
         ["MISSING   id=gfternary from the catalog "
          "(2-bit {-phi, 0, +phi} alphabet, not a float)"],
         absent=("DANGLING", "COLLAPSED", "ALIAS", "LOST"),
         rows=drop("id=gfternary"))

    # 2b. The other half: the row stays, the spec file is deleted by a glob.
    case("2b neighbour file gone",
         ["DANGLING  gfternary -> specs/numeric/gfternary.t27",
          "MISSING   specs/numeric/gfternary.t27 on disk "
          "(2-bit {-phi, 0, +phi} alphabet, not a float)"],
         absent=("COLLAPSED", "ALIAS", "LOST"),
         specs=[s for s in _SPECS if s != "gfternary.t27"])

    # 3. A family losing its last rung. MISSING rides along because bnf16 is
    #    both the family's only member here and one of the five named
    #    neighbours; asserting the exact pair pins which branch said what.
    case("3 family collapsed",
         ["MISSING   id=bnf16 from the catalog "
          "(theorem axis, binary -- the control for TNF)",
          "COLLAPSED  family bnf has no rungs left"],
         absent=("DANGLING", "ALIAS", "LOST", "on disk"),
         rows=drop("id=bnf16"))

    # 4. Two rows claiming one spec file -- aliasing where it actually happens.
    case("4 two rows, one spec",
         ["ALIAS      specs/numeric/gf16.t27 is claimed by 2 rows: "
          "['gf16', 'gf16_alias']"],
         absent=("DANGLING", "MISSING", "COLLAPSED", "LOST"),
         rows=_ROWS + ['id=gf16_alias name="GF16" source="specs/numeric/gf16.t27"'])

    # 5. A tnf rung stripped of the name its measurements were recorded under.
    #    Shares the `LOST` marker with case 6, so each names the OTHER's text as
    #    the thing that must be absent -- the exit code alone cannot tell them
    #    apart, and neither can the first word.
    case("5 former_name stripped",
         ['LOST      tnf16 no longer carries former_name="GF-T16"'],
         absent=("DANGLING", "MISSING", "COLLAPSED", "ALIAS", "golden-section"),
         rows=[r.replace(' former_name="GF-T16"', '') for r in _ROWS])

    # 6. The golden-section rule string gone from the file.
    case("6 golden-section rule gone",
         ["LOST      the golden-section rule from the GF-T block"],
         absent=("DANGLING", "MISSING", "COLLAPSED", "ALIAS", "no longer carries"),
         rule=False)

    # T85b: everything above proves check(). Nothing above proves that a
    # non-empty problem list becomes a non-zero EXIT CODE -- and that wiring
    # lives in main(), which no case touches. Measured: with main()'s `return 1`
    # changed to `return 0` the gate printed OK on a catalog with a dangling
    # source=, and every case above still reported "RED, correct branch".
    #
    # So run the WHOLE program, twice, end to end. The script is COPIED into the
    # planted tree, which makes ROOT resolve there by the ordinary
    # parent.parent rule -- no --root flag and no environment override, so this
    # adds no way to aim a live gate somewhere harmless.
    def spawned(label, want, expect_text, **kw):
        nonlocal ok
        with tempfile.TemporaryDirectory() as td:
            root = _plant(td, **kw)
            (root / "tools").mkdir(exist_ok=True)
            # plant() copies what this file imports, not only this file.
            # A copy of the script alone dies on ImportError the day a
            # sibling import is added, and every control below then reads
            # as "expected text absent" -- four of them, here.
            plant(__file__, root / "tools")
            proc = subprocess.run(
                [sys.executable, str(root / "tools" / pathlib.Path(__file__).name)],
                capture_output=True, text=True)
        said = expect_text in proc.stdout
        good = proc.returncode == want and said
        print("  %-26s %s" % (label, "exit %d, says it" % want if good else "CONTROL FAILED"))
        if not good:
            ok = False
            print("       exit %r (want %r); text present: %s" % (proc.returncode, want, said))
            print("       stdout %r" % (proc.stdout[:300],))

    spawned("end-to-end clean tree", 0, "OK: 5 catalog rows")
    spawned("end-to-end broken tree", 1, "FAIL: 1 problem(s)", rows=drop("id=gfternary"))

    print("  self-check: %s" % ("all branches proven red" if ok else "FAILED"))
    return 0 if ok else 1


def main():
    if "--self-check" in sys.argv:
        return self_check()
    # A crash is not a verdict. Run where the tracked input is absent, this
    # raised FileNotFoundError and a traceback -- which check_gate_preconditions
    # scores WRONG: "it went red, but not through the branch that explains
    # why". broken(), not skip(): a missing TOOL is the environment, a missing
    # file this repository tracks is the repository.
    ssot = ROOT / SSOT_REL
    if not ssot.is_file():
        broken(f"{SSOT_REL} is missing. It is the catalog this gate compares "
               "against, and it is tracked in git.")
    problems, rows, fam = check(ROOT)
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
