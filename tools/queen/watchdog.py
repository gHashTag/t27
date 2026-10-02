#!/usr/bin/env python3
"""The swarm is not answering. Say so, and put it back.

WHY THIS EXISTS

2026-09-20, measured: the agent server crash-looped every few minutes, and
Railway's `restartPolicyMaxRetries: 10` is a budget - after ten restarts the
platform stops trying and leaves the service dead. It stayed dead for nine
hours. `/queen/status` answered

    {"status":"error","code":502,"message":"Application failed to respond"}

for all of them, and nothing anywhere was reading that answer. The pusher reads
the swarm's OWN numbers; when the swarm cannot answer at all it exits 2, which
turns one scheduled run red and does nothing else.

So this is the outside watchman: it asks one question - does the swarm answer? -
and it is the only thing here allowed to restart the service.

WHAT IT REFUSES TO DO

  restart on one bad probe   a deploy is a few seconds of 502 and is not an
                             outage; two probes a minute apart must both fail
  restart without a token    with no RAILWAY_TOKEN it alarms and stops; it does
                             not pretend to have healed anything
  restart twice in a row     a redeploy that did not help is not fixed by
                             another, so it alarms instead and names the
                             deployment it already tried

Usage:

    python3 tools/queen/watchdog.py --dry-run
    python3 tools/queen/watchdog.py
    python3 tools/queen/watchdog.py --self-test
"""
from __future__ import annotations

import argparse
import datetime
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request

REPO = os.environ.get("WATCHDOG_REPO", "gHashTag/t27")
STATUS = os.environ.get(
    "QUEEN_STATUS_URL",
    "https://trios-agent-server-production.up.railway.app/queen/status",
)
ALARM_TITLE = "The swarm is not answering"
RAILWAY_API = "https://backboard.railway.com/graphql/v2"
PROBE_GAP_SECONDS = 60


def log(message: str) -> None:
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"{stamp} {message}", flush=True)


def probe(url: str = STATUS) -> tuple[bool, str]:
    """(answers, what it said). A 200 carrying `workers` is the only yes.

    The platform answers 502 with a JSON body of its own, so "valid JSON" is not
    the question: the swarm's own shape is.
    """
    try:
        with urllib.request.urlopen(url, timeout=30) as response:
            body = response.read().decode("utf-8", "replace")
            # `file://` has no HTTP status, and the self-test's fixtures are
            # files: a missing status is not a failing one.
            status = getattr(response, "status", None)
            if status is not None and status != 200:
                return False, f"HTTP {status}"
    except urllib.error.HTTPError as error:
        return False, f"HTTP {error.code}: {error.read()[:120].decode('utf-8', 'replace')}"
    except Exception as error:  # noqa: BLE001 - any failure to reach it is a no
        return False, f"unreachable: {error}"
    try:
        payload = json.loads(body)
    except json.JSONDecodeError:
        return False, f"not JSON: {body[:120]}"
    if not isinstance(payload, dict) or "workers" not in payload:
        return False, f"answered without workers: {body[:120]}"
    workers = payload.get("workers") or {}
    return True, (
        f"capacity {workers.get('capacity')}, active {workers.get('active')}"
    )


def sh(args: list[str], timeout: int = 120) -> tuple[int, str]:
    done = subprocess.run(args, capture_output=True, text=True, timeout=timeout,
                          stdin=subprocess.DEVNULL)
    return done.returncode, ((done.stdout or "") + (done.stderr or "")).strip()


def gh_json(args: list[str], default):
    code, out = sh(["gh", *args])
    if code != 0 or not out:
        return default
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        return default


def alarm_issue() -> dict | None:
    for issue in gh_json(["issue", "list", "--repo", REPO, "--state", "open",
                          "--limit", "50", "--search", ALARM_TITLE,
                          "--json", "number,title,body"], []):
        if issue.get("title") == ALARM_TITLE:
            return issue
    return None


