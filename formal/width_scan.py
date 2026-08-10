#!/usr/bin/env python3
"""Fail if a declared width cannot hold the range it is supposed to carry.

Wave 630. Prop. 80 found a real defect that had survived since Wave 33:
`adder_tree_27`'s level-2 array summed three values of range [-3,+3], so it
spanned [-9,+9], and was declared `signed [3:0]` -- which spans [-8,+7]. Any
group of nine trits summing to +8, +9 or -9 wrapped by 16, and the tree returned
-14 where the true sum was +2.

**The RTL's own comment said `range [-9, +9] -> signed [3:0]`.** The correct
range was written directly above the declaration that could not hold it. A unit
test asserted the buggy width verbatim, so the defect was not merely untested
but protected. Nothing mechanical ever compared the two numbers.

Three checks over the emitted tree:

  DOCUMENTED  a `range [-N, +M]` comment whose declaration cannot hold it.
              Exact, and it catches the Wave 628 defect as written.
  PROPAGATED  a reduction `assign T[i] = A + B + C;` where the operands' own
              ranges sum past what T is declared to hold. No comment on T
              required, so it still fires if someone "corrects" the comment
              instead of the width.
  CONSISTENT  a reduction whose operands sum past the range T's *comment*
              claims. This catches the documentation drifting from the design
              even when the width happens to be wide enough.

Ranges propagate from comments, not from widths. That distinction is the whole
design: `val` is declared `signed [1:0]` but holds only {-1, 0, +1}, because a
trit needs three values and two bits carry four. Reasoning from the declared
width would compute [-2,+1] per element, make level 1 span [-6,+3], and fail a
correct design -- worst-case-by-width is unsound wherever an encoding is
narrower in value than in bits, which for ternary hardware is everywhere. So a
declaration's range is what its comment says when it has one, and only falls
back to its declared width when it does not.

Scope, stated rather than implied: this reads emitted RTL only, and only the
annotated-declaration and array-reduction forms above. It is not a general
Verilog width checker. It is the specific check that would have caught the
specific defect, generalised as far as the emitters' own conventions allow.

Usage:  python3 formal/width_scan.py [--self-test]
"""

import pathlib
import re
import sys

DECL = re.compile(r"^\s*(?:(?:input|output)\s+)?wire\s+signed\s*\[(\d+):(\d+)\]"
                  r"\s+(\w+)")
RANGE_COMMENT = re.compile(r"range\s*\[\s*(-?\d+)\s*,\s*\+?(-?\d+)\s*\]")
REDUCE = re.compile(r"^\s*assign\s+(\w+)\s*(?:\[[^\]]*\])?\s*=\s*([^;]+);", re.M)
OPERAND = re.compile(r"\b([A-Za-z_]\w*)\s*(?:\[|\b)")


def span(hi, lo):
    """Signed range representable by a [hi:lo] declaration."""
    w = hi - lo + 1
    return -(2 ** (w - 1)), 2 ** (w - 1) - 1


def top_level_plus(expr):
    """Count `+` outside any bracket -- `val[i*3+1]` adds no operand."""
    depth = n = 0
    for c in expr:
        if c == "[":
            depth += 1
        elif c == "]":
            depth -= 1
        elif c == "+" and depth == 0:
            n += 1
    return n


def parse(src):
    """Return (declared, documented) name -> range, for one file."""
    lines = src.split("\n")
    declared, documented = {}, {}
    pending = None
    for line in lines:
        # Wave 637c: this used to `continue` here, so a line carrying BOTH a
        # range comment and a declaration was consumed as a comment only and
        # the declaration entered neither dict -- invisible to every check
        # below. Moving an existing comment to TRAIL its declaration, a
        # formatting change, took a provably broken adder tree from exit 1 to
        # exit 0. A same-line comment now annotates its own declaration.
        rc = RANGE_COMMENT.search(line) if "//" in line else None
        if rc:
            pending = (int(rc.group(1)), int(rc.group(2)))
        d = DECL.match(line)
        if d:
            name = d.group(3)
            declared[name] = span(int(d.group(1)), int(d.group(2)))
            if pending:
                documented[name] = pending
        if line.strip() and not line.strip().startswith("//"):
            pending = None
    return declared, documented


