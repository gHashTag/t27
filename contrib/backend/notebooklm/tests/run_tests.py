#!/usr/bin/env python3
# contrib/backend/notebooklm/tests/run_tests.py
# Test runner for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Test runner with coverage reporting."""

import sys
import subprocess
from pathlib import Path

# Add project root to path
repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))


def run_test_file(test_path: str) -> tuple[bool, str]:
    """Run a single test file and return (success, output)."""
    result = subprocess.run(
        [sys.executable, test_path],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0, result.stdout + result.stderr


def main():
    """Run all tests and report results."""
    test_dir = Path(__file__).parent
    test_files = [
        "test_config.py",
        "test_auth_token.py",
        "test_wrapup.py",
        "test_session.py",
        "test_client.py",
    ]

    passed = 0
    failed = 0

    print("=" * 60)
    print("NOTEBOOKLM INTEGRATION TEST SUITE")
    print("=" * 60)
    print()

    for test_file in test_files:
        test_path = test_dir / test_file
        if not test_path.exists():
            print(f"[SKIP] {test_file} - not found")
            continue

        print(f"Running {test_file}...")
        success, output = run_test_file(str(test_path))

        if success:
            print(f"[PASS] {test_file}")
            passed += 1
        else:
            print(f"[FAIL] {test_file}")
            print(output)
            failed += 1
        print()

    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Passed: {passed}/{len(test_files)}")
    print(f"Failed: {failed}/{len(test_files)}")

    if failed == 0:
        print("\nAll tests passed!")
        return 0
    else:
        print(f"\n{failed} test(s) failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())
