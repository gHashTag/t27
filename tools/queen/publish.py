#!/usr/bin/env python3
"""Open a pull request for the work a bee already pushed. Nothing else did.

WHY THIS EXISTS

Measured 2026-09-20: **401 `queen-*` branches on the remote, and the last bee
pull request was opened on 2026-09-17.** Of the forty most recent branches, 24
carry real commits, 9 carry only a salvage commit and 7 are empty. Every one of
the 24 was invisible: nothing in the system opens a pull request. The Queen
skips issues that ARE pull requests (queen-tick.ts:349) and never creates one,
the container holds no push credential by design, and the job that used to do it
was somebody running a script on a laptop.

So twenty bees ran at ninety percent utilisation for three days and shipped
nothing, and every instrument said the swarm was healthy - because every
instrument was measuring dispatch, not delivery.

WHAT IT REFUSES TO PUBLISH, AND WHY EACH ONE

  no commits          an empty branch is a turn that ended without work
  no diff             commits that cancel out are not a change
  a pull request      already open, closed or merged for that head
  the issue is shut   nobody is waiting for it
  outside the boundary  the issue names one file; a branch that wrote elsewhere
                      fails the Queen's own rule and would fail review
  does not merge      a conflicting branch reports as "waiting for CI" forever,
                      because GitHub runs no required check on it (13 such were
                      closed by hand on 2026-09-19)
  too large           more than MAX_COMMITS or MAX_FILES is not one bee's turn;
                      queen-3678 sits 3269 commits ahead of master

WHAT IT ADDS

One `docs/now/` entry, because every pull request must add exactly one and a bee
cannot know that - its brief names a boundary file and acceptance criteria, and
`docs/now/` is neither. The entry says which branch it came from and does not
pretend a bee wrote it. The commit carries `Closes #N` because L1 TRACEABILITY
reads commit messages, not pull request bodies.

Usage, from a checkout with `origin` fetched:

    python3 tools/queen/publish.py --dry-run --limit 5
    python3 tools/queen/publish.py --limit 5
    python3 tools/queen/publish.py --self-test
"""
from __future__ import annotations

import argparse
import datetime
import json
import os
import re
import subprocess
import sys

REPO = os.environ.get("PUBLISH_REPO", "gHashTag/t27")
BRANCH_RE = re.compile(r"^queen-(\d+)$")
# One bee, one turn, one boundary. Above these a branch is something else.
MAX_COMMITS = 20
MAX_FILES = 25


def sh(args: list[str], timeout: int = 180) -> tuple[int, str]:
    done = subprocess.run(args, capture_output=True, text=True, timeout=timeout,
                          stdin=subprocess.DEVNULL)
    return done.returncode, ((done.stdout or "") + (done.stderr or "")).strip()


def git(*args: str, timeout: int = 180) -> str:
    return sh(["git", *args], timeout=timeout)[1]


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


def heads_with_pull_requests() -> set[str]:
    """Every branch that already has a pull request, in any state."""
    rows = gh_json(["pr", "list", "--repo", REPO, "--state", "all", "--limit", "1000",
                    "--json", "headRefName"], [])
    return {row.get("headRefName", "") for row in rows}


def open_issues() -> dict[int, dict]:
    rows = gh_json(["issue", "list", "--repo", REPO, "--state", "open", "--limit", "1000",
                    "--json", "number,title,body"], [])
    return {row["number"]: row for row in rows}


def boundary_of(issue: dict) -> list[str]:
    body = issue.get("body") or ""
    match = re.search(r"(?ims)^##\s*boundary\s*$(.*?)(?=^#|\Z)", body)
    if not match:
        return []
    return re.findall(r"[\w./-]+\.(?:t27|py|rs|md|ya?ml|json|txt|zig|c|h)", match.group(1))


def branches() -> list[tuple[str, int]]:
    out = git("for-each-ref", "--format=%(refname:short)", "--sort=-committerdate",
              "refs/remotes/origin/queen-*")
    found = []
    for line in out.split("\n"):
        name = line.strip().replace("origin/", "", 1)
        match = BRANCH_RE.match(name)
        if match:
            found.append((name, int(match.group(1))))
    return found


def merges_cleanly(branch: str) -> bool:
    """`git merge-tree` writes conflict markers rather than failing, so read them.

    A conflicting branch is worse than an unpublished one: GitHub runs no
    required check on a pull request it cannot merge, so it reports as waiting
    for CI forever and a human closes it by hand later.
    """
    code, out = sh(["git", "merge-tree", "--write-tree", "origin/master",
                    f"origin/{branch}"])
    return code == 0 and "<<<<<<<" not in out


