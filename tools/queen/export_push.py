#!/usr/bin/env python3
"""Take the bees' finished work out of the container and push it.

WHY THIS EXISTS

The worker container holds no push credential, by design: a checkout the agents
can write is a checkout whose `.git/config` and hooks they control, so a token
placed there is a token they can take. Instead the container EXPORTS - it serves
a git bundle per branch at `/queen/export` - and the push happens somewhere they
cannot write.

Nothing in the cloud did that push. It was somebody running a script on a
laptop, and when that stopped so did the pipeline: measured 2026-09-23, the last
bee branch reached GitHub at 10:33 the previous day, while 59 issues carried an
`accept` whose work had never left the container. The Queen skips those as "the
work already landed", so they hold their files, the backlog empties, and the
swarm idles at 0 of 20 lanes with a full queue of finished work nobody can see.

WHAT IT DOES

    GET  /queen/export            what is ahead of the base, per branch
    GET  /queen/export/<issue>    that branch as a base64 git bundle
    git fetch <bundle> ; git push origin <branch>

and then `queen-publish.yml`, which runs on a push to `queen-*`, opens the pull
request. So one secret turns the whole chain back on.

WHAT IT REFUSES

    already pushed      the remote head equals the exported head; nothing to do
    not fast-forward    the remote has commits this bundle does not, and the
                        exported head is not the Queen's accepted, newer
                        attempt (below); this tool never destroys a bee's work
    a bundle that does not verify, or whose prerequisite commits this checkout
                        does not have (fetch the base first)

WHY THE LIMIT COUNTS PUSHES, NOT TRIES

Measured 2026-10-06 (#6657): "waiting: 1446 branch(es); taking 10 ... done:
not-fast-forward=10", on every run, for a day. The listing was cut to the ten
oldest issues BEFORE anything was tried, those ten were all refused, and the next
run cut the same ten. 102 heads the Queen had accepted, on open issues, never
left the container; 21 of them were plain fast-forwards the run never reached.
So a branch the remote already carries is dropped before the cut, the Queen's
accepted heads go first, and the limit counts branches pushed, with a separate
cap on tries.

WHEN A NEWER ACCEPTED ATTEMPT REPLACES AN OLDER ONE

The other 73 of those 102 were all the same shape: the remote carried an older
attempt at the issue, never published (71 of 73 had no pull request at all), and
the Queen had since accepted a NEWER attempt that a re-dispatch had cut from the
base instead of from that older tip. Never forcing meant the accepted work could
never arrive, and that the issue was re-offered forever. The tool now replaces
the remote tip only when every one of these holds (`may_supersede`, self-tested):

    the board's verdict is accept AND its judgedHead is the exported head
    the exported head is newer than the remote tip (a newer remote tip is a
        later attempt still waiting for her judgement: never touched)
    no open pull request has the branch as its head

and it destroys nothing: the old tip is first pushed to the tag
`queen-superseded/<issue>/<sha12>`, and the replacement is a
--force-with-lease on exactly the tip it read. The structural fix - a new
attempt starts FROM the remote tip - is the #6657 contract in
specs/queen/control.t27; this is the recovery for the attempts already cut.

Usage:
    python3 tools/queen/export_push.py --dry-run --limit 20
    python3 tools/queen/export_push.py --limit 10
    python3 tools/queen/export_push.py --self-test
"""
from __future__ import annotations

import argparse
import base64
import json
import os
import re
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
# The board reader and "accepted at THIS head" rule have one home: publish.py.
from publish import accepted_at, queen_verdicts  # noqa: E402

QUEEN = os.environ.get(
    "QUEEN_EXPORT_URL",
    "https://trios-agent-server-production.up.railway.app/queen/export",
)
BRANCH_RE = re.compile(r"^queen-(\d+)$")
TIMEOUT = 120


def log(message: str) -> None:
    print(message, flush=True)


