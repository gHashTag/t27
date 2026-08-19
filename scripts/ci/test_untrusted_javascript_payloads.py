#!/usr/bin/env python3
"""Negative fixtures for the actions/github-script JavaScript sink.

The vulnerable form pastes a title into JavaScript source. The safe form reads
that same title from process.env. Expectations are declared before execution;
otherwise a payload that happens to survive the vulnerable form would provide no
evidence about the safe form.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

CHECKER = Path(__file__).with_name("check_untrusted_javascript_interp.py")

# Expected effect of the vulnerable, source-interpolated form:
#   executes -- injected JavaScript writes the marker
#   breaks   -- the source is invalid and the marker is absent
#   control  -- the source remains intact, included only to test classification
PAYLOADS = [
    ("single quote", "x'; require('fs').writeFileSync('MARKER', 'executed');//", "executes"),
    ("double quote control", 'a "quoted" title', "control"),
    ("backtick control", "x`quoted`y", "control"),
    ("newline", "first\nsecond", "breaks"),
]

VULNERABLE = "const title = '{payload}';\nrequire('fs').writeFileSync('out.txt', title);\n"
SAFE = "const title = process.env.PR_TITLE;\nrequire('fs').writeFileSync('out.txt', title);\n"


def run_node(source: str, cwd: Path, env_extra: dict[str, str] | None = None):
    env = dict(os.environ, PR_TITLE="unused", SECRET="fixture-secret")
    env.update(env_extra or {})
    return subprocess.run(["node", "-e", source], cwd=cwd, env=env,
                          capture_output=True, text=True)


def static_fixture_result(workflow: str) -> int:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        workflows = root / ".github" / "workflows"
        workflows.mkdir(parents=True)
        (workflows / "fixture.yml").write_text(workflow, encoding="utf-8")
        result = subprocess.run([sys.executable, str(CHECKER)], cwd=root,
                                capture_output=True, text=True)
        return result.returncode


def main() -> int:
    failures: list[str] = []
    print("%-22s %24s %20s  %s" % ("payload", "interpolated", "env-passed", "vs expected"))
    print("-" * 82)

    for name, payload, expected in PAYLOADS:
        with tempfile.TemporaryDirectory() as directory:
            cwd = Path(directory)
            marker = cwd / "MARKER"
            body = payload.replace("MARKER", str(marker).replace("\\", "\\\\"))

            vulnerable = run_node(VULNERABLE.format(payload=body), cwd)
            executed = marker.exists()
            if executed:
                marker.unlink()
            if executed:
                observed, shown = "executes", "EXECUTED payload"
            elif vulnerable.returncode != 0:
                observed, shown = "breaks", "syntax/runtime error"
            else:
                observed, shown = "control", "passed through"

            safe = run_node(SAFE, cwd, {"PR_TITLE": payload})
            out = cwd / "out.txt"
            safe_ok = (safe.returncode == 0 and not marker.exists()
                       and out.read_text(encoding="utf-8") == payload)
            safe_shown = "clean, verbatim" if safe_ok else "FAILED"
            verdict = "ok" if observed == expected else "UNEXPECTED"
            print("%-22s %24s %20s  %s" % (name, shown, safe_shown, verdict))

            if observed != expected:
                failures.append("%s: expected %s, observed %s" % (name, expected, observed))
            if not safe_ok:
                failures.append("%s: env form did not preserve the payload" % name)

    vulnerable_workflow = """name: fixture\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/github-script@v7\n        with:\n          script: |\n            const title = '${{ github.event.issue.title }}';\n            console.log(title);\n"""
    safe_workflow = """name: fixture\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/github-script@v7\n        env:\n          ISSUE_TITLE: ${{ github.event.issue.title }}\n        with:\n          script: |\n            const title = process.env.ISSUE_TITLE;\n            console.log(title);\n"""
    before = static_fixture_result(vulnerable_workflow)
    after = static_fixture_result(safe_workflow)
    print()
    print("static JavaScript contract: before=%d unsafe, after=%d unsafe" %
          (1 if before == 1 else 0, 0 if after == 0 else 1))
    if before != 1:
        failures.append("static vulnerable fixture was not rejected (rc=%d)" % before)
    if after != 0:
        failures.append("static env fixture was rejected (rc=%d)" % after)

    print()
    if failures:
        print("FAIL (%d):" % len(failures))
        for failure in failures:
            print("  -", failure)
        return 1

    print("OK: %d payloads matched their declared class; env form preserved all "
          "payloads byte for byte; static contract changed 1 -> 0 unsafe "
          "interpolations." % len(PAYLOADS))
    print("Scope: JavaScript source in actions/github-script. Shell has a separate "
          "checker and payload harness.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
