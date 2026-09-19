#!/usr/bin/env python3
"""
Check that changed .t27 files still parse -- a ratchet against new NOPARSE.

This script compares `t27c spec-status` at the merge-base (or base ref) against
HEAD for every changed .t27 file. It fails when a file that parsed at the base
does not parse at HEAD, naming each file and the parse error. A file that was
already NOPARSE at the base is not a new failure (this is a ratchet, not a
cleanup mandate).

Exit codes:
  0 -- all changed .t27 files that parsed at base still parse at HEAD
  1 -- one or more files regressed from parsing to NOPARSE
  2 -- could not run (t27c not buildable/runnable, git errors, etc.)

Self-test (--self-test): proves the checker rejects the shape it exists to
reject by creating a temporary spec that parses, breaking it, and verifying
the checker catches the regression. Prints a line beginning with "ok" on success.
"""

import argparse
import os
import re
import subprocess
import sys
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Tuple


REPO_ROOT = Path(__file__).resolve().parents[2]
T27C = shutil.which("t27c") or "/usr/local/bin/t27c"


class SpecStatusChecker:
    def __init__(self, base_ref: str, head_ref: str = "HEAD"):
        self.base_ref = base_ref
        self.head_ref = head_ref
        self.changed_files: List[str] = []
        self.base_status: Dict[str, Tuple[str, str]] = {}  # file -> (status, error)
        self.head_status: Dict[str, Tuple[str, str]] = {}

    def run_cmd(self, cmd: List[str], cwd: Optional[Path] = None) -> subprocess.CompletedProcess:
        """Run a command and return the result."""
        return subprocess.run(cmd, cwd=cwd or REPO_ROOT, capture_output=True, text=True)

    def get_changed_t27_files(self) -> List[str]:
        """Get list of changed .t27 files between base and head."""
        # Try merge-base first for PRs, fall back to base ref directly
        result = self.run_cmd(["git", "merge-base", self.base_ref, self.head_ref])
        if result.returncode == 0:
            merge_base = result.stdout.strip()
        else:
            merge_base = self.base_ref

        result = self.run_cmd(["git", "diff", "--name-only", merge_base, self.head_ref])
        if result.returncode != 0:
            raise RuntimeError(f"git diff failed: {result.stderr}")

        files = [f for f in result.stdout.strip().split("\n") if f.endswith(".t27")]
        return files

    def get_spec_status(self, file: str, ref: str) -> Tuple[str, str]:
        """
        Get spec-status for a file at a given ref.
        Returns (status, error_message). status is one of: PARSED, NOPARSE, MISSING, ERROR.
        """
        # Check if file exists at ref
        result = self.run_cmd(["git", "show", f"{ref}:{file}"])
        if result.returncode != 0:
            return ("MISSING", "File not found at ref")

        # Write to temp file and run t27c spec-status
        with tempfile.NamedTemporaryFile(mode="w", suffix=".t27", delete=False) as f:
            f.write(result.stdout)
            temp_path = f.name

        try:
            result = self.run_cmd([T27C, "spec-status", temp_path])
        finally:
            os.unlink(temp_path)

        if result.returncode != 0:
            # t27c spec-status returns non-zero for NOPARSE
            stderr = result.stderr.strip()
            stdout = result.stdout.strip()
            error_msg = stderr or stdout or "t27c spec-status failed"
            return ("NOPARSE", error_msg)

        # Parse the output: "file.t27: PARSED" or "file.t27: NOPARSE ..."
        stdout = result.stdout.strip()
        if ": PARSED" in stdout:
            return ("PARSED", "")
        elif ": NOPARSE" in stdout:
            # Extract error message after NOPARSE
            parts = stdout.split("NOPARSE", 1)
            error_msg = parts[1].strip() if len(parts) > 1 else "Parse error"
            return ("NOPARSE", error_msg)
        else:
            return ("ERROR", f"Unexpected output: {stdout}")

    def check_t27c_available(self) -> bool:
        """Check if t27c is available and runnable."""
        result = self.run_cmd([T27C, "--version"])
        return result.returncode == 0

    def run_check(self) -> int:
        """Run the main check. Returns exit code."""
        # Check t27c availability
        if not self.check_t27c_available():
            print(f"ERROR: t27c not found or not runnable at {T27C}", file=sys.stderr)
            return 2

        # Get changed files
        try:
            self.changed_files = self.get_changed_t27_files()
        except RuntimeError as e:
            print(f"ERROR: {e}", file=sys.stderr)
            return 2

        if not self.changed_files:
            print("No .t27 files changed.")
            return 0

        print(f"Checking {len(self.changed_files)} changed .t27 file(s)...")

        # Get status at base and head for each changed file
        regressed_files = []

        for file in self.changed_files:
            base_status, base_error = self.get_spec_status(file, self.base_ref)
            head_status, head_error = self.get_spec_status(file, self.head_ref)

            self.base_status[file] = (base_status, base_error)
            self.head_status[file] = (head_status, head_error)

            # Ratchet logic: only fail if it parsed at base but doesn't at head
            if base_status == "PARSED" and head_status == "NOPARSE":
                regressed_files.append((file, head_error))
            elif base_status == "PARSED" and head_status == "MISSING":
                # File was deleted - this is a different kind of change
                print(f"  {file}: PARSED -> MISSING (deleted)")
            elif base_status == "NOPARSE" and head_status == "PARSED":
                print(f"  {file}: NOPARSE -> PARSED (fixed!)")
            elif base_status == "NOPARSE" and head_status == "NOPARSE":
                # Already broken, not a new failure
                pass
            elif base_status == "MISSING" and head_status == "PARSED":
                print(f"  {file}: NEW FILE (PARSED)")
            elif base_status == "MISSING" and head_status == "NOPARSE":
                print(f"  {file}: NEW FILE (NOPARSE)")
            elif base_status == "ERROR" or head_status == "ERROR":
                print(f"  {file}: ERROR checking status")
                regressed_files.append((file, "ERROR checking status"))

        # Report results
        if regressed_files:
            print("\nREGRESSION: The following files parsed at base but do not parse at HEAD:")
            for file, error in regressed_files:
                print(f"  {file}")
                print(f"    Parse error: {error}")
            return 1

        print("All changed .t27 files that parsed at base still parse at HEAD.")
        return 0

    def run_self_test(self) -> int:
        """
        Self-test: prove the checker rejects the shape it exists to reject.
        Creates a temporary spec that parses, breaks it, and verifies the checker catches it.
        """
        print("Running self-test...")

        # Create a temporary directory for the test
        with tempfile.TemporaryDirectory() as tmpdir:
            tmpdir_path = Path(tmpdir)
            test_file = tmpdir_path / "test_spec.t27"

            # A valid spec that parses
            valid_spec = """spec test_spec {
    test trivial {
        assert true
    }
}"""

            # An invalid spec that doesn't parse (missing closing brace)
            invalid_spec = """spec test_spec {
    test trivial {
        assert true
    }
"""

            # Write valid spec
            test_file.write_text(valid_spec)

            # Initialize a git repo
            self.run_cmd(["git", "init"], cwd=tmpdir_path)
            self.run_cmd(["git", "config", "user.email", "test@test.com"], cwd=tmpdir_path)
            self.run_cmd(["git", "config", "user.name", "Test"], cwd=tmpdir_path)
            self.run_cmd(["git", "add", "test_spec.t27"], cwd=tmpdir_path)
            self.run_cmd(["git", "commit", "-m", "Initial commit"], cwd=tmpdir_path)

            # Get the base commit
            result = self.run_cmd(["git", "rev-parse", "HEAD"], cwd=tmpdir_path)
            base_commit = result.stdout.strip()

            # Write invalid spec
            test_file.write_text(invalid_spec)
            self.run_cmd(["git", "add", "test_spec.t27"], cwd=tmpdir_path)
            self.run_cmd(["git", "commit", "-m", "Break the spec"], cwd=tmpdir_path)

            # Run the checker logic manually on this temp repo
            # We need to check status at base vs HEAD
            base_status, base_error = self.get_spec_status_at_ref(tmpdir_path, "test_spec.t27", base_commit)
            head_status, head_error = self.get_spec_status_at_ref(tmpdir_path, "test_spec.t27", "HEAD")

            if base_status != "PARSED":
                print(f"FAIL: Base spec should parse but got {base_status}: {base_error}", file=sys.stderr)
                return 1

            if head_status != "NOPARSE":
                print(f"FAIL: Head spec should not parse but got {head_status}: {head_error}", file=sys.stderr)
                return 1

            # Now test the checker would catch this
            # Simulate the check by creating a checker instance with the temp repo
            checker = SpecStatusChecker(base_commit, "HEAD")
            # Override REPO_ROOT for this test
            old_root = globals()['REPO_ROOT']
            globals()['REPO_ROOT'] = tmpdir_path
            try:
                exit_code = checker.run_check()
            finally:
                globals()['REPO_ROOT'] = old_root

            if exit_code != 1:
                print(f"FAIL: Checker should exit 1 for regression but got {exit_code}", file=sys.stderr)
                return 1

            # Now test that a file already NOPARSE at base is NOT a failure
            # Create another commit where base is already NOPARSE
            test_file.write_text(invalid_spec)  # Keep it broken
            self.run_cmd(["git", "add", "test_spec.t27"], cwd=tmpdir_path)
            self.run_cmd(["git", "commit", "-m", "Another commit on broken spec"], cwd=tmpdir_path)

            checker2 = SpecStatusChecker("HEAD~1", "HEAD")
            globals()['REPO_ROOT'] = tmpdir_path
            try:
                exit_code2 = checker2.run_check()
            finally:
                globals()['REPO_ROOT'] = old_root

            if exit_code2 != 0:
                print(f"FAIL: Checker should exit 0 for already-broken spec but got {exit_code2}", file=sys.stderr)
                return 1

        print("ok self-test passed")
        return 0

    def get_spec_status_at_ref(self, repo_root: Path, file: str, ref: str) -> Tuple[str, str]:
        """Get spec-status for a file at a given ref in a specific repo."""
        result = subprocess.run(
            ["git", "show", f"{ref}:{file}"],
            cwd=repo_root,
            capture_output=True,
            text=True
        )
        if result.returncode != 0:
            return ("MISSING", "File not found at ref")

        with tempfile.NamedTemporaryFile(mode="w", suffix=".t27", delete=False) as f:
            f.write(result.stdout)
            temp_path = f.name

        try:
            result = subprocess.run(
                [T27C, "spec-status", temp_path],
                cwd=repo_root,
                capture_output=True,
                text=True
            )
        finally:
            os.unlink(temp_path)

        if result.returncode != 0:
            stderr = result.stderr.strip()
            stdout = result.stdout.strip()
            error_msg = stderr or stdout or "t27c spec-status failed"
            return ("NOPARSE", error_msg)

        stdout = result.stdout.strip()
        if ": PARSED" in stdout:
            return ("PARSED", "")
        elif ": NOPARSE" in stdout:
            parts = stdout.split("NOPARSE", 1)
            error_msg = parts[1].strip() if len(parts) > 1 else "Parse error"
            return ("NOPARSE", error_msg)
        else:
            return ("ERROR", f"Unexpected output: {stdout}")


def main():
    parser = argparse.ArgumentParser(
        description="Check that changed .t27 files still parse (ratchet against new NOPARSE)"
    )
    parser.add_argument(
        "--base",
        default="origin/master",
        help="Base ref to compare against (default: origin/master)"
    )
    parser.add_argument(
        "--head",
        default="HEAD",
        help="Head ref to check (default: HEAD)"
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run self-test to prove the checker rejects regressions"
    )
    args = parser.parse_args()

    if args.self_test:
        checker = SpecStatusChecker(args.base, args.head)
        sys.exit(checker.run_self_test())

    checker = SpecStatusChecker(args.base, args.head)
    sys.exit(checker.run_check())


if __name__ == "__main__":
    main()