def check_file(path):
    """Return (findings, declared_count, annotated_count, reduction_count)."""
    src = path.read_text()
    declared, documented = parse(src)
    # A declaration's range is what it is DOCUMENTED to carry, and nothing else.
    #
    # Wave 637c: this used to fall back to the declared width for an
    # unannotated operand, which is precisely the worst-case-by-width rule the
    # docstring above calls unsound for ternary -- `val` is signed [1:0] but
    # holds only {-1,0,+1}. Deleting one of the three range comments therefore
    # made the gate report a FALSE finding against correct RTL: `l1` "reaches
    # [-6, 3]" using [-2,+1] per operand instead of the true [-1,+1].
    #
    # An unannotated operand now makes the reduction UNCHECKABLE rather than
    # checkable-by-a-wrong-rule. That is a real loss of coverage, so it is
    # counted and printed rather than absorbed silently.
    rng = dict(documented)
    bad = []

    for name, (lo_req, hi_req) in documented.items():
        lo_can, hi_can = declared[name]
        if lo_req < lo_can or hi_req > hi_can:
            n = next(i for i, l in enumerate(src.split("\n"), 1)
                     if DECL.match(l) and DECL.match(l).group(3) == name)
            bad.append(f"{path.name}:{n}: `{name}` is declared to span "
                       f"[{lo_can}, {hi_can}], but the comment above it states "
                       f"range [{lo_req}, {hi_req}] -- the declaration cannot "
                       "hold the range written next to it")

    # Wave 637c: `seen` used to hold TARGET NAMES and was consulted before the
    # check ran, so `assign l2[0]`, `l2[1]`, `l2[2]` collapsed to one -- 2 of
    # the 5 checkable reductions in the bundle were never examined, both inside
    # adder_tree_27, the module this gate was written for. Worse, len(seen) was
    # the coverage figure, so the summary counted distinct names and read as
    # full coverage. Every reduction is now checked; only the ERROR is deduped,
    # so one narrow declaration still reports once rather than three times.
    checked, reported, skipped = 0, set(), 0
    for m in REDUCE.finditer(src):
        target, expr = m.group(1), m.group(2)
        if target not in declared or "?" in expr:
            continue
        plus = top_level_plus(expr)
        if plus < 1:
            continue
        ops = OPERAND.findall(expr)
        known = [o for o in ops if o in rng]
        if len([o for o in ops if o in declared]) != plus + 1:
            continue
        if len(known) != plus + 1:
            # An operand with no documented range. Not checkable without the
            # unsound width fallback; counted so the loss is visible.
            skipped += 1
            continue
        ops = known
        checked += 1
        lo_req = sum(rng[o][0] for o in ops)
        hi_req = sum(rng[o][1] for o in ops)
        n = src[:m.start()].count("\n") + 1
        lo_can, hi_can = declared[target]
        if (lo_req < lo_can or hi_req > hi_can) and target not in reported:
            reported.add(target)
            bad.append(f"{path.name}:{n}: `{target}` is declared to span "
                       f"[{lo_can}, {hi_can}], but is assigned the sum of "
                       f"{len(ops)} operands whose ranges reach "
                       f"[{lo_req}, {hi_req}]")
        elif target in documented and target not in reported:
            lo_doc, hi_doc = documented[target]
            if lo_req < lo_doc or hi_req > hi_doc:
                reported.add(target)
                bad.append(f"{path.name}:{n}: `{target}` is documented as range "
                           f"[{lo_doc}, {hi_doc}], but is assigned the sum of "
                           f"{len(ops)} operands whose ranges reach "
                           f"[{lo_req}, {hi_req}] -- the comment understates "
                           "what the design puts there")
    return bad, len(declared), len(documented), checked, skipped


def scan(root):
    files = sorted((root / "build" / "rtl").glob("*.sv"))
    if not files:
        print(f"::error::width_scan found no RTL under {root}/build/rtl -- "
              "emit the bundle before running this gate")
        return 1
    bad, declared, annotated, reduced, skipped = [], 0, 0, 0, 0
    for f in files:
        b, d, a, r, k = check_file(f)
        bad += b
        declared += d
        annotated += a
        reduced += r
        skipped += k
    for b in bad:
        print(f"::error::{b}")
    # A scan that parsed nothing reports zero findings and reads as a pass.
    # Prop. 58's lesson turned on this instrument: refuse to report clean
    # unless something was actually read. The reduction count is here because
    # an earlier draft of this file skipped level 1 of the very tree it was
    # written for -- `val[i*3+1]` put a `+` inside an index, the operand count
    # disagreed with the term count, and the check silently declined. It still
    # printed a clean result.
    if annotated == 0 or declared == 0 or reduced == 0:
        print(f"::error::width_scan parsed {declared} signed declarations, "
              f"{annotated} of them range-annotated, and checked {reduced} "
              f"reductions across {len(files)} files -- it checked too little "
              "for its silence to mean anything. Either the emitters stopped "
              "writing these forms or the parser broke.")
        return 1
    print(f"width scan: {len(files)} emitted files, {declared} signed "
          f"declarations ({annotated} range-annotated), {reduced} reductions "
          f"checked, {skipped} uncheckable for want of an annotated operand, "
          f"{len(bad)} carrying less than the design puts in them")
    return 1 if bad else 0


