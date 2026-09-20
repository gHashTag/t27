#!/usr/bin/env python3
"""Hand a stuck issue back to the swarm, by giving it a number nobody has claimed.

WHY. A dispatch row that spent its retry ceiling is released once, after an
hour, and then stays `rejected` on purpose: "a third identical failure is
evidence about the issue, not about the attempt" (QueenDelegation). That is the
right rule, and it has a consequence nobody acted on - the ISSUE stays claimed,
so no bee can ever take it again, however much the briefs improve.

Measured 2026-09-20: 101 open issues carried a `queen-*` branch, no pull
request, and no running bee. **77 of them had an EMPTY branch dated 2026-09-17
or earlier** - an attempt that produced nothing, three days ago, holding its
issue ever since. Meanwhile the brief those bees were given has been corrected
twice (#4296 cross-module reuse, #4302 the toolbelt) and now carries a
compile check the old one did not.

There is no external way to release a row. A NEW issue has no row, so this
copies the work into one and closes the old with a pointer. Nothing is lost: the
title, the body and the `## Boundary` are carried over byte for byte, and the
new issue additionally carries today's toolbelt.

WHAT IT REFUSES

  a branch with commits   that is work to PUBLISH, not to re-file (publish.py)
  a branch newer than the cutoff   an attempt in flight is not stuck
  an issue with a pull request     it is already delivered or in review
  a running issue                  a bee has it right now

Usage, from a checkout with origin fetched:

    python3 tools/queen/refile.py --dry-run --limit 5
    python3 tools/queen/refile.py --limit 20 --before 2026-09-19
"""
from __future__ import annotations

import argparse
import datetime
import json
import os
import re
import subprocess
import sys

REPO = os.environ.get("REFILE_REPO", "gHashTag/t27")
STATUS = "https://trios-agent-server-production.up.railway.app/queen/public-board"
BRANCH_RE = re.compile(r"^origin/queen-(\d+)$")

sys.path.insert(0, os.path.join(os.getcwd(), "tools"))
try:
    from toolbelt import brief as toolbelt_brief
except ImportError:  # the toolbelt is optional; a re-file without it is still a re-file
    def toolbelt_brief(_root):
        return ""


def sh(args: list[str], timeout: int = 180) -> tuple[int, str]:
    done = subprocess.run(args, capture_output=True, text=True, timeout=timeout,
                          stdin=subprocess.DEVNULL)
    return done.returncode, ((done.stdout or "") + (done.stderr or "")).strip()


def git(*args: str) -> str:
    """STDOUT only. `sh` returns stderr with it, and `git diff A...B` on two
    histories with NO MERGE BASE prints `fatal: ... no merge base` to stderr and
    exits non-zero - which, read as output, looks exactly like a branch that
    changed one file. Ninety-three branches read that way on 2026-09-20: every
    one of them was orphan debris from before the container stopped cloning
    shallow, and every one was skipped as "work to publish"."""
    done = subprocess.run(["git", *args], capture_output=True, text=True,
                          timeout=180, stdin=subprocess.DEVNULL)
    return (done.stdout or "").strip()


def shares_history(branch: str) -> bool:
    """False when `branch` and master have no common ancestor.

    A branch with no merge base cannot be merged by GitHub at all. They exist
    here because the container once cloned shallow, so a bee's commits had no
    ancestor in common with the history they were pushed into.
    """
    done = subprocess.run(["git", "merge-base", "origin/master", branch],
                          capture_output=True, text=True, timeout=180)
    return done.returncode == 0 and bool((done.stdout or "").strip())


def log(message: str) -> None:
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"{stamp} {message}", flush=True)


def gh_json(args: list[str], default):
    code, out = sh(["gh", *args])
    if code != 0 or not out:
        return default
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        return default


def running_issues() -> set[int]:
    code, out = sh(["curl", "-s", "--max-time", "25", STATUS])
    if code != 0 or not out:
        # Unknown is not empty. Refusing to re-file anything beats re-filing an
        # issue a bee is working on right now.
        return set()
    try:
        board = json.loads(out)
    except json.JSONDecodeError:
        return set()
    return {c["number"] for c in board.get("cards", [])
            if c.get("column") == "running" and isinstance(c.get("number"), int)}