def slug(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")[:60] or "bee-work"


def entry_for(issue_number: int, title: str, branch: str, files: list[str],
              stat: str, today: str) -> tuple[str, str]:
    path = f"docs/now/{today}-published-{slug(title)}.md"
    body = "\n".join([
        f"# NOW -- {title} (published {today})",
        "",
        f"## A bee's work on #{issue_number}, published from `{branch}` (Closes #{issue_number})",
        "",
        f"- The branch changes {len(files)} file(s): {', '.join('`' + f + '`' for f in files[:6])}"
        + (f" and {len(files) - 6} more" if len(files) > 6 else "") + ".",
        f"- `git diff --stat origin/master...{branch}` reads: {stat}",
        "- This entry is written by the publisher, not by the bee. A pull request must",
        "  add exactly one `docs/now/` entry and a bee has no way to know that: its brief",
        "  names a boundary file and acceptance criteria, and `docs/now/` is neither.",
        "- What this entry does NOT establish: that the work is correct. The gates on the",
        "  pull request judge that, and they are the same gates every other change meets.",
        "",
    ])
    return path, body


def publish(branch: str, issue_number: int, issue: dict, today: str,
            dry_run: bool) -> bool:
    files = [f for f in git("diff", "--name-only",
                            f"origin/master...origin/{branch}").split("\n") if f]
    stat = git("diff", "--shortstat", f"origin/master...origin/{branch}") or "(no stat)"
    bounds = boundary_of(issue)
    if bounds:
        stray = [f for f in files if f not in bounds and not f.startswith("docs/now/")]
        if stray:
            log(f"skip {branch}: wrote outside its boundary: {stray[:3]}")
            return False
    title = issue.get("title") or f"Work on #{issue_number}"
    path, body = entry_for(issue_number, title, branch, files, stat, today)
    if dry_run:
        log(f"dry-run: would publish {branch} for #{issue_number} ({stat}) with {path}")
        return True

    git("checkout", "-B", branch, f"origin/{branch}")
    os.makedirs("docs/now", exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(body)
    git("add", path)
    message = (
        "docs: the coordination entry this branch needs to land\n\n"
        "A pull request must add exactly one docs/now entry and a bee has no way\n"
        "to know that: its brief names a boundary file and acceptance criteria,\n"
        "and docs/now/ is neither. The publisher adds it rather than failing the\n"
        "gate.\n\n"
        f"Closes #{issue_number}\n\n"
        "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
    )
    code, out = sh(["git", "commit", "-q", "-m", message])
    if code != 0:
        log(f"skip {branch}: commit failed: {out[:160]}")
        return False
    code, out = sh(["git", "push", "-q", "origin", branch])
    if code != 0:
        log(f"skip {branch}: push failed: {out[:160]}")
        return False

    pr_body = "\n".join([
        f"Closes #{issue_number}",
        "",
        f"Written by a bee on `{branch}` and published by "
        "`tools/queen/publish.py`. The branch itself is the bee's; the second "
        "commit is the coordination entry every pull request must add, which a "
        "bee has no way to know about.",
        "",
        "```",
        stat,
        "```",
        "",
        "🤖 Generated with [Claude Code](https://claude.com/claude-code)",
    ])
    body_path = os.path.join(os.environ.get("RUNNER_TEMP", "/tmp"),
                             f"pr-{issue_number}.md")
    with open(body_path, "w", encoding="utf-8") as handle:
        handle.write(pr_body)
    code, out = sh(["gh", "pr", "create", "--repo", REPO, "--base", "master",
                    "--head", branch, "--title", title[:120],
                    "--body-file", body_path])
    if code != 0:
        log(f"skip {branch}: gh pr create failed: {out[:200]}")
        return False
    log(f"published {branch} for #{issue_number}: {out.strip().split()[-1]}")
    return True


def self_test() -> int:
    """The refusals, against branches whose verdict is known."""
    checks = [
        ("a queen branch name parses", bool(BRANCH_RE.match("queen-4286")), True),
        ("a feature branch does not", bool(BRANCH_RE.match("feat/publisher")), False),
        ("a suffixed name does not", bool(BRANCH_RE.match("queen-4286-retry")), False),
        ("a boundary is read from the body",
         boundary_of({"body": "## Boundary\n\nspecs/a.t27\n"}) == ["specs/a.t27"], True),
        ("no boundary section reads as none",
         boundary_of({"body": "no section here"}) == [], True),
        ("a slug is a filename",
         slug("Restore the 1 function(s) dropped!") == "restore-the-1-function-s-dropped", True),
    ]
    bad = 0
    for name, got, want in checks:
        if got != want:
            print(f"  self-test FAILED: {name} -> {got}, expected {want}")
            bad += 1
    if bad:
        return 1
    print(f"ok: {len(checks)} shapes, including two branch names this must NOT take")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--limit", type=int, default=5)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    today = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")
    git("fetch", "-q", "origin", "master", timeout=600)
    git("fetch", "-q", "origin", "refs/heads/queen-*:refs/remotes/origin/queen-*",
        "--prune", timeout=900)

    have_pr = heads_with_pull_requests()
    issues = open_issues()
    if not issues:
        print("could not run: `gh issue list` returned nothing, which this "
              "repository has never had", file=sys.stderr)
        return 2

    counts = {"no commits": 0, "already a PR": 0, "issue not open": 0,
              "too large": 0, "conflicts": 0, "published": 0, "refused": 0}
    for branch, number in branches():
        if counts["published"] >= args.limit:
            break
        if branch in have_pr:
            counts["already a PR"] += 1
            continue
        if number not in issues:
            counts["issue not open"] += 1
            continue
        ahead = git("rev-list", "--count", f"origin/master..origin/{branch}")
        if not ahead.isdigit() or int(ahead) == 0:
            counts["no commits"] += 1
            continue
        files = [f for f in git("diff", "--name-only",
                                f"origin/master...origin/{branch}").split("\n") if f]
        if not files:
            counts["no commits"] += 1
            continue
        if int(ahead) > MAX_COMMITS or len(files) > MAX_FILES:
            log(f"skip {branch}: {ahead} commit(s) over {len(files)} file(s) is not one turn")
            counts["too large"] += 1
            continue
        if not merges_cleanly(branch):
            log(f"skip {branch}: conflicts with master, and GitHub runs no required "
                "check on a branch it cannot merge")
            counts["conflicts"] += 1
            continue
        if publish(branch, number, issues[number], today, args.dry_run):
            counts["published"] += 1
        else:
            counts["refused"] += 1

    log("done: " + ", ".join(f"{k}={v}" for k, v in counts.items()))
    return 0


if __name__ == "__main__":
    sys.exit(main())
