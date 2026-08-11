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


def positive_arm(root, steps, only=None):
    """Run each step with the tree INTACT. A step that fails here is broken.

    Wave 648. The sweep is a negative control: it establishes that a step fails
    when starved. That licenses nothing on its own, because a step that is
    ALREADY BROKEN also fails when starved -- and the sweep records it as
    "fails, correct". Two live instances were confirmed this way:

      * "Prove zero-size properties" carried a stray third element in a tuple
        list the loop unpacked as two, so it raised ValueError after the first
        two suites. Four of the eight zero-size properties were never proved.
      * "Baseline, control, and mutation" named the engine emitter's
        pre-2026-08-09 text as a mutation target. The emitter now writes the
        declaration and the assignment on separate lines, so the target
        appeared zero times, the mutation was never applied, and the suite
        silently tested 7 of 8 mutants.

    Both exited non-zero in normal operation and both were invisible to the
    sweep, which only ever asked the starved question. This runs the other arm.

    Opt-in via --positive: it executes the real proofs and takes as long as CI
    does, where the sweep itself is minutes.
    """
    bad = []
    print(f"\n{'step':58s} {'exit':>4s}   verdict")
    print("-" * 92)
    for name, script in steps:
        if only and only not in name:
            continue
        r = subprocess.run(["bash", "-c", script], cwd=root,
                           capture_output=True, text=True)
        ok = r.returncode == 0
        print(f"{name[:58]:58s} {r.returncode:>4d}   "
              f"{'ok' if ok else 'BROKEN IN NORMAL OPERATION'}")
        if not ok:
            tail = (r.stderr or r.stdout or "").strip().splitlines()[-1:]
            bad.append((name, r.returncode, tail[0][:120] if tail else ""))
    for name, rc, msg in bad:
        print(f"::error::step '{name}' exits {rc} with the tree INTACT -- it is "
              f"broken in normal operation, and the starved sweep reads that "
              f"same failure as 'fails, correct'. Last line: {msg}")
    print(f"\npositive arm: {len(bad)} step(s) broken in normal operation")
    return 1 if bad else 0


