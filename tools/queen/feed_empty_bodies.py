#!/usr/bin/env python3
"""
Feeder for empty function bodies in specs/**.
Runs on a schedule via GitHub Actions, builds t27c, and reports counts.
"""

import json
import os
import re
import subprocess
import sys
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class EmptyBodyCounts:
    total_empty: int
    total_files: int
    details: list[str]


def get_t27c_binary() -> Path:
    """Get the t27c binary path from T27C_BIN env var or default."""
    bin_path = os.environ.get("T27C_BIN")
    if bin_path:
        return Path(bin_path)
    # Default to the one on PATH
    return Path("t27c")


def run_impl_status(t27c: Path) -> EmptyBodyCounts:
    """Run t27c impl-status --verbose and parse the output."""
    cmd = [str(t27c), "impl-status", "--verbose"]
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=Path.cwd())
    if result.returncode != 0:
        raise RuntimeError(f"t27c impl-status failed: {result.stderr}")

    # Parse output like:
    # UNWRITTEN: 245 empty bodies in 70 files
    output = result.stdout.strip()
    match = re.search(r"UNWRITTEN:\s+(\d+)\s+empty bodies? in\s+(\d+)\s+files?", output)
    if not match:
        # Try alternative format
        match = re.search(r"(\d+)\s+empty.*?(\d+)\s+files?", output)
    if not match:
        raise RuntimeError(f"Could not parse impl-status output: {output}")

    total_empty = int(match.group(1))
    total_files = int(match.group(2))

    # Extract per-file details if present
    details = []
    for line in output.splitlines():
        if "empty" in line.lower() and ":" in line:
            details.append(line.strip())

    return EmptyBodyCounts(total_empty=total_empty, total_files=total_files, details=details)


def claims_hold(commands: list[str], expected_outputs: list[str]) -> bool:
    """
    Execute each command and verify its output matches the expected value.
    Returns True only if all claims hold.
    """
    for cmd, expected in zip(commands, expected_outputs):
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=Path.cwd())
        actual = result.stdout.strip()
        if actual != expected.strip():
            print(f"Claim failed: command '{cmd}' produced '{actual}', expected '{expected}'", file=sys.stderr)
            return False
    return True


def verify_claims_hold(t27c: Path, counts: EmptyBodyCounts) -> bool:
    """
    Verify that the commands we will quote in the issue actually produce
    the values the issue will claim.
    """
    commands = [
        f"{t27c} impl-status --verbose | grep -c UNWRITTEN",
        f"{t27c} impl-status --verbose | grep -o '[0-9]\\+ empty' | head -1",
    ]
    expected = [
        str(counts.total_files) if counts.total_files > 0 else "0",
        f"{counts.total_empty} empty",
    ]
    return claims_hold(commands, expected)


def get_existing_issue_number() -> Optional[int]:
    """Check if there's already an open issue for empty bodies."""
    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        return None

    repo = os.environ.get("GITHUB_REPOSITORY", "gHashTag/t27")
    url = f"https://api.github.com/repos/{repo}/issues?state=open&labels=empty-bodies-feeder&per_page=10"

    req = urllib.request.Request(url)
    req.add_header("Authorization", f"Bearer {token}")
    req.add_header("Accept", "application/vnd.github+json")

    try:
        with urllib.request.urlopen(req) as resp:
            issues = json.load(resp)
            for issue in issues:
                if "empty function bodies" in issue.get("title", "").lower():
                    return issue["number"]
    except Exception:
        pass
    return None


def create_or_update_issue(counts: EmptyBodyCounts, t27c: Path) -> bool:
    """Create or update a GitHub issue with the current empty body counts."""
    token = os.environ.get("GITHUB_TOKEN")
    if not token:
        print("No GITHUB_TOKEN, skipping issue creation", file=sys.stderr)
        return False

    repo = os.environ.get("GITHUB_REPOSITORY", "gHashTag/t27")
    existing = get_existing_issue_number()

    title = f"[feeder] Empty function bodies: {counts.total_empty} in {counts.total_files} files"
    body = f"""## Empty Function Bodies Report

**Total empty bodies**: {counts.total_empty}
**Files affected**: {counts.total_files}

### Verification commands
These commands were executed and their outputs verified before opening this issue:

```bash
# Count files with UNWRITTEN status
$ {t27c} impl-status --verbose | grep -c UNWRITTEN
{counts.total_files}

# Show empty body count
$ {t27c} impl-status --verbose | grep -o '[0-9]\\+ empty' | head -1
{counts.total_empty} empty
```

### Per-file details
```
{chr(10).join(counts.details) if counts.details else 'No details available'}
```

---
*This issue is managed by the empty-bodies feeder (scheduled workflow).*
"""

    url = f"https://api.github.com/repos/{repo}/issues"
    if existing:
        url = f"{url}/{existing}"
        data = {"title": title, "body": body, "state": "open"}
        method = "PATCH"
    else:
        data = {"title": title, "body": body, "labels": ["empty-bodies-feeder", "feeder"]}
        method = "POST"

    req = urllib.request.Request(url, data=json.dumps(data).encode(), method=method)
    req.add_header("Authorization", f"Bearer {token}")
    req.add_header("Accept", "application/vnd.github+json")
    req.add_header("Content-Type", "application/json")

    try:
        with urllib.request.urlopen(req) as resp:
            print(f"Issue {'updated' if existing else 'created'}: {resp.geturl()}")
            return True
    except urllib.error.HTTPError as e:
        print(f"Failed to {'update' if existing else 'create'} issue: {e.read().decode()}", file=sys.stderr)
        return False


def main() -> int:
    t27c = get_t27c_binary()
    print(f"Using t27c at: {t27c}")

    if not t27c.exists() and not any(t27c.name in p for p in os.environ.get("PATH", "").split(":")):
        print(f"t27c not found at {t27c}", file=sys.stderr)
        return 1

    try:
        counts = run_impl_status(t27c)
    except RuntimeError as e:
        print(f"Failed to get impl-status: {e}", file=sys.stderr)
        return 1

    print(f"Found {counts.total_empty} empty bodies in {counts.total_files} files")

    # Verify claims hold before opening/updating issue
    if not verify_claims_hold(t27c, counts):
        print("Claims verification failed, refusing to open issue", file=sys.stderr)
        return 1

    print("All claims hold")

    if not create_or_update_issue(counts, t27c):
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())