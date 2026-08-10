#!/usr/bin/env python3
"""Time a command and record the machine state that timing is a claim about.

Wave 634. Prop. 85f corrected a published figure: two properties were reported
as costing an engine proof 4x, from measurements taken while three other provers
were competing for the machine. The real figure was 1.58x. Nothing malfunctioned
-- the stopwatch was accurate, and it measured a machine I had not described.

Correctness results here are reproducible: a proof discharges or it does not,
and the answer does not depend on what else was running. Timings are not. They
are claims about contention, core count and thermal state, and this campaign has
twenty waves of gates checking whether its tools lie while its performance
numbers were being read off a wall clock with no provenance at all.

Three rules, enforced rather than remembered:

  PAIRED     a before/after is run in one invocation, alternating, so both
             arms see the same machine. Comparing a number measured today
             against one recorded eight waves ago is not a comparison.
  WITNESSED  load average and competing prover count are sampled before and
             after each run and printed beside the seconds. A timing without
             them is not reportable.
  REPEATED   each arm runs N times and the spread is reported. A single sample
             cannot distinguish a real regression from a noisy machine, which
             is exactly the mistake Prop. 85f records.

It refuses to print a comparison when the machine was contended, rather than
printing one with a caveat -- a caveat is something a reader can skip.

Usage:
  python3 formal/bench.py --label A "<cmd>" --label B "<cmd>" [--repeat 3]
  python3 formal/bench.py --self-test
"""

import argparse
import glob as globlib
import hashlib
import os
import pathlib
import re
import statistics
import subprocess
import sys
import time


def fingerprint(patterns, cwd):
    """Digest of every file under test, so a mid-run edit is detectable.

    Wave 634, found the hard way twice in one session. The first version of this
    harness witnessed the MACHINE -- load, competing provers -- and reported a
    clean 0.88x for a change that should have been slower. The inputs had been
    regenerated a third of the way through the run, so the early and late
    samples were measuring different RTL. A benchmark whose inputs move mid-run
    is exactly as broken as one whose machine is contended, and neither is
    visible in the seconds.
    """
    h = hashlib.sha256()
    files = []
    for pat in patterns:
        files += sorted(globlib.glob(os.path.join(cwd, pat)))
    for f in files:
        try:
            h.update(pathlib.Path(f).read_bytes())
        except OSError:
            h.update(b"<missing>")
        h.update(f.encode())
    return h.hexdigest()[:16], len(files)


def provers():
    """Competing yosys/z3 processes, excluding our own children."""
    try:
        out = subprocess.run(["ps", "-Ao", "comm"], capture_output=True,
                             text=True, timeout=10).stdout
    except Exception:
        return -1
    return sum(1 for l in out.splitlines()
               if re.search(r"(^|/)(yosys|z3|boolector|cvc5)$", l.strip()))


def loadavg():
    try:
        return os.getloadavg()[0]
    except Exception:
        return -1.0


def run_once(cmd, cwd):
    before_p, before_l = provers(), loadavg()
    t = time.monotonic()
    r = subprocess.run(cmd, shell=True, cwd=cwd, capture_output=True, text=True)
    secs = time.monotonic() - t
    after_p, after_l = provers(), loadavg()
    return {
        "secs": secs,
        "rc": r.returncode,
        "provers": max(before_p, after_p),
        "load": max(before_l, after_l),
        "tail": (r.stderr or r.stdout or "").strip().splitlines()[-1:],
    }


def bench(arms, repeat, cwd, quiet_provers=1, quiet_load=None,
          watch=None):
    """Run every arm `repeat` times, alternating, on one machine state."""
    cpus = os.cpu_count() or 1
    if quiet_load is None:
        quiet_load = cpus * 0.75
    fp_before = fingerprint(watch, cwd) if watch else None
    runs = {label: [] for label, _ in arms}
    for _ in range(repeat):
        for label, cmd in arms:            # alternate, do not batch by arm
            runs[label].append(run_once(cmd, cwd))
    fp_after = fingerprint(watch, cwd) if watch else None

    print(f"{'arm':10s} {'runs':>4s} {'median s':>9s} {'min':>8s} {'max':>8s} "
          f"{'rc':>3s} {'provers':>8s} {'load':>7s}")
    print("-" * 64)
    contended, failed = [], []
    for label, _ in arms:
        rs = runs[label]
        secs = [r["secs"] for r in rs]
        # A prover of our own is expected: the command under test. More than
        # that is somebody else's work landing in our measurement.
        peak_p = max(r["provers"] for r in rs)
        peak_l = max(r["load"] for r in rs)
        rcs = {r["rc"] for r in rs}
        print(f"{label:10s} {len(rs):>4d} {statistics.median(secs):>9.1f} "
              f"{min(secs):>8.1f} {max(secs):>8.1f} "
              f"{','.join(str(c) for c in sorted(rcs)):>3s} "
              f"{peak_p:>8d} {peak_l:>7.1f}")
        if peak_p > quiet_provers or peak_l > quiet_load:
            contended.append(label)
        if rcs != {0}:
            failed.append(label)

    if watch:
        print(f"inputs: {fp_before[1]} files, {fp_before[0]} -> {fp_after[0]}")
    for label in failed:
        print(f"::error::arm '{label}' exited nonzero -- a timing for a command "
              "that failed is not a measurement of anything")
    if watch and fp_before != fp_after:
        print(f"::error::the files under test CHANGED during the run "
              f"({fp_before[0]} -> {fp_after[0]}). Early and late samples "
              "measured different inputs, so no comparison is printed. This "
              "guard exists because the first run of this harness reported a "
              "clean 0.88x for a change that was slower -- the RTL had been "
              "regenerated a third of the way through.")
        return 1, runs
    if contended:
        print(f"::error::the machine was contended during {contended} "
              f"(>{quiet_provers} prover or load >{quiet_load:.1f} on "
              f"{cpus} cores). No comparison is printed: Prop. 85f was a 4x "
              "figure that was really 1.58x, measured exactly this way.")
        return 1, runs
    if len(arms) == 2 and not failed:
        (la, _), (lb, _) = arms
        sa = [r["secs"] for r in runs[la]]
        sb = [r["secs"] for r in runs[lb]]
        a, b = statistics.median(sa), statistics.median(sb)
        ratio = b / a if a else float("inf")
        # Disjointness, not "bigger than one arm's spread". If the observed
        # ranges overlap at all, some run of the slower arm beat some run of
        # the faster one, and no ordering between them is supportable. This is
        # deliberately conservative: it is the criterion that would have
        # refused to print Prop. 85f's 4x.
        overlap = min(max(sa), max(sb)) >= max(min(sa), min(sb))
        print(f"\n{lb} / {la} = {ratio:.2f}x ({b - a:+.1f} s)   "
              f"{la} range [{min(sa):.1f}, {max(sa):.1f}]   "
              f"{lb} range [{min(sb):.1f}, {max(sb):.1f}]")
        if overlap:
            print("::error::the two arms' observed ranges OVERLAP, so some run "
                  "of the slower arm beat some run of the faster one. No "
                  "ordering between them is supportable; report no ratio.")
            return 1, runs
    return (1 if failed else 0), runs


