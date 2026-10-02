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
     r"(?<![\w.@])cast_i8\s*\(", 1081, "#3497; 2026-10-02 (#5497) 1079 -> 1081 at 769f3252: 1079 at d3224e69, core 1079->1079, specs/port/ +2"),
    ("cast_i16 uses", "uses",
     r"(?<![\w.@])cast_i16\s*\(", 38, "#3497"),
    ("[]T{} empty slice literals", "literals",
     r"\[\]\s*[A-Za-z_][\w:]*\s*\{\s*\}", 525, "#3495; 2026-10-02 (#5497) 478 -> 525 at 769f3252: 481 at d3224e69, core 481->505, specs/port/ +20"),
    ("x.len() with an identifier base", "call sites",
     r"\b[A-Za-z_]\w*\s*\.\s*len\s*\(", 1414, "#3489, corrected from 1322; 2026-10-02 (#5497) 1319 -> 1414 at 769f3252: 1319 at d3224e69, core 1319->1301, specs/port/ +113"),
    ("x.len with an identifier base", "field reads",
     r"\b[A-Za-z_]\w*\s*\.\s*len\b(?!\s*\()", 2197, "#3489, corrected from 687; 2026-10-02 (#5497) 680 -> 2197 at 769f3252: 1075 at d3224e69, core 1075->1567, specs/port/ +630"),
    ("len(x) free-function spelling", "call sites",
     r"(?<![\w.@])len\s*\(", 339, "#3489 said 142 -- that was a DIAGNOSTIC count; 2026-10-02 (#5497) 296 -> 339 at 769f3252: 296 at d3224e69, core 296->332, specs/port/ +7"),
    ("three-segment paths a::b::c", "occurrences",
     r"\b[A-Za-z_]\w*::[A-Za-z_]\w*::[A-Za-z_]\w*", 619, "#3473, corrected from 477; 2026-10-02 (#5497) 473 -> 619 at 769f3252: 474 at d3224e69, core 474->503, specs/port/ +116"),
    ("pub const OP_* declarations", "declarations",
     r"^\s*pub\s+const\s+OP_\w+", 61, "#3497 said 20 -- that was a SITE count in the C; 2026-10-02 (#5497) 11 -> 61 at 769f3252: 61 at d3224e69, core 61->61, specs/port/ +0"),
    ("abs( uses", "uses",
     r"(?<![\w.@])abs\s*\(", 418, "#3501; 2026-10-02 (#5497) 389 -> 418 at 769f3252: 389 at d3224e69, core 389->418, specs/port/ +0"),
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
    # #3567 (S05) rewrote specs/isa/ternary_encoding.t27 (eleven) and added specs/isa/tri27_machine.t27
    # (ten), specs/isa/tri27_bytecode.t27 (six), specs/vm/trinity_vm.t27 (five), specs/api/c_abi.t27
    # (four): 12673 + 36 = 12709.
    # #3568 (S06) added specs/tools/catalog.t27 (five) and specs/tools/mcp_protocol.t27 (five); the 29
    # cards under specs/tools/trinity/tri/ carry no test block: 12709 + 10 = 12719.
    # #3596 added specs/automation/crm-lead-magnet.t27 with five: 12719 + 5 = 12724.
    # #3598 added specs/automation/crm-sellers.t27 with five: 12724 + 5 = 12729.
    # #3600 added specs/automation/leela-agent-link.t27 with five: 12729 + 5 = 12734.
    # #3602 added specs/automation/crm-duet.t27 with five: 12734 + 5 = 12739.
    # #3604 added specs/automation/crm-client-workspace.t27 with five: 12739 + 5 = 12744.
    # #3608 added specs/automation/crm-client-ownership.t27 with five: 12744 + 5 = 12749.
    # #3613 added three to specs/automation/crm-duet.t27: 12749 + 3 = 12752.
    # #3615 added one to crm-duet.t27 and three in the new
    # specs/automation/agent-provider-chain.t27: 12752 + 4 = 12756.
    # #3617 added two to crm-duet.t27 (discovery gate, negation): 12756 + 2 = 12758.
    # reserve max_tokens added one to agent-provider-chain.t27: 12758 + 1 = 12759.
    # #3576 added specs/memory/tmem/session.t27 (durable session record layout
    # and recovery rules) with 24 test blocks, re-derived directly rather than
    # trusted from the branch's own stale comment (which said 22): 12759 + 24 = 12783.
    # 2026-10-02 (#5497): no pin had moved since d3224e69 (2026-09-16), and five
    # had already drifted AT that commit -- the merge that resolved this file's
    # conflict kept the older numbers. Since then specs/ grew from 946 to 1146
    # files, 200 of them under specs/port/, which did not exist at d3224e69: the
    # port waves added specs without moving these pins. Each note below gives the
    # value at d3224e69 and the split of the change into the rest of specs/
    # ("core") and specs/port/. No matcher changed in this file's history.
    ("test blocks", "blocks",
     r"^\s*test\s+(?:\"[^\"]*\"|[A-Za-z_][\w\-]*)\s*\{?\s*$", 14350,
     "#3479 pinned 12644; #3482 removed 188; #3557 added 6; #3556 added 3; #3560 added 77; #3561 added 51; #3596 added 5; #3598 added 5; #3600 added 5; #3613 added 3; #3576 added 24; 2026-10-02 (#5497) 12783 -> 14350 at 769f3252: 13056 at d3224e69, core 13056->13419, specs/port/ +931"),
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
