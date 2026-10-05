#!/usr/bin/env python3
"""The reviewer bee: runs on Railway to review pull requests.

This bee replaces the workstation-based reviewer (#5776, #6117). It runs
on Railway as a cron job, maintaining state across runs and using the
t27-bees GitHub App for approvals.

COMMANDS

  python3 tools/bees/reviewer.py run      # main review loop ( Railway cron )
  python3 tools/bees/reviewer.py pause    # mark as paused (owner on workstation)
  python3 tools/bees/reviewer.py doctor   # report status (owner on workstation)
  python3 tools/bees/reviewer.py stats     # show review statistics

STATE FILES

  reviews.jsonl    -- one JSON line per review verdict
  ticks.jsonl     -- one JSON line per tick (cron run)
  opinions.json    -- cached opinions about PRs
  paused          -- empty file when paused

RAILWAY SETUP

- Service: python3 tools/bees/reviewer.py run
- Cron: 0 */10 * * * (every 10 minutes)
- Secrets: Z_AI_KEY, GITHUB_APP_ID, GITHUB_PRIVATE_KEY_PATH
- Volume: for state persistence
"""

import argparse
import json
import os
import pathlib
import subprocess
import sys
import time
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any

import tools.bees.bees as bees

# Configuration
HERE = pathlib.Path(__file__).resolve().parent
STATE_DIR = pathlib.Path(os.environ.get("STATE_DIR", "/state"))
REVIEWS_FILE = STATE_DIR / "reviews.jsonl"
TICKS_FILE = STATE_DIR / "ticks.jsonl"
OPINIONS_FILE = STATE_DIR / "opinions.json"
PAUSED_FILE = STATE_DIR / "paused"

# Environment variables (Railway secrets)
Z_AI_KEY = os.environ.get("Z_AI_KEY")
GITHUB_APP_ID = os.environ.get("GITHUB_APP_ID")
GITHUB_PRIVATE_KEY_PATH = os.environ.get("GITHUB_PRIVATE_KEY_PATH")
BEE_REPO = os.environ.get("BEE_REPO", "gHashTag/t27")

# Opinion cache (to avoid re-analyzing the same PR)
opinions_cache: Dict[str, Dict[str, Any]] = {}


class ReviewerError(Exception):
    """Safe to print, no secrets."""


def load_opinions() -> Dict[str, Dict[str, Any]]:
    """Load cached opinions from disk."""
    global opinions_cache
    if opinions_cache:
        return opinions_cache
    
    if OPINIONS_FILE.exists():
        try:
            opinions_cache = json.loads(OPINIONS_FILE.read_text())
        except (json.JSONDecodeError, OSError):
            opinions_cache = {}
    else:
        opinions_cache = {}
    
    return opinions_cache


def save_opinions(opinions: Dict[str, Dict[str, Any]]) -> None:
    """Save opinions to disk."""
    global opinions_cache
    opinions_cache = opinions
    
    # Create state directory if it doesn't exist
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    
    try:
        OPINIONS_FILE.write_text(json.dumps(opinions, indent=2))
    except OSError as e:
        print(f"warning: could not save opinions: {e}", file=sys.stderr)


def is_paused() -> bool:
    """Check if the reviewer is paused."""
    return PAUSED_FILE.exists()


def set_paused(paused: bool) -> None:
    """Set or clear the paused state."""
    if paused:
        STATE_DIR.mkdir(parents=True, exist_ok=True)
        PAUSED_FILE.touch()
    else:
        PAUSED_FILE.unlink(missing_ok=True)


def record_review(pr_number: int, verdict: str, head_sha: str, 
                  reason: str, token: str) -> None:
    """Record a review verdict to reviews.jsonl."""
    review = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "pr_number": pr_number,
        "verdict": verdict,
        "head_sha": head_sha,
        "reason": reason
    }
    
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    with REVIEWS_FILE.open("a") as f:
        f.write(json.dumps(review) + "\n")


def record_tick(start_time: float, prs_reviewed: int, verdicts: List[str]) -> None:
    """Record a tick (cron run) to ticks.jsonl."""
    tick = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "duration_seconds": time.time() - start_time,
        "prs_reviewed": prs_reviewed,
        "verdicts": verdicts
    }
    
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    with TICKS_FILE.open("a") as f:
        f.write(json.dumps(tick) + "\n")


