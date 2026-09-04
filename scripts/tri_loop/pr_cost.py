#!/usr/bin/env python3
r"""tri pr-cost -- what the up-to-date rule costs, in minutes rather than commits.

`strict_required_status_checks_policy` means every merge that lands makes every
other open pull request stale, and each one then takes an `update-branch` merge
and a full CI rerun before it can try again. #3134 priced that in COMMITS -- more
reruns than commits of content. This prices it in TIME.

MEASURED 2026-09-04, seven pull requests landed in one morning:

    7 pull requests, 377 minutes alive, 10 commits of content
    11 update-branch merges
    mean CI cycle 17.6 minutes, 32-42 checks

    11 x 17.6 = 193 minutes -- 3.2 hours of CI in one morning, caused by the
    up-to-date rule alone, for ten commits of work.

WHAT THE 193 MINUTES BOUGHT
---------------------------
Nothing, on those eleven occasions: every rerun came back green. That is NOT an
argument against the rule -- it exists for the case where a rerun catches two
pull requests that pass alone and break together, and this repository has no
other defence against that. It is an argument that the price is now known.

TWO COSTS THAT LOOK LIKE ONE
----------------------------
Of a 17.6-minute cycle, the last **15.4 minutes on average** happen after the four
required contexts are already green. So part of the tax is the rerun and part is
waiting for checks that cannot block the merge. A merge queue removes the first;
`tri pr ready --required-only` removes the second. **Neither substitutes for the
other**, and reporting one number for both hides that.

WHAT THIS DOES NOT ESTABLISH
----------------------------
That a merge queue is worth enabling -- that is a ruleset change and the owner's
(#3134). Nor that this morning is typical: two mornings is not a week, and the
cheapest option in #3134 was always to keep measuring before spending a decision.

An `update-branch` merge is counted by its commit message on the pull request's
own branch, so a differently-worded merge of the base is invisible here.

    tri pr-cost                 # the last 10 merged PRs
    tri pr-cost --last 25
    tri pr-cost --json
"""
from __future__ import annotations

import datetime as dt
import json
import subprocess
import sys

UPDATE_PREFIXES = ("Merge branch 'master'", "Merge remote-tracking", "Merge branch \"master\"")


def sh(args: list[str]) -> str:
    return subprocess.run(args, capture_output=True, text=True).stdout


def refuse(msg: str) -> None:
    print(f"tri pr-cost: {msg}", file=sys.stderr)
    print("  Exit 2 = could not run, not a zero cost.", file=sys.stderr)
    raise SystemExit(2)


def when(s: str) -> dt.datetime | None:
    if not s or s in ("null", "None"):
        return None
    return dt.datetime.fromisoformat(s.replace("Z", "+00:00"))


def repo() -> str:
    r = sh(["gh", "repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"]).strip()
    if not r:
        refuse("could not read the repository from the current directory.")
    return r


def main() -> int:
    last = 10
    if "--last" in sys.argv:
        try:
            last = int(sys.argv[sys.argv.index("--last") + 1])
        except (IndexError, ValueError):
            refuse("--last needs a number.")
    slug = repo()

    raw = sh(["gh", "pr", "list", "--repo", slug, "--state", "merged", "--limit", str(last),
              "--json", "number"])
    try:
        nums = [p["number"] for p in json.loads(raw or "[]")]
    except json.JSONDecodeError:
        refuse("gh did not return JSON for the merged list.")
    if not nums:
        refuse("no merged pull request was returned; that is an unanswered question, not a zero.")

    rows = []
    for n in nums:
        meta = sh(["gh", "api", f"repos/{slug}/pulls/{n}", "--jq",
                   "[.created_at,.merged_at,.commits,.head.sha]|@tsv"]).strip()
        if meta.count("\t") != 3:
            continue
        created, merged, commits, sha = meta.split("\t")
        c, m = when(created), when(merged)
        if not (c and m):
            continue
        msgs = sh(["gh", "api", f"repos/{slug}/pulls/{n}/commits", "--jq", ".[].commit.message"])
        updates = sum(1 for l in msgs.split("\n") if l.startswith(UPDATE_PREFIXES))
        checks = sh(["gh", "api", "--paginate", f"repos/{slug}/commits/{sha}/check-runs",
                     "--jq", r'.check_runs[]|"\(.started_at)\t\(.completed_at)"'])
        st, en = [], []
        for line in checks.split("\n"):
            if "\t" not in line:
                continue
            a, b = line.split("\t", 1)
            if (x := when(a)):
                st.append(x)
            if (y := when(b)):
                en.append(y)
        cycle = (max(en) - min(st)).total_seconds() / 60 if st and en else None
        rows.append({"pr": n, "alive_min": (m - c).total_seconds() / 60,
                     "commits": int(commits), "updates": updates,
                     "cycle_min": cycle, "checks": len(en)})

    if not rows:
        refuse("not one merged pull request could be read; refusing to report a cost of zero.")

    cycles = [r["cycle_min"] for r in rows if r["cycle_min"]]
    mean_cycle = sum(cycles) / len(cycles) if cycles else None
    updates = sum(r["updates"] for r in rows)
    content = sum(r["commits"] for r in rows) - updates

    if "--json" in sys.argv:
        print(json.dumps({"rows": rows, "updates": updates, "content_commits": content,
                          "mean_cycle_min": mean_cycle,
                          "tax_min": updates * mean_cycle if mean_cycle else None}, indent=1))
        return 0

    print(f"tri pr-cost -- the up-to-date rule, over the last {len(rows)} merged pull request(s)")
    print()
    print(f"  {'PR':<8}{'alive':>8}{'commits':>9}{'updates':>9}{'cycle':>9}{'checks':>8}")
    for r in sorted(rows, key=lambda x: x["pr"]):
        cyc = f"{r['cycle_min']:.1f}m" if r["cycle_min"] else "-"
        print(f"  #{r['pr']:<7}{r['alive_min']:>7.0f}m{r['commits']:>9}{r['updates']:>9}{cyc:>9}{r['checks']:>8}")
    print()
    print(f"  content commits          {content}")
    print(f"  update-branch merges     {updates}")
    if mean_cycle:
        print(f"  mean CI cycle            {mean_cycle:.1f} minutes")
        print(f"  cost of the rule         {updates * mean_cycle:.0f} minutes "
              f"({updates} reruns x {mean_cycle:.1f})")
    print()
    print("  This does NOT establish that a merge queue is worth enabling -- that is a")
    print("  ruleset change and the owner's (#3134). Nor that this window is typical.")
    print("  And part of a cycle is waiting for checks that cannot block the merge:")
    print("  a merge queue removes the reruns, `tri pr ready --required-only` removes")
    print("  the waiting, and neither substitutes for the other.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
