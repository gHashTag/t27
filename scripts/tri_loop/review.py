#!/usr/bin/env python3
"""tri review -- the reviewer bee's health, queue and outcomes, read-only.

The review jam of 2026-10-03 (#5776) was invisible from the board: the merger
merged nothing for two weeks, and the reason sat in three places nobody read
together -- a launchd job's exit code, a JSONL file of verdicts, and the skip
lines of a log. This puts them on one page.

  tri review            = tri review status
  tri review status     the job, the log's age, the installed copy's drift from
                        this checkout, keys (count only), t27c, zig, disk, old run
                        dirs, whether a head's CLAUDE.md can reach the agent
                        (the last `probe --tamper` against today's CLI), and
                        what the last day of verdicts and raw answers say keeps
                        going wrong
  tri review queue      who the bee reviews next, and why every other open pull
                        request waits, grouped by reason (reads GitHub, ~2 min)
  tri review stats [--days N]
                        outcomes per day, median review time, leading reasons
  tri review tick [--json]
                        one look -- health, queue, reviews and merges since the
                        last look -- appended to ticks.jsonl, then what the run of
                        looks shows that one cannot: a stalled queue, a failure
                        that came back after a repair, approvals not merged
  tri review golden     the newest golden-set eval: each pinned pull request's
                        known verdict beside the bee's, and the score (read from
                        eval.jsonl; running one is `reviewer.py eval`, which
                        holds the run lock like a live run)
  tri review wire [--samples N --every S]
                        each running agent's bytes in and out and its open
                        connections, from outside the process: a slow stream,
                        a request sent again, a review restarted on another key
                        -- none of which reaches the log (ps, nettop, lsof)
  tri review self-test  the reviewer's own checks: no network, no agent

All of it is `tools/bees/reviewer.py`; this file only routes to it, so the long
form of every rule lives there and nowhere else.

WHAT THIS DOES NOT ESTABLISH
---------------------------
That an approval is right. `status` counts outcomes and flags shapes of
failure (no verdict block, a recurring reason, an answer claiming to have run
a command when the agent has no shell); whether a given verdict is correct is
a reading of that pull request, and only a person or a later revert says so.
`golden` is that reading done once, for five pull requests, and kept.

It also never repairs. `status` writes its answer to
~/.local/state/t27-bees/doctor.json, `tick` appends one line to ticks.jsonl
beside it, and neither changes anything else; the repairs that
are safe (reload a job that is neither loaded nor paused, prune old run dirs)
are `python3 tools/bees/reviewer.py doctor --fix`, run on purpose.
"""
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
REVIEWER = ROOT / "tools" / "bees" / "reviewer.py"
ROUTES = {"status": ["doctor"], "queue": ["queue"], "stats": ["stats"], "tick": ["tick"],
          "golden": ["eval", "--last"], "wire": ["wire"], "self-test": ["self-test"]}


def main(argv):
    if argv and argv[0] in ("-h", "--help", "help"):
        print(__doc__)
        return 0
    sub = argv[0] if argv else "status"
    if sub not in ROUTES:
        print(f"tri review: unknown subcommand {sub!r}; one of {', '.join(ROUTES)}", file=sys.stderr)
        return 2
    if not REVIEWER.exists():
        print(f"tri review: {REVIEWER} is missing", file=sys.stderr)
        return 2
    extra = argv[1:]
    if sub == "status" and "--fix" in extra:
        print("tri review: tri helpers never mutate; run `python3 tools/bees/reviewer.py doctor --fix`",
              file=sys.stderr)
        return 2
    return subprocess.run([sys.executable, str(REVIEWER), *ROUTES[sub], *extra], env=os.environ).returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
