#!/usr/bin/env python3
"""
Pusher — a scheduled reader of system health that files issues when the
system stops evolving.

Each rule carries a `command` field so a reader can independently verify
the condition. The tool runs in three modes:

  --self-test   exercise rules against known readings (must print "ok")
  --dry-run     print a JSON reading and file nothing
  (default)     read live status, evaluate rules, file issues
"""

import argparse
import json
import os
import subprocess
import sys
import time
import urllib.request
from dataclasses import dataclass, asdict
from typing import Any, Callable, Optional

STATUS_URL = os.environ.get(
    "QUEEN_STATUS_URL",
    "https://trios-agent-server-production.up.railway.app/queen/status",
)


@dataclass
class Rule:
    name: str
    command: str
    check: Callable[[dict], Optional[str]]  # returns issue body or None


def fetch_status() -> dict:
    """Fetch the queen status endpoint."""
    with urllib.request.urlopen(STATUS_URL, timeout=30) as resp:
        return json.load(resp)


def rule_feed_frequency(reading: dict) -> Optional[str]:
    """
    The feeder should fire roughly every 36 hours. If it fires twice in
    12 hours, something is wrong with the schedule or the feeder is
    re-running spuriously.
    """
    # The status endpoint doesn't directly expose feeder runs, so we
    # infer from the queue state and lastTick timing.
    # In self-test we'll inject a synthetic reading with a 'feeder_runs'
    # field containing timestamps.
    feeder_runs = reading.get("feeder_runs", [])
    if len(feeder_runs) < 2:
        return None
    # Check if any two consecutive runs are < 12 hours apart
    for i in range(1, len(feeder_runs)):
        delta = feeder_runs[i] - feeder_runs[i - 1]
        if delta < 12 * 3600:  # 12 hours in seconds
            return (
                f"## Feeder frequency anomaly\n\n"
                f"Feeder ran twice within {delta / 3600:.1f} hours "
                f"(expected ~36h cadence).\n\n"
                f"Last runs (unix): {feeder_runs[-2:]}\n\n"
                f"**Verify:** `{RuleSet.rules[0].command}`"
            )
    return None


def rule_merge_failures(reading: dict) -> Optional[str]:
    """
    The batch merge should succeed at least once every 48 hours.
    If it has failed every run for 2+ days, the merge lane is stuck.
    """
    merge_runs = reading.get("merge_runs", [])
    if not merge_runs:
        return None
    now = reading.get("now", time.time())
    # Check if there's been a successful merge in the last 48 hours
    recent_success = any(
        r.get("success") and (now - r.get("timestamp", 0)) < 48 * 3600
        for r in merge_runs
    )
    if not recent_success:
        last_run = merge_runs[-1]
        return (
            f"## Batch merge stalled\n\n"
            f"No successful merge in 48+ hours. Last run: "
            f"{'success' if last_run.get('success') else 'failed'} "
            f"at {last_run.get('timestamp')}\n\n"
            f"**Verify:** `{RuleSet.rules[1].command}`"
        )
    return None


def rule_worker_utilization(reading: dict) -> Optional[str]:
    """
    If there are claimed issues but no lanes running, workers are stuck
    or the scheduler isn't dispatching.
    """
    queue = reading.get("queue", {})
    workers = reading.get("workers", {})
    claimed = queue.get("claimed", 0)
    active = workers.get("active", 0)
    capacity = workers.get("capacity", 0)
    refusal = (reading.get("lastTick") or {}).get("refusal", "")

    # Trigger: claimed > 0, active == 0, capacity > 0
    if claimed > 0 and active == 0 and capacity > 0:
        return (
            f"## Workers idle with claimed work\n\n"
            f"Queue claims {claimed} issues but 0/{capacity} lanes active.\n"
            f"Refusal: {refusal or 'none'}\n\n"
            f"**Verify:** `{RuleSet.rules[2].command}`"
        )
    return None


