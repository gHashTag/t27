#!/usr/bin/env python3
r"""tri swarm -- one screen of the live swarm: lanes, throughput, and which model the bees are on.

WHY THIS EXISTS
---------------
On 2026-09-21 the swarm read `active 20` for eighteen minutes while it finished
six turns. Every bee was asleep in a 30-second backoff: NVIDIA answered 1,369
requests with 429 or 503. `active` is the number of bees that have started and
not finished, and it cannot tell working from waiting. The only reading that
can is finished-per-interval, and the model ranking the server now keeps
(BrowserOS #496) says why: each candidate's success rate over the last fifteen
minutes, its measured speed, and whether it can call a tool.

This prints both. With --since N it samples twice, N seconds apart, and reports
the finished delta as a rate, which is the number to judge a lane change by.

WHAT THIS ESTABLISHES, AND WHAT IT DOES NOT
-------------------------------------------
Established: what /queen/status answered, and, with --since, how many
dispatches finished between two readings.
Not established: that a finished turn produced useful work (read the verdicts),
or why a model's success rate is low - 503 is the model overloaded, 429 is one
key's rate limit, and this page does not separate them. The server log does.
"""
import argparse
import json
import os
import sys
import time
import urllib.request

DEFAULT_URL = "https://trios-agent-server-production.up.railway.app/queen/status"


def read(url: str) -> dict:
    with urllib.request.urlopen(url, timeout=30) as response:
        return json.load(response)


def main() -> int:
    parser = argparse.ArgumentParser(prog="tri swarm", description=__doc__.split("\n")[0])
    parser.add_argument("--url", default=os.environ.get("TRI_SWARM_STATUS_URL", DEFAULT_URL))
    parser.add_argument("--since", type=int, default=0, metavar="SECONDS",
                        help="sample twice, SECONDS apart, and report finished per hour")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    try:
        first = read(args.url)
    except Exception as error:  # the swarm not answering IS the finding
        print(f"tri swarm: status did not answer: {error}", file=sys.stderr)
        return 2

    rate = None
    last = first
    if args.since > 0:
        time.sleep(args.since)
        try:
            last = read(args.url)
        except Exception as error:
            print(f"tri swarm: second reading failed: {error}", file=sys.stderr)
            return 2
        done = (last.get("dispatches", {}).get("finished") or 0) - (
            first.get("dispatches", {}).get("finished") or 0)
        rate = done * 3600 / args.since

    if args.json:
        print(json.dumps({"status": last, "finishedPerHour": rate}, indent=1))
        return 0

    workers = last.get("workers", {})
    dispatches = last.get("dispatches", {})
    print(f"lanes      {workers.get('active')}/{workers.get('capacity')} active "
          f"(active counts sleeping bees too; judge by finished/hour)")
    print(f"dispatches total {dispatches.get('total')}  finished {dispatches.get('finished')}  "
          f"unreviewed {dispatches.get('unreviewed')}")
    if rate is not None:
        print(f"throughput {rate:.1f} finished/hour over {args.since}s")
    models = last.get("models")
    if not models:
        print("models     ranking off (TRIOS_QUEEN_WORKER_MODEL_CANDIDATES unset)")
        return 0
    print(f"model      {models.get('chosen')}")
    for m in models.get("ranking", []):
        score = m.get("score")
        cost = f"{score:6.1f}s" if isinstance(score, (int, float)) else "     -"
        flags = " GONE" if m.get("gone") else ""
        print(f"  {cost}  ok {m.get('successRate'):.2f} n={m.get('samples'):<4} "
              f"tps {str(m.get('tokensPerSecond') or '-'):>4} tools {str(m.get('toolCalls')):<5} "
              f"{m.get('model')}{flags}")
    print("cost = expected seconds per 500-token step; '-' = not yet measured or no tool call")
    return 0


if __name__ == "__main__":
    sys.exit(main())