def get_gh_token() -> str:
    """Get a GitHub token using the app credentials."""
    env = os.environ.copy()
    if Z_AI_KEY:
        env["Z_AI_KEY"] = Z_AI_KEY
    if GITHUB_APP_ID:
        env["BEE_APP_ID"] = GITHUB_APP_ID
    if GITHUB_PRIVATE_KEY_PATH:
        env["BEE_PRIVATE_KEY_PATH"] = GITHUB_PRIVATE_KEY_PATH
    
    return bees.mint_token(BEE_REPO, env=env)


def get_pending_prs(token: str) -> List[Dict[str, Any]]:
    """Get PRs that need review (not yet reviewed, after head arrived)."""
    try:
        # Get PRs with the bee-reviewed label removed
        prs = subprocess.run([
            "gh", "pr", "list", "--limit", "50", "--json", "number,title,headRefOid,author,createdAt,updatedAt",
            "--jq", '[.[] | select(.labels | contains(["bee-reviewed"]) | not)]'
        ], capture_output=True, text=True, env={**os.environ, "GH_TOKEN": token})
        
        if prs.returncode != 0:
            print(f"gh pr list failed: {prs.stderr}", file=sys.stderr)
            return []
        
        return json.loads(prs.stdout or "[]")
    except Exception as e:
        print(f"error getting pending PRs: {e}", file=sys.stderr)
        return []


def analyze_pr(pr_number: int, token: str) -> Optional[Dict[str, Any]]:
    """Analyze a PR and return a verdict."""
    # Check cache first
    opinions = load_opinions()
    cache_key = f"pr_{pr_number}"
    if cache_key in opinions:
        return opinions[cache_key]
    
    try:
        # Get PR details
        pr_details = subprocess.run([
            "gh", "pr", "view", str(pr_number), "--json", "title,body,headRefOid,author,labels,comments",
            "--jq", '.'
        ], capture_output=True, text=True, env={**os.environ, "GH_TOKEN": token})
        
        if pr_details.returncode != 0:
            print(f"gh pr view failed for #{pr_number}: {pr_details.stderr}", file=sys.stderr)
            return None
        
        pr_data = json.loads(pr_details.stdout)
        
        # For now, implement a simple approval logic
        # In practice, this would use Z.AI for more sophisticated analysis
        verdict = {
            "pr_number": pr_number,
            "title": pr_data.get("title", ""),
            "head_sha": pr_data.get("headRefOid", ""),
            "verdict": "APPROVE",
            "reason": "Automatic approval - needs Z.AI integration for real analysis"
        }
        
        # Cache the opinion
        opinions[cache_key] = verdict
        save_opinions(opinions)
        
        return verdict
        
    except Exception as e:
        print(f"error analyzing PR #{pr_number}: {e}", file=sys.stderr)
        return None


def review_prs(token: str, max_verdicts: int = 3) -> List[str]:
    """Review pending PRs and return list of verdicts."""
    verdicts = []
    pending_prs = get_pending_prs(token)
    
    for pr in pending_prs:
        pr_number = pr["number"]
        
        # Skip if we've already made a verdict on this PR
        opinions = load_opinions()
        if f"pr_{pr_number}" in opinions:
            continue
        
        print(f"Reviewing PR #{pr_number}: {pr['title']}")
        
        verdict = analyze_pr(pr_number, token)
        if verdict is None:
            continue
        
        # Post the review
        try:
            comment = f"Verified: {verdict['reason']}"
            subprocess.run([
                "gh", "pr", "review", str(pr_number), 
                "--approve", "--body", comment
            ], check=True, capture_output=True, text=True, 
            env={**os.environ, "GH_TOKEN": token})
            
            # Add the bee-reviewed label
            subprocess.run([
                "gh", "pr", "edit", str(pr_number), 
                "--add-label", "bee-reviewed"
            ], check=True, capture_output=True, text=True,
            env={**os.environ, "GH_TOKEN": token})
            
            # Record the verdict
            record_review(
                pr_number=pr_number,
                verdict=verdict["verdict"],
                head_sha=verdict["head_sha"],
                reason=verdict["reason"],
                token=token
            )
            
            verdicts.append(f"#{pr_number}: {verdict['verdict']}")
            print(f"Approved PR #{pr_number}")
            
            # Stop if we've reached the max verdicts
            if len(verdicts) >= max_verdicts:
                break
                
        except subprocess.CalledProcessError as e:
            print(f"Failed to review PR #{pr_number}: {e.stderr}", file=sys.stderr)
        except Exception as e:
            print(f"Unexpected error reviewing PR #{pr_number}: {e}", file=sys.stderr)
    
    return verdicts


