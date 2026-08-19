#!/usr/bin/env python3
"""Negative tests for untrusted-input handling in workflow steps.

Two forms are compared on the same payloads:

  interpolated   TITLE="<payload pasted into the script text>"
  env-passed     TITLE="$PR_TITLE"      with PR_TITLE=<payload> in the environment

The first is what GitHub does with `${{ github.event.pull_request.title }}` inside
a `run:` block -- textual substitution before the shell parses the line. The
second is what `env:` does.

Every case asserts BOTH directions, because only one of them is evidence:

  * the interpolated form must be shown to lose integrity on the payload -- it
    executes it, or it dies on a syntax error. A payload the vulnerable form
    survives proves nothing about the safe form.
  * the env form must pass the payload through byte for byte and leave no side
    effect.

The side effect is a marker file in a private temporary directory. Nothing is
written to /tmp, and no payload runs anything beyond `touch` on that marker.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

# Each payload is a title someone could type into a pull request, together with
# the effect the INTERPOLATED form is expected to have on it. Declaring the
# expectation in advance is what makes the run a test rather than a description:
# a surprise is then a failure, not a paragraph written after the fact.
#
#   executes  -- the payload runs a command
#   breaks    -- the payload destroys the script's integrity without running
#   control   -- the payload is expected to pass through both forms unharmed;
#                it is here to show the harness can tell the cases apart
PAYLOADS = [
    ("command substitution", "x$(touch MARKER)y",                    "executes"),
    ("backticks",            "x`touch MARKER`y",                     "executes"),
    ("statement separator",  "a\"; touch MARKER; echo \"b",           "executes"),
    ("IFS for the space",    "x$(touch${IFS}MARKER)y",               "executes"),
    ("env exfiltration",     "x$(echo $GITHUB_TOKEN > MARKER)y",     "executes"),
    # A newline is literal inside a double-quoted assignment, so the shell form
    # is unharmed by it. It was declared "breaks" here and the harness refused
    # the run: the danger of a newline lives in a DIFFERENT sink -- the
    # GITHUB_OUTPUT file, checked at the end of this script. Which character is
    # dangerous is a property of the sink, not of the character.
    ("newline",              "first\ntouch MARKER\nsecond",           "control"),
    ("double quote",         'a "quoted" title',                     "breaks"),
    # The real title of PR #2168, verbatim. See the note below.
    ("PR #2168, verbatim",
     "bootstrap: accept `pub const Name(T) = struct` as a parameterised type "
     "declaration",                                                 "breaks"),
    ("single quote",         "it's a title",                         "control"),
    ("parenthesis alone",    "accept pub const Name(T) = struct",    "control"),
]

# Correction to the record, established by this harness on 2026-08-15:
#
# The CI failure that exposed this defect was first reported -- in issue #2171, in
# the tick D report and in a commit message -- as being caused by the parenthesis
# in the title of PR #2168. That attribution is wrong, and the last row above
# refutes it: a parenthesis inside a double-quoted assignment is an ordinary
# character, and the "parenthesis alone" payload passes through untouched.
#
# The entry point was the pair of BACKTICKS in that title. They opened a command
# substitution, and `(` was then a syntax error inside it:
#
#     bash: command substitution: line 1: syntax error near unexpected token `('
#     bash: command substitution: line 1: `pub const Name(T) = struct'
#
# reproduced here byte for byte against the CI log.
#
# The correction makes the finding worse rather than better. Had the backticked
# text been a valid command instead of a declaration, it would have RUN. The
# parenthesis is what stopped it. And titles in this repository quote code in
# backticks as a matter of style, so the dangerous construct is the ordinary one.

VULNERABLE = 'TITLE="{payload}"\nprintf %s "$TITLE" > out.txt\n'
SAFE = 'set -eu\nTITLE="$PR_TITLE"\nprintf %s "$TITLE" > out.txt\n'


def run(script: str, cwd: Path, env_extra: dict | None = None):
    env = dict(os.environ, GITHUB_TOKEN="fake-token-for-test")
    env.update(env_extra or {})
    p = subprocess.run(["bash", "-c", script], cwd=cwd, env=env,
                       capture_output=True, text=True)
    return p.returncode, (cwd / "out.txt").read_text() if (cwd / "out.txt").exists() else None


def main() -> int:
    failures = []
    print("%-22s %26s %22s  %s" % ("payload", "interpolated", "env-passed", "vs expected"))
    print("-" * 86)

    for name, payload, expected in PAYLOADS:
        with tempfile.TemporaryDirectory() as d:
            cwd = Path(d)
            marker = cwd / "MARKER"
            body = payload.replace("MARKER", str(marker))

            # --- the vulnerable form
            rc, out = run(VULNERABLE.format(payload=body), cwd)
            executed = marker.exists()
            if executed:
                marker.unlink()
            if executed:
                observed, shown = "executes", "EXECUTED payload"
            elif rc != 0 or out != body:
                observed, shown = "breaks", ("died rc=%d" % rc if rc else "mangled value")
            else:
                observed, shown = "control", "passed through"

            # --- the safe form
            rc2, out2 = run(SAFE, cwd, {"PR_TITLE": body})
            leaked = marker.exists()
            safe_ok = (not leaked) and out2 == body and rc2 == 0
            safe = "clean, verbatim" if safe_ok else (
                "SIDE EFFECT" if leaked else "rc=%d not verbatim" % rc2)

            verdict = "ok" if observed == expected else "UNEXPECTED"
            print("%-22s %26s %22s  %s" % (name, shown, safe, verdict))

            if observed != expected:
                failures.append("%s: interpolated form was expected to %s, "
                                "observed %s" % (name, expected, observed))
            if not safe_ok:
                failures.append("%s: env form failed -- %s" % (name, safe))

    # --- GITHUB_OUTPUT is a file, and a newline in a value defines new outputs
    print()
    with tempfile.TemporaryDirectory() as d:
        cwd = Path(d)
        gho = cwd / "gh_output"
        gho.write_text("")
        payload = "harmless\nis_admin=true"
        subprocess.run(["bash", "-c", 'printf "title=%s\\n" "$PR_TITLE" >> "$GITHUB_OUTPUT"'],
                       cwd=cwd, env=dict(os.environ, PR_TITLE=payload,
                                         GITHUB_OUTPUT=str(gho)),
                       capture_output=True, text=True)
        lines = gho.read_text().splitlines()
        injected = any(l.strip() == "is_admin=true" for l in lines)
        print("GITHUB_OUTPUT with a newline in the value ->",
              "defines an extra output" if injected else "contained")
        for l in lines:
            print("   ", repr(l))
        if not injected:
            failures.append("GITHUB_OUTPUT case did not reproduce; the "
                            "justification for validating values before writing "
                            "them is then unproven, not disproven")
        else:
            print("    Quoting in the shell does not help here: the value is")
            print("    already data, and the file format is what is being abused.")
            print("    This is why the issue number is checked against ^[0-9]+$")
            print("    before it is written, and why the title is not written.")

    print()
    if failures:
        print(f"FAIL ({len(failures)}):")
        for f in failures:
            print("  -", f)
        return 1
    n_exec = sum(1 for _, _, e in PAYLOADS if e == "executes")
    n_break = sum(1 for _, _, e in PAYLOADS if e == "breaks")
    n_ctrl = sum(1 for _, _, e in PAYLOADS if e == "control")
    print("OK: %d payloads, each matching its declared expectation -- %d execute a "
          "command\nunder the interpolated form, %d destroy the script, %d are "
          "controls that pass\nthrough both. The env form passed all %d through byte "
          "for byte with no side effect."
          % (len(PAYLOADS), n_exec, n_break, n_ctrl, len(PAYLOADS)))
    print("Scope: this tests the two shell forms, not the live workflow. It cannot")
    print("show that no other step interpolates untrusted data -- that is what")
    print("scripts/ci/check_untrusted_shell_interp.py is for.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
