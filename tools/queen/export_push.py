#!/usr/bin/env python3
"""Export and push branches from the Queen container to GitHub.

WHY THIS EXISTS

Measured 2026-09-23: 59 open issues carried an `accept` whose work had never left
the container. The Queen skips those as "the work already landed", so they hold
their files, the delegatable backlog empties, and the swarm sits at 0 of 20 lanes
with a queue of finished work nobody can see. The container holds no push
credential by design (a checkout the agents can write is one whose git config they
control), but it exports instead:

- GET /queen/export lists what is ahead of the base
- GET /queen/export/<issue> returns that branch as a base64 git bundle

This script reads the list, fetches each bundle, and pushes the branch. Never
--force: a remote that has commits the bundle lacks is a bee's work, and that is
a person's decision. queen-publish.yml already runs on a push to queen-* branches,
so the pull request follows by itself.

Usage:
    python3 tools/queen/export_push.py --dry-run     # print the reading, push nothing
    python3 tools/queen/export_push.py               # push what is exported
"""
from __future__ import annotations

import argparse
import base64
import json
import os
import subprocess
import sys
import tempfile
import urllib.request
import urllib.error

REPO = os.environ.get("EXPORT_PUSH_REPO", "gHashTag/t27")
QUEEN_EXPORT_URL = os.environ.get("QUEEN_EXPORT_URL", "https://trios-agent-server-production.up.railway.app/queen")
QUEEN_EXPORT_TOKEN = os.environ.get("QUEEN_EXPORT_TOKEN")


def sh(args: list[str], timeout: int = 180) -> tuple[int, str]:
    """Run a shell command and return (returncode, output)."""
    done = subprocess.run(args, capture_output=True, text=True, timeout=timeout,
                          stdin=subprocess.DEVNULL)
    return done.returncode, ((done.stdout or "") + (done.stderr or "")).strip()


def log(message: str) -> None:
    """Log a message with timestamp."""
    from datetime import datetime
    stamp = datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"{stamp} {message}", flush=True)


def get_json(url: str, default) -> dict:
    """Get JSON from a URL with timeout and error handling."""
    try:
        headers = {}
        if QUEEN_EXPORT_TOKEN:
            headers["Authorization"] = f"Bearer {QUEEN_EXPORT_TOKEN}"
        
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=30) as response:
            return json.load(response)
    except urllib.error.HTTPError as e:
        log(f"HTTP {e.code} fetching {url}: {e.reason}")
        return default
    except Exception as e:
        log(f"Error fetching {url}: {e}")
        return default


def get_export_list() -> list[dict]:
    """Get the list of exported branches from /queen/export."""
    url = f"{QUEEN_EXPORT_URL}/export"
    data = get_json(url, {})
    
    if not data:
        log("No data received from export endpoint")
        return []
    
    # The endpoint should return a list of branches with issue numbers
    if isinstance(data, list):
        return data
    elif isinstance(data, dict) and "branches" in data:
        return data["branches"]
    else:
        log(f"Unexpected data format from export endpoint: {type(data)}")
        return []


def fetch_bundle(issue_number: int) -> str | None:
    """Fetch the git bundle for a specific issue."""
    url = f"{QUEEN_EXPORT_URL}/export/{issue_number}"
    data = get_json(url, {})
    
    if not data or "bundle" not in data:
        log(f"No bundle data for issue #{issue_number}")
        return None
    
    return data["bundle"]


def create_branch_from_bundle(bundle_data: str, branch_name: str, temp_dir: str) -> bool:
    """Create a branch from a base64 git bundle in a temporary directory."""
    try:
        # Decode the bundle
        bundle_bytes = base64.b64decode(bundle_data)
        bundle_path = os.path.join(temp_dir, f"{branch_name}.bundle")
        
        with open(bundle_path, "wb") as f:
            f.write(bundle_bytes)
        
        # Initialize git repo and fetch from bundle
        sh(["git", "init"], timeout=30)
        sh(["git", "config", "user.name", "queen-export[bot]"], timeout=30)
        sh(["git", "config", "user.email", "noreply@anthropic.com"], timeout=30)
        
        # Fetch from bundle
        code, out = sh(["git", "fetch", bundle_path, f"{branch_name}:{branch_name}"], timeout=60)
        if code != 0:
            log(f"Failed to fetch from bundle for {branch_name}: {out}")
            return False
        
        # Switch to the branch
        code, out = sh(["git", "checkout", branch_name], timeout=30)
        if code != 0:
            log(f"Failed to checkout {branch_name}: {out}")
            return False
        
        return True
        
    except Exception as e:
        log(f"Error creating branch from bundle for {branch_name}: {e}")
        return False


def push_branch(branch_name: str) -> bool:
    """Push a branch to the remote without --force."""
    # Check if remote branch exists
    code, out = sh(["git", "ls-remote", "--heads", "origin", branch_name], timeout=30)
    
    if code == 0:
        # Remote branch exists, check if it's ahead or behind
        local_rev = sh(["git", "rev-parse", f"HEAD"], timeout=30)[1]
        remote_rev = out.split()[0] if out else ""
        
        if local_rev == remote_rev:
            log(f"Branch {branch_name} is identical to remote, skipping")
            return True
        elif local_rev != remote_rev:
            # Check if remote is ahead (has commits we don't have)
            # This would be a bee's work that someone else modified, so don't force
            log(f"Branch {branch_name} differs from remote (remote ahead), skipping to preserve bee's work")
            return False
    
    # Push without --force
    code, out = sh(["git", "push", "origin", branch_name], timeout=60)
    if code == 0:
        log(f"Pushed {branch_name} successfully")
        return True
    else:
        log(f"Failed to push {branch_name}: {out}")
        return False


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true", help="Print what would be done, don't push")
    args = dry_run = args.dry_run
    
    # Check for required token
    if not QUEEN_EXPORT_TOKEN:
        log("Error: QUEEN_EXPORT_TOKEN environment variable is required")
        log("Set this to the agent server's TRIOS_API_TOKEN")
        return 1
    
    log("Starting export and push process")
    
    # Get the list of exported branches
    export_list = get_export_list()
    if not export_list:
        log("No branches found in export list")
        return 0
    
    log(f"Found {len(export_list)} branches in export list")
    
    # Process each branch
    success_count = 0
    skip_count = 0
    
    for export in export_list:
        issue_number = export.get("issue")
        branch_name = export.get("branch")
        
        if not issue_number or not branch_name:
            log(f"Skipping invalid export entry: {export}")
            skip_count += 1
            continue
        
        log(f"Processing issue #{issue_number}, branch {branch_name}")
        
        if dry_run:
            log(f"[dry-run] Would fetch bundle for #{issue_number} and push {branch_name}")
            success_count += 1
            continue
        
        # Fetch the bundle
        bundle_data = fetch_bundle(issue_number)
        if not bundle_data:
            log(f"Failed to fetch bundle for #{issue_number}")
            skip_count += 1
            continue
        
        # Create temporary directory for this branch
        with tempfile.TemporaryDirectory(prefix=f"export-{issue_number}-") as temp_dir:
            # Create branch from bundle
            if not create_branch_from_bundle(bundle_data, branch_name, temp_dir):
                log(f"Failed to create branch from bundle for #{issue_number}")
                skip_count += 1
                continue
            
            # Push the branch
            if not push_branch(branch_name):
                log(f"Failed to push branch {branch_name}")
                skip_count += 1
                continue
            
            success_count += 1
    
    log(f"Done: success={success_count}, skip={skip_count}")
    return 0


if __name__ == "__main__":
    sys.exit(main())