def rule_stuck_attempts(reading: dict) -> Optional[str]:
    """
    Issues held by attempts that have exhausted retries indicate the
    retry loop isn't resolving (e.g., codegen consistently fails).
    """
    stuck = reading.get("stuck_attempts", 0)
    if stuck > 10:
        return (
            f"## Stuck attempts accumulating\n\n"
            f"{stuck} issues held by attempts that spent all retries.\n"
            f"This usually means generated code consistently fails checks.\n\n"
            f"**Verify:** `{RuleSet.rules[3].command}`"
        )
    return None


def rule_candidate_pileup(reading: dict) -> Optional[str]:
    """
    Many candidates but few claimed means the selector isn't picking
    work, or the queue is clogged with unpickable issues.
    """
    queue = reading.get("queue", {})
    candidates = queue.get("candidates", 0)
    claimed = queue.get("claimed", 0)
    if candidates > 500 and claimed < 50:
        return (
            f"## Candidate pileup\n\n"
            f"{candidates} candidates but only {claimed} claimed.\n"
            f"Selector may be rejecting everything or queue is stale.\n\n"
            f"**Verify:** `{RuleSet.rules[4].command}`"
        )
    return None


def rule_refusal_loop(reading: dict) -> Optional[str]:
    """
    If the scheduler keeps refusing with the same reason, it's not
    making progress.
    """
    last_tick = reading.get("lastTick", {})
    refusal = last_tick.get("refusal", "")
    refusal_history = reading.get("refusal_history", [])

    if not refusal:
        return None

    # Count consecutive same refusals
    consecutive = 1
    for r in reversed(refusal_history):
        if r == refusal:
            consecutive += 1
        else:
            break

    if consecutive >= 5:
        return (
            f"## Scheduler refusal loop\n\n"
            f"Same refusal '{refusal}' for {consecutive} consecutive ticks.\n"
            f"The system is not evolving.\n\n"
            f"**Verify:** `{RuleSet.rules[5].command}`"
        )
    return None


class RuleSet:
    rules: list[Rule] = [
        Rule(
            name="feed_frequency",
            command="gh api /repos/gHashTag/t27/actions/runs --jq '.workflow_runs[] | select(.name==\"queen-feed-empty-bodies\") | .created_at' | head -5",
            check=rule_feed_frequency,
        ),
        Rule(
            name="merge_failures",
            command="gh api /repos/gHashTag/t27/actions/runs --jq '.workflow_runs[] | select(.name==\"queen-batch-merge\") | {conclusion, created_at}' | head -10",
            check=rule_merge_failures,
        ),
        Rule(
            name="worker_utilization",
            command="curl -s $QUEEN_STATUS_URL | jq '{claimed: .queue.claimed, active: .workers.active, capacity: .workers.capacity, refusal: .lastTick.refusal}'",
            check=rule_worker_utilization,
        ),
        Rule(
            name="stuck_attempts",
            command="curl -s $QUEEN_STATUS_URL | jq '.stuck_attempts // 0'",
            check=rule_stuck_attempts,
        ),
        Rule(
            name="candidate_pileup",
            command="curl -s $QUEEN_STATUS_URL | jq '{candidates: .queue.candidates, claimed: .queue.claimed}'",
            check=rule_candidate_pileup,
        ),
        Rule(
            name="refusal_loop",
            command="curl -s $QUEEN_STATUS_URL | jq '.lastTick.refusal'",
            check=rule_refusal_loop,
        ),
    ]


def evaluate_rules(reading: dict) -> list[tuple[str, str]]:
    """Run all rules against a reading. Returns list of (rule_name, issue_body)."""
    issues = []
    for rule in RuleSet.rules:
        body = rule.check(reading)
        if body:
            issues.append((rule.name, body))
    return issues


