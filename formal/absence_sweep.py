#!/usr/bin/env python3
"""Take the subject away and run every checking step. Anything still green is
not measuring what its name claims.

Wave 608. Waves 592-607 found nine defects in the RTL and six in the harnesses.
Every one of the six harness defects had the same shape -- an absence read as a
pass: output truncated before the verdict line, a crash instead of a decision,
a glob matching no files, a gate never wired into CI, a `grep` on a file that
did not exist, a parse check on an emission with nothing in it.

Five of those six were found by looking. This is the version that does not
depend on looking: empty `build/rtl/` and `formal/`, then run each step of
formal-yosys.yml verbatim. A step that reports success with no design and no
properties present is measuring something other than the design.

The steps are cheap in this configuration -- with no input, yosys fails
immediately -- so the sweep costs far less than the suite it audits.

Usage:  python3 formal/absence_sweep.py [workflow.yml]
"""

import os
import pathlib
import shutil
import subprocess
import sys

import yaml

# Steps whose job is to CREATE the subject rather than check it.
BUILDERS = {"Install Yosys", "Build t27c", "Emit the BitNet RTL bundle"}

# Steps that legitimately pass without build/rtl or formal/, with the reason
# each one is exempt. An exemption has to be argued, not assumed: this list is
# the only way a step escapes the sweep, and a wrong entry here is how the
# sweep would come to pass while checking less than it claims.
EXEMPT = {
    "Behavior-DSL subset still emits and parses":
        "writes its own behaviours.json and emits the SystemVerilog it checks, "
        "so it depends on neither directory. Its own absence case -- an "
        "emission containing no assertions -- is covered by the assertion "
        "count inside that step (Prop. 59).",
    # Wave 643. Both surfaced the moment the sweep stopped deleting the gate
    # scripts along with the subject: they had been "failing correctly" because
    # their own script was gone, which established nothing. Their subject is
    # not the RTL, so starving build/rtl cannot make them fail -- and demanding
    # that it does would be shape 7, failing a correct artifact.
    "Benchmark harness self-test":
        "bench.py --self-test exercises its own guards with synthetic commands "
        "and has no RTL subject at all. Its absence case is internal: six "
        "self-test cases, each of which must fire (Prop. 87).",
    "No gate mutates a path its docstring never names":
        "faith_check's subject is formal/*.py -- the gate scripts themselves -- "
        "which this sweep now deliberately PRESERVES (Prop. 109). Starving "
        "build/rtl cannot make it fail, and making it fail would require "
        "deleting the very instruments the sweep was fixed to keep. Its own "
        "absence case is the FLOOR on resolved mutated paths (Prop. 111).",
    "Every proposition carries the gate that keeps it true":
        "doc_gate reads FORMAL_FOUNDATIONS.md and the workflow step names, not "
        "the design. Emptying build/rtl leaves its subject untouched. Its own "
        "absence case is the props==0 guard and the Gate:-resolution count "
        "added in Prop. 107.",
}


DEFAULT_WORKFLOWS = [".github/workflows/formal-yosys.yml",
                     ".github/workflows/formal-mutation.yml"]


def collect(root, wf_path):
    """Checking steps of every job in one workflow, minus builders and self.

    Wave 608 swept only formal-yosys.yml and said so (Prop. 59f): this file runs
    as a step of formal-mutation.yml, so sweeping that workflow would invoke
    this sweep from inside itself. The exclusion is by CONTENT -- any step whose
    script invokes absence_sweep.py -- rather than by step name, so renaming the
    step cannot silently reintroduce the recursion. It is reported, not
    swallowed, and the sweep's own absence case is covered by the empty-steps
    check below and by its self-test.
    """
    wf = yaml.safe_load(open(wf_path))
    steps, skipped, builders = [], [], []
    for job in wf["jobs"].values():
        for s in job.get("steps", []):
            if "run" not in s:
                continue
            name = s.get("name", "<unnamed>")
            if name in BUILDERS:
                # Counted. Wave 639: builder exclusions were invisible, so a
                # CHECKING step that happened to be named like a builder would
                # have vanished from the sweep with nothing in the summary to
                # say so -- the same shape as Prop. 100's silent declines.
                builders.append(name)
                continue
            if "absence_sweep" in s["run"]:
                skipped.append(name)
                continue
            steps.append((name, s["run"]))
    return steps, skipped, builders