def self_test():
    """The guards must fire on the conditions they exist for.

    Every case except the contention one runs with the contention thresholds
    disabled. A self-test whose verdict depends on how loaded the machine
    happens to be is not testing its own logic -- and this one failed exactly
    that way in Wave 636b, rejecting two clean runs because the campaign's own
    proof runs had pushed the load average past the default threshold. The
    contention case sets its own threshold explicitly so it still fires.
    """
    bad = []
    cwd = str(pathlib.Path(__file__).resolve().parent.parent)

    QUIET = dict(quiet_provers=10**6, quiet_load=10**6)
    rc, _ = bench([("quick", "python3 -c 'pass'")], 2, cwd, **QUIET)
    print(f"  {'ok  ' if rc == 0 else 'FAIL'} a clean run reports (exit {rc})")
    if rc != 0:
        bad.append("a clean quiet run was rejected")

    rc, _ = bench([("boom", "exit 3")], 2, cwd, **QUIET)
    print(f"  {'ok  ' if rc else 'FAIL'} a failing command is not timed "
          f"(exit {rc})")
    if rc == 0:
        bad.append("a nonzero-exit command was reported as a timing")

    # Two arms that are really the same command: any ratio found is noise, and
    # the harness must refuse to report it rather than dress it up.
    rc, _ = bench([("a", "python3 -c 'import time;time.sleep(0.05)'"),
                   ("b", "python3 -c 'import time;time.sleep(0.05)'")], 3, cwd,
                  **QUIET)
    print(f"  {'ok  ' if rc else 'FAIL'} identical arms yield no ratio "
          f"(exit {rc})")
    if rc == 0:
        bad.append("a difference smaller than the noise was reported anyway")

    rc, _ = bench([("x", "python3 -c 'pass'")], 1, cwd, quiet_provers=-1)
    print(f"  {'ok  ' if rc else 'FAIL'} contention blocks the report "
          f"(exit {rc})")
    if rc == 0:
        bad.append("a contended measurement was reported")

    # Inputs that move under the benchmark's feet.
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        t = pathlib.Path(td) / "subject.txt"
        t.write_text("before")
        rc, _ = bench([("edits", f"python3 -c \"open(r'{t}','w').write('after')\"")],
                      2, td, watch=["*.txt"], **QUIET)
        print(f"  {'ok  ' if rc else 'FAIL'} an input edited mid-run blocks the "
              f"report (exit {rc})")
        if rc == 0:
            bad.append("a run whose inputs changed underneath it was reported")

        t.write_text("stable")
        rc, _ = bench([("stable", "python3 -c 'pass'")], 2, td,
                      watch=["*.txt"], **QUIET)
        print(f"  {'ok  ' if rc == 0 else 'FAIL'} stable inputs still report "
              f"(exit {rc})")
        if rc != 0:
            bad.append("the fingerprint guard rejected an unchanged tree")

    for b in bad:
        print(f"::error::bench self-test: {b}")
    return 1 if bad else 0


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(self_test())
    ap = argparse.ArgumentParser()
    ap.add_argument("--label", action="append", nargs=2, metavar=("NAME", "CMD"),
                    required=True)
    ap.add_argument("--repeat", type=int, default=3)
    ap.add_argument("--watch", action="append", default=None,
                    help="glob of files under test; the run is rejected if they "
                         "change mid-benchmark")
    a = ap.parse_args()
    root = str(pathlib.Path(__file__).resolve().parent.parent)
    sys.exit(bench([(n, c) for n, c in a.label], a.repeat, root,
                   watch=a.watch)[0])