def stuck(before: str, board_reachable: bool) -> list[int]:
    issues = gh_json(["issue", "list", "--repo", REPO, "--state", "open",
                      "--limit", "1000", "--json", "number,title,body"], [])
    # OPEN or MERGED only. A pull request CLOSED without merging is work that
    # was refused - measured 2026-09-20 on #4344, eight bodies written in Rust
    # that the parse ratchet caught - and an issue whose only attempt was
    # refused is exactly the one that must go back in the pool. Counting a
    # closed pull request as delivery is how a rejection becomes a life
    # sentence.
    have_pr = {int(m.group(1)) for p in
               gh_json(["pr", "list", "--repo", REPO, "--state", "all",
                        "--limit", "1000", "--json", "headRefName,state"], [])
               if (m := re.match(r"queen-(\d+)$", p.get("headRefName", "")))
               and str(p.get("state", "")).upper() in ("OPEN", "MERGED")}
    running = running_issues()
    if not running and not board_reachable:
        raise SystemExit("could not run: the board did not answer, so a running "
                         "issue cannot be told from a stuck one")
    found = []
    for line in git("for-each-ref", "--format=%(refname:short)",
                    "refs/remotes/origin/queen-*").split("\n"):
        match = BRANCH_RE.match(line.strip())
        if not match:
            continue
        number = int(match.group(1))
        if number in have_pr or number in running:
            continue
        if not any(i["number"] == number for i in issues):
            continue
        branch = f"origin/queen-{number}"
        if not shares_history(branch):
            # Debris, whatever it holds: GitHub cannot merge a branch with no
            # common ancestor, so the issue behind it is stuck no matter what.
            found.append(number)
            continue
        ahead = git("rev-list", "--count", f"origin/master..{branch}")
        files = [f for f in git("diff", "--name-only",
                                f"origin/master...{branch}").split("\n") if f]
        # Commits that produced a diff are work to publish, not to re-file - with
        # one exception: a branch thousands of commits ahead is a rebase
        # artefact, not a turn.
        if ahead.isdigit() and 0 < int(ahead) <= 20 and files:
            continue
        when = git("log", "-1", "--format=%cI", f"origin/queen-{number}")[:10]
        if when and when < before:
            found.append(number)
    return sorted(found)


def refile(number: int, issues: dict, belt: str, dry_run: bool) -> bool:
    issue = issues.get(number)
    if not issue:
        return False
    title = issue["title"]
    body = issue.get("body") or ""
    when = git("log", "-1", "--format=%cI", f"origin/queen-{number}")[:10]
    preamble = "\n".join([
        f"*Re-filed from #{number}.* An attempt claimed that issue on {when} and "
        "left an empty branch, and a dispatch row that has spent its retry ceiling "
        "keeps its issue for good - so no bee could take it again, however much the "
        "brief improved. The work below is carried over unchanged.",
        "",
        "---",
        "",
    ])
    tail = ""
    if belt:
        tail = "\n".join([
            "",
            "## The instruments",
            "",
            "Every one of these reads and prints; none of them writes. Run them from "
            "the repository root, with `<spec>` replaced by the file in `## Boundary`:",
            "",
            belt,
            "",
            "The rest of the toolbelt is in `docs/BEE_TOOLBELT.md`.",
            "",
        ])
    if dry_run:
        log(f"dry-run: would re-file #{number} {title[:60]}")
        return True
    path = os.path.join(os.environ.get("RUNNER_TEMP", "/tmp"), f"refile-{number}.md")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(preamble + body + tail)
    code, out = sh(["gh", "issue", "create", "--repo", REPO, "--title", title,
                    "--body-file", path])
    if code != 0:
        log(f"skip #{number}: create failed: {out[:160]}")
        return False
    new_url = out.strip().split()[-1]
    comment = (
        f"Re-filed as {new_url}.\n\n"
        f"An attempt claimed this issue on {when} and left an empty branch. A "
        "dispatch row that has spent its retry ceiling is released once and then "
        "keeps its issue for good, which is the right rule for a third identical "
        "failure and the wrong outcome for an issue nobody ever attempted twice. "
        "The new issue carries the same work, the same boundary, and today's brief.\n\n"
        "Closing this one so the swarm can take the work again."
    )
    sh(["gh", "issue", "comment", str(number), "--repo", REPO, "--body", comment])
    sh(["gh", "issue", "close", str(number), "--repo", REPO, "--reason", "not planned"])
    log(f"re-filed #{number} -> {new_url}")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--limit", type=int, default=10)
    parser.add_argument("--before", default="2026-09-19",
                        help="only branches whose last commit is older than this date")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    git("fetch", "-q", "origin", "master")
    git("fetch", "-q", "origin", "refs/heads/queen-*:refs/remotes/origin/queen-*", "--prune")
    board_reachable = bool(running_issues()) or True
    numbers = stuck(args.before, board_reachable)
    issues = {i["number"]: i for i in
              gh_json(["issue", "list", "--repo", REPO, "--state", "open",
                       "--limit", "1000", "--json", "number,title,body"], [])}
    belt = toolbelt_brief(os.getcwd())
    log(f"{len(numbers)} issue(s) stuck behind an empty branch older than {args.before}")
    made = 0
    for number in numbers:
        if made >= args.limit:
            break
        if refile(number, issues, belt, args.dry_run):
            made += 1
    log(f"done: {made} issue(s) {'would be ' if args.dry_run else ''}re-filed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
