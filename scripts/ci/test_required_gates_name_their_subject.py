#!/usr/bin/env python3
"""#3061: the two required gates must not report the wrong subject.

Both are in docs/BRANCH-PROTECTION.md's required list, so a wrong message here
lands on a check nobody can merge past.

  NOW Sync Gate -- `${PR_BASE_SHA:?}` catches unset and empty, not a SHA absent
  from the checkout. `git diff` then exits 128, the `|| true` that exists to
  absorb grep's no-match absorbs that identically, and the gate printed
  "SYNC REQUIRED: this PR/push adds no docs/now/ entry" about a comparison it
  never made.

  Issue Gate -- workflow_dispatch carries no `pull_request` object, so PR_TITLE
  and PR_BODY are empty and the gate printed "L1 TRACEABILITY violation: No
  issue reference found in PR title/body" against a pull request that does not
  exist.

Each assertion has a control: the same gate on a real input must still produce
its real verdict, or this file cannot fail.

The Issue Gate's logic is extracted from the workflow rather than restated, so
that editing the YAML and not this file is caught.
"""

import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FAILURES = []


def check(name, ok, detail=""):
    print(f"  {'ok      ' if ok else 'FAILED  '}{name}")
    if not ok:
        FAILURES.append(f"{name}: {detail}")


def run_now_gate(env):
    e = dict(os.environ)
    e.update(env)
    return subprocess.run(
        ["bash", str(REPO / "scripts/ci/now-sync-gate-diff.sh")],
        capture_output=True, text=True, env=e, cwd=str(REPO),
    )


def issue_gate_script():
    """The `run:` body of the 'Check for linked issues in PR' step, as shipped."""
    y = (REPO / ".github/workflows/issue-gate.yml").read_text()
    m = re.search(r"- name: Check for linked issues in PR\b.*?\n        run: \|\n(.*?)(?=\n      - |\n\Z)", y, re.S)
    if not m:
        return None
    body = "\n".join(l[10:] if l.startswith(" " * 10) else l for l in m.group(1).split("\n"))
    return body


def run_issue_gate(env):
    body = issue_gate_script()
    if body is None:
        return None
    e = dict(os.environ)
    e.setdefault("PR_TITLE", "")
    e.setdefault("PR_BODY", "")
    e.setdefault("PR_NUMBER", "")
    e.update(env)
    return subprocess.run(["bash", "-c", body], capture_output=True, text=True, env=e, cwd=str(REPO))


def main():
    head = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=str(REPO)).stdout.strip()
    absent = "0" * 39 + "1"

    print("NOW Sync Gate")
    bad = run_now_gate({"GITHUB_EVENT_NAME": "pull_request", "PR_BASE_SHA": absent, "PR_HEAD_SHA": head})
    out = bad.stdout + bad.stderr
    check("a base revision absent from the checkout exits 2, not 1", bad.returncode == 2,
          f"got {bad.returncode}; output={out[:400]!r}")
    check("and does not claim the entry is missing",
          "SYNC REQUIRED" not in out, f"output={out[:400]!r}")
    check("and names the variable and the value it could not resolve",
          "PR_BASE_SHA" in out and absent in out, f"output={out[:400]!r}")

    bad2 = run_now_gate({"GITHUB_EVENT_NAME": "push", "PUSH_BEFORE": "0" * 40, "PUSH_AFTER": absent})
    check("the push arm refuses an absent AFTER the same way",
          bad2.returncode == 2, f"got {bad2.returncode}")

    # CONTROL. Without it, a script that exits 2 for everything passes above.
    # HEAD against itself, deliberately: it needs no history, so this control
    # survives a `fetch-depth: 1` checkout instead of silently skipping there --
    # a control that can vanish with the runner's clone depth is not one.
    ctl = run_now_gate({"GITHUB_EVENT_NAME": "pull_request", "PR_BASE_SHA": head, "PR_HEAD_SHA": head})
    check("control: two resolvable revisions still produce a real verdict",
          ctl.returncode in (0, 1), f"got {ctl.returncode}; output={(ctl.stdout+ctl.stderr)[:300]!r}")

    print("Issue Gate")
    body = issue_gate_script()
    check("the step body was found in the workflow", body is not None,
          "the extractor no longer matches -- the test would assert nothing")
    if body is not None:
        no_pr = run_issue_gate({"GITHUB_EVENT_NAME": "workflow_dispatch"})
        o = no_pr.stdout + no_pr.stderr
        check("an event with no pull request does not report a violation",
              "L1 TRACEABILITY violation" not in o, f"output={o[:400]!r}")
        check("and says it examined nothing", "asserts nothing" in o, f"output={o[:400]!r}")
        check("and does not fail the run", no_pr.returncode == 0, f"got {no_pr.returncode}")

        # CONTROLS, both directions.
        good = run_issue_gate({"PR_NUMBER": "42", "PR_TITLE": "fix: x", "PR_BODY": "Closes #7"})
        check("control: a real PR carrying a reference still passes",
              good.returncode == 0 and "Issue gate passed" in good.stdout,
              f"got {good.returncode}; out={good.stdout[:200]!r}")
        bad3 = run_issue_gate({"PR_NUMBER": "42", "PR_TITLE": "fix: x", "PR_BODY": "no reference here"})
        check("control: a real PR with no reference is still a violation",
              bad3.returncode == 1 and "L1 TRACEABILITY violation" in (bad3.stdout + bad3.stderr),
              f"got {bad3.returncode}")

    print()
    if FAILURES:
        print("FAILED:")
        for f in FAILURES:
            print(f"  - {f}")
        return 1
    print("ok: both required gates name their own subject.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