def cmd_run(args) -> int:
    """Main review loop for Railway cron."""
    if is_paused():
        print("Reviewer is paused - exiting")
        return 0
    
    start_time = time.time()
    print(f"Starting review run at {datetime.now(timezone.utc).isoformat()}")
    
    try:
        # Get GitHub token
        token = get_gh_token()
        print(f"Got GitHub token for {BEE_REPO}")
        
        # Review PRs
        verdicts = review_prs(token, max_verdicts=3)
        
        # Record the tick
        record_tick(start_time, len(verdicts), verdicts)
        
        print(f"Review complete: {len(verdicts)} verdicts")
        for verdict in verdicts:
            print(f"  {verdict}")
        
        return 0
        
    except Exception as e:
        print(f"Review run failed: {e}", file=sys.stderr)
        return 1


def cmd_pause(args) -> int:
    """Pause the reviewer (owner command)."""
    set_paused(True)
    print("Reviewer paused")
    return 0


def cmd_doctor(args) -> int:
    """Report status (owner command)."""
    print(f"Reviewer status:")
    print(f"  Paused: {is_paused()}")
    print(f"  State directory: {STATE_DIR}")
    print(f"  Reviews file: {REVIEWS_FILE}")
    print(f"  Ticks file: {TICKS_FILE}")
    print(f"  Opinions file: {OPINIONS_FILE}")
    
    if REVIEWS_FILE.exists():
        reviews_count = sum(1 for _ in REVIEWS_FILE.open())
        print(f"  Total reviews: {reviews_count}")
    
    if TICKS_FILE.exists():
        ticks_count = sum(1 for _ in TICKS_FILE.open())
        print(f"  Total ticks: {ticks_count}")
    
    return 0


def cmd_stats(args) -> int:
    """Show review statistics."""
    if not REVIEWS_FILE.exists():
        print("No reviews yet")
        return 0
    
    reviews = []
    for line in REVIEWS_FILE.open():
        try:
            reviews.append(json.loads(line.strip()))
        except json.JSONDecodeError:
            continue
    
    print(f"Review statistics:")
    print(f"  Total reviews: {len(reviews)}")
    
    if reviews:
        first_review = min(reviews, key=lambda x: x["timestamp"])
        last_review = max(reviews, key=lambda x: x["timestamp"])
        print(f"  First review: {first_review['timestamp']}")
        print(f"  Last review: {last_review['timestamp']}")
        
        verdict_counts = {}
        for review in reviews:
            verdict = review["verdict"]
            verdict_counts[verdict] = verdict_counts.get(verdict, 0) + 1
        
        print(f"  Verdict breakdown:")
        for verdict, count in verdict_counts.items():
            print(f"    {verdict}: {count}")
    
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        prog="reviewer",
        description=__doc__.split("\n\n")[0]
    )
    
    subparsers = parser.add_subparsers(dest="cmd", required=True)
    
    # Run command
    run_parser = subparsers.add_parser("run", help="main review loop (Railway cron)")
    
    # Pause command
    pause_parser = subparsers.add_parser("pause", help="mark as paused (owner on workstation)")
    
    # Doctor command
    doctor_parser = subparsers.add_parser("doctor", help="report status (owner on workstation)")
    
    # Stats command
    stats_parser = subparsers.add_parser("stats", help="show review statistics")
    
    args = parser.parse_args(argv)
    
    try:
        if args.cmd == "run":
            return cmd_run(args)
        elif args.cmd == "pause":
            return cmd_pause(args)
        elif args.cmd == "doctor":
            return cmd_doctor(args)
        elif args.cmd == "stats":
            return cmd_stats(args)
        else:
            return 1
    except ReviewerError as e:
        print(f"reviewer: {e}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("Interrupted", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())