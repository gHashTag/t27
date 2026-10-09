#!/usr/bin/env python3
"""Gate 29: a suspended check must name what would end the suspension.

Prop. 201: `continue-on-error: true` converts a failing check into an
INDEFINITELY suspended one. The step emits no signal distinguishing "green, safe
to enforce" from "red for a year" -- both render as a passing workflow -- so the
suspension becomes invisible at exactly the moment it should end. One step here
read "non-blocking until observed green" and had never been green.

This gate does not run the suspended checks; that would mean rebuilding Coq trees
and re-resolving git refs on every invocation. It enforces the bookkeeping that
makes the promise checkable:

  * every `continue-on-error: true` step must appear in the baseline below, and
  * the baseline entry records the step and the workflow it lives in.

A NEW suspension therefore fails the build until someone writes it down, and a
suspension that is REMOVED shows up as a baseline entry with no step -- which is
the signal that the flag was flipped and the record should follow.

COVERAGE. Examines every `- name:` step in `.github/workflows/*.yml`: 5 steps
carry the flag today, of which 2 are checks and 3 are infrastructure (an artifact
upload, a status sync, a matrix generator). The gate does not distinguish those --
it cannot tell a check from a side effect by parsing YAML, and guessing would be
the kind of classification-by-name this campaign keeps retracting (Prop. 197). It
therefore ratchets ALL of them and lets the baseline carry the distinction in
prose. It says nothing about whether a suspended check currently passes.

ARTIFACTS. Reads `.github/workflows/*.yml`. WRITES
`formal/suspension_baseline.txt`, and only when `--init` is passed. Nothing else.

Prop. 202.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
WF = ROOT / ".github" / "workflows"
BASELINE = ROOT / "formal" / "suspension_baseline.txt"


def suspended():
    out = []
    for y in sorted(WF.glob("*.yml")):
        text = y.read_text(errors="ignore")
        for m in re.finditer(r"- name: ([^\n]+)\n((?:(?!\n      - name:).)*)",
                             text, re.S):
            if "continue-on-error: true" in m.group(2):
                out.append(f"{y.name}\t{m.group(1).strip()}")
    return out


def main():
    if not WF.exists():
        print("::error::suspension scan: no such directory "
              "'.github/workflows' -- nothing was scanned")
        return 1
    files = list(WF.glob("*.yml"))
    if not files:
        print("::error::suspension scan: found no workflow files under "
              ".github/workflows -- nothing was scanned")
        return 1

    now = sorted(suspended())
    print(f"suspension scan: {len(files)} workflow file(s), {len(now)} step(s) "
          f"marked continue-on-error")

    if not BASELINE.exists():
        # Prop. 211c: writing a baseline is an explicit act, never a fallback.
        # `if not exists(): write(now); return 0` resets the ratchet on one
        # `rm`, and on a clone that never had the file it rubber-stamps the tree
        # it was handed and exits 0. Measured before f66561f33: 8 of the 13
        # baselines in this suite were on disk and in no commit, and 8 of the 13
        # gates owning them re-baseline a possibly-broken tree and pass.
        if "--init" not in sys.argv[1:]:
            print(f"::error::suspension scan: {BASELINE.name} does not exist and "
                  f"--init was not given. Writing one here would record "
                  f"whatever this tree contains as the accepted state -- on a "
                  f"fresh clone that is a green run which checked nothing. "
                  f"Genuine first run: `python3 formal/suspension_scan.py --init`. "
                  f"Otherwise the baseline was lost and belongs in the commit "
                  f"that lost it (Prop. 211)")
            return 1
        BASELINE.write_text("\n".join(now) + ("\n" if now else ""))
        print(f"suspension scan: baseline written to {BASELINE.name} "
              f"({len(now)} suspensions)")
        return 0

    was = [l for l in BASELINE.read_text().splitlines()
           if l.strip() and not l.startswith("#")]
    new = [s for s in now if s not in was]
    if new:
        print(f"::error::suspension scan: {len(new)} new continue-on-error "
              f"step(s). A suspended check reports success whether it is green "
              f"or has been red for a year -- record it in "
              f"{BASELINE.name} with what would end the suspension, or do not "
              f"suspend it (Prop. 202)")
        for s in new:
            print(f"  {s}")
        return 1
    gone = [w for w in was if w not in now]
    if gone:
        print(f"suspension scan: {len(gone)} suspension(s) removed -- the flag "
              f"was flipped; update {BASELINE.name} to lock it in")
        for g in gone:
            print(f"  {g}")
    print(f"suspension scan: ratchet holds ({len(now)} <= {len(was)} suspended)")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:
        print(f"::error::suspension scan: could not read the workflows "
              f"({type(exc).__name__}: {exc}) -- nothing was scanned")
        sys.exit(1)
