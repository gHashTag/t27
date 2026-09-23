#!/usr/bin/env python3
"""Draft a `## Boundary` for issues that already name their own files.

WHY

565 of 649 candidates are skipped every tick for `missingBoundary`: no
`## Boundary` section, so the Queen can reserve nothing and no bee can take
them. `needs_boundary.py` made that list visible; this proposes a first draft of
what is missing, for the subset where the issue already says which files it is
about.

Measured 2026-09-23 over the 553 labelled issues: **238 (43%) name at least one
concrete file path** in their title or body, and 103 of those name a `.t27`.

WHAT IT REFUSES TO DO, AND WHY THAT IS THE POINT

It never edits an issue. A WRONG boundary is worse than a missing one: it
reserves files the work does not own, and blocks whatever really owns them
until the claim expires. And a path in an issue body is not always a path the
work touches - #4122 and #4115 both name
`docs/audit/inngest-improvement-plan-2026-09-13.md`, which is the document the
issue CITES, not the file it changes.

So this prints a proposal per issue and stops. A person (or a reviewer who
knows the work) decides. `--as-comment` posts the draft as a comment, which is
still not a boundary: the Queen reads the BODY, so a comment cannot make an
issue dispatchable by accident.

WHAT IT LEANS ON

  * a path that ends in `.t27` is the strongest signal: the issue is about a
    spec, and specs are what this project is
  * a path named in the TITLE outranks one named only in the body
  * a path under `docs/` is quoted as a citation more often than edited, so it
    is proposed last and marked

Usage:
    python3 tools/queen/propose_boundary.py --limit 20
    python3 tools/queen/propose_boundary.py --issue 3927
    python3 tools/queen/propose_boundary.py --self-test
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import urllib.request

REPO = "gHashTag/t27"
LABEL = "needs-boundary"

# Directories this project actually keeps work in. A match outside them is
# prose that happens to contain a slash.
ROOTS = "specs|tools|scripts|src|bootstrap|conformance|docs|bindings|apps"
PATH = re.compile(rf"((?:{ROOTS})/[\w./-]+\.[A-Za-z0-9]{{1,6}})")


def rank(path: str, in_title: bool) -> tuple[int, str]:
    """Sort key: lower is more likely to be the file the work touches."""
    if path.endswith(".t27"):
        return (0 if in_title else 1, path)
    if path.startswith("docs/"):
        # Cited far more often than edited; last, and marked in the output.
        return (8 if in_title else 9, path)
    return (2 if in_title else 3, path)


def paths_of(title: str, body: str) -> list[str]:
    """Every path the issue names, best candidate first. Pure: --self-test drives it."""
    in_title = set(PATH.findall(title or ""))
    everywhere = in_title | set(PATH.findall(body or ""))
    return sorted(everywhere, key=lambda p: rank(p, p in in_title))


def proposal(paths: list[str]) -> str:
    lines = ["## Boundary", ""]
    for path in paths:
        note = "  <!-- cited in the issue; delete if the work does not touch it -->" if path.startswith("docs/") else ""
        lines.append(f"- `{path}`{note}")
    return "\n".join(lines)


def api(url: str) -> object:
    req = urllib.request.Request(
        url, headers={"accept": "application/vnd.github+json", "user-agent": "propose-boundary"}
    )
    with urllib.request.urlopen(req, timeout=60) as res:
        return json.load(res)


def issues(limit: int, only: int | None) -> list[dict]:
    if only is not None:
        return [api(f"https://api.github.com/repos/{REPO}/issues/{only}")]
    found: list[dict] = []
    for page in range(1, 8):
        rows = api(
            f"https://api.github.com/repos/{REPO}/issues"
            f"?state=open&labels={LABEL}&per_page=100&page={page}"
        )
        if not isinstance(rows, list) or not rows:
            break
        found += [r for r in rows if not r.get("pull_request")]
        if len(rows) < 100:
            break
    return found[: limit if limit > 0 else None]


def self_test() -> int:
    cases = [
        (("Port tools/x.py", ""), ["tools/x.py"], "a path in the title"),
        (("", "see specs/base/types.t27 and tools/y.sh"),
         ["specs/base/types.t27", "tools/y.sh"], "a spec outranks a tool"),
        (("fix specs/a.t27", "also docs/plan.md"),
         ["specs/a.t27", "docs/plan.md"], "docs come last"),
        (("nothing here", "prose with a/b and 1/2"), [], "prose is not a path"),
        (("", "apps/website/src/x.tsx"), ["apps/website/src/x.tsx"], "a nested path"),
    ]
    bad = 0
    for (title, body), want, why in cases:
        got = paths_of(title, body)
        if got != want:
            print(f"FAIL ({why}): {got} != {want}")
            bad += 1
    text = proposal(["specs/a.t27", "docs/plan.md"])
    if not text.startswith("## Boundary") or "cited in the issue" not in text:
        print("FAIL: the proposal must carry the heading the Queen reads, and mark a cited doc")
        bad += 1
    print("propose_boundary self-test:", "PASS" if bad == 0 else f"{bad} FAILED")
    return 1 if bad else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--limit", type=int, default=20)
    parser.add_argument("--issue", type=int)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument(
        "--as-comment",
        action="store_true",
        help="post each draft as a comment (needs gh auth). Never edits the body.",
    )
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    rows = issues(args.limit, args.issue)
    drafted = 0
    for row in rows:
        paths = paths_of(row.get("title", ""), row.get("body") or "")
        if not paths:
            continue
        drafted += 1
        text = proposal(paths)
        print(f"\n#{row['number']} {row.get('title','')[:70]}")
        print(text)
        if args.as_comment:
            body = (
                "A boundary drafted from the paths this issue already names, so a bee can "
                "take it. **Nothing is reserved until this is in the issue body** - the "
                "Queen reads the body, not comments. Check the paths first: a wrong "
                "boundary reserves files the work does not own.\n\n" + text
            )
            subprocess.run(
                ["gh", "issue", "comment", str(row["number"]), "--repo", REPO, "--body", body],
                check=True,
            )
    print(f"\n{drafted} of {len(rows)} issue(s) carry enough to draft a boundary.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