def main(argv):
    root = pathlib.Path(__file__).resolve().parent.parent
    paths = argv[1:] or [str(root / w) for w in DEFAULT_WORKFLOWS]

    steps, recursive, builders = [], [], []
    for p in paths:
        s, sk, bl = collect(root, p)
        steps += s
        recursive += sk
        builders += bl
        print(f"{pathlib.Path(p).name}: {len(s)} checking steps"
              f"{f', {len(sk)} recursive (skipped: {sk})' if sk else ''}"
              f"{f', {len(bl)} builders not swept' if bl else ''}")

    if not steps:
        print(f"::error::absence_sweep found no checking steps in {paths}")
        return 1

    bak = root / "build" / "_absence_bak"
    shutil.rmtree(bak, ignore_errors=True)
    os.makedirs(bak)
    # Starve the SUBJECTS, not the instruments. Wave 643: this moved the whole
    # of `formal/` aside -- including all ten gate SCRIPTS. Every python step
    # then failed with "No such file or directory: formal/<gate>.py", and the
    # sweep recorded "fails, correct". For roughly a quarter of the swept steps
    # the only thing established was that deleting a script breaks the step
    # that runs it, which is circular and proves nothing about whether the gate
    # reads its subject.
    #
    # `build/rtl` goes entirely (it is all subject). From `formal/` only the
    # property files and RTL go; the *.py gates stay, so a step that passes now
    # passes while genuinely starved.
    moved = []
    src = root / "build" / "rtl"
    if src.exists():
        dst = bak / "build_rtl"
        shutil.move(str(src), str(dst))
        os.makedirs(src, exist_ok=True)
        moved.append((str(dst), str(src)))

    fbak = bak / "formal_subjects"
    os.makedirs(fbak, exist_ok=True)
    for f in sorted((root / "formal").glob("*")):
        if f.is_file() and f.suffix != ".py":
            shutil.move(str(f), str(fbak / f.name))
            moved.append((str(fbak / f.name), str(f)))

    green, applied = [], []
    try:
        print(f"{'step':58s} {'exit':>4s}   verdict")
        print("-" * 92)
        for name, run in steps:
            script = bak / "step.sh"
            open(script, "w").write(run)
            try:
                r = subprocess.run(["bash", str(script)], cwd=root,
                                   capture_output=True, text=True, timeout=1800)
                rc = r.returncode
            except subprocess.TimeoutExpired:
                rc = -1
            exempt = name in EXEMPT
            if exempt:
                applied.append(name)
            bad = rc == 0 and not exempt
            if bad:
                green.append(name)
            verdict = ("exempt" if exempt and rc == 0 else
                       "PASSES ON NOTHING" if bad else "fails, correct")
            print(f"{name[:58]:58s} {rc:>4d}   {verdict}")
    finally:
        for dst, src in moved:
            d, sp = pathlib.Path(dst), pathlib.Path(src)
            if d.is_dir():
                shutil.rmtree(src, ignore_errors=True)
            elif sp.exists():
                sp.unlink()
            shutil.move(dst, src)
        shutil.rmtree(bak, ignore_errors=True)

    for n in green:
        print(f"::error::step '{n}' exits 0 with no RTL and no properties "
              "present -- it is not measuring the design")
    # `applied`, not `len(EXEMPT)`: report the exemptions that were actually
    # used, not the size of the list. Reporting the list size says "1 exempt"
    # on a run where nothing was exempted -- a small lie of exactly the kind
    # this file exists to find.
    print(f"\nabsence sweep: {len(steps)} steps across {len(paths)} workflows, "
          f"{len(builders)} builders not swept, "
          f"{len(applied)} exempt, {len(recursive)} recursive, "
          f"{len(green)} passing on nothing")
    return 1 if green else 0


def self_test():
    """Four synthetic workflows whose right answers are known.

    Self-exclusion (`collect` drops any step invoking this file) is what lets
    the sweep run inside the workflow it audits -- and it is also a new way for
    the sweep to check nothing and report success. The fourth case is that one:
    a workflow whose only step is the sweep must FAIL, not quietly pass having
    examined zero steps.
    """
    import tempfile
    cases = [
        ("a step that passes on nothing",
         [{"name": "Decorative", "run": "echo 'looks fine to me'"}], 1),
        ("a step that reads the subject",
         [{"name": "Honest", "run": "test -f build/rtl/dma_controller.sv"}], 0),
        ("a workflow with no run steps",
         [{"name": "Checkout", "uses": "actions/checkout@v4"}], 1),
        ("a workflow whose only step is this sweep",
         [{"name": "Sweep", "run": "python3 formal/absence_sweep.py"}], 1),
    ]
    bad = []
    for name, steps, want in cases:
        with tempfile.NamedTemporaryFile("w", suffix=".yml", delete=False) as fh:
            yaml.safe_dump({"on": "push", "jobs": {"j": {"steps": steps}}}, fh)
        got = main(["absence_sweep", fh.name])
        print(f"  {'ok  ' if got == want else 'FAIL'} {name}  (exit {got}, want {want})")
        if got != want:
            bad.append(name)
    for b in bad:
        print(f"::error::absence_sweep self-test: '{b}' gave the wrong answer")
    return 1 if bad else 0


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(self_test())
    sys.exit(main(sys.argv))
