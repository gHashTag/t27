#!/usr/bin/env python3
"""Gate 26: a gate that reports only its numerator cannot be read as coverage.

Prop. 193 proved that a scan matching an observed shape covers exactly the
members containing that shape, and that its residue is undetectable by
construction. This gate is that theorem turned on the campaign's own instruments,
and the first measurement is worse than the theorem predicts -- because two gates
have been printing their residue in their summary line all along, in green, and
nobody read it as coverage:

    units scan: 13 files, 41 connections compared,
                122 SKIPPED AS UNRECOGNISED, 0 new disagreements
    width scan: 16 signed declarations (3 RANGE-ANNOTATED),
                5 reductions checked, 0 uncheckable

`units_scan` infers a quantity from a NAME, against a hand-written `FAMILIES`
table. A port whose name is not in that table is not compared -- so the gate is
blind to **75%** of what it looks at. `width_scan` can only check a declaration
that carries a range annotation: **3 of 16**. Both exit 0. Both have exited 0
every wave since they landed.

Neither is broken. Each answers its question correctly. The defect is that the
green was read as "no width defects" when it means "no width defects among the
19% of declarations that told us their range".

WHAT THIS GATE REQUIRES. Every checking script in `formal/` must carry a
`COVERAGE.` paragraph in its module docstring, stating what it examined, what it
could not examine, and why. That is a documentation requirement, deliberately --
no scanner can compute another scanner's true denominator (that is the halting
problem wearing a lab coat), but a gate whose author cannot state the denominator
in one paragraph has not established coverage of anything.

It RATCHETS. Landing red on 29 gates would get it disabled rather than obeyed
(Prop. 26); it fails when a gate without a `COVERAGE.` paragraph is ADDED, and
when a gate that had one loses it.

COVERAGE. Examines every `formal/*.py` whose module docstring exists: 29 of 29
files. It checks for the PRESENCE of the marker, never the truth of the paragraph
beneath it -- a gate can satisfy this with a false denominator, and this gate
cannot tell. Helper modules that perform no checking (`bench`, `mutate`,
`trace_reader`, `scale_probe`) are exempted BY NAME below, with the reason.

ARTIFACTS. Reads `formal/*.py` and `.github/workflows/*.yml` (Prop. 200: to
resolve which scripts CI actually runs). WRITES `formal/coverage_baseline.txt`,
and only when `--init` is passed. Nothing else.

Prop. 194.
"""
import ast
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
FORMAL = ROOT / "formal"

# Prop. 198: this set was written from a general justification -- "not checking
# scripts, so coverage is not a question they answer" -- and then coded as four
# literal filenames. Re-deriving the property it claims ("never returns a failure
# status and emits no ::error::") and counting the class refutes three of the four:
#
#   bench.py       ::error::arm '{label}' exited nonzero -- it CAN fail the build
#   mutate.py      ::error::mutate self-test found no RTL -- so can this
#   scale_probe.py returns 1 on a failure path
#   trace_reader.py  the only one that matches the stated property
#
# The exemption was not too narrow (Prop. 197's shape) but simply WRONG: three
# scripts that gate CI were excused from stating a denominator by a sentence that
# was not true of them. Narrowed to the one entry the property holds for; the
# other three fall into the ratcheted baseline, where being undeclared is
# recorded rather than hidden behind a false reason.
EXEMPT = {
    "trace_reader.py": "parses a counterexample another gate produced; it has no "
                       "failure path and emits no ::error::",
}


