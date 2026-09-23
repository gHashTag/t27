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
    not fast-forward    the remote has commits this bundle does not; a force
                        push here would destroy a bee's work, and this tool
                        never forces
    a bundle that does not verify, or whose prerequisite commits this checkout
                        does not have (fetch the base first)

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
import urllib.error
import urllib.request

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


def wanted(branches: list[dict], limit: int) -> list[dict]:
    """The branches worth asking for: named `queen-<issue>`, ahead of the base.

    Pure, so --self-test can drive it: the ordering is oldest issue first, so a
    run that hits its limit still makes progress through the backlog rather than
    re-taking the same newest few.
    """
    keep = []
    for entry in branches:
        branch = str(entry.get("branch", ""))
        if not BRANCH_RE.match(branch):
            continue
        if int(entry.get("ahead", 0) or 0) <= 0:
            continue
        keep.append(entry)
    keep.sort(key=lambda e: int(BRANCH_RE.match(str(e["branch"])).group(1)))
    return keep[:limit]


def push_one(entry: dict, token: str, dry_run: bool) -> str:
    """One branch, from the container to the remote. Returns a one-word outcome."""
    branch = str(entry["branch"])
    issue = int(BRANCH_RE.match(branch).group(1))
    exported = str(entry.get("head", "") or "")
    there = remote_head(branch)
    if there and exported and there == exported:
        return "already"

    if dry_run:
        log(f"dry-run: would push {branch} ({entry.get('ahead')} commit(s))")
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
        # NEVER --force: the remote having commits this bundle lacks is a bee's
        # work that would be destroyed, and that is a person's decision.
        pushed = git(["push", "origin", f"{ref}:refs/heads/{branch}"])
        git(["update-ref", "-d", ref])
        if pushed.returncode != 0:
            reason = pushed.stderr.strip().splitlines()[-1] if pushed.stderr.strip() else "push failed"
            log(f"skip {branch}: {reason[:200]}")
            return "not-fast-forward" if "fetch first" in pushed.stderr or "non-fast-forward" in pushed.stderr else "push-failed"
    log(f"pushed {branch} ({entry.get('ahead')} commit(s), {len(entry.get('files', []) or [])} file(s))")
    return "pushed"


def self_test() -> int:
    cases = [
        ([{"branch": "queen-2", "ahead": 1}, {"branch": "queen-1", "ahead": 3}], 10,
         ["queen-1", "queen-2"], "oldest issue first"),
        ([{"branch": "queen-1", "ahead": 0}], 10, [], "a branch level with the base carries no work"),
        ([{"branch": "main", "ahead": 5}, {"branch": "queen-x", "ahead": 5}], 10, [],
         "only queen-<issue> branches"),
        ([{"branch": f"queen-{n}", "ahead": 1} for n in range(1, 8)], 3,
         ["queen-1", "queen-2", "queen-3"], "the limit is honoured"),
    ]
    failures = 0
    for branches, limit, expected, why in cases:
        got = [e["branch"] for e in wanted(branches, limit)]
        if got != expected:
            print(f"FAIL ({why}): {got} != {expected}")
            failures += 1
    print("export_push self-test:", "PASS" if failures == 0 else f"{failures} FAILED")
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--limit", type=int, default=10)
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

    branches = wanted(listing.get("branches", []), args.limit)
    log(f"waiting: {listing.get('count')} branch(es); taking {len(branches)} (base {listing.get('base')})")
    counts: dict[str, int] = {}
    for entry in branches:
        outcome = push_one(entry, token, args.dry_run)
        counts[outcome] = counts.get(outcome, 0) + 1
    log("done: " + (", ".join(f"{k}={v}" for k, v in sorted(counts.items())) or "nothing to do"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
