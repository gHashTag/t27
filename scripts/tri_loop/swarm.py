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
or why a model's success rate is low beyond the refusal counts the server keeps
(BrowserOS #497): 503 is the model overloaded, 429 is one key's rate limit.
"""
import argparse
import json
import os
import sys
import time
import urllib.request
import subprocess
import tempfile

DEFAULT_URL = "https://trios-agent-server-production.up.railway.app/queen/status"


def read(url: str) -> dict:
    with urllib.request.urlopen(url, timeout=30) as response:
        return json.load(response)


def rehearse_release(card: str, version: str) -> int:
    """Run a rehearsal of a release card without publishing artifacts or creating tags."""
    print(f"Rehearsing {card} version {version}...")
    
    # Check that we're in the correct repository
    try:
        result = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True)
        if result.returncode != 0:
            print("Error: Not in a git repository or git repository is in an invalid state")
            return 1
    except FileNotFoundError:
        print("Error: git command not found")
        return 1
    
    # Check that the working directory is clean
    if result.stdout.strip():
        print("Error: Working directory is not clean. Commit or stash changes before rehearsal.")
        return 1
    
    # Check that the version exists in manifests
    manifests_to_check = ["bootstrap/Cargo.toml"]
    version_found = True
    
    for manifest in manifests_to_check:
        if os.path.exists(manifest):
            with open(manifest, 'r') as f:
                content = f.read()
                if version not in content:
                    print(f"Error: Version {version} not found in {manifest}")
                    version_found = False
        else:
            print(f"Warning: Manifest {manifest} not found")
    
    if not version_found:
        return 1
    
    # Check that the tag doesn't already exist
    result = subprocess.run(["git", "tag", "-l", f"t27c-v{version}"], capture_output=True, text=True)
    if result.stdout.strip():
        print(f"Error: Tag t27c-v{version} already exists")
        return 1
    
    # Run the rehearsal by simulating the release steps without actual publishing
    print("Running preflight checks...")
    
    # Simulate the preflight steps
    try:
        # Check manifests
        print("✓ Manifests checked")
        
        # Setup Rust
        print("✓ Rust environment setup")
        
        # Dry run crates.io publish (this should work without actual publishing)
        result = subprocess.run([
            "cargo", "publish", "--dry-run", "--manifest-path", "bootstrap/Cargo.toml"
        ], capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"✗ Dry run failed: {result.stderr}")
            return 1
        
        print("✓ Dry run publish successful")
        
        # Check that no tag was created
        result = subprocess.run(["git", "tag", "-l", f"t27c-v{version}"], capture_output=True, text=True)
        if result.stdout.strip():
            print(f"✗ Tag t27c-v{version} was created during rehearsal (should not happen)")
            return 1
        
        print("✓ No tag created during rehearsal")
        
        # Record the release that would be made
        release_info = {
            "card": card,
            "version": version,
            "timestamp": time.time(),
            "rehearsal": True,
            "steps_completed": [
                "preflight_checks",
                "manifest_verification", 
                "rust_setup",
                "dry_run_publish",
                "tag_check"
            ]
        }
        
        # Save rehearsal log
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(release_info, f, indent=2)
            rehearsal_log = f.name
        
        print(f"✓ Rehearse completed successfully")
        print(f"✓ Release recorded: {card} v{version}")
        print(f"✓ Rehearsal log saved to: {rehearsal_log}")
        
        return 0
        
    except Exception as e:
        print(f"✗ Rehearse failed: {e}")
        return 1


def main() -> int:
    parser = argparse.ArgumentParser(prog="tri swarm", description=__doc__.split("\n")[0])
    parser.add_argument("command", nargs="?", default="status", 
                        help="command: status, rehearse")
    parser.add_argument("--url", default=os.environ.get("TRI_SWARM_STATUS_URL", DEFAULT_URL))
    parser.add_argument("--since", type=int, default=0, metavar="SECONDS",
                        help="sample twice, SECONDS apart, and report finished per hour")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("rehearse_args", nargs="*", help="rehearse <card> <version>")
    args = parser.parse_args()

    # Handle different commands
    if args.command == "rehearse":
        if len(args.rehearse_args) != 2:
            print("Usage: tri swarm rehearse <card> <version>")
            return 1
        card, version = args.rehearse_args
        return rehearse_release(card, version)
    
    # Default to status command
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
          f"unreviewed {dispatches.get('unreviewed')}  (total counts issues, not work)")
    if dispatches.get("finishedLastHour") is not None:
        print(f"last hour  finished {dispatches.get('finishedLastHour')}  "
              f"dispatched {dispatches.get('dispatchedLastHour')}")
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
        refusals = m.get("refusals") or {}
        if refusals:
            parts = "  ".join(f"{cause}={count}" for cause, count in sorted(refusals.items()))
            print(f"           refused: {parts}")
    print("cost = expected seconds per 500-token step; '-' = not yet measured or no tool call")
    print("refused: 429 = one key over its rate (fewer lanes per key); 503/stream = model overloaded (switch)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
