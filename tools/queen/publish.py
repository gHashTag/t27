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

THE QUEEN ONLY MANAGES. SHE DOES NOT MERGE.

Owner's rule, 2026-10-02, overriding the rule of 2026-10-01: "the Queen must
not merge by herself!! the Queen only manages!" She assigns, judges, accepts
and sends back. A merge happens only after a REVIEWER BEE - a code-review agent
with real tools - has reviewed and verified the pull request, and says so with
the `bee-reviewed` label that `auto-merge-ready-prs.yml` requires.

The rule this replaces (2026-10-01, #5422) was "her accept IS the merge": this
script armed `gh pr merge --auto --squash` on every head she accepted, so her
verdict was the last thing standing between a bee's branch and master. Her
verdict is a judgement about whether the work answers the issue; it is not a
review of the code by something that ran it. So now:

  - a branch is still PUBLISHED only when `/queen/public-board` carries
    `accept` for its issue AND the head she judged (`judgedHead`) is the
    branch's head, or its parent under this publisher's one docs/now commit.
    Her accept decides what becomes a pull request - that is managing.
  - this script NEVER arms auto-merge and never merges. The only `gh pr merge`
    it runs is `--disable-auto`, and `--self-test` reads this module's own
    source to prove there is no other.
  - any open bee pull request that carries auto-merge is DISARMED, whatever
    her verdict, because whoever armed it was not a reviewer bee.

If the board cannot be read, nothing is published: a missing answer is not a
verdict.

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
        # `check` (tools/check_now_entry_shape.py) requires the heading to END in
        # `(YYYY-MM-DD)`. `(published DATE)` failed it on every pull request this
        # module opened, so the bees refused all of them; self_test now runs
        # that checker over this entry rather than restating its rule here.
        f"# NOW -- Published: {title} ({today})",
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
    # NO AUTO-MERGE. Until 2026-10-02 this armed `gh pr merge --auto --squash`
    # here, which made the Queen's accept the merge. The owner's rule of that
    # day: she only manages; a reviewer bee reviews, and only its `bee-reviewed`
    # label lets the scheduled merger take the pull request.
    log(f"published {branch} for #{issue_number}: {url} "
        "(waits for a reviewer bee; nothing here merges it)")
    return True


def reconcile_open(dry_run: bool) -> dict[str, int]:
    """Disarm auto-merge on every open bee pull request. Never arm one.

    Before 2026-10-02 this armed what the Queen accepted and disarmed the rest.
    Now nothing she decides arms a merge, so any bee pull request carrying
    auto-merge was armed by something that is not a reviewer bee - this
    publisher's own earlier runs, most likely - and is taken back.
    """
    rows = gh_json(["pr", "list", "--repo", REPO, "--state", "open", "--limit", "300",
                    "--json", "number,headRefName,autoMergeRequest"], [])
    counts = {"disarmed": 0, "unarmed": 0}
    for row in rows:
        if not BRANCH_RE.match(row.get("headRefName", "")):
            continue
        if row.get("autoMergeRequest"):
            if not dry_run:
                sh(["gh", "pr", "merge", str(row["number"]), "--repo", REPO,
                    "--disable-auto"])
            log(f"disarm #{row['number']} ({row['headRefName']}): the Queen does "
                "not merge, and only a reviewer bee's `bee-reviewed` label does")
            counts["disarmed"] += 1
        else:
            counts["unarmed"] += 1
    return counts


def merge_calls(source: str) -> list[list[str]]:
    """Every literal argv in `source` that runs `gh pr merge`. Pure, for the self-test.

    Reads the module as a syntax tree, so a comment or a docstring that names
    `gh pr merge --auto` is not a call, and a list literal that is one cannot
    hide behind formatting.
    """
    import ast
    found = []
    for node in ast.walk(ast.parse(source)):
        # A tuple is an argv too: `sh(list(("gh", "pr", "merge", ...)))`.
        if not isinstance(node, (ast.List, ast.Tuple)):
            continue
        words = [e.value for e in node.elts
                 if isinstance(e, ast.Constant) and isinstance(e.value, str)]
        if words[:3] == "gh pr merge".split():
            found.append(words)
    return found


def api_merge_routes(source: str) -> list[str]:
    """String constants that reach a merge without `gh pr merge`. Pure, for the self-test.

    `gh api -X PUT repos/.../pulls/N/merge` and the GraphQL mutations merge or
    arm a pull request just as well, and `merge_calls` cannot see them. An
    f-string's literal parts are constants too, so `f".../pulls/{n}/merge"` is
    found by its "/merge" tail. The bodies of this function and of
    `self_test` are skipped: they hold the pattern and its fixtures, and
    neither runs during a publish.
    """
    import ast
    import re
    route = re.compile(r"/merge$|enablePullRequestAutoMerge|mergePullRequest")
    tree = ast.parse(source)
    skipped = {id(n) for f in ast.walk(tree)
               if isinstance(f, ast.FunctionDef) and f.name in ("api_merge_routes", "self_test")
               for n in ast.walk(f)}
    return [node.value for node in ast.walk(tree)
            if id(node) not in skipped
            and isinstance(node, ast.Constant) and isinstance(node.value, str)
            and route.search(node.value)]


def only_disarms(calls: list[list[str]]) -> bool:
    """True when every `gh pr merge` call is `--disable-auto` and nothing else."""
    forbidden = {"--auto", "--squash", "--merge", "--rebase", "--admin"}
    return all("--disable-auto" in call and not forbidden & set(call) for call in calls)


def entry_shape_faults(title: str, heading: str | None = None) -> list[str]:
    """What `check` would say about the entry `entry_for` writes for `title`.

    The rule lives in tools/check_now_entry_shape.py and is imported, not
    copied: a second copy is how this module came to write a heading the gate
    refuses. `heading`, when given, replaces the first line -- the negative
    control that proves the gate can still say no.
    """
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
    import check_now_entry_shape
    path, body = entry_for(4286, title, "queen-4286", ["specs/a.t27"],
                           "1 file changed", "2026-10-03")
    if heading is not None:
        body = "\n".join([heading] + body.split("\n")[1:])
    return check_now_entry_shape.check_entry(path, body)


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
        ("this module runs `gh pr merge` only to disarm",
         only_disarms(merge_calls(open(__file__, encoding="utf-8").read())), True),
        ("and it does run that disarm",
         len(merge_calls(open(__file__, encoding="utf-8").read())) >= 1, True),
        ("an arming call is caught",
         only_disarms(merge_calls('x = ["gh", "pr", "merge", u, "--auto", "--squash"]')), False),
        ("a plain merge is caught",
         only_disarms(merge_calls('x = ["gh", "pr", "merge", n, "--merge"]')), False),
        ("nor does it reach a merge through `gh api`",
         api_merge_routes(open(__file__, encoding="utf-8").read()), []),
        ("an arming tuple is caught",
         only_disarms(merge_calls('sh(list(("gh", "pr", "merge", u, "--auto")))')), False),
        ("a REST merge is caught",
         api_merge_routes('sh(["gh", "api", "-X", "PUT", f"repos/{R}/pulls/{n}/merge"])'),
         ["/merge"]),
        ("a GraphQL auto-merge is caught",
         bool(api_merge_routes('q = "mutation { enablePullRequestAutoMerge(input: $i) { x } }"')),
         True),
        ("a comment naming --auto is not a call",
         merge_calls('# gh pr merge --auto\ny = 1'), []),
        ("a slug is a filename",
         slug("Restore the 1 function(s) dropped!") == "restore-the-1-function-s-dropped", True),
        ("the entry this writes passes the `check` gate",
         entry_shape_faults("Restore the 1 function(s) dropped (gen)"), []),
        ("and that gate is not a rubber stamp",
         bool(entry_shape_faults("x", heading="# NOW -- x (published 2026-10-03)")), True),
    ]
    bad = 0
    for name, got, want in checks:
        if got != want:
            print(f"  self-test FAILED: {name} -> {got}, expected {want}")
            bad += 1
    if bad:
        return 1
    print(f"ok: {len(checks)} shapes, including two branch names this must NOT take "
          "and no `gh pr merge` that is not a disarm")
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
              "them nothing may be published", file=sys.stderr)
        return 2
    gated = reconcile_open(args.dry_run)
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
