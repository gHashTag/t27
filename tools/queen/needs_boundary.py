#!/usr/bin/env python3
"""Label the open issues no bee can take, because they declare no boundary.

WHY

The Queen reserves paths before she dispatches, so that two workers never edit
the same file at once. An issue that never says which paths it touches has
nothing to reserve, so she skips it -- and says so only as a number inside a
tick summary nobody reads.

Measured 2026-09-23: 563 of 654 open issues had no `## Boundary` section, while
the Queen's tick reported `missingBoundary` for 565 candidates and the swarm sat
at 2 of 20 lanes. The backlog was not empty. It was unreachable, and the issue
list said nothing about which ones or why.

WHAT IT DOES, AND WHAT IT REFUSES TO DO

It adds `needs-boundary` to open issues without the section, and removes it from
issues that have since gained one. That is all. It never writes a boundary into
an issue body: which paths a task may touch is a judgement about the work, and a
WRONG boundary is worse than a missing one, because it reserves the wrong files
and blocks whatever really owns them.

THE PARSER IS THE QUEEN'S, NOT A NEW ONE

A boundary is a line beginning `## Boundary` or `## Границы` -- the rule in
`boundaryPathsOf` (BrowserOS trios/agent-server queen-tick.ts) and its Swift
twin in queend. This file matches that rule and nothing wider: if it accepted a
heading the Queen does not, the label would say "dispatchable" about an issue
she will keep skipping.

Usage:
    python3 tools/queen/needs_boundary.py --dry-run
    python3 tools/queen/needs_boundary.py
    python3 tools/queen/needs_boundary.py --self-test
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys

REPO = "gHashTag/t27"
LABEL = "needs-boundary"
LABEL_COLOR = "d93f0b"
LABEL_ABOUT = (
    "No '## Boundary' section, so the Queen can reserve nothing for it and no "
    "bee can take it"
)

# The Queen's rule, matched exactly: a level-two heading that starts with
# Boundary or the Russian heading her parser also accepts.
BOUNDARY = re.compile(r"^##[ \t]+(Boundary|Границы)", re.MULTILINE)


def has_boundary(body: str | None) -> bool:
    """Whether the Queen would find a boundary section in this issue body."""
    return bool(BOUNDARY.search(body or ""))


def gh(args: list[str]) -> str:
    done = subprocess.run(["gh", *args], capture_output=True, text=True)
    if done.returncode != 0:
        raise RuntimeError(done.stderr.strip()[:400])
    return done.stdout


def open_issues() -> list[dict]:
    """Every open issue, pull requests excluded: a PR is not dispatchable work."""
    rows: list[dict] = []
    out = gh([
        "api", "--paginate",
        f"repos/{REPO}/issues?state=open&per_page=100",
        "--jq",
        "[.[] | select(.pull_request == null) "
        "| {number, body, labels: [.labels[].name]}]",
    ])
    for line in out.strip().splitlines():
        if line.strip():
            rows.extend(json.loads(line))
    return rows


# A goal is not a task. The roadmap stages and the epics are deliberately not
# dispatchable -- they are containers for the issues that are -- so labelling
# them "needs-boundary" would put eight permanent entries at the top of a list
# whose whole value is that everything on it is worth fixing.
NOT_TASKS = frozenset({"roadmap", "epic"})


def decide(issues: list[dict]) -> tuple[list[int], list[int]]:
    """(to label, to unlabel). Pure, so --self-test can drive it."""
    add, drop = [], []
    for issue in issues:
        labels = issue.get("labels", [])
        labelled = LABEL in labels
        if NOT_TASKS & set(labels):
            # A goal that somehow carries the label still gets it removed.
            if labelled:
                drop.append(issue["number"])
            continue
        if has_boundary(issue.get("body")):
            if labelled:
                drop.append(issue["number"])
        elif not labelled:
            add.append(issue["number"])
    return add, drop


def ensure_label() -> None:
    try:
        gh(["label", "create", LABEL, "--repo", REPO,
            "--color", LABEL_COLOR, "--description", LABEL_ABOUT])
    except RuntimeError as error:
        # Already there is the normal case after the first run.
        if "already exists" not in str(error):
            raise


def self_test() -> int:
    cases = [
        ("## Boundary\n- a.py\n", True, "the heading the Queen accepts"),
        ("## Границы\n- a.py\n", True, "the Russian heading she also accepts"),
        ("## Boundary: files\n", True, "a heading with a trailing colon"),
        ("### Boundary\n", False, "a level-three heading is not the one she reads"),
        ("## boundary\n", False, "she matches the capital B"),
        ("Boundary: src/\n", False, "prose is not a section"),
        ("text\n\n## Boundary\n- a\n", True, "not only at the top of the body"),
        ("", False, "an empty body"),
        (None, False, "no body at all"),
    ]
    failures = 0
    for body, want, why in cases:
        got = has_boundary(body)
        if got != want:
            print(f"FAIL ({why}): {got} != {want}")
            failures += 1

    add, drop = decide([
        {"number": 1, "body": "nothing", "labels": []},
        {"number": 2, "body": "## Boundary\n- a", "labels": [LABEL]},
        {"number": 3, "body": "## Boundary\n- a", "labels": []},
        {"number": 4, "body": "nothing", "labels": [LABEL]},
        # A goal is not a task: never labelled, and cleared if it ever was.
        {"number": 5, "body": "nothing", "labels": ["roadmap"]},
        {"number": 6, "body": "nothing", "labels": ["epic", LABEL]},
    ])
    if add != [1] or drop != [2, 6]:
        print(f"FAIL (decide): add={add} drop={drop}, expected add=[1] drop=[2, 6]")
        failures += 1

    print("needs_boundary self-test:", "PASS" if failures == 0 else f"{failures} FAILED")
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    issues = open_issues()
    add, drop = decide(issues)
    goals = sum(1 for i in issues if NOT_TASKS & set(i.get("labels", [])))
    print(
        f"open issues: {len(issues)}; without a boundary: "
        f"{sum(1 for i in issues if not has_boundary(i.get('body')))}; "
        f"goals held back: {goals}; to label: {len(add)}; to unlabel: {len(drop)}"
    )
    if args.dry_run:
        print("dry run: nothing was changed")
        print("would label:", ", ".join(f"#{n}" for n in add[:25]) or "(none)")
        return 0

    if add:
        ensure_label()
    for number in add:
        gh(["issue", "edit", str(number), "--repo", REPO, "--add-label", LABEL])
    for number in drop:
        gh(["issue", "edit", str(number), "--repo", REPO, "--remove-label", LABEL])
    print(f"labelled {len(add)}, cleared {len(drop)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
