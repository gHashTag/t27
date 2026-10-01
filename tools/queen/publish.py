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

THE QUEEN'S ACCEPT IS THE MERGE, AND NOTHING ELSE IS

Until 2026-10-01 this published every branch with commits and armed auto-merge
on it, and never asked the Queen: 249 bee pull requests merged and 17 more
armed, none of them gated on her verdict, while 216 of her cards sat in review.
The owner's rule: no person is needed once she has approved - her accept merges
by itself - and the one thing that must hold is that she checked.

So a branch is published only when `/queen/public-board` carries `accept` for
its issue AND the head she judged (`judgedHead`) is the branch's head, or its
parent under this publisher's one docs/now commit. Auto-merge is armed with
`--match-head-commit`, so a push after the accept cannot ride through on it.
An open bee pull request whose head she has not accepted is DISARMED. If the
board cannot be read, nothing is published and nothing is disarmed: a missing
answer is not a verdict either way.

WHAT IT REFUSES TO PUBLISH, AND WHY EACH ONE

  not accepted        the Queen has not accepted this head (no verdict, a
                      send-back, an escalation, or an accept of an older head)
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
import time

REPO = os.environ.get("PUBLISH_REPO", "gHashTag/t27")
QUEEN_API = os.environ.get(
    "QUEEN_API", "https://trios-agent-server-production.up.railway.app"
).rstrip("/")
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
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


def gh_json(args: list[str], default, attempts: int = 3):
    """Parse `gh`'s STDOUT only, retrying a failed call, and say why it failed.

    This used to parse stdout and stderr joined, and return the default on any
    failure without a word. From 2026-09-20 19:02 every scheduled run exited
    "`gh issue list` returned nothing" and published nothing for sixteen hours,
    while the same command answered in ten seconds from a laptop: a warning on
    stderr, a GraphQL 502 on a thousand issue bodies, an expired token - any of
    them reads the same when the cause is thrown away.
    """
    last = ""
    for attempt in range(1, attempts + 1):
        try:
            done = subprocess.run(["gh", *args], capture_output=True, text=True,
                                  timeout=300, stdin=subprocess.DEVNULL)
        except subprocess.TimeoutExpired:
            last = "timed out after 300s"
        else:
            if done.returncode == 0 and done.stdout.strip():
                try:
                    return json.loads(done.stdout)
                except json.JSONDecodeError as error:
                    last = f"stdout is not JSON ({error}): {done.stdout[:200]!r}"
            else:
                last = f"exit {done.returncode}: {(done.stderr or done.stdout).strip()[:400]}"
        log(f"gh {' '.join(args[:2])} failed (attempt {attempt}/{attempts}): {last}")
        if attempt < attempts:
            time.sleep(10 * attempt)
    return default


def queen_verdicts() -> dict[int, dict] | None:
    """Issue number -> {verdict, judgedHead}, from the Queen's public board.

    None when the board cannot be read or does not carry verdicts at all - a
    supervisor deployed before the field existed - so the caller can tell
    "she has not accepted this" from "nobody could ask her".
    """
    import urllib.request
    url = f"{QUEEN_API}/queen/public-board"
    for attempt in range(1, 4):
        try:
            with urllib.request.urlopen(url, timeout=60) as response:
                board = json.load(response)
            break
        except Exception as error:  # noqa: BLE001 - any failure is "could not ask"
            log(f"queen board unreadable (attempt {attempt}/3): {error}")
            time.sleep(5 * attempt)
    else:
        return None
    cards = board.get("cards") if isinstance(board, dict) else None
    if not isinstance(cards, list) or not cards:
        return None
    if not any("verdict" in card for card in cards if isinstance(card, dict)):
        log("queen board carries no verdict field: the supervisor predates the gate")
        return None
    return {
        card["number"]: {"verdict": card.get("verdict"),
                         "judgedHead": card.get("judgedHead")}
        for card in cards
        if isinstance(card, dict) and isinstance(card.get("number"), int)
    }


