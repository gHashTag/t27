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
}


def main(argv):
    root = pathlib.Path(__file__).resolve().parent.parent
    wf_path = argv[1] if len(argv) > 1 else str(root / ".github/workflows/formal-yosys.yml")
    wf = yaml.safe_load(open(wf_path))
    job = wf["jobs"][list(wf["jobs"])[0]]
    steps = [(s["name"], s["run"]) for s in job["steps"]
             if "run" in s and s.get("name") not in BUILDERS]

    if not steps:
        print(f"::error::absence_sweep found no checking steps in {wf_path}")
        return 1

    bak = root / "build" / "_absence_bak"
    shutil.rmtree(bak, ignore_errors=True)
    os.makedirs(bak)
    moved = []
    for d in ["build/rtl", "formal"]:
        src = root / d
        if src.exists():
            dst = bak / d.replace("/", "_")
            shutil.move(str(src), str(dst))
            os.makedirs(src, exist_ok=True)
            moved.append((str(dst), str(src)))

    green = []
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
            bad = rc == 0 and not exempt
            if bad:
                green.append(name)
            verdict = ("exempt" if exempt and rc == 0 else
                       "PASSES ON NOTHING" if bad else "fails, correct")
            print(f"{name[:58]:58s} {rc:>4d}   {verdict}")
    finally:
        for dst, src in moved:
            shutil.rmtree(src, ignore_errors=True)
            shutil.move(dst, src)
        shutil.rmtree(bak, ignore_errors=True)

    for n in green:
        print(f"::error::step '{n}' exits 0 with no RTL and no properties "
              "present -- it is not measuring the design")
    print(f"\nabsence sweep: {len(steps)} steps, {len(EXEMPT)} exempt, "
          f"{len(green)} passing on nothing")
    return 1 if green else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