def api(path: str, token: str) -> dict:
    request = urllib.request.Request(
        f"{QUEEN}{path}",
        headers={"Authorization": f"Bearer {token}", "Accept": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=TIMEOUT) as response:
        return json.loads(response.read().decode("utf-8"))


def git(args: list[str], **kw) -> subprocess.CompletedProcess:
    return subprocess.run(["git", *args], capture_output=True, text=True, **kw)


def remote_head(branch: str) -> str | None:
    """The sha the remote has for this branch, or None when it has no such branch."""
    done = git(["ls-remote", "origin", f"refs/heads/{branch}"])
    if done.returncode != 0:
        return None
    line = done.stdout.strip()
    return line.split("\t")[0] if line else None


def commits_of(entry: dict) -> int:
    """How far this branch is ahead of the base, as /queen/export names it.

    THE FIELD IS `commits`. This read `ahead` for its whole first day and took
    nothing: the listing said 348 branches were waiting and the run took 0,
    because a name that is not there defaults to zero and zero is filtered out.
    The self-test did not catch it - its fixtures were written from the same
    wrong assumption as the code, so it was checking a guess against itself.

    `ahead` stays accepted as a fallback rather than removed: if the route is
    ever taught to send it, a reader of this function should not have to
    discover which of two names wins.
    """
    for name in ("commits", "ahead"):
        if name in entry:
            try:
                return int(entry[name] or 0)
            except (TypeError, ValueError):
                return 0
    return 0


def remote_heads() -> dict[str, str] | None:
    """Every `queen-*` head the remote has, in ONE call; None when it cannot say."""
    done = git(["ls-remote", "origin", "refs/heads/queen-*"])
    if done.returncode != 0:
        log(f"ls-remote failed: {done.stderr.strip()[-200:]}")
        return None
    heads = {}
    for line in done.stdout.splitlines():
        sha, _, ref = line.partition("\t")
        heads[ref.removeprefix("refs/heads/")] = sha
    return heads


def is_accepted(entry: dict, verdicts: dict[int, dict] | None) -> bool:
    """The Queen accepted the very head this entry exports."""
    match = BRANCH_RE.match(str(entry.get("branch", "")))
    if not match or not verdicts:
        return False
    return accepted_at(verdicts.get(int(match.group(1))), str(entry.get("head", "") or ""),
                       None, False)[0]


def wanted(branches: list[dict], remote: dict[str, str] | None = None,
           verdicts: dict[int, dict] | None = None) -> list[dict]:
    """The branches worth trying, in the order to try them. NOT cut to a limit.

    Pure, so --self-test can drive it. Dropped: anything not `queen-<issue>`,
    anything level with the base, and anything the remote already carries at
    the exported head. Order: the heads the Queen accepted first - those are the
    ones a pull request waits for - then oldest issue first. The caller stops
    after `limit` PUSHES; cutting here, before trying, is what starved it.
    """
    remote = remote or {}
    keep = []
    for entry in branches:
        branch = str(entry.get("branch", ""))
        if not BRANCH_RE.match(branch):
            continue
        if commits_of(entry) <= 0:
            continue
        head = str(entry.get("head", "") or "")
        if head and remote.get(branch) == head:
            continue
        keep.append(entry)
    keep.sort(key=lambda e: (not is_accepted(e, verdicts),
                             int(BRANCH_RE.match(str(e["branch"])).group(1))))
    return keep


def may_supersede(accepted: bool, exported_time: int | None, remote_time: int | None,
                  open_pull_request: bool) -> tuple[bool, str]:
    """Whether a refused push may replace the remote tip. Pure, self-tested.

    Only the Queen's accepted head, only over an OLDER tip, and never under an
    open pull request. A remote tip newer than the export is a later attempt
    still waiting for her: that one is the work, not the leftover.
    """
    if not accepted:
        return False, "the Queen has not accepted the exported head"
    if open_pull_request:
        return False, "an open pull request has this branch as its head"
    if exported_time is None or remote_time is None:
        return False, "cannot date both heads"
    if exported_time <= remote_time:
        return False, "the remote tip is the newer attempt"
    return True, "accepted, newer than the remote tip, no open pull request"


def commit_time(sha: str) -> int | None:
    done = git(["log", "-1", "--format=%ct", sha])
    try:
        return int(done.stdout.strip()) if done.returncode == 0 else None
    except ValueError:
        return None


def open_pr_heads() -> set[str] | None:
    """Head branch names of every open pull request; None when gh cannot say."""
    done = subprocess.run(["gh", "pr", "list", "--state", "open", "--limit", "1000",
                           "--json", "headRefName"], capture_output=True, text=True)
    if done.returncode != 0:
        log(f"gh pr list failed: {done.stderr.strip()[-200:]}")
        return None
    try:
        return {row["headRefName"] for row in json.loads(done.stdout or "[]")}
    except (json.JSONDecodeError, KeyError, TypeError):
        return None


def push_one(entry: dict, token: str, dry_run: bool, accepted: bool = False,
             open_prs: set[str] | None = None) -> str:
    """One branch, from the container to the remote. Returns a one-word outcome."""
    branch = str(entry["branch"])
    issue = int(BRANCH_RE.match(branch).group(1))
    exported = str(entry.get("head", "") or "")
    there = remote_head(branch)
    if there and exported and there == exported:
        return "already"

    if dry_run:
        mark = " [accepted]" if accepted else ""
        log(f"dry-run: would push {branch}{mark} ({commits_of(entry)} commit(s))")
        return "would-push"

    try:
        payload = api(f"/{issue}", token)
    except urllib.error.HTTPError as error:
        log(f"skip {branch}: export said {error.code}")
        return "export-refused"
    bundle_b64 = payload.get("bundleBase64")
    if not bundle_b64:
        log(f"skip {branch}: the export carried no bundle")
        return "no-bundle"

    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, f"{branch}.bundle")
        with open(path, "wb") as handle:
            handle.write(base64.b64decode(bundle_b64))
        verified = git(["bundle", "verify", path])
        if verified.returncode != 0:
            log(f"skip {branch}: {verified.stderr.strip().splitlines()[-1:] or ['bundle did not verify']}")
            return "bad-bundle"
        ref = f"refs/queen-import/{branch}"
        fetched = git(["fetch", "--no-tags", path, f"refs/heads/{branch}:{ref}"])
        if fetched.returncode != 0:
            log(f"skip {branch}: fetch failed: {fetched.stderr.strip()[-200:]}")
            return "fetch-failed"
        # A plain push first: a fast-forward never needs anything else.
        pushed = git(["push", "origin", f"{ref}:refs/heads/{branch}"])
        if pushed.returncode != 0:
            refused = "fetch first" in pushed.stderr or "non-fast-forward" in pushed.stderr
            if not refused:
                git(["update-ref", "-d", ref])
                reason = pushed.stderr.strip().splitlines()[-1] if pushed.stderr.strip() else "push failed"
                log(f"skip {branch}: {reason[:200]}")
                return "push-failed"
            outcome = supersede(branch, ref, there, accepted, open_prs)
            git(["update-ref", "-d", ref])
            return outcome
        git(["update-ref", "-d", ref])
    log(f"pushed {branch} ({commits_of(entry)} commit(s), {entry.get('files', 0)} file(s))")
    return "pushed"


