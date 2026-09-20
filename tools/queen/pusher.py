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
import datetime
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
    # WHOSE merges, not how many. The first version counted every merged pull
    # request in the repository, so one human merge suppressed `nothing-lands`
    # entirely - and that is exactly what hid the real outage: measured
    # 2026-09-20, seventeen pull requests merged in twenty-four hours and NOT
    # ONE came from a bee, while the swarm ran at ninety percent utilisation.
    # The branch name is what separates them.
    since = (
        sh(["date", "-u", "-v-6H", "+%Y-%m-%dT%H:%M:%SZ"]).strip()
        if sys.platform == "darwin"
        else sh(["date", "-u", "-d", "6 hours ago", "+%Y-%m-%dT%H:%M:%SZ"]).strip()
    )
    merged_recent = gh_json(
        ["pr", "list", "--repo", REPO, "--state", "merged", "--limit", "100",
         "--search", "merged:>=" + since,
         "--json", "number,headRefName"], [])
    bee_merged_recent = [
        p for p in merged_recent
        if str(p.get("headRefName", "")).startswith("queen-")
    ]
    # The bodies, not just the numbers: an issue with no `## Boundary` can never
    # be dispatched, so the count of open issues says nothing about how much work
    # the swarm can actually take. Measured 2026-09-20: 673 open, 119 with a
    # boundary.
    open_issues = gh_json(
        ["issue", "list", "--repo", REPO, "--state", "open", "--limit", "1000",
         "--json", "number,body"], [])
    with_boundary = sum(
        1 for issue in open_issues
        if re.search(r"(?ims)^##\s*boundary\s*$", issue.get("body") or "")
    )

    queued = get_json(
        f"https://api.github.com/repos/{REPO}/actions/runs?status=queued&per_page=1", {}
    ).get("total_count", -1)

    return {
        "swarm_state": status.get("swarmState", "?"),
        "observed_at": (status.get("queue") or {}).get("observedAt", ""),
        "workers_active": workers.get("active", -1),
        "workers_capacity": workers.get("capacity", -1),
        "workers_idle": workers.get("idle", -1),
        "dispatch_finished": dispatches.get("finished", -1),
        "dispatch_unreviewed": dispatches.get("unreviewed", -1),
        "issues_with_boundary": with_boundary,
        "skip_completed": skips.get("completed", 0),
        "queue_state": (status.get("queue") or {}).get("state", "?"),
        "refusal": (status.get("lastTick") or {}).get("refusal"),
        "dispatch_total": dispatches.get("total", -1),
        "dispatch_running": dispatches.get("running", -1),
        # A tick that refused on capacity never scanned the board, so its
        # skipSummary is empty - and an empty summary reads exactly like a board
        # with nothing skipped. Measured 2026-09-20T10:28Z: `10 workers already
        # running (limit 10)` with every skip count zero while 554 issues had no
        # boundary. Zero because nobody looked is not zero.
        "skips_are_fresh": bool(skips),
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
        "bee_merged_last_6h": len(bee_merged_recent),
        "actions_queued": queued,
        # EVERY gh failure in this file returns the default, and the default is
        # a number. A rate-limited token, an expired scope or a network blip
        # therefore collapses every count to zero - and an all-zero reading is
        # exactly the shape of `out-of-fuel`, which would file "nothing is
        # dispatchable" about a repository with a full backlog. This repository
        # has never had zero open issues; when it does, that is news, not a
        # reading to write rules on.
        "github_readable": len(open_issues) > 0,
    }


def hours_between(now: dict, before: dict | None) -> float:
    """Hours between two readings, by the SERVER's clock, or 0 when unknown.

    The server's own `queue.observedAt`, not this runner's clock: a scheduled
    job that starts late would otherwise read as a swarm that slowed down.
    """
    if not before:
        return 0.0
    try:
        a = datetime.datetime.fromisoformat(str(before.get("observed_at", "")).replace("Z", "+00:00"))
        b = datetime.datetime.fromisoformat(str(now.get("observed_at", "")).replace("Z", "+00:00"))
    except ValueError:
        return 0.0
    return max(0.0, (b - a).total_seconds() / 3600.0)