def accepted_at(verdict: dict | None, head: str, parent: str | None,
                head_only_docs_now: bool) -> tuple[bool, str]:
    """Whether the Queen accepted THIS head. Pure, so the self-test can drive it.

    `parent` and `head_only_docs_now` describe the publisher's own commit: once
    published, the branch head is the judged head plus one docs/now entry, and
    that entry is not new work.
    """
    if not verdict or not verdict.get("verdict"):
        return False, "the Queen has no verdict on it"
    if verdict["verdict"] != "accept":
        return False, f"the Queen's verdict is {verdict['verdict']}"
    judged = verdict.get("judgedHead") or ""
    if not SHA_RE.match(judged):
        return False, "accepted, but the board names no head it was judged at"
    if head == judged:
        return True, "accepted at this head"
    if parent == judged and head_only_docs_now:
        return True, "accepted at the parent of the publisher's docs/now entry"
    return False, f"accepted at {judged[:9]}, but the branch is at {head[:9]}"


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

    # IN A WORKTREE OF ITS OWN, never in the checkout this runs from. Measured
    # 2026-09-21 on #4338: the publisher ran from a checkout that carried its
    # own copy of tools/queen/publish.py, `checkout -B` switched branches in
    # place, and the file travelled into the bee's branch - outside the issue's
    # boundary, which named one spec. The pull request then conflicted with
    # master on a file the bee never touched. A fresh worktree cut from the
    # bee's own branch holds exactly what that branch holds and nothing else.
    import tempfile
    workdir = tempfile.mkdtemp(prefix=f"publish-{issue_number}-")
    code, out = sh(["git", "worktree", "add", "--force", "-B", branch, workdir,
                    f"origin/{branch}"])
    if code != 0:
        log(f"skip {branch}: could not cut a worktree: {out[:160]}")
        return False
    try:
        return _commit_push_and_open(branch, issue_number, title, path, body,
                                     stat, workdir)
    finally:
        sh(["git", "worktree", "remove", "--force", workdir])


def _commit_push_and_open(branch: str, issue_number: int, title: str, path: str,
                          body: str, stat: str, workdir: str) -> bool:
    os.makedirs(os.path.join(workdir, "docs", "now"), exist_ok=True)
    with open(os.path.join(workdir, path), "w", encoding="utf-8") as handle:
        handle.write(body)
    sh(["git", "-C", workdir, "add", path])
    message = (
        "docs: the coordination entry this branch needs to land\n\n"
        "A pull request must add exactly one docs/now entry and a bee has no way\n"
        "to know that: its brief names a boundary file and acceptance criteria,\n"
        "and docs/now/ is neither. The publisher adds it rather than failing the\n"
        "gate.\n\n"
        f"Closes #{issue_number}\n\n"
        "Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>"
    )
    code, out = sh(["git", "-C", workdir, "commit", "-q", "-m", message])
    if code != 0:
        log(f"skip {branch}: commit failed: {out[:160]}")
        return False
    code, out = sh(["git", "-C", workdir, "push", "-q", "origin", branch])
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
    url = out.strip().split()[-1]
    # ARM AUTO-MERGE IMMEDIATELY, while every check is still pending. GitHub
    # refuses `--auto` on a pull request whose checks have already settled into
    # an unstable state - "Pull request is in unstable status" - so the moment
    # to ask is now, not on a later sweep. A refusal here is not a failure of
    # the publish: the pull request exists either way, and the scheduled merger
    # can still take it.
    head = git("-C", workdir, "rev-parse", "HEAD")
    armed = sh(["gh", "pr", "merge", url, "--repo", REPO, "--auto", "--squash",
                "--match-head-commit", head])
    log(f"published {branch} for #{issue_number}: {url}"
        + ("" if armed[0] == 0 else f" (auto-merge not armed: {armed[1][:60]})"))
    return True


