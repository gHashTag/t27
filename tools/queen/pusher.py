#!/usr/bin/env python3
"""The pusher: read the whole system, say what moved, and file the work.

WHY. Every stall this swarm has had was visible in a number nobody was looking
at. On 2026-09-17 the feeder had fired twice in twelve hours instead of 36 and
the swarm idled all night. On 2026-09-19 the batch merge had been failing in
three seconds on a model id for two days, silently. On 2026-09-20 every one of
71 issues sat claimed by an attempt that had spent its retries, 0 of 8 lanes
ran, and the tick refused 673 candidates - honestly, because the Zig the bees
wrote did not compile - and the swarm simply stopped. In all three the operator
found out by asking.

WHAT THIS IS. A reading, taken on a schedule, of the things that can stop this
system, each next to what it was last time; then the rules that turn a reading
into work. It keeps its memory in its own GitHub issue, so the history is where
anyone can read it and no state file has to be committed back to master.

WHAT IT IS NOT. It does not judge code, it does not merge, it does not close
other people's work. It measures, it compares, and it opens an issue with the
command that produced every number. A rule that cannot name the command that
would falsify it does not belong here.

Usage:
    python3 tools/queen/pusher.py --dry-run     # print the reading, file nothing
    python3 tools/queen/pusher.py               # file what the rules find
    python3 tools/queen/pusher.py --self-test   # the rules, against fixtures
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.request

REPO = os.environ.get("PUSHER_REPO", "gHashTag/t27")
QUEEN = os.environ.get(
    "QUEEN_STATUS_URL",
    "https://trios-agent-server-production.up.railway.app/queen",
)
PULSE_TITLE = "System pulse: what the swarm is doing, and what has stopped"
PULSE_LABEL = "pulse"
# A reading older than this is not evidence about now.
STALE_READING_HOURS = 6


def sh(args: list[str], timeout: int = 180) -> str:
    done = subprocess.run(
        args, capture_output=True, text=True, timeout=timeout, stdin=subprocess.DEVNULL
    )
    return done.stdout if done.returncode == 0 else ""


def gh_json(args: list[str], default):
    out = sh(["gh", *args])
    if not out.strip():
        return default
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        return default


def get_json(url: str, default):
    try:
        with urllib.request.urlopen(url, timeout=30) as response:
            return json.load(response)
    except Exception:
        return default


def reading() -> dict:
    """Every number a rule below is allowed to use, and where it came from."""
    status = get_json(f"{QUEEN}/status", {})
    board = get_json(f"{QUEEN}/public-board", {})
    cards = board.get("cards", [])
    columns: dict[str, int] = {}
    for card in cards:
        columns[card.get("column", "?")] = columns.get(card.get("column", "?"), 0) + 1

    workers = status.get("workers") or {}
    dispatches = status.get("dispatches") or {}
    skips = {
        key: (value or {}).get("count", 0)
        for key, value in ((status.get("lastTick") or {}).get("skipSummary") or {}).items()
    }

    open_prs = gh_json(
        ["pr", "list", "--repo", REPO, "--state", "open", "--limit", "200",
         "--json", "number,headRefName,mergeStateStatus"], [])
    bee_prs = [p for p in open_prs if str(p.get("headRefName", "")).startswith("queen-")]
    merged_recent = gh_json(
        ["pr", "list", "--repo", REPO, "--state", "merged", "--limit", "100",
         "--search", "merged:>=" + sh(["date", "-u", "-v-6H", "+%Y-%m-%dT%H:%M:%SZ"]).strip()
         if sys.platform == "darwin" else
         "merged:>=" + sh(["date", "-u", "-d", "6 hours ago", "+%Y-%m-%dT%H:%M:%SZ"]).strip(),
         "--json", "number"], [])
    open_issues = gh_json(
        ["issue", "list", "--repo", REPO, "--state", "open", "--limit", "1000",
         "--json", "number"], [])

    queued = get_json(
        f"https://api.github.com/repos/{REPO}/actions/runs?status=queued&per_page=1", {}
    ).get("total_count", -1)

    return {
        "swarm_state": status.get("swarmState", "?"),
        "workers_active": workers.get("active", -1),
        "workers_capacity": workers.get("capacity", -1),
        "queue_state": (status.get("queue") or {}).get("state", "?"),
        "refusal": (status.get("lastTick") or {}).get("refusal"),
        "dispatch_total": dispatches.get("total", -1),
        "dispatch_running": dispatches.get("running", -1),
        "skip_claimed": skips.get("claimed", 0),
        "skip_missing_boundary": skips.get("missingBoundary", 0),
        "skip_file_conflict": skips.get("fileConflict", 0),
        "column_review": columns.get("review", 0),
        "column_backlog": columns.get("backlog", 0),
        "column_done": columns.get("done", 0),
        "open_issues": len(open_issues),
        "open_bee_prs": len(bee_prs),
        "conflicted_bee_prs": sum(
            1 for p in bee_prs if p.get("mergeStateStatus") == "DIRTY"),
        "merged_last_6h": len(merged_recent),
        "actions_queued": queued,
    }


# Each rule: (key, does it fire, the sentence, the command a reader can run).
def rules(now: dict, before: dict | None) -> list[dict]:
    found: list[dict] = []

    def fire(key: str, why: str, command: str, detail: str = "") -> None:
        found.append({"key": key, "why": why, "command": command, "detail": detail})

    if now["workers_active"] == 0 and now["dispatch_running"] == 0:
        # Idle is only a fault when there IS work it could take.
        free_fuel = now["open_issues"] - now["skip_missing_boundary"] - now["skip_claimed"]
        if free_fuel > 0:
            fire(
                "idle-with-fuel",
                f"No bee is running while {free_fuel} open issue(s) are neither "
                "claimed nor boundary-less.",
                f"curl -s {QUEEN}/status | python3 -m json.tool",
                f"refusal: {now['refusal']!r}",
            )
        else:
            fire(
                "out-of-fuel",
                "No bee is running and nothing is dispatchable: every open issue is "
                "claimed or carries no boundary.",
                "python3 tools/queen/feed_empty_bodies.py --dry-run --limit 5",
            )

    if now["skip_claimed"] >= 40:
        fire(
            "work-parked",
            f"{now['skip_claimed']} issues are held by attempts that are not running.",
            f"curl -s {QUEEN}/status | python3 -c \"import json,sys;"
            "print(json.load(sys.stdin)['lastTick']['skipSummary'])\"",
        )

    if now["conflicted_bee_prs"] >= 10:
        fire(
            "prs-conflicted",
            f"{now['conflicted_bee_prs']} of {now['open_bee_prs']} bee pull requests "
            "cannot merge: GitHub runs no required check on a branch that conflicts, "
            "so they report as waiting for CI.",
            f"gh pr list --repo {REPO} --state open --limit 200 "
            "--json number,headRefName,mergeStateStatus",
        )

    if now["merged_last_6h"] == 0 and now["open_bee_prs"] > 0:
        fire(
            "nothing-lands",
            f"Nothing has merged in six hours while {now['open_bee_prs']} bee pull "
            "requests are open.",
            f"gh run list --repo {REPO} --workflow auto-merge-ready-prs.yml --limit 5",
        )

    if now["actions_queued"] >= 300:
        fire(
            "ci-queue",
            f"{now['actions_queued']} workflow runs are queued; required checks wait "
            "behind them.",
            f"gh api 'repos/{REPO}/actions/runs?status=queued&per_page=1' -q .total_count",
        )

    if before:
        moved = now["column_done"] - before.get("column_done", now["column_done"])
        if moved <= 0 and now["dispatch_total"] == before.get("dispatch_total"):
            fire(
                "not-evolving",
                "Since the last reading nothing finished and nothing new was "
                "dispatched: the system is not moving at all.",
                f"curl -s {QUEEN}/status | python3 -m json.tool",
                f"done {before.get('column_done')} -> {now['column_done']}, "
                f"dispatches {before.get('dispatch_total')} -> {now['dispatch_total']}",
            )

    return found


def render(now: dict, before: dict | None, found: list[dict]) -> str:
    def row(label: str, key: str) -> str:
        was = "" if not before else str(before.get(key, "?"))
        arrow = "" if not before or str(before.get(key)) == str(now[key]) else f" ({was} ->)"
        return f"| {label} | {now[key]}{arrow} |"

    lines = [
        "<!-- pusher: this issue is the pusher's memory; the JSON below is read back -->",
        "",
        "## The reading",
        "",
        "| | now |",
        "|---|---|",
        row("bees running", "dispatch_running"),
        row("lanes", "workers_capacity"),
        row("queue", "queue_state"),
        row("issues claimed by a held attempt", "skip_claimed"),
        row("issues with no boundary", "skip_missing_boundary"),
        row("board: review", "column_review"),
        row("board: done", "column_done"),
        row("open issues", "open_issues"),
        row("open bee PRs", "open_bee_prs"),
        row("of those, conflicted", "conflicted_bee_prs"),
        row("merged in the last 6h", "merged_last_6h"),
        row("workflow runs queued", "actions_queued"),
        "",
    ]
    if found:
        lines += ["## What has stopped", ""]
        for item in found:
            lines += [
                f"### {item['why']}",
                "",
                f"```\n{item['command']}\n```",
            ]
            if item["detail"]:
                lines += ["", item["detail"]]
            lines += [""]
    else:
        lines += ["## What has stopped", "", "Nothing these rules can name.", ""]
    lines += [
        "## Rules that produced this",
        "",
        "Each rule names the command that would falsify it. A rule that cannot is not "
        "in this file.",
        "",
        "```json",
        json.dumps(now, indent=1, sort_keys=True),
        "```",
    ]
    return "\n".join(lines)


def previous_reading(body: str) -> dict | None:
    match = re.search(r"```json\n(\{.*?\})\n```", body or "", re.S)
    if not match:
        return None
    try:
        return json.loads(match.group(1))
    except json.JSONDecodeError:
        return None


def pulse_issue() -> dict | None:
    issues = gh_json(
        ["issue", "list", "--repo", REPO, "--state", "open", "--limit", "100",
         "--search", PULSE_TITLE, "--json", "number,title,body"], [])
    for issue in issues:
        if issue.get("title") == PULSE_TITLE:
            return issue
    return None


def self_test() -> int:
    """The rules, against readings whose verdict is known."""
    idle_with_fuel = {
        "workers_active": 0, "dispatch_running": 0, "open_issues": 700,
        "skip_missing_boundary": 553, "skip_claimed": 14, "skip_file_conflict": 0,
        "conflicted_bee_prs": 0, "open_bee_prs": 0, "merged_last_6h": 3,
        "actions_queued": 0, "refusal": "nothing to choose",
        "column_done": 300, "dispatch_total": 800, "column_review": 14,
    }
    starved = {**idle_with_fuel, "open_issues": 567, "skip_claimed": 14}
    busy = {**idle_with_fuel, "workers_active": 8, "dispatch_running": 8}
    cases = [
        ("idle with fuel fires", idle_with_fuel, "idle-with-fuel", True),
        ("out of fuel fires instead", starved, "out-of-fuel", True),
        ("a busy swarm fires neither", busy, "idle-with-fuel", False),
        ("a busy swarm is not starved either", busy, "out-of-fuel", False),
        ("parked work fires at 40", {**busy, "skip_claimed": 71}, "work-parked", True),
        ("parked work is quiet at 14", busy, "work-parked", False),
        ("conflicted PRs fire at 10",
         {**busy, "conflicted_bee_prs": 33, "open_bee_prs": 33}, "prs-conflicted", True),
        ("nothing landing fires",
         {**busy, "merged_last_6h": 0, "open_bee_prs": 5}, "nothing-lands", True),
        ("a full CI queue fires", {**busy, "actions_queued": 2267}, "ci-queue", True),
    ]
    bad = 0
    for name, now, key, want in cases:
        fired = any(item["key"] == key for item in rules(now, None))
        if fired != want:
            print(f"  self-test FAILED: {name} -> {fired}, expected {want}")
            bad += 1
    # The stillness rule needs two readings.
    still = any(
        item["key"] == "not-evolving"
        for item in rules(busy, {"column_done": 300, "dispatch_total": 800})
    )
    if not still:
        print("  self-test FAILED: two identical readings must read as not evolving")
        bad += 1
    moved = any(
        item["key"] == "not-evolving"
        for item in rules(busy, {"column_done": 290, "dispatch_total": 790})
    )
    if moved:
        print("  self-test FAILED: a reading that moved must not read as stalled")
        bad += 1
    if bad:
        return 1
    print(f"ok: {len(cases) + 2} rule shapes, including three a moving system must NOT fire")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    now = reading()
    if now["workers_capacity"] < 0:
        print("could not run: the Queen's status endpoint did not answer", file=sys.stderr)
        return 2

    issue = pulse_issue()
    before = previous_reading(issue.get("body", "")) if issue else None
    found = rules(now, before)
    body = render(now, before, found)

    print(json.dumps(now, indent=1, sort_keys=True))
    for item in found:
        print(f"STOPPED: {item['why']}")
    if args.dry_run:
        print(f"dry-run: would {'update' if issue else 'open'} the pulse issue")
        return 0

    path = os.path.join(os.environ.get("RUNNER_TEMP", "/tmp"), "pulse.md")
    with open(path, "w") as handle:
        handle.write(body)
    if issue:
        sh(["gh", "issue", "edit", str(issue["number"]), "--repo", REPO,
            "--body-file", path])
        print(f"updated the pulse issue #{issue['number']}")
        if found:
            comment = "\n".join(
                [f"- {item['why']}\n  ```\n  {item['command']}\n  ```" for item in found])
            comment_path = path + ".comment"
            with open(comment_path, "w") as handle:
                handle.write("This reading found:\n\n" + comment)
            sh(["gh", "issue", "comment", str(issue["number"]), "--repo", REPO,
                "--body-file", comment_path])
    else:
        sh(["gh", "issue", "create", "--repo", REPO, "--title", PULSE_TITLE,
            "--body-file", path])
        print("opened the pulse issue")
    return 0


if __name__ == "__main__":
    sys.exit(main())