def finished_per_hour(now: dict, before: dict | None) -> float | None:
    """Bees finished per hour between the two readings, or None when unmeasurable."""
    hours = hours_between(now, before)
    if hours < 0.05 or not before:
        return None
    moved = now.get("dispatch_finished", -1) - before.get("dispatch_finished", -1)
    if moved < 0:
        # The counter went backwards: the service restarted. Not a slowdown.
        return None
    return moved / hours


# `dispatchable` is an ESTIMATE and is named as one. The tick reports how many
# candidates it skipped as claimed or completed, but caps the issue lists it
# prints, so the exact set cannot be subtracted - only the counts.
def dispatchable(now: dict) -> int:
    return max(
        0,
        now.get("issues_with_boundary", 0)
        - now.get("skip_claimed", 0)
        - now.get("skip_completed", 0),
    )


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

    # PARTIAL idleness, which every rule above was blind to. The first version
    # fired only when `workers_active == 0`, so a swarm running two bees in ten
    # lanes - 80% idle, with 221 issues it could have taken - read as healthy.
    # Measured 2026-09-20T10:16Z: capacity 10, active 2, idle 8, and no rule
    # said anything.
    # Idle lanes are read off the worker counts, which the tick always reports,
    # so this does NOT need a fresh skipSummary - only the "is there work"
    # half does, and over-counting there can only silence this rule, never make
    # it cry wolf.
    if (now.get("workers_idle", -1) >= 2 and dispatchable(now) > 0
            and now.get("workers_active", 0) > 0):
        fire(
            "lanes-idle",
            f"{now['workers_idle']} of {now.get('workers_capacity', '?')} lanes are empty while "
            f"about {dispatchable(now)} issue(s) are dispatchable.",
            f"curl -s {QUEEN}/status | python3 -c \"import json,sys;"
            "d=json.load(sys.stdin);print(d['workers'], d['lastTick']['refusal'])\"",
            f"refusal: {now['refusal']!r}",
        )

    # The tank, read before it is empty. `out-of-fuel` above fires when the swarm
    # has already stopped; this fires while it is still running, because a feeder
    # takes minutes to build a compiler and open an issue.
    lanes = now.get("workers_capacity", -1)
    if (now.get("skips_are_fresh", True) and "issues_with_boundary" in now
            and lanes > 0 and dispatchable(now) < lanes):
        fire(
            "fuel-runway",
            f"About {dispatchable(now)} dispatchable issue(s) for "
            f"{lanes} lanes: the swarm runs dry within one tick. "
            f"({now['issues_with_boundary']} open issues carry a boundary, "
            f"{now['skip_claimed']} are claimed, {now['skip_completed']} are completed.)",
            "python3 tools/queen/feed_untested.py --dry-run --limit 3",
        )

    # A swarm that got smaller without anyone saying so. The cap lives in the
    # deployment's environment (TRIOS_QUEEN_MAX_WORKERS), so it can change
    # between readings with no commit anywhere to show for it.
    if before and 0 <= now.get("workers_capacity", -1) < before.get("workers_capacity", -1):
        fire(
            "capacity-shrank",
            f"The swarm has fewer lanes than at the last reading: "
            f"{before.get('workers_capacity')} -> {now['workers_capacity']}.",
            f"curl -s {QUEEN}/status | python3 -c \"import json,sys;"
            "print(json.load(sys.stdin)['workers'])\"",
        )

    rate = finished_per_hour(now, before)
    was = before.get("rate_per_hour") if before else None
    if rate is not None and isinstance(was, (int, float)) and was >= 2 and rate <= was / 2:
        fire(
            "throughput-down",
            f"Bees are finishing at {rate:.1f}/hour against {was:.1f}/hour at the last "
            "reading - half or less.",
            f"curl -s {QUEEN}/status | python3 -c \"import json,sys;"
            "print(json.load(sys.stdin)['dispatches'])\"",
            f"finished {before.get('dispatch_finished')} -> {now['dispatch_finished']} "
            f"over {hours_between(now, before):.2f}h",
        )

    # Review cost is linear in running workers, so the backlog is what binds
    # next after the lanes are full. Reported, not repaired: there is no
    # mechanical answer to "the reviewers are behind" that this file may take.
    # ONE LANE'S WORTH, not two. The threshold was 2x the lanes, which at twenty
    # lanes means forty unreviewed - and the queue that mattered on 2026-09-20
    # sat at sixteen to twenty all afternoon, under the threshold, while every
    # bee that finished waited. The number that matters is whether the backlog
    # is bigger than what the swarm can produce in one round, because that is
    # the point at which it can only grow.
    #
    # THE CAUSE IS A BUDGET, and the rule names it: the review sweep buys
    # `TRIOS_QUEEN_REVIEWS_PER_ROUND` reviews a round (default 3) and
    # `TRIOS_QUEEN_MEASUREMENTS_PER_ROUND` criteria measurements (default 3),
    # both capped at 32. Faster bees do not make the swarm faster if the sweep
    # still buys three.
    unreviewed = now.get("dispatch_unreviewed", -1)
    lanes_for_review = max(1, now.get("workers_capacity", 1))
    if (before and unreviewed >= lanes_for_review
            and unreviewed >= before.get("dispatch_unreviewed", unreviewed)):
        fire(
            "review-backlog",
            f"{unreviewed} finished bees are unreviewed, against "
            f"{now.get('workers_capacity')} lanes and a sweep that buys "
            "TRIOS_QUEEN_REVIEWS_PER_ROUND reviews a round (default 3). Nothing "
            "they wrote can land until it is judged.",
            f"curl -s {QUEEN}/status | python3 -c \"import json,sys;"
            "print(json.load(sys.stdin)['dispatches'])\"",
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

    if now.get("bee_merged_last_6h", -1) == 0 and now["open_bee_prs"] > 0:
        fire(
            "nothing-lands",
            f"No bee pull request has merged in six hours while "
            f"{now['open_bee_prs']} are open"
            + (f" (the repository merged {now['merged_last_6h']} in total)."
               if now.get("merged_last_6h") else "."),
            f"gh run list --repo {REPO} --workflow auto-merge-ready-prs.yml --limit 5",
        )

    # DELIVERY, which nothing measured. A swarm at full utilisation that has
    # published nothing looks identical to a swarm that is shipping, from every
    # number above: measured 2026-09-20, 401 `queen-*` branches on the remote
    # and the last bee pull request three days old, because nothing opened one.
    if (now.get("bee_merged_last_6h", -1) == 0 and now["open_bee_prs"] == 0
            and now["dispatch_running"] > 0):
        fire(
            "nothing-published",
            f"{now['dispatch_running']} bee(s) are running, no bee pull request is "
            "open, and none has merged in six hours: the work is not reaching a "
            "pull request at all.",
            f"gh pr list --repo {REPO} --state all --limit 200 "
            "--json headRefName,createdAt | grep -c queen-",
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
        # THREE conditions, not two. With two it fired on a healthy swarm:
        # measured 2026-09-20T10:28Z, nine bees running and the total unchanged
        # for sixteen minutes, which is what a nine-bee swarm with a twenty-
        # minute cycle looks like. A finished counter that has also not moved,
        # over at least three quarters of an hour, is the stall this was for.
        age = hours_between(now, before)
        still = (
            age <= STALE_READING_HOURS
            and moved <= 0
            and now["dispatch_total"] == before.get("dispatch_total")
            and now.get("dispatch_finished", -1) == before.get("dispatch_finished", -2)
            and age >= 0.75
        )
        if still:
            fire(
                "not-evolving",
                "Since the last reading nothing finished and nothing new was "
                "dispatched: the system is not moving at all.",
                f"curl -s {QUEEN}/status | python3 -m json.tool",
                f"done {before.get('column_done')} -> {now['column_done']}, "
                f"dispatches {before.get('dispatch_total')} -> {now['dispatch_total']}, "
                f"finished {before.get('dispatch_finished')} -> {now.get('dispatch_finished')}, "
                f"over {hours_between(now, before):.2f}h",
            )

    return found


# THE PART THAT IS NOT A REPORT. Every stall this file was written for was a
# stall somebody then had to fix by hand, and the hands were not always awake.
# Two of the rules have a mechanical answer - a feeder run - so the pusher takes
# it, and prints what it took.
#
# Bounded on purpose: only the feeders, only when a fuel-shaped rule fired, and
# never twice inside twenty minutes. A watchdog that can dispatch anything is a
# second Queen with none of her gates.
FEEDERS = ("queen-feed-untested.yml", "queen-feed-empty-bodies.yml")
FUEL_KEYS = {"out-of-fuel", "fuel-runway", "lanes-idle"}
REFILL_COOLDOWN_MINUTES = 20


def ran_recently(workflow: str, minutes: int = REFILL_COOLDOWN_MINUTES) -> bool:
    runs = gh_json(
        ["run", "list", "--repo", REPO, "--workflow", workflow, "--limit", "1",
         "--json", "createdAt"], [])
    if not runs:
        return False
    try:
        started = datetime.datetime.fromisoformat(
            str(runs[0].get("createdAt", "")).replace("Z", "+00:00"))
    except ValueError:
        return False
    age = datetime.datetime.now(datetime.timezone.utc) - started
    return age.total_seconds() < minutes * 60


def refill(found: list[dict], dry_run: bool) -> list[str]:
    """Run the feeders when the tank is the thing that stopped the swarm."""
    reasons = [item["key"] for item in found if item["key"] in FUEL_KEYS]
    if not reasons:
        return []
    acted = []
    for workflow in FEEDERS:
        if ran_recently(workflow):
            acted.append(f"`{workflow}` left alone: it ran inside the last "
                         f"{REFILL_COOLDOWN_MINUTES} minutes")
            continue
        if dry_run:
            acted.append(f"`{workflow}` would be dispatched (dry run)")
            continue
        done = subprocess.run(
            ["gh", "workflow", "run", workflow, "--repo", REPO],
            capture_output=True, text=True, timeout=120,
        )
        acted.append(
            f"`{workflow}` dispatched" if done.returncode == 0
            else f"`{workflow}` FAILED to dispatch: {done.stderr.strip()[:120]}"
        )
    return [f"because {', '.join(reasons)} fired:", *acted]


def render(now: dict, before: dict | None, found: list[dict],
           acted: list[str] | None = None) -> str:
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
        row("lanes standing empty", "workers_idle"),
        row("finished, all time", "dispatch_finished"),
        row("finished but unreviewed", "dispatch_unreviewed"),
        row("open issues carrying a boundary", "issues_with_boundary"),
        row("the tick scanned the board", "skips_are_fresh"),
        row("queue", "queue_state"),
        row("issues claimed by a held attempt", "skip_claimed"),
        row("issues with no boundary", "skip_missing_boundary"),
        row("board: review", "column_review"),
        row("board: done", "column_done"),
        row("open issues", "open_issues"),
        row("open bee PRs", "open_bee_prs"),
        row("of those, conflicted", "conflicted_bee_prs"),
        row("merged in the last 6h", "merged_last_6h"),
        row("of those, from a bee", "bee_merged_last_6h"),
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
    if acted:
        lines += ["## What was done about it", ""]
        lines += [f"- {line}" for line in acted]
        lines += [""]
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


def fired_keys(found: list[dict]) -> list[str]:
    return sorted({item["key"] for item in found})


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
        "bee_merged_last_6h": 3,
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
         {**busy, "merged_last_6h": 0, "bee_merged_last_6h": 0, "open_bee_prs": 5},
         "nothing-lands", True),
        ("a full CI queue fires", {**busy, "actions_queued": 2267}, "ci-queue", True),
    ]
    # The four rules added after 2026-09-20, when a swarm running two bees in ten
    # lanes read as healthy because every rule asked whether active was ZERO.
    ten_lanes = {**busy, "workers_capacity": 10, "workers_active": 2,
                 "workers_idle": 8, "issues_with_boundary": 240,
                 "skip_completed": 5, "dispatch_finished": 700,
                 "observed_at": "2026-09-20T10:00:00Z"}
    full = {**ten_lanes, "workers_active": 10, "workers_idle": 0}
    dry_tank = {**full, "issues_with_boundary": 20, "skip_claimed": 14,
                "skip_completed": 5}
    stale = {**ten_lanes, "skips_are_fresh": False, "issues_with_boundary": 20,
             "workers_active": 10, "workers_idle": 0}
    outage = {**busy, "bee_merged_last_6h": 0, "open_bee_prs": 0,
              "dispatch_running": 20}
    shipping = {**outage, "bee_merged_last_6h": 4}
    cases += [
        ("a swarm that publishes nothing fires", outage, "nothing-published", True),
        ("a swarm whose work lands does not", shipping, "nothing-published", False),
        ("and a repository that merged only its operator's work still fires",
         {**busy, "bee_merged_last_6h": 0, "merged_last_6h": 17, "open_bee_prs": 5},
         "nothing-lands", True),
        ("a tick that never scanned cannot say the tank is thin", stale,
         "fuel-runway", False),
        ("empty lanes fire while work exists", ten_lanes, "lanes-idle", True),
        ("a full swarm has no empty lanes", full, "lanes-idle", False),
        ("a thin tank fires before it is empty", dry_tank, "fuel-runway", True),
        ("a deep tank does not", full, "fuel-runway", False),
    ]
    bad = 0
    for name, now, key, want in cases:
        fired = any(item["key"] == key for item in rules(now, None))
        if fired != want:
            print(f"  self-test FAILED: {name} -> {fired}, expected {want}")
            bad += 1
    # The stillness rule needs two readings.
    stale_busy = {**busy, "dispatch_finished": 700,
                  "observed_at": "2026-09-20T11:00:00Z"}
    an_hour_ago = {"column_done": 300, "dispatch_total": 800,
                   "dispatch_finished": 700, "observed_at": "2026-09-20T10:00:00Z"}
    still = any(
        item["key"] == "not-evolving" for item in rules(stale_busy, an_hour_ago)
    )
    if not still:
        print("  self-test FAILED: an hour with nothing finished must read as not evolving")
        bad += 1
    fresh = any(
        item["key"] == "not-evolving"
        for item in rules({**stale_busy, "observed_at": "2026-09-20T10:16:00Z"}, an_hour_ago)
    )
    if fresh:
        print("  self-test FAILED: sixteen minutes of a twenty-minute cycle is not a stall")
        bad += 1
    moved = any(
        item["key"] == "not-evolving"
        for item in rules(stale_busy, {**an_hour_ago, "column_done": 290,
                                       "dispatch_total": 790, "dispatch_finished": 690})
    )
    if moved:
        print("  self-test FAILED: a reading that moved must not read as stalled")
        bad += 1
    piling_up = any(
        item["key"] == "review-backlog"
        for item in rules({**full, "dispatch_unreviewed": 12},
                          {**full, "dispatch_unreviewed": 11})
    )
    if not piling_up:
        print("  self-test FAILED: 11 -> 12 unreviewed against 10 lanes must fire")
        bad += 1
    draining = any(
        item["key"] == "review-backlog"
        for item in rules({**full, "dispatch_unreviewed": 8},
                          {**full, "dispatch_unreviewed": 30})
    )
    if draining:
        print("  self-test FAILED: a backlog that is going DOWN must not fire")
        bad += 1
    under = any(
        item["key"] == "review-backlog"
        for item in rules({**full, "dispatch_unreviewed": 3},
                          {**full, "dispatch_unreviewed": 2})
    )
    if under:
        print("  self-test FAILED: three unreviewed against ten lanes is not a backlog")
        bad += 1

    # Capacity shrinking needs two readings, and so does throughput halving.
    shrank = any(
        item["key"] == "capacity-shrank"
        for item in rules(full, {**full, "workers_capacity": 16})
    )
    if not shrank:
        print("  self-test FAILED: 16 lanes becoming 10 must read as a shrink")
        bad += 1
    grew = any(
        item["key"] == "capacity-shrank"
        for item in rules(full, {**full, "workers_capacity": 4})
    )
    if grew:
        print("  self-test FAILED: 4 lanes becoming 10 is not a shrink")
        bad += 1
    halved = any(
        item["key"] == "throughput-down"
        for item in rules({**full, "dispatch_finished": 705,
                           "observed_at": "2026-09-20T11:00:00Z"},
                          {**full, "dispatch_finished": 700,
                           "observed_at": "2026-09-20T10:00:00Z",
                           "rate_per_hour": 20.0})
    )
    if not halved:
        print("  self-test FAILED: 20/hour falling to 5/hour must fire")
        bad += 1
    steady = any(
        item["key"] == "throughput-down"
        for item in rules({**full, "dispatch_finished": 720,
                           "observed_at": "2026-09-20T11:00:00Z"},
                          {**full, "dispatch_finished": 700,
                           "observed_at": "2026-09-20T10:00:00Z",
                           "rate_per_hour": 20.0})
    )
    if steady:
        print("  self-test FAILED: a steady 20/hour must not read as a fall")
        bad += 1
    restarted = finished_per_hour(
        {**full, "dispatch_finished": 3, "observed_at": "2026-09-20T11:00:00Z"},
        {**full, "dispatch_finished": 700, "observed_at": "2026-09-20T10:00:00Z"},
    )
    if restarted is not None:
        print("  self-test FAILED: a counter that went backwards is a restart, not a rate")
        bad += 1
    if bad:
        return 1
    print(f"ok: {len(cases) + 11} rule shapes, including eleven a moving system must NOT fire")
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
    # A reading taken through a blind `gh` is not a reading. Every helper here
    # returns its default on failure and every default is a number, so a
    # rate-limited token turns the whole board into zeros - and an all-zero
    # board is the exact shape of `out-of-fuel`. Refusing is the same answer
    # this file already gives when the status endpoint is silent.
    if not now.get("github_readable", True):
        print("could not run: `gh` returned no open issues, which this repository "
              "has never had. Every count would be a zero it did not measure.",
              file=sys.stderr)
        return 2

    issue = pulse_issue()
    before = previous_reading(issue.get("body", "")) if issue else None
    # The rate this reading establishes, stored so the NEXT reading can compare.
    # A rate is a property of two readings, so it cannot come out of one.
    measured_rate = finished_per_hour(now, before)
    if measured_rate is not None:
        now["rate_per_hour"] = round(measured_rate, 2)
    found = rules(now, before)
    now["fired"] = fired_keys(found)
    acted = refill(found, args.dry_run)
    body = render(now, before, found, acted)

    print(json.dumps(now, indent=1, sort_keys=True))
    for item in found:
        print(f"STOPPED: {item['why']}")
    for line in acted:
        print(f"ACTED: {line}")
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
        # A comment per reading is 96 comments a day while one rule holds, and a
        # log nobody can scroll is a log nobody reads - the alarm-fatigue shape
        # that hides the next real stall. The BODY is rewritten every reading
        # and always current; a comment is for a CHANGE.
        changed = fired_keys(found) != sorted(before.get("fired", [])) if before else bool(found)
        if found and changed:
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
