#!/usr/bin/env python3
"""Fail if a property file is not run by any CI step.

Wave 618. Wave 617 found eight properties that README counted as proved while no
job in the repository ran them: `zero_size_props.sv` appeared exactly once in all
of `.github/`, inside the *weekly* mutation harness, and two of its four wrappers
appeared nowhere at all. Nothing was broken -- all eight held -- which is exactly
why it sat unnoticed. **An ungated property that happens to hold is
indistinguishable from a gated one.**

That was found by accident, from one line of a bound audit that I nearly wrote
off as my own bug. This is the systematic version, and it is cheap: counting
properties tells you nothing, so count the STEPS THAT RUN THEM.

Two levels of finding:

  ORPHAN   no workflow references the file at all -- an error
  WEEKLY   referenced only by a schedule-triggered workflow, so a defect in it
           is invisible on a pull request. Reported, not failed: weekly is a
           deliberate choice for expensive harnesses. Silence is what is not
           allowed.

Usage:  python3 formal/orphan_scan.py [--self-test]
"""

import pathlib
import re
import sys

import yaml


def workflows(root):
    """[(name, is_scheduled, whole_text)] for every workflow."""
    out = []
    for p in sorted((root / ".github" / "workflows").glob("*.yml")):
        text = p.read_text()
        try:
            doc = yaml.safe_load(text) or {}
        except yaml.YAMLError:
            doc = {}
        # `on:` parses as the boolean True in YAML 1.1 -- check both spellings.
        trig = doc.get("on", doc.get(True, {})) or {}
        scheduled = isinstance(trig, dict) and set(trig) <= {"schedule", "workflow_dispatch"}
        out.append((p.name, scheduled, text))
    return out


def scan(root):
    wfs = workflows(root)
    files = sorted((root / "formal").glob("*.sv"))
    if not files:
        print(f"::error::orphan_scan found no property files under {root}/formal")
        return 1
    if not wfs:
        print(f"::error::orphan_scan found no workflows under {root}/.github/workflows")
        return 1

    bad, weekly = [], []
    print(f"{'property file':34s} {'workflows referencing it':>26s}")
    print("-" * 64)
    for f in files:
        hits = [(n, sched) for n, sched, text in wfs if f.name in text]
        names = ", ".join(n.replace(".yml", "") for n, _ in hits) or "-- NONE --"
        print(f"{f.name:34s} {names:>26s}")
        if not hits:
            bad.append(f.name)
        elif all(sched for _, sched in hits):
            weekly.append(f.name)

    for b in bad:
        print(f"::error::formal/{b} is referenced by no workflow -- its properties "
              "are counted but never run, and an ungated property that happens to "
              "hold looks exactly like a gated one")
    for w in weekly:
        print(f"::warning::formal/{w} is referenced only by schedule-triggered "
              "workflows, so a defect in it is invisible on a pull request")
    print(f"\norphan scan: {len(files)} property files, {len(wfs)} workflows, "
          f"{len(bad)} orphaned, {len(weekly)} weekly-only")
    return 1 if bad else 0


def self_test():
    """The gate must catch a file nothing runs, and must not cry wolf."""
    import shutil
    import tempfile
    root = pathlib.Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory() as td:
        td = pathlib.Path(td)
        shutil.copytree(root / "formal", td / "formal")
        shutil.copytree(root / ".github" / "workflows", td / ".github" / "workflows")
        cases = [("the real tree", lambda: None, 0),
                 ("a property file nothing references",
                  lambda: (td / "formal" / "orphaned_props.sv").write_text(
                      "module x_props (input wire clk);\n"
                      "    always @(posedge clk) a_x: assert (1'b1);\n"
                      "endmodule\n"), 1)]
        bad = []
        for name, setup, want in cases:
            setup()
            got = scan(td)
            print(f"  {'ok  ' if got == want else 'FAIL'} {name}  "
                  f"(exit {got}, want {want})\n")
            if got != want:
                bad.append(name)
        # And it must fail loudly rather than pass on an empty tree.
        shutil.rmtree(td / "formal")
        (td / "formal").mkdir()
        got = scan(td)
        print(f"  {'ok  ' if got == 1 else 'FAIL'} no property files at all "
              f"(exit {got}, want 1)")
        if got != 1:
            bad.append("empty tree")
    for b in bad:
        print(f"::error::orphan_scan self-test: '{b}' gave the wrong answer")
    return 1 if bad else 0


if __name__ == "__main__":
    r = pathlib.Path(__file__).resolve().parent.parent
    sys.exit(self_test() if "--self-test" in sys.argv else scan(r))
