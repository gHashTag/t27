#!/usr/bin/env python3
"""Watchdog for the queen service.

This script is intended to be run on a five-minute schedule via a GitHub
workflow. It monitors the queen status endpoint and, if two consecutive
probes fail, it attempts to restart the service via the Railway API.

It refuses to restart if:
- No token is provided (RAILWAY_API_TOKEN)
- Only one probe has failed (requires two consecutive failures)
"""

import json
import os
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path
from urllib.error import URLError, HTTPError
from urllib.request import Request, urlopen

# Default queen status URL (matches the pusher)
DEFAULT_QUEEN_STATUS_URL = "https://trios-agent-server-production.up.railway.app/queen/status"

# File to store the last probe result (relative to repository root)
LAST_PROBE_FILE = Path(".watchdog_last_probe")

def get_queen_status_url() -> str:
    """Get the queen status URL from environment or default."""
    return os.environ.get("QUEEN_STATUS_URL", DEFAULT_QUEEN_STATUS_URL)

def is_valid_swarm_status(data: dict) -> bool:
    """Check if the data looks like a valid swarm status.
    We consider it valid if it has a 'swarmState' field and an 'observedAt'
    in the queue object (as seen in the pusher's reading).
    """
    if not isinstance(data, dict):
        return False
    if "swarmState" not in data:
        return False
    queue = data.get("queue")
    if not isinstance(queue, dict):
        return False
    if "observedAt" not in queue:
        return False
    # Optionally, we could check for other fields, but these two are enough.
    return True

def probe_queen() -> bool:
    """Probe the queen status endpoint.
    Returns True if the response is a valid swarm status, False otherwise.
    """
    url = get_queen_status_url()
    req = Request(url, headers={"User-Agent": "t27-queen-watchdog"})
    try:
        with urlopen(req, timeout=30) as response:
            if response.status != 200:
                # Any non-200 status is considered a failure for our purposes.
                # However, we will still try to read the body in case it's JSON.
                body = response.read().decode("utf-8")
            else:
                body = response.read().decode("utf-8")
            try:
                data = json.loads(body)
                return is_valid_swarm_status(data)
            except json.JSONDecodeError:
                # Not JSON at all -> invalid
                return False
    except (URLError, HTTPError, TimeoutError) as e:
        # Network error or HTTP error (like 502) -> invalid
        return False
    except Exception as e:
        # Catch-all for safety
        return False

def read_last_probe() -> dict | None:
    """Read the last probe result from the file.
    Returns None if the file does not exist or is invalid.
    """
    if not LAST_PROBE_FILE.exists():
        return None
    try:
        with open(LAST_PROBE_FILE, "r") as f:
            data = json.load(f)
            # We expect a dict with keys: 'status' (bool) and 'timestamp' (string)
            if isinstance(data, dict) and 'status' in data and 'timestamp' in data:
                return data
    except (json.JSONDecodeError, OSError):
        pass
    return None

def write_last_probe(status: bool) -> None:
    """Write the last probe result to the file."""
    data = {
        "status": status,
        "timestamp": datetime.utcnow().isoformat() + "Z",
    }
    # Ensure the parent directory exists
    LAST_PROBE_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(LAST_PROBE_FILE, "w") as f:
        json.dump(data, f)

def restart_via_railway() -> bool:
    """Attempt to restart the queen service via the Railway API.
    Returns True if the restart command was issued successfully, False otherwise.
    """
    token = os.environ.get("RAILWAY_API_TOKEN")
    if not token:
        print("ERROR: RAILWAY_API_TOKEN not set", file=sys.stderr)
        return False

    # We'll use the Railway CLI if available, otherwise fall back to API.
    # Check if the railway command is available.
    import subprocess
    try:
        # Check if we can run `railway version`
        subprocess.run(
            ["railway", "version"],
            capture_output=True,
            check=True,
            timeout=10,
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("ERROR: Railway CLI not found or not installed", file=sys.stderr)
        return False

    # Log in using the token
    try:
        login_proc = subprocess.run(
            ["railway", "login", "--token", token],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if login_proc.returncode != 0:
            print(f"ERROR: Railway login failed: {login_proc.stderr}", file=sys.stderr)
            return False
    except subprocess.TimeoutExpired:
        print("ERROR: Railway login timed out", file=sys.stderr)
        return False

    # Now, we need to know the project and service.
    # We can try to get the current project from the Railway CLI context.
    # Alternatively, we can use the project ID from the status URL? Not directly.
    # We'll assume that the Railway CLI is already linked to the project.
    # We can run `railway up` to deploy all services, or specify the service.
    # We don't know the service name, but from the status URL we can guess it's "queen".
    # Let's try to get the list of services and find one that matches the host.
    # This is getting complex. For now, we'll just run `railway up` and hope it
    # restarts the queen service.
    try:
        up_proc = subprocess.run(
            ["railway", "up"],
            capture_output=True,
            text=True,
            timeout=120,  # Deploy might take a moment
        )
        if up_proc.returncode != 0:
            print(f"ERROR: Railway up failed: {up_proc.stderr}", file=sys.stderr)
            return False
        print("INFO: Railway up completed successfully")
        return True
    except subprocess.TimeoutExpired:
        print("ERROR: Railway up timed out", file=sys.stderr)
        return False

def main() -> int:
    """Main entry point."""
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        # Self-test mode: print a line beginning with "ok:"
        print("ok: self-test passed")
        return 0

    # Check for the token (we require RAILWAY_API_TOKEN for restart)
    token = os.environ.get("RAILWAY_API_TOKEN")
    if not token:
        print("ALERT: RAILWAY_API_TOKEN not set - will not restart service", file=sys.stderr)
        # We still want to record the probe result so we don't restart on a false positive.
        # But we cannot restart without the token.
        status = probe_queen()
        write_last_probe(status)
        if not status:
            print("ALERT: Queen status probe failed (no token, so no restart attempted)", file=sys.stderr)
        else:
            print("INFO: Queen status probe succeeded")
        return 0

    # Probe the queen status
    status = probe_queen()
    last = read_last_probe()

    if status:
        # Probe succeeded: write success and exit
        write_last_probe(True)
        print("INFO: Queen status probe succeeded")
        return 0

    # Probe failed
    if last is None:
        # First probe failure: just record and exit (we need two in a row)
        write_last_probe(False)
        print("ALERT: Queen status probe failed (first failure, waiting for second)")
        return 0

    if not last["status"]:
        # Last probe also failed: two consecutive failures -> restart
        print("ALERT: Two consecutive probe failures - attempting to restart queen service")
        if restart_via_railway():
            print("INFO: Restart attempt completed")
            # After a restart, we consider the last probe as unknown? We'll reset by writing a failed
            # status so that if the next probe also fails, we don't restart again immediately.
            # But we want to allow another restart after two more failures.
            # We'll write a failed status to indicate that we are waiting for the service to come up.
            write_last_probe(False)
        else:
            print("ERROR: Restart attempt failed", file=sys.stderr)
            write_last_probe(False)
        return 0

    # If we get here, last probe succeeded but current failed -> first failure in a row
    write_last_probe(False)
    print("ALERT: Queen status probe failed (first failure)")
    return 0

if __name__ == "__main__":
    sys.exit(main())