def main():
    if not FORMAL.exists():
        print("::error::coverage gate: no such directory 'formal' under the "
              "repository root -- nothing was scanned")
        return 1
    files = sorted(p for p in FORMAL.glob("*.py"))
    if not files:
        print("::error::coverage gate: found no .py files under formal/ -- "
              "nothing was scanned")
        return 1

    # Prop. 200: ALIVE bar. Copied alone into an empty tree, this gate found one
    # script (itself), declared it compliant and exited 0 -- the absence sweep
    # caught it. "The corpus is present" is not "the corpus is non-empty": a
    # directory holding only the scanner satisfies every count it makes. Resolve
    # what the workflows actually run and require those to be here, rather than
    # trusting the glob (the same rule as Prop. 162/165: coverage is an
    # invocation's OUTPUT, not the flag that requested it).
    wf_dir = ROOT / ".github" / "workflows"
    cited = set()
    if wf_dir.exists():
        for y in wf_dir.glob("*.yml"):
            cited.update(re.findall(r"python3 formal/(\w+\.py)",
                                    y.read_text(errors="ignore")))
    if not cited:
        print("::error::coverage gate: found no `python3 formal/*.py` step in "
              "any workflow -- there is nothing to check the scanned corpus "
              "against, so this gate can establish nothing. A decline must exit "
              "nonzero naming what is missing, not pass quietly (Prop. 103)")
        return 1

    present = {p.name for p in files}
    absent = sorted(cited - present)
    if absent:
        print(f"::error::coverage gate: {len(absent)} script(s) the workflows "
              f"run are not present under formal/ -- the corpus this gate "
              f"measured is not the corpus CI executes, so its verdict is about "
              f"the wrong set (Prop. 200)")
        for a in absent[:8]:
            print(f"  formal/{a}")
        return 1

    missing, ok, exempt = [], 0, 0
    for p in files:
        if p.name in EXEMPT:
            exempt += 1
            continue
        try:
            doc = ast.get_docstring(ast.parse(p.read_text())) or ""
        except SyntaxError:
            print(f"::error::coverage gate: {p.name} does not parse -- a gate "
                  f"that cannot be imported checks nothing")
            return 1
        if "COVERAGE." in doc:
            ok += 1
        else:
            missing.append(p.name)

    print(f"coverage gate: {len(files)} scripts, {ok} declare a denominator, "
          f"{len(missing)} do not, {exempt} exempt as non-checking")

    baseline = FORMAL / "coverage_baseline.txt"
    now = sorted(missing)
    if not baseline.exists():
        # Prop. 211c: writing a baseline is an explicit act, never a fallback.
        # `if not exists(): write(now); return 0` resets the ratchet on one
        # `rm`, and on a clone that never had the file it rubber-stamps the tree
        # it was handed and exits 0. Measured before f66561f33: 8 of the 13
        # baselines in this suite were on disk and in no commit, and 8 of the 13
        # gates owning them re-baseline a possibly-broken tree and pass.
        if "--init" not in sys.argv[1:]:
            print(f"::error::coverage gate: {baseline.name} does not exist and "
                  f"--init was not given. Writing one here would record "
                  f"whatever this tree contains as the accepted state -- on a "
                  f"fresh clone that is a green run which checked nothing. "
                  f"Genuine first run: `python3 formal/coverage_gate.py --init`. "
                  f"Otherwise the baseline was lost and belongs in the commit "
                  f"that lost it (Prop. 211)")
            return 1
        baseline.write_text("\n".join(now) + ("\n" if now else ""))
        print(f"coverage gate: baseline written to {baseline.name} "
              f"({len(now)} without a denominator)")
        return 0

    was = [l for l in baseline.read_text().splitlines() if l.strip()]
    new = [m for m in now if m not in was]
    if new:
        print(f"::error::coverage gate: {len(new)} checking script(s) do not "
              f"state a COVERAGE. denominator. A gate reporting only its "
              f"numerator cannot be read as coverage -- `units_scan` skips 122 "
              f"of 163 connections and exits 0, and has done so every wave "
              f"since it landed (Prop. 194)")
        for m in new:
            print(f"  formal/{m}")
        return 1
    fixed = [w for w in was if w not in now]
    if fixed:
        print(f"coverage gate: {len(fixed)} script(s) now state a denominator; "
              f"update {baseline.name} to lock it in")
    print(f"coverage gate: ratchet holds ({len(now)} <= {len(was)} silent)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::coverage gate: could not scan formal/ "
              f"({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)
