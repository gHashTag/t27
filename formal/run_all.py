#!/usr/bin/env python3
"""Run every gate the workflows run, enumerated from the workflows themselves.

Prop. 200: for six waves this campaign ended each wave with "all 20 gates green".
The suite has 58 steps. The 20 were a list typed by hand, and `absence_sweep` --
the check that validates suite membership -- was not on it. It had been red since
wave 694, and its absence was **unobservable from inside the subset that was run**.

The fix cannot be discipline, because the failure mode of recall is silence. It
has to be construction: this script parses `.github/workflows/*.yml`, extracts
every `python3 formal/*.py` invocation, and runs all of them. There is no list to
forget.

It is deliberately NOT a gate. It runs gates. Adding it to a workflow would make
CI run every python step twice, and it would then need exempting from the absence
sweep -- which is the class of bookkeeping that produced Prop. 200 in the first
place. It is an operator tool, and its own correctness rests on one property that
IS checked: if it enumerates zero steps, it fails.

COVERAGE. Recovers an interrupted destructive run before doing anything
(Prop. 207) -- but only for gates it launches. **A gate invoked by hand while a
stash exists still measures a starved tree**, and that window is open until
someone runs this or the sweep. Closing it fully would mean a guard inside every
one of the 32 scripts; that is not done, and the residual gap is stated rather
than implied away.

Enumerates `python3 formal/<name>.py` invocations across all workflow
files -- 43 distinct invocations of 30 scripts, out of 58 total workflow steps.
The residue is explicit and large: the other 15 steps are yosys proofs, cargo
builds and shell, and this tool does not run them. It therefore establishes "every
PYTHON gate the workflows run passes", never "CI would pass". Two scripts under
formal/ are invoked by no workflow (`scale_probe.py`, `trace_reader.py`); both are
the non-checking helpers named in `coverage_gate.EXEMPT`, and that is reported
rather than assumed.

ARTIFACTS. Reads `.github/workflows/*.yml` and executes `formal/*.py`. Writes
nothing itself; the gates it runs write their own baselines as they always do.

Prop. 201.
"""
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
TIMEOUT = 900


def invocations():
    """Every `python3 formal/X.py [args]` step, in workflow order, deduplicated.

    Keyed on the full command including arguments: several scripts are run twice,
    once with `--self-test` and once for real, and collapsing those would drop the
    self-test -- the exact shape of loss this tool exists to prevent.
    """
    wf = ROOT / ".github" / "workflows"
    if not wf.exists():
        return None
    seen, out = set(), []
    for y in sorted(wf.glob("*.yml")):
        for m in re.finditer(r"python3 (formal/[a-z_0-9]+\.py[^\n|&;]*)",
                             y.read_text(errors="ignore")):
            cmd = m.group(1).strip()
            if cmd in seen:
                continue
            seen.add(cmd)
            # Prop. 201: CI's verdict on a step is `continue-on-error`-aware, so
            # a runner that ignores the flag disagrees with the thing it is
            # simulating -- and disagreeing in the STRICT direction is not safe
            # either: it trains the operator to expect one red and ignore it,
            # which is how a second red gets missed. Read the flag from the step
            # this command belongs to: scan backwards to the enclosing `- name:`.
            head = y.read_text(errors="ignore")[:m.start()]
            step = head.rsplit("- name:", 1)[-1] if "- name:" in head else ""
            soft = "continue-on-error: true" in step
            out.append((y.name, cmd, soft))
    return out


def recover_stash():
    """Restore an interrupted destructive run before enumerating anything.

    Prop. 207. Prop. 206 put recovery at the START of `absence_sweep`, which
    closes the window only for the next run of THAT gate. Every other gate run in
    between still measures a starved tree -- and reports its result with complete
    confidence, because a gate cannot tell "my subject is absent" from "my
    subject is clean" unless it was written to (which is the whole point of the
    sweep, and is why exactly 17 of 60 steps are exempt from it).

    This runner is the enumerated entry point, so it recovers first. That does
    not close the window for a gate invoked by hand; see COVERAGE.
    """
    bak = ROOT / "build" / "_absence_bak"
    if not bak.exists():
        return 0
    sys.path.insert(0, str(ROOT / "formal"))
    try:
        import absence_sweep
        absence_sweep.recover(ROOT)
    except Exception as exc:
        print(f"::error::run_all: a stash from an interrupted destructive run "
              f"exists at build/_absence_bak and could not be restored "
              f"({type(exc).__name__}: {exc}). Every gate below would measure a "
              f"starved tree and report it as a finding -- refusing rather than "
              f"running (Prop. 207)")
        return 1
    return 0


def main():
    if recover_stash():
        return 1
    steps = invocations()
    if steps is None:
        print("::error::run_all: no .github/workflows directory -- there is "
              "nothing to enumerate, so this tool ran no gates")
        return 1
    if not steps:
        print("::error::run_all: found no `python3 formal/*.py` step in any "
              "workflow. Enumerating zero steps and reporting success is the "
              "failure this tool exists to prevent (Prop. 200)")
        return 1

    scripts = {s.name for s in ROOT.glob("formal/*.py")}
    run_names = {c.split()[0].split("/")[-1] for _, c, _ in steps}
    never_run = sorted(scripts - run_names - {"run_all.py"})

    print(f"run_all: {len(steps)} python gate invocations enumerated from "
          f"{len({w for w, _, _ in steps})} workflow file(s), "
          f"{len(run_names)} distinct scripts")
    if never_run:
        print(f"run_all: {len(never_run)} script(s) under formal/ are run by no "
              f"workflow: {', '.join(never_run)}")

    failed, timed_out, soft_failed = [], [], []
    for wf_name, cmd, soft in steps:
        try:
            r = subprocess.run(["python3"] + cmd.split(), cwd=str(ROOT),
                               capture_output=True, text=True, timeout=TIMEOUT)
            rc = r.returncode
        except subprocess.TimeoutExpired:
            timed_out.append(cmd)
            print(f"  TIMEOUT  {cmd}")
            continue
        if rc != 0 and soft:
            soft_failed.append(cmd)
            print(f"  SOFT({rc}) {cmd}  [continue-on-error in the workflow]")
            continue
        if rc != 0:
            failed.append((cmd, rc))
            tail = [l for l in (r.stdout + r.stderr).splitlines()
                    if "::error::" in l]
            print(f"  FAIL({rc}) {cmd}")
            for t in tail[:2]:
                print(f"           {t[:110]}")
        else:
            print(f"  ok       {cmd}")

    print(f"\nrun_all: {len(steps) - len(failed) - len(timed_out) - len(soft_failed)} "
          f"passed, {len(failed)} failed, {len(soft_failed)} failed but marked "
          f"continue-on-error, {len(timed_out)} timed out")
    if soft_failed:
        print("run_all: the soft failures are RED CHECKS THE WORKFLOW IGNORES, "
              "not passes -- each one is a claim nobody is enforcing:")
        for c in soft_failed:
            print(f"  {c}")
    # A timeout is not a pass. Prop. 103: a decline that is not counted is the
    # shape this campaign keeps finding, and "it was slow" is a decline.
    return 1 if (failed or timed_out) else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::run_all: could not enumerate or run the gates "
              f"({type(exc).__name__}: {exc}) -- no conclusion was reached")
        sys.exit(1)