def main(argv):
    root = pathlib.Path(__file__).resolve().parent.parent
    positive = "--positive" in argv
    argv = [a for a in argv if a != "--positive"]
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

    if positive:
        return positive_arm(root, steps)

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
    diagnosed_steps, indeterminate = [], []
    try:
        print(f"{'step':58s} {'exit':>4s}   verdict")
        print("-" * 92)
        for name, run in steps:
            script = bak / "step.sh"
            open(script, "w").write(run)
            out = ""
            try:
                r = subprocess.run(["bash", str(script)], cwd=root,
                                   capture_output=True, text=True, timeout=1800)
                rc = r.returncode
                out = (r.stdout or "") + (r.stderr or "")
            except subprocess.TimeoutExpired:
                rc = -1
            exempt = name in EXEMPT
            # Count an exemption only when it actually SUPPRESSED a green
            # verdict. Wave 649: this appended on name membership alone, so a
            # step in EXEMPT that FAILED was still reported as exempt, and the
            # summary read identically whether the exemption did any work.
            if exempt and rc == 0:
                applied.append(name)
            bad = rc == 0 and not exempt
            if bad:
                green.append(name)

            # A THIRD verdict. Wave 649: every non-zero exit was read as
            # "fails, correct" -- so a missing binary (rc 127), an unrelated
            # crash, and a hang all counted as a healthy gate. Classifying the
            # captured output of the real swept set gave 11 steps that produced
            # a designed diagnosis of the absence, 10 that died with a raw
            # traceback, and 19 that exited non-zero saying nothing at all.
            # All forty printed the same words, and the summary said
            # "0 passing on nothing" either way.
            #
            # DIAGNOSED requires positive evidence that the step noticed its
            # subject was gone. Anything else that merely failed is
            # INDETERMINATE: not a pass, but not evidence of a working gate.
            # DIAGNOSED means the failure output NAMES THE STARVED SUBJECT.
            #
            # Wave 650 corrects Wave 649's classifier, which looked only for
            # this repository's own `::error::` convention and reported 9
            # diagnosed against 28 indeterminate. Re-reading the captured
            # output showed all 28 name the exact missing file, in the tool's
            # own words -- `ERROR: File 'build/rtl/x.sv' not found` from yosys,
            # `FileNotFoundError: ... 'formal/x.sv'` from Python. Those ARE
            # diagnoses: a step broken for some other reason does not say that.
            # The Wave 649 figure was over-detection in the measurement, which
            # is the fourth consecutive wave a new check has fired on correct
            # behaviour (Prop. 115).
            #
            # The distinction that matters is Prop. 114's: can this failure be
            # told apart from "the step was already broken"? A message naming a
            # starved path can. `ValueError: too many values to unpack`, rc 127
            # and a timeout cannot -- and that ValueError is precisely how one
            # of Prop. 114's two defects hid.
            low = out.lower()
            names_subject = ("not found" in low or "filenotfounderror" in low
                             or "no such file" in low
                             or "found no" in low or "emit the bundle" in low)
            starved = "build/rtl" in low or "formal/" in low
            diagnosed = "::error::" in out or (names_subject and starved)
            if not (bad or (exempt and rc == 0)):
                (diagnosed_steps if diagnosed else indeterminate).append(name)
            verdict = ("exempt" if exempt and rc == 0 else
                       "PASSES ON NOTHING" if bad else
                       "diagnosed" if diagnosed else "INDETERMINATE")
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

    # A WALL, now that the corrected classifier puts the true count at zero.
    # Wave 649 set this to 28 on a measurement that was itself over-detecting;
    # every one of those steps names its starved subject in the tool's own
    # words. With the real figure at 0 a ratchet would be pointless: any step
    # that fails without naming what it was missing is indistinguishable from
    # one that was already broken, which is how Prop. 114's two defects hid.
    INDETERMINATE_CEILING = 0
    if len(indeterminate) > INDETERMINATE_CEILING:
        print(f"::error::{len(indeterminate)} steps merely CRASH when starved "
              f"rather than diagnosing the absence, above the ceiling of "
              f"{INDETERMINATE_CEILING}. A step that dies with a bare traceback "
              "proves nothing about whether it reads its subject: it fails "
              "when starved and would fail just as readily if it were simply "
              "broken. Lower the ceiling as steps gain real diagnostics; never "
              "raise it. Prop. 116.")
        return 1

    for n in green:
        print(f"::error::step '{n}' exits 0 with no RTL and no properties "
              "present -- it is not measuring the design")
    # `applied`, not `len(EXEMPT)`: report the exemptions that were actually
    # used, not the size of the list. Reporting the list size says "1 exempt"
    # on a run where nothing was exempted -- a small lie of exactly the kind
    # this file exists to find.
    print(f"\nabsence sweep: {len(steps)} steps across {len(paths)} workflows, "
          f"{len(builders)} builders not swept, "
          f"{len(diagnosed_steps)} diagnosed, {len(indeterminate)} "
          f"indeterminate, {len(applied)} exempt, {len(recursive)} recursive, "
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
        # Wave 650: the synthetic step used to be a bare `test -f`, which fails
        # SILENTLY. Every real step in this repository names the file it could
        # not find -- yosys prints `ERROR: File '...' not found`, Python raises
        # FileNotFoundError with the path -- and the ceiling is now 0 because
        # that is the measured truth. A fixture that fails without saying why
        # was modelling something the tree does not contain.
        ("a step that reads the subject",
         [{"name": "Honest",
           "run": "test -f build/rtl/dma_controller.sv || "
                  "{ echo \"ERROR: File 'build/rtl/dma_controller.sv' not found\"; "
                  "exit 1; }"}], 0),
        # And its counterpart: a step that fails for some OTHER reason must be
        # INDETERMINATE, not credited as a working gate. This is Prop. 114a's
        # shape -- the stray-tuple ValueError that let a broken step hide.
        ("a step that fails without naming its subject",
         [{"name": "Opaque", "run": "python3 -c \"raise ValueError('boom')\""}], 1),
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
