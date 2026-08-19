#!/usr/bin/env python3
"""Forbid GitHub event data interpolated into actions/github-script source.

`actions/github-script` executes the value of `with.script` as JavaScript. GitHub
expressions are substituted into that source before the JavaScript parser runs.
A title such as `x'); await github.rest...` can therefore change the program,
even when the expression appears inside a quoted string.

The contract is deliberately narrow and mechanical: a `github.*` expression is
not allowed inside a `script:` block of an `actions/github-script` step. Pass the
value through `env:` and read `process.env.NAME` instead. This checker does not
claim to validate JavaScript, YAML, JSON, or other interpreters.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

WORKFLOW_DIR = Path(".github/workflows")
EXPR = re.compile(r"\$\{\{\s*([^}]+?)\s*\}\}")
GITHUB_TOKEN = re.compile(r"\bgithub(?:\.[A-Za-z_][A-Za-z0-9_]*)+")
USES = re.compile(r"^\s*-?\s*uses:\s*actions/github-script@[^\s#]+")
SCRIPT = re.compile(r"^(\s*)script:\s*([|>].*)?$")


def script_blocks(text: str):
    """Yield (line number, script text) for github-script steps."""
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        if not USES.match(lines[i]):
            i += 1
            continue
        step_indent = len(lines[i]) - len(lines[i].lstrip())
        j = i + 1
        script = None
        while j < len(lines):
            line = lines[j]
            indent = len(line) - len(line.lstrip())
            if line.strip() and indent <= step_indent and re.match(r"^\s*-\s+", line):
                break
            match = SCRIPT.match(line)
            if match:
                script_indent = len(match.group(1))
                body, start = [], j + 1
                j += 1
                while j < len(lines):
                    body_line = lines[j]
                    body_indent = len(body_line) - len(body_line.lstrip())
                    if body_line.strip() and body_indent <= script_indent:
                        break
                    body.append(body_line)
                    j += 1
                script = (start + 1, "\n".join(body))
                break
            j += 1
        if script is not None:
            yield script
        i = max(j, i + 1)


def main() -> int:
    files = sorted(WORKFLOW_DIR.glob("*.yml")) + sorted(WORKFLOW_DIR.glob("*.yaml"))
    if not files:
        print("FAIL: no workflow files found -- wrong working directory?")
        return 2

    steps = 0
    errors = []
    for path in files:
        text = path.read_text(encoding="utf-8", errors="replace")
        for start_line, block in script_blocks(text):
            steps += 1
            for match in EXPR.finditer(block):
                refs = GITHUB_TOKEN.findall(match.group(1))
                if not refs:
                    continue
                line = start_line + block[: match.start()].count("\n")
                errors.append((path, line, match.group(1).strip()))

    if errors:
        print("FAIL: %d github.* interpolation(s) into actions/github-script source:" % len(errors))
        for path, line, expression in errors:
            print("  %s:%d" % (path, line))
            print("    ${{ %s }}" % expression)
        print()
        print("Fix: pass event data through env: and read process.env.NAME.")
        print("The environment value is data; it is not substituted into JavaScript source.")
        return 1

    print("OK: %d actions/github-script step(s), 0 github.* interpolations into script source." % steps)
    print("Scope: JavaScript source in actions/github-script only; shell run: blocks have a separate gate.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
