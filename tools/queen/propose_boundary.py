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
import http.client
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request

REPO = "gHashTag/t27"
LABEL = "needs-boundary"
# The first line of every comment this posts. It is also how a re-run knows it
# has already spoken here: a tool that posts must be safe to run twice, or the
# first network hiccup turns a second attempt into a second comment on every
# issue it already reached.
PREFACE = (
    "A boundary drafted from the paths this issue already names, so a bee can "
    "take it. **Nothing is reserved until this is in the issue body** - the "
    "Queen reads the body, not comments. Check the paths first: a wrong "
    "boundary reserves files the work does not own."
)

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


def token() -> str:
    """A GitHub token if one is to hand. Anonymous reads work; they are just
    rationed at sixty an hour, and this walks six pages of a large repository."""
    for name in ("GH_TOKEN", "GITHUB_TOKEN"):
        if os.environ.get(name):
            return os.environ[name]
    try:
        out = subprocess.run(
            ["gh", "auth", "token"], capture_output=True, text=True, timeout=20
        )
        return out.stdout.strip() if out.returncode == 0 else ""
    except (OSError, subprocess.SubprocessError):
        return ""


def api(url: str, tries: int = 3) -> object:
    """One GitHub read, retried.

    `json.load(res)` straight off the socket raised `IncompleteRead` on a full
    page of issues: GitHub sends these chunked, and a chunked body that ends
    early dies mid-parse with half the rows already consumed and nothing to
    retry from. Read the bytes first, parse second, and a short read is an
    error that can simply be asked again.
    """
    headers = {
        "accept": "application/vnd.github+json",
        "user-agent": "propose-boundary",
    }
    auth = token()
    if auth:
        headers["authorization"] = f"Bearer {auth}"
    last: Exception | None = None
    for attempt in range(tries):
        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=60) as res:
                return json.loads(res.read().decode("utf-8"))
        except (http.client.IncompleteRead, urllib.error.URLError, TimeoutError) as err:
            last = err
            time.sleep(2 * (attempt + 1))
    raise RuntimeError(f"{url}: {last}")


# A SMALL PAGE, BECAUSE A BIG ONE ARRIVES BROKEN.
#
# GitHub sends these chunked and the bodies here are long - a spec issue
# carries its whole brief. Measured against this repository on 2026-09-23,
# four pages read in a row:
#
#     per_page=100  ->  1 ok, 3 IncompleteRead
#     per_page=50   ->  3 ok, 1 IncompleteRead
#     per_page=30   ->  4 ok, 0 failures
#
# Retrying did not help, because the same page truncated again. Fewer rows per
# response is what fixes it; the extra round trips are cheap next to a run that
# dies two thirds of the way through.
PER_PAGE = 30
# 553 labelled issues at 30 a page is 19; the cap is a bound on cost, not an
# expectation. A run that hits it says so rather than silently answering short.
MAX_PAGES = 60


def issues(only: int | None) -> list[dict]:
    if only is not None:
        return [api(f"https://api.github.com/repos/{REPO}/issues/{only}")]
    found: list[dict] = []
    page = 1
    while page <= MAX_PAGES:
        rows = api(
            f"https://api.github.com/repos/{REPO}/issues"
            f"?state=open&labels={LABEL}&per_page={PER_PAGE}&page={page}"
        )
        if not isinstance(rows, list) or not rows:
            return found
        found += [r for r in rows if not r.get("pull_request")]
        if len(rows) < PER_PAGE:
            return found
        page += 1
    print(
        f"note: stopped at {MAX_PAGES} pages; there may be more labelled issues.",
        file=sys.stderr,
    )
    return found


def already_drafted(number: int) -> bool:
    """Has this tool commented on the issue before? Keeps a re-run silent."""
    rows = api(f"https://api.github.com/repos/{REPO}/issues/{number}/comments?per_page=100")
    if not isinstance(rows, list):
        return False
    return any(PREFACE[:60] in str(c.get("body") or "") for c in rows)


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
    spec_first = paths_of("fix specs/a.t27", "tools/y.sh")
    if not spec_first or not spec_first[0].endswith(".t27"):
        print("FAIL: --specs-only leans on a .t27 sorting first")
        bad += 1
    text = proposal(["specs/a.t27", "docs/plan.md"])
    if not text.startswith("## Boundary") or "cited in the issue" not in text:
        print("FAIL: the proposal must carry the heading the Queen reads, and mark a cited doc")
        bad += 1
    print("propose_boundary self-test:", "PASS" if bad == 0 else f"{bad} FAILED")
    return 1 if bad else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--limit", type=int, default=20, help="how many DRAFTS to print (0 = all)"
    )
    parser.add_argument("--issue", type=int)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument(
        "--specs-only",
        action="store_true",
        help="only issues naming a .t27 - law L0's own work, and the draft "
        "least likely to be wrong, because a spec path in an issue about a "
        "spec is the file it changes",
    )
    parser.add_argument(
        "--as-comment",
        action="store_true",
        help="post each draft as a comment (needs gh auth). Never edits the body.",
    )
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    rows = issues(args.issue)
    drafted = 0
    failed: list[int] = []
    for row in rows:
        # The limit counts DRAFTS, not issues read: cutting the list first made
        # `--limit 1` mean "look at one issue", which on a board where most
        # issues name nothing printed nothing at all.
        if args.limit > 0 and drafted >= args.limit:
            break
        paths = paths_of(row.get("title", ""), row.get("body") or "")
        if not paths:
            continue
        if args.specs_only and not any(p.endswith(".t27") for p in paths):
            continue
        drafted += 1
        text = proposal(paths)
        print(f"\n#{row['number']} {row.get('title','')[:70]}")
        print(text)
        if args.as_comment:
            if already_drafted(int(row["number"])):
                print("  (already drafted here; left alone)")
                continue
            # ONE FAILED POST MUST NOT END THE RUN. A single 502 two thirds
            # of the way through a hundred issues used to abort everything
            # after it; the re-run guard above makes finishing the rest and
            # coming back the cheap thing to do.
            done = subprocess.run(
                [
                    "gh", "issue", "comment", str(row["number"]),
                    "--repo", REPO, "--body", f"{PREFACE}\n\n{text}",
                ],
                capture_output=True,
                text=True,
            )
            if done.returncode != 0:
                failed.append(int(row["number"]))
                print(f"  (could not comment: {done.stderr.strip()[:120]})")
    print(f"\n{drafted} draft(s) from {len(rows)} labelled issue(s).")
    if failed:
        print(f"{len(failed)} could not be commented on: {failed}", file=sys.stderr)
        print("Run the same command again - what is already drafted is left alone.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