def self_test():
    """Catch the Wave 628 defect every way it could have been written."""
    import tempfile
    root = pathlib.Path(__file__).resolve().parent.parent
    real = (root / "build" / "rtl" / "trit_stdlib.sv").read_text()

    narrow = real.replace("wire signed [4:0] l2 [0:2];",
                          "wire signed [3:0] l2 [0:2];")
    silent = narrow.replace("// Level 2: 3 groups of 3, range [-9, +9] -> "
                            "signed [4:0].", "// Level 2: 3 groups of 3.")
    drift = real.replace("// Level 2: 3 groups of 3, range [-9, +9] -> "
                         "signed [4:0].",
                         "// Level 2: 3 groups of 3, range [-8, +8].")
    cases = [
        ("the corrected tree, exactly as shipped", real, 0),
        ("the Wave 628 defect re-injected", narrow, 2),
        ("the same defect with the comment 'corrected' away", silent, 1),
        ("a wide-enough width whose comment understates it", drift, 1),
    ]

    bad = []
    # An injection that silently fails to apply grades the scan on unmodified
    # text and calls it a pass. Check that each case really moved.
    for name, text, _ in cases[1:]:
        if text == real:
            bad.append(f"{name}: the injection changed nothing")

    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td) / "build" / "rtl"
        d.mkdir(parents=True)
        for name, text, want in cases:
            f = d / "trit_stdlib.sv"
            f.write_text(text)
            got = check_file(f)[0]
            ok = len(got) == want
            print(f"  {'ok  ' if ok else 'FAIL'} {name}: {len(got)} finding(s), "
                  f"want {want}")
            for g in got:
                print(f"         {g.split(': ', 1)[1][:98]}")
            if not ok:
                bad.append(f"{name}: got {len(got)}, want {want}")

        # The trit encoding is the reason ranges propagate from comments. If
        # this ever reasons from declared widths again, level 1 turns into a
        # false positive on a correct design -- assert it does not.
        f.write_text(real)
        got = check_file(f)[0]
        if any("`l1`" in g for g in got):
            bad.append("level 1 flagged on the shipped tree -- the scan is "
                       "reasoning from declared widths, not documented ranges")

        # Wave 637c regressions. Each of these was a live defect found by an
        # adversarial audit, and each is kept as a case so it cannot return.
        f = d / "trit_stdlib.sv"

        # (1) Coverage: all three assignments to l2 must be checked, not one.
        f.write_text(real)
        checked = check_file(f)[3]
        ok = checked == 5
        print(f"  {'ok  ' if ok else 'FAIL'} every reduction is checked, not "
              f"one per target name: {checked} (want 5)")
        if not ok:
            bad.append(f"reduction coverage is {checked}, want 5 -- the "
                       "dedup-by-name defect is back")

        # (2) A same-line range comment must still annotate its declaration.
        same = real.replace(
            "    // Level 2: 3 groups of 3, range [-9, +9] -> signed [4:0].",
            "    // Level 2: three groups.").replace(
            "    wire signed [4:0] l2 [0:2];",
            "    wire signed [3:0] l2 [0:2]; // range [-9, +9]")
        if same == real:
            bad.append("the same-line-comment injection changed nothing")
        else:
            f.write_text(same)
            got = check_file(f)[0]
            hit = any("comment" in g for g in got)
            print(f"  {'ok  ' if hit else 'FAIL'} a TRAILING range comment "
                  f"still annotates its declaration: {len(got)} finding(s)")
            if not hit:
                bad.append("a same-line range comment was consumed as a "
                           "comment only, deleting the declaration from view")

        # (3) A missing annotation must make a reduction UNCHECKABLE, never
        # produce a finding via the unsound width fallback.
        drop = real.replace(
            "    // Decode each trit to signed {-1, 0, +1} (2-bit signed, range [-1, +1]).",
            "    // Decode each trit to signed {-1, 0, +1}.")
        if drop == real:
            bad.append("the annotation-removal injection changed nothing")
        else:
            f.write_text(drop)
            got, _, _, _, skip = check_file(f)
            ok = len(got) == 0 and skip > 0
            print(f"  {'ok  ' if ok else 'FAIL'} a missing annotation is "
                  f"uncheckable, not a false finding: {len(got)} finding(s), "
                  f"{skip} skipped")
            if not ok:
                bad.append("removing one annotation produced a finding against "
                           "correct RTL via the width fallback")

        (d / "trit_stdlib.sv").write_text("module m; endmodule\n")
        rc = scan(pathlib.Path(td))
        print(f"  {'ok  ' if rc else 'FAIL'} a tree with nothing to check fails "
              f"rather than passing silently: exit {rc}")
        if rc == 0:
            bad.append("the zero-parse guard let an unreadable tree pass")

    for b in bad:
        print(f"::error::width_scan self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else scan(r))