def reconcile_open(verdicts: dict[int, dict], dry_run: bool) -> dict[str, int]:
    """Arm what she accepted, disarm what she did not, on open bee PRs.

    The 17 pull requests armed before this gate existed were armed without her;
    this is what takes that back, and what arms one she accepts later.
    """
    rows = gh_json(["pr", "list", "--repo", REPO, "--state", "open", "--limit", "300",
                    "--json", "number,headRefName,headRefOid,autoMergeRequest"], [])
    counts = {"armed": 0, "disarmed": 0, "kept": 0, "waiting": 0}
    for row in rows:
        match = BRANCH_RE.match(row.get("headRefName", ""))
        if not match:
            continue
        number, branch = int(match.group(1)), row["headRefName"]
        head = row.get("headRefOid", "")
        parent = git("rev-parse", f"origin/{branch}^") or None
        only_docs = bool(parent) and all(
            f.startswith("docs/now/")
            for f in git("diff", "--name-only", f"origin/{branch}^", f"origin/{branch}").split("\n")
            if f)
        ok, why = accepted_at(verdicts.get(number), head, parent, only_docs)
        armed = bool(row.get("autoMergeRequest"))
        if ok and not armed:
            if not dry_run:
                sh(["gh", "pr", "merge", str(row["number"]), "--repo", REPO, "--auto",
                    "--squash", "--match-head-commit", head])
            log(f"arm #{row['number']} ({branch}): {why}")
            counts["armed"] += 1
        elif not ok and armed:
            if not dry_run:
                sh(["gh", "pr", "merge", str(row["number"]), "--repo", REPO,
                    "--disable-auto"])
            log(f"disarm #{row['number']} ({branch}): {why}")
            counts["disarmed"] += 1
        elif ok:
            counts["kept"] += 1
        else:
            counts["waiting"] += 1
    return counts


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
        ("the Queen's accept at this head publishes",
         accepted_at({"verdict": "accept", "judgedHead": "a" * 40}, "a" * 40, None, False)[0], True),
        ("an accept of an OLDER head does not",
         accepted_at({"verdict": "accept", "judgedHead": "a" * 40}, "b" * 40, None, False)[0], False),
        ("the publisher's own docs/now commit over the judged head does",
         accepted_at({"verdict": "accept", "judgedHead": "a" * 40}, "b" * 40, "a" * 40, True)[0], True),
        ("but not when that commit touches more than docs/now",
         accepted_at({"verdict": "accept", "judgedHead": "a" * 40}, "b" * 40, "a" * 40, False)[0], False),
        ("a send-back does not publish",
         accepted_at({"verdict": "sendBack", "judgedHead": "a" * 40}, "a" * 40, None, False)[0], False),
        ("an escalation does not publish",
         accepted_at({"verdict": "escalate", "judgedHead": "a" * 40}, "a" * 40, None, False)[0], False),
        ("no verdict does not publish", accepted_at(None, "a" * 40, None, False)[0], False),
        ("an accept with no head does not publish",
         accepted_at({"verdict": "accept"}, "a" * 40, None, False)[0], False),
        ("an accept with a short head does not publish",
         accepted_at({"verdict": "accept", "judgedHead": "a" * 7}, "a" * 7, None, False)[0], False),
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

    verdicts = queen_verdicts()
    if verdicts is None:
        print("could not run: the Queen's verdicts are unreadable, and without "
              "them nothing may be published or disarmed", file=sys.stderr)
        return 2
    gated = reconcile_open(verdicts, args.dry_run)
    log("open bee pull requests: " + ", ".join(f"{k}={v}" for k, v in gated.items()))

    have_pr = heads_with_pull_requests()
    issues = open_issues()
    if not issues:
        print("could not run: `gh issue list` returned nothing, which this "
              "repository has never had", file=sys.stderr)
        return 2

    counts = {"not accepted": 0, "no commits": 0, "already a PR": 0, "issue not open": 0,
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
        head = git("rev-parse", f"origin/{branch}")
        ok, why = accepted_at(verdicts.get(number), head, None, False)
        if not ok:
            counts["not accepted"] += 1
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
        # A NEW WORKFLOW MOVES TWO LEDGERS a bee has no way to know about: the
        # census (tools/census/*.txt, checked by `tri census pin --gate`) and
        # the classification in scripts/ci/check_pr_branch_filters.py. Each has
        # turned master red before when a workflow landed without it (#4303,
        # #4319). Measured 2026-09-21 on #4498. Such a branch is left for a
        # person, and said so, rather than published into a guaranteed red.
        adds_workflow = [f for f in files if f.startswith(".github/workflows/")]
        if adds_workflow:
            log(f"skip {branch}: it changes {adds_workflow[0]}, which moves the census "
                "and the gate-topology ledger - a person's change, not a publish")
            counts["refused"] += 1
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