def supersede(branch: str, ref: str, there: str | None, accepted: bool,
              open_prs: set[str] | None) -> str:
    """The remote refused a fast-forward. Replace its tip only by `may_supersede`."""
    if not there:
        log(f"skip {branch}: refused, and the remote tip could not be read")
        return "not-fast-forward"
    git(["fetch", "--no-tags", "origin", there])  # usually already here
    remote_time = commit_time(there)
    ok, why = may_supersede(accepted, commit_time(ref), remote_time,
                            open_prs is None or branch in open_prs)
    if not ok:
        log(f"skip {branch}: not a fast-forward, and {why}")
        return "not-fast-forward"
    issue = BRANCH_RE.match(branch).group(1)
    # Destroy nothing: the old tip is kept where anyone can fetch it.
    tag = f"refs/tags/queen-superseded/{issue}/{there[:12]}"
    kept = git(["push", "origin", f"{there}:{tag}"])
    if kept.returncode != 0 and "already exists" not in kept.stderr:
        log(f"skip {branch}: could not keep the old tip: {kept.stderr.strip()[-200:]}")
        return "keep-failed"
    replaced = git(["push", f"--force-with-lease=refs/heads/{branch}:{there}", "origin",
                    f"{ref}:refs/heads/{branch}"])
    if replaced.returncode != 0:
        log(f"skip {branch}: lease refused: {replaced.stderr.strip()[-200:]}")
        return "lease-refused"
    log(f"superseded {branch}: {there[:9]} kept at {tag.removeprefix('refs/tags/')}; {why}")
    return "superseded"


