#!/usr/bin/env python3
"""Every spec population this campaign has published, with the matcher that
produced it, re-derivable on demand.

WHY. A population column in a reader counted COMMENTS for one pass and inflated
seven rows; `sign` was published as "63 uses in 48 specs" and is one use in one
spec, and that 63 had already become a recommendation to treat it as a
language-level question (#3501). The obvious response is to re-count every
figure. Doing so found something else: almost all of the movement was NOT
miscounting. It was that the published sentence and the matcher behind it asked
different questions.

    `len(` "142"        was a DIAGNOSTIC count in the generated C.
                        The spec population is 296 in 29 specs.
    `pub const OP_*` 20 was a count of LIST SITES in the generated C.
                        The declarations are 11, in one spec.
    `[T]` 220           depends on which names count as a type: 220 with one
                        set of primitives, 228 with `float` and `int` added.

A number without its matcher is not reproducible, and a number whose unit is
implicit is not comparable. So this file pins BOTH, and `--check` re-derives
each and prints the drift. Comments are excluded everywhere, which is what the
one genuine miscount was about.

Usage:
  tools/published_figures.py            the table, re-derived now
  tools/published_figures.py --check    exit 1 if a figure has drifted
  tools/published_figures.py --self-check  negative control

Exit codes:
  0  every figure matches its pin (or the table printed)
  1  a figure drifted -- re-derive, decide whether the code or the pin is wrong
  2  COULD NOT RUN (no specs)
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPECS = os.path.join(ROOT, "specs")

# (claim, unit, regex over CODE lines, pinned value, where it was published)
FIGURES = [
    ("cast_i8 uses", "uses",
     r"(?<![\w.@])cast_i8\s*\(", 1079, "#3497"),
    ("cast_i16 uses", "uses",
     r"(?<![\w.@])cast_i16\s*\(", 38, "#3497"),
    ("[]T{} empty slice literals", "literals",
     r"\[\]\s*[A-Za-z_][\w:]*\s*\{\s*\}", 480, "#3495; queen harvest 2026-09-10 added 2"),
    ("x.len() with an identifier base", "call sites",
     r"\b[A-Za-z_]\w*\s*\.\s*len\s*\(", 1319, "#3489, corrected from 1322"),
    ("x.len with an identifier base", "field reads",
     r"\b[A-Za-z_]\w*\s*\.\s*len\b(?!\s*\()", 710, "#3489, corrected from 687; queen harvest 2026-09-10 added 30"),
    ("len(x) free-function spelling", "call sites",
     r"(?<![\w.@])len\s*\(", 296, "#3489 said 142 -- that was a DIAGNOSTIC count"),
    ("three-segment paths a::b::c", "occurrences",
     r"\b[A-Za-z_]\w*::[A-Za-z_]\w*::[A-Za-z_]\w*", 473, "#3473, corrected from 477"),
    ("pub const OP_* declarations", "declarations",
     r"^\s*pub\s+const\s+OP_\w+", 11, "#3497 said 20 -- that was a SITE count in the C"),
    ("abs( uses", "uses",
     r"(?<![\w.@])abs\s*\(", 389, "#3501"),
    # The pin FOLLOWED the corpus, and the movement is explained rather than
    # blessed away: #3482 deleted 188 duplicate test blocks whose bodies were
    # byte-identical to their twin. 12644 - 188 = 12456, which is what a
    # re-derivation gives -- the first thing this file caught, on its first run.
    # #3557 added specs/ui/viewport.t27 with six test blocks: 12456 + 6 = 12462.
    # #3556 (Closes #3559) added specs/automation/inngest-probe-suite.t27 with
    # three test blocks: 12462 + 3 = 12465.
    # The 2026-09-10 queen harvest (#3560) landed 9 bee patches (#3508 #3515
    # #3524 #3525 #3530 #3531 #3534 #3536 #3538), each adding test blocks
    # beside the functions it implemented: 12465 + 77 = 12542.
    # #3561 added specs/memory/tmem/ (six Trinity Memory contract specs) with
    # 51 test blocks: 12542 + 51 = 12593.
    # #3563 (S01 of gHashTag/trinity#988) added specs/trinity/ (project.t27 with
    # five test blocks, fifty capability cards with one each) and the canonical
    # copy specs/catalog/discovery.t27 with five: 12593 + 60 = 12653.
    # #3564 (S02) added specs/trinity/compiler_matrix.t27 with six: 12653 + 6 = 12659.
    # Its fixtures live under bootstrap/tests/fixtures/trinity_matrix/, outside
    # specs/, and are not counted here.
    # #3565 (S03) added specs/trinity/build_graph.t27 with four: 12659 + 4 = 12663.
    # #3566 (S04) added specs/vsa/trinity_compat.t27 with ten: 12663 + 10 = 12673.
    ("test blocks", "blocks",
     r"^\s*test\s+(?:\"[^\"]*\"|[A-Za-z_][\w\-]*)\s*\{?\s*$", 12673,
     "#3479 pinned 12644; #3482 removed 188; #3557 added 6; #3556 added 3; #3560 added 77; #3561 added 51"),
]


def code_lines():
    if not os.path.isdir(SPECS):
        print("published_figures: no specs directory. Exit 2.", file=sys.stderr)
        sys.exit(2)
    for r, _, fs in os.walk(SPECS):
        for f in sorted(fs):
            if not f.endswith(".t27"):
                continue
            p = os.path.join(r, f)
            for line in open(p, encoding="utf-8", errors="replace"):
                t = line.lstrip()
                # The one genuine miscount this file exists for.
                if t.startswith("//") or t.startswith("#"):
                    continue
                yield p, line


def derive():
    rows = [(re.compile(pat), 0, set()) for _, _, pat, _, _ in FIGURES]
    for p, line in code_lines():
        for i, (rx, _, _) in enumerate(rows):
            k = len(rx.findall(line))
            if k:
                rows[i] = (rx, rows[i][1] + k, rows[i][2] | {p})
    return [(n, len(f)) for _, n, f in rows]


def self_check() -> int:
    """A comment line must not count, and a code line must. Without both, a
    file that reads nothing and a file that reads everything look alike."""
    import tempfile
    ok = True
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "x.t27")
        open(p, "w").write("// cast_i8(1) in a comment\nvar a = cast_i8(2);\n")
        rx = re.compile(r"(?<![\w.@])cast_i8\s*\(")
        seen = sum(
            len(rx.findall(l))
            for l in open(p)
            if not l.lstrip().startswith("//")
        )
        print(f"  one in code, one in a comment -> counted {seen} "
              f"({'PASS' if seen == 1 else 'FAIL'})")
        ok &= seen == 1
    return 0 if ok else 2


def main() -> int:
    if "--self-check" in sys.argv:
        return self_check()
    got = derive()
    check = "--check" in sys.argv
    drift = 0
    print(f"{'figure':38} {'unit':13} {'pinned':>7} {'now':>7} {'specs':>6}  published as")
    for (name, unit, _, pinned, where), (now, nspecs) in zip(FIGURES, got):
        mark = "" if now == pinned else "  DRIFT"
        if now != pinned:
            drift += 1
        print(f"{name:38} {unit:13} {pinned:7} {now:7} {nspecs:6}  {where}{mark}")
    if drift:
        print(f"\n{drift} figure(s) drifted. Either the corpus moved and the pin "
              f"should follow it,\nor a matcher changed meaning -- and only reading "
              f"the diff says which.")
    return 1 if (check and drift) else 0


if __name__ == "__main__":
    sys.exit(main())