def redeploy() -> tuple[bool, str]:
    """Ask Railway to redeploy the service. Needs RAILWAY_TOKEN."""
    token = os.environ.get("RAILWAY_TOKEN", "").strip()
    service = os.environ.get("RAILWAY_SERVICE_ID", "").strip()
    environment = os.environ.get("RAILWAY_ENVIRONMENT_ID", "").strip()
    if not token or not service or not environment:
        return False, ("no RAILWAY_TOKEN / RAILWAY_SERVICE_ID / "
                       "RAILWAY_ENVIRONMENT_ID, so nothing was restarted")
    query = {
        "query": (
            "mutation redeploy($environmentId: String!, $serviceId: String!) {"
            " serviceInstanceRedeploy(environmentId: $environmentId,"
            " serviceId: $serviceId) }"
        ),
        "variables": {"environmentId": environment, "serviceId": service},
    }
    request = urllib.request.Request(
        RAILWAY_API,
        data=json.dumps(query).encode(),
        headers={"Authorization": f"Bearer {token}",
                 "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            body = json.loads(response.read().decode("utf-8", "replace"))
    except Exception as error:  # noqa: BLE001
        return False, f"the redeploy call failed: {error}"
    if body.get("errors"):
        return False, f"Railway refused: {json.dumps(body['errors'])[:200]}"
    return True, "Railway accepted the redeploy"


def self_test() -> int:
    """The probe, against answers whose verdict is known."""
    cases = [
        ("a swarm answer is a yes", '{"workers":{"capacity":20,"active":19}}', True),
        ("the platform's own 502 body is a no",
         '{"status":"error","code":502,"message":"Application failed to respond"}', False),
        ("a 200 without workers is a no", '{"status":"ok"}', False),
        ("prose is a no", "<html>bad gateway</html>", False),
    ]
    bad = 0
    for name, body, want in cases:
        path = os.path.join(os.environ.get("RUNNER_TEMP", "/tmp"), "watchdog-probe.json")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(body)
        got, _ = probe("file://" + path)
        if got != want:
            print(f"  self-test FAILED: {name} -> {got}, expected {want}")
            bad += 1
    if bad:
        return 1
    print(f"ok: {len(cases)} answers, including three a live swarm never sends")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--gap", type=int, default=PROBE_GAP_SECONDS)
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    first, first_said = probe()
    log(f"probe 1: {'answers' if first else 'silent'} - {first_said}")
    if first:
        issue = alarm_issue()
        if issue and not args.dry_run:
            sh(["gh", "issue", "close", str(issue["number"]), "--repo", REPO,
                "--comment", f"The swarm answers again: {first_said}."])
            log(f"closed the alarm #{issue['number']}")
        return 0

    # A deploy is a few seconds of 502 and is not an outage.
    log(f"waiting {args.gap}s before asking again, because a deploy answers 502 too")
    if not args.dry_run:
        time.sleep(args.gap)
    second, second_said = probe()
    log(f"probe 2: {'answers' if second else 'silent'} - {second_said}")
    if second:
        return 0

    issue = alarm_issue()
    already_tried = bool(issue and "Railway accepted the redeploy" in (issue.get("body") or ""))
    if already_tried:
        acted = "a redeploy was already asked for and it did not help; not asking again"
        log(acted)
    elif args.dry_run:
        acted = "dry-run: would ask Railway to redeploy"
        log(acted)
    else:
        ok, acted = redeploy()
        log(("restarted: " if ok else "could not restart: ") + acted)

    body = "\n".join([
        f"`{STATUS}` did not answer twice, {args.gap} seconds apart.",
        "",
        "```",
        f"probe 1: {first_said}",
        f"probe 2: {second_said}",
        "```",
        "",
        f"**What this watchdog did:** {acted}",
        "",
        "Railway's `restartPolicyMaxRetries` is a budget, not a promise: past it "
        "the platform stops restarting and the service stays dead. That is how "
        "the swarm spent nine hours down on 2026-09-20 with nobody reading the "
        "502.",
        "",
        "This issue closes itself when the swarm answers again.",
    ])
    if args.dry_run:
        log("dry-run: would open or update the alarm issue")
        return 1
    path = os.path.join(os.environ.get("RUNNER_TEMP", "/tmp"), "watchdog.md")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(body)
    if issue:
        sh(["gh", "issue", "edit", str(issue["number"]), "--repo", REPO,
            "--body-file", path])
        log(f"updated the alarm #{issue['number']}")
    else:
        code, out = sh(["gh", "issue", "create", "--repo", REPO,
                        "--title", ALARM_TITLE, "--body-file", path])
        log(f"opened the alarm: {out.strip().splitlines()[-1] if out else code}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
