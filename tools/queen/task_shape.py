#!/usr/bin/env python3
"""Refuse to publish a task the Queen could never dispatch.

WHY

An issue with no `## Boundary` is not a task the swarm declines - it is a task
it CANNOT SEE. The Queen reserves files before she hands work out, so with
nothing to reserve the issue is skipped every round, for good. Measured
2026-09-23: 565 of 649 candidates skipped for `missingBoundary`, twenty lanes
idle, and the tick answering `refusal="nothing to choose"` while the board held
six hundred open issues.

Every one of those was opened by something that never checked. Labelling them
afterwards (`needs_boundary.py`) and drafting boundaries for them
(`propose_boundary.py`) are both repairs. This is the gate: a task that cannot
be dispatched is not published in the first place.

THE RULE IS THE QUEEN'S, COPIED

`boundary_paths` below is a PINNED TWIN of `boundaryPathsOf` in
`agent-server/apps/server/src/api/services/queen-tick.ts`. Not an
approximation: a gate that is stricter than the Queen rejects work she would
have taken, and one that is looser passes work she will silently skip - which
is the failure this exists to end. `--self-test` pins the twin against the
cases the TypeScript carries, including the ones that surprise:

  * the section ends at the NEXT `## ` heading, not at a blank line
  * a line contributes its FIRST token holding `/` or ending in `.ext`
  * backticks, quotes and trailing punctuation are stripped
  * `## Границы` is the same heading

WHAT IS NOT CHECKED, ON PURPOSE

Whether the path EXISTS. A port task names the `.t27` it is about to create,
and a gate demanding the file first would refuse every one of them.

Usage:
    python3 tools/queen/task_shape.py --self-test
    python3 tools/queen/task_shape.py --issue 4379
    python3 tools/queen/task_shape.py --issue 4379 --comment
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys

REPO = "gHashTag/t27"
# A goal is not a task. The Queen should never dispatch these, so they are not
# held to a task's shape - the same list `needs_boundary.py` holds back.
NOT_TASKS = frozenset({"roadmap", "epic"})
# The file name decides, not the directory: `docs/diagram.png` is not
# documentation and `trios/docs/x.md` is. The twin of DOC_FILE_SUFFIXES.
DOC_SUFFIXES = (".md",)


def boundary_paths(body: str) -> list[str]:
    """Pinned twin of `boundaryPathsOf` (queen-tick.ts)."""
    inside = False
    paths: list[str] = []
    for raw in (body or "").split("\n"):
        line = raw.strip()
        if line.startswith("## "):
            if inside:
                break
            inside = line.startswith("## Boundary") or line.startswith("## Границы")
            continue
        if not inside or not line:
            continue
        for token in line.split():
            cleaned = token.lstrip("`\"'(").rstrip("`\"'.,;:!?)")
            if "/" in cleaned or _has_extension(cleaned):
                paths.append(cleaned)
                break
    return paths


def _has_extension(token: str) -> bool:
    """`/\\.\\w{1,10}$/` - a dot followed by 1-10 word characters, at the end."""
    dot = token.rfind(".")
    if dot < 0 or dot == len(token) - 1:
        return False
    tail = token[dot + 1 :]
    return len(tail) <= 10 and all(c.isalnum() or c == "_" for c in tail)


def reaches_source(paths: list[str]) -> bool:
    """True when at least one path is not documentation.

    A boundary of one `.md` file has length 1, so the Queen calls it work - and
    an issue worked exactly as written changes no behaviour. Reported, and not
    fatal: whether prose-only tasks may be dispatched is the operator's
    decision, and this gate does not get to make it silently.
    """
    return any(
        not path[path.rfind("/") + 1 :].lower().endswith(DOC_SUFFIXES)
        for path in paths
    )


def problems(body: str, labels: set[str]) -> list[str]:
    """What stops this issue being dispatchable. Empty means it is ready."""
    if NOT_TASKS & labels:
        return []
    paths = boundary_paths(body)
    if not paths:
        if "## Boundary" in (body or "") or "## Границы" in (body or ""):
            return [
                "the `## Boundary` section holds no path: the Queen reads "
                "nothing from it, which she treats exactly as a missing section"
            ]
        return [
            "no `## Boundary` section, so there is nothing to reserve and no "
            "bee can take this issue - it will be skipped every round, for good"
        ]
    return []


def warnings(body: str, labels: set[str]) -> list[str]:
    """Worth saying, but not worth refusing over."""
    if NOT_TASKS & labels:
        return []
    paths = boundary_paths(body)
    if paths and not reaches_source(paths):
        return [
            "every path in the boundary is documentation, so work done exactly "
            "as written changes no behaviour"
        ]
    return []


def gh_json(args: list[str]) -> object:
    out = subprocess.run(["gh", *args], capture_output=True, text=True)
    if out.returncode != 0:
        raise RuntimeError(out.stderr.strip()[:200])
    return json.loads(out.stdout)


def report(number: int, body: str, labels: set[str], comment: bool) -> int:
    bad = problems(body, labels)
    warn = warnings(body, labels)
    for line in bad:
        print(f"#{number} REFUSED: {line}")
    for line in warn:
        print(f"#{number} warning: {line}")
    if not bad and not warn:
        print(f"#{number} ready: {', '.join(boundary_paths(body)) or 'not a task'}")
    if bad and comment:
        text = (
            "**This issue cannot be dispatched as it stands.**\n\n"
            + "\n".join(f"- {line}" for line in bad)
            + "\n\nAdd a section naming the files the work owns, and the Queen "
            "will pick it up on her next round:\n\n"
            "```markdown\n## Boundary\n\n- `path/to/the/file`\n```\n\n"
            "A boundary is a claim: name only what the work changes. Reserving "
            "a file the work does not own blocks whoever really owns it."
        )
        subprocess.run(
            ["gh", "issue", "comment", str(number), "--repo", REPO, "--body", text],
            check=True,
        )
    return 1 if bad else 0


def self_test() -> int:
    bad = 0
    cases: list[tuple[str, list[str], str]] = [
        ("## Boundary\n- src/a.py\n", ["src/a.py"], "a list item"),
        ("## Boundary\n\nspecs/x.t27\n", ["specs/x.t27"], "a bare line"),
        ("## Boundary\n- `specs/x.t27`\n", ["specs/x.t27"], "backticks stripped"),
        ("## Границы\n- src/a.py\n", ["src/a.py"], "the Russian heading"),
        ("## Boundary\n- a.py\n## Other\n- b.py\n", ["a.py"],
         "the section ends at the next ## heading"),
        ("## Boundary\n\n- a.py\n\n- b.py\n", ["a.py", "b.py"],
         "a blank line does NOT end the section"),
        ("## Boundary\n- see src/a.py and src/b.py\n", ["src/a.py"],
         "the FIRST token of a line, not every one"),
        ("## Boundary\n- prose with no path\n", [],
         "a line holding no path contributes none"),
        ("### Boundary\n- a.py\n", [], "a level-three heading is not the one she reads"),
        ("Boundary: src/\n", [], "prose is not a section"),
        ("## Boundary\n- (`src/a.py`),\n", ["src/a.py"], "punctuation stripped"),
        ("text\n\n## Boundary\n- a.py\n", ["a.py"], "not only at the top"),
    ]
    for body, want, why in cases:
        got = boundary_paths(body)
        if got != want:
            print(f"FAIL (boundary_paths, {why}): {got} != {want}")
            bad += 1
    for token, want, why in [
        ("a.py", True, "an extension"),
        ("e.g", True, "two letters is still an extension by the Queen's rule"),
        ("word", False, "no dot"),
        ("trailing.", False, "a dot at the end is not an extension"),
        ("a.verylongextension", False, "more than ten characters is not one"),
    ]:
        if _has_extension(token) is not want:
            print(f"FAIL (_has_extension, {why}): {token}")
            bad += 1
    for paths, want, why in [
        (["src/a.py"], True, "code reaches source"),
        (["docs/x.md"], False, "one document does not"),
        (["docs/x.md", "src/a.py"], True, "one of the two does"),
        (["docs/diagram.png"], True, "the NAME decides, not the directory"),
        (["trios/docs/x.MD"], False, "case does not matter"),
    ]:
        if reaches_source(paths) is not want:
            print(f"FAIL (reaches_source, {why}): {paths}")
            bad += 1
    for body, labels, want, why in [
        ("## Boundary\n- a.py\n", set(), 0, "a shaped task passes"),
        ("no section", set(), 1, "a task with no boundary is refused"),
        ("## Boundary\n\nprose only\n", set(), 1, "an empty section is refused"),
        ("no section", {"roadmap"}, 0, "a goal is not held to a task's shape"),
        ("no section", {"epic"}, 0, "nor is an epic"),
    ]:
        if len(problems(body, labels)) != want:
            print(f"FAIL (problems, {why})")
            bad += 1
    if warnings("## Boundary\n- docs/x.md\n", set()) == []:
        print("FAIL: a documentation-only boundary should warn")
        bad += 1
    print("task_shape self-test:", "PASS" if bad == 0 else f"{bad} FAILED")
    return 1 if bad else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--issue", type=int)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument(
        "--comment",
        action="store_true",
        help="say on the issue what is missing, when it is refused",
    )
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.issue is None:
        parser.error("give --issue N or --self-test")
    row = gh_json(
        ["issue", "view", str(args.issue), "--repo", REPO, "--json", "body,labels"]
    )
    labels = {str(l.get("name")) for l in (row.get("labels") or [])}
    return report(args.issue, str(row.get("body") or ""), labels, args.comment)


if __name__ == "__main__":
    sys.exit(main())