def file_issue(title: str, body: str, labels: list[str]) -> None:
    """Create a GitHub issue via gh CLI."""
    cmd = [
        "gh",
        "issue",
        "create",
        "--repo",
        "gHashTag/t27",
        "--title",
        title,
        "--body",
        body,
    ]
    for label in labels:
        cmd.extend(["--label", label])
    subprocess.run(cmd, check=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="Pusher — system health monitor")
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run rules against known readings and verify expected verdicts",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Fetch live status, evaluate rules, print JSON reading, file nothing",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    reading = fetch_status()

    if args.dry_run:
        # Add metadata for dry-run output
        reading["_meta"] = {
            "fetched_at": time.time(),
            "rules_evaluated": [r.name for r in RuleSet.rules],
        }
        print(json.dumps(reading, indent=2))
        return 0

    # Default mode: evaluate and file issues
    issues = evaluate_rules(reading)
    for rule_name, body in issues:
        title = f"[pusher] {rule_name.replace('_', ' ').title()}"
        file_issue(title, body, ["pusher", "automated"])
        print(f"Filed issue for {rule_name}")

    if not issues:
        print("No issues filed — all rules passed")

    return 0


def self_test() -> int:
    """Exercise rules against synthetic readings with known verdicts."""
    now = time.time()

    # Reading 1: Healthy system — no rules should fire
    healthy = {
        "feeder_runs": [now - 36 * 3600, now - 72 * 3600],
        "merge_runs": [
            {"timestamp": now - 12 * 3600, "success": True},
            {"timestamp": now - 36 * 3600, "success": True},
        ],
        "queue": {"claimed": 5, "candidates": 100},
        "workers": {"active": 3, "capacity": 8},
        "lastTick": {"refusal": "nothing to choose"},
        "refusal_history": ["nothing to choose"] * 3,
        "stuck_attempts": 0,
        "now": now,
    }

    # Reading 2: Feeder too frequent
    feeder_bad = {
        **healthy,
        "feeder_runs": [now - 6 * 3600, now - 12 * 3600],  # 6h apart
    }

    # Reading 3: Merge stalled
    merge_bad = {
        **healthy,
        "merge_runs": [
            {"timestamp": now - 60 * 3600, "success": False},
            {"timestamp": now - 84 * 3600, "success": False},
        ],
    }

    # Reading 4: Workers idle with claimed work
    workers_idle = {
        **healthy,
        "queue": {"claimed": 71, "candidates": 673},
        "workers": {"active": 0, "capacity": 8},
        "lastTick": {"refusal": "nothing to choose"},
    }

    # Reading 5: Stuck attempts
    stuck_bad = {
        **healthy,
        "stuck_attempts": 71,
    }

    # Reading 6: Candidate pileup
    pileup_bad = {
        **healthy,
        "queue": {"claimed": 10, "candidates": 800},
    }

    # Reading 7: Refusal loop
    refusal_bad = {
        **healthy,
        "lastTick": {"refusal": "no eligible candidates"},
        "refusal_history": ["no eligible candidates"] * 10,
    }

    test_cases = [
        ("healthy", healthy, []),
        ("feeder_frequency", feeder_bad, ["feed_frequency"]),
        ("merge_failures", merge_bad, ["merge_failures"]),
        ("worker_utilization", workers_idle, ["worker_utilization"]),
        ("stuck_attempts", stuck_bad, ["stuck_attempts"]),
        ("candidate_pileup", pileup_bad, ["candidate_pileup"]),
        ("refusal_loop", refusal_bad, ["refusal_loop"]),
    ]

    all_passed = True
    for name, reading, expected in test_cases:
        fired = [r for r, _ in evaluate_rules(reading)]
        if set(fired) == set(expected):
            print(f"  PASS: {name}")
        else:
            print(f"  FAIL: {name} — expected {expected}, got {fired}")
            all_passed = False

    if all_passed:
        print("ok — all self-test cases passed")
        return 0
    else:
        print("FAIL — some self-test cases failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())