def self_test() -> int:
    A, B, C = "a" * 40, "b" * 40, "c" * 40
    accept = {"verdict": "accept", "judgedHead": A}
    cases = [
        # The field names are the route's own (queen-export.ts): `commits`.
        # These fixtures said `ahead` and the code read `ahead`, so the pair
        # agreed with each other and disagreed with production.
        ([{"branch": "queen-2", "commits": 1}, {"branch": "queen-1", "commits": 3}], {}, {},
         ["queen-1", "queen-2"], "oldest issue first"),
        ([{"branch": "queen-1", "commits": 0}], {}, {}, [], "a branch level with the base carries no work"),
        ([{"branch": "main", "commits": 5}, {"branch": "queen-x", "commits": 5}], {}, {}, [],
         "only queen-<issue> branches"),
        # The name this file used to read, kept working on purpose.
        ([{"branch": "queen-5", "ahead": 2}], {}, {}, ["queen-5"], "`ahead` still accepted"),
        ([{"branch": "queen-6"}], {}, {}, [], "neither name means nothing to take"),
        # #6657: the cut happened before trying, so ten refusals filled every run.
        ([{"branch": f"queen-{n}", "commits": 1} for n in range(1, 30)], {}, {},
         [f"queen-{n}" for n in range(1, 30)], "nothing is cut before it is tried"),
        ([{"branch": "queen-1", "commits": 1, "head": A}, {"branch": "queen-2", "commits": 1, "head": B}],
         {"queen-1": A, "queen-2": C}, {}, ["queen-2"], "a head the remote already carries is dropped"),
        ([{"branch": "queen-1", "commits": 1, "head": B}, {"branch": "queen-9", "commits": 1, "head": A}],
         {}, {9: accept}, ["queen-9", "queen-1"], "the Queen's accepted head goes first"),
        ([{"branch": "queen-1", "commits": 1, "head": B}, {"branch": "queen-9", "commits": 1, "head": B}],
         {}, {9: accept}, ["queen-1", "queen-9"], "accepted at another head is not accepted"),
    ]
    failures = 0
    for branches, remote, verdicts, expected, why in cases:
        got = [e["branch"] for e in wanted(branches, remote, verdicts)]
        if got != expected:
            print(f"FAIL ({why}): {got} != {expected}")
            failures += 1
    supersede_cases = [
        ((True, 200, 100, False), True, "accepted, newer, no open pull request"),
        ((False, 200, 100, False), False, "not accepted: never replace"),
        ((True, 100, 200, False), False, "the remote tip is the newer attempt"),
        ((True, 100, 100, False), False, "same second: not provably newer"),
        ((True, 200, 100, True), False, "an open pull request heads the branch"),
        ((True, None, 100, False), False, "an undatable head is not replaced"),
        ((True, 200, None, False), False, "an undatable remote tip is not replaced"),
    ]
    for args, expected, why in supersede_cases:
        got = may_supersede(*args)[0]
        if got != expected:
            print(f"FAIL ({why}): may_supersede{args} = {got}")
            failures += 1
    total = len(cases) + len(supersede_cases)
    print("export_push self-test:", "PASS" if failures == 0 else f"{failures} FAILED", f"({total} cases)")
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--limit", type=int, default=10, help="branches to PUSH")
    parser.add_argument("--max-tries", type=int, default=0,
                        help="bundles to ask for at most (default 6 x --limit)")
    parser.add_argument("--budget-seconds", type=int, default=14 * 60)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    token = os.environ.get("QUEEN_EXPORT_TOKEN", "").strip()
    if not token:
        # Not a silent skip: say which secret is missing and what it costs.
        print(
            "::warning::QUEEN_EXPORT_TOKEN is not set, so the bees' work stays "
            "in the container and no pull request can be opened for it. Set it "
            "to the agent server's TRIOS_API_TOKEN.",
            flush=True,
        )
        return 0

    try:
        listing = api("", token)
    except urllib.error.HTTPError as error:
        print(f"::error::the export refused this token ({error.code})", flush=True)
        return 1
    except Exception as error:  # noqa: BLE001 - the container may be restarting
        print(f"::error::could not reach the export: {error}", flush=True)
        return 1

    remote = remote_heads()
    verdicts = queen_verdicts()
    open_prs = open_pr_heads()
    branches = wanted(listing.get("branches", []), remote, verdicts)
    accepted = sum(1 for e in branches if is_accepted(e, verdicts))
    max_tries = args.max_tries or 6 * args.limit
    log(f"waiting: {listing.get('count')} branch(es); {len(branches)} not on the remote at their "
        f"head, {accepted} of them accepted by the Queen; pushing up to {args.limit}, "
        f"trying up to {max_tries} (base {listing.get('base')})")
    if verdicts is None:
        log("the Queen's board could not be read: no accepted-first order, and nothing superseded")
    counts: dict[str, int] = {}
    started = time.monotonic()
    landed = 0
    for tries, entry in enumerate(branches):
        if landed >= args.limit or tries >= max_tries:
            break
        if time.monotonic() - started > args.budget_seconds:
            log(f"stopping: {args.budget_seconds}s budget spent")
            break
        outcome = push_one(entry, token, args.dry_run, is_accepted(entry, verdicts), open_prs)
        counts[outcome] = counts.get(outcome, 0) + 1
        if outcome in ("pushed", "superseded", "would-push"):
            landed += 1
    log("done: " + (", ".join(f"{k}={v}" for k, v in sorted(counts.items())) or "nothing to do"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
