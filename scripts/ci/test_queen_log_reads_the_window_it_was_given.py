#!/usr/bin/env python3
"""Test that queen_log reads the window it was given correctly."""

import subprocess
import sys
import tempfile
import os


def run_queen_log(file_path):
    """Run queen_log with the given file and return (exit_code, stdout, stderr)."""
    script_path = "/workspace/t27/.worktrees/queen-6056/scripts/tri_loop/queen_log.py"
    cmd = [sys.executable, script_path, "--file", file_path]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.returncode, result.stdout, result.stderr


def test_fixture_file():
    """Test queen_log with the fixture file."""
    fixture_path = "/workspace/t27/.worktrees/queen-6056/scripts/ci/fixtures/queen_log/window.jsonl"
    
    exit_code, stdout, stderr = run_queen_log(fixture_path)
    
    expected_output = """lines: 7 (1 not JSON)
window: 2026-10-04T11:00:00Z .. 2026-10-04T11:05:00Z
last tick: chosen=5835 candidates=774
dispatches: 1
last review: accept=1 send=0 wait=2 escalate=0 empty=1
diff-read failures: 2 on 1 issues (#5567)
ring-00 disagreements: 1
NOT ESTABLISHED: This card describes only the window it was given, not the engine's state now.
"""
    
    if exit_code != 0:
        print(f"FAIL: Expected exit code 0, got {exit_code}")
        print(f"stderr: {stderr}")
        return False
    
    if stdout != expected_output:
        print("FAIL: Output doesn't match expected")
        print("Expected:")
        print(repr(expected_output))
        print("Got:")
        print(repr(stdout))
        return False
    
    print("PASS: Fixture file test")
    return True


def test_missing_file():
    """Test queen_log with a missing file."""
    exit_code, stdout, stderr = run_queen_log("/nonexistent/file.jsonl")
    
    if exit_code != 2:
        print(f"FAIL: Expected exit code 2, got {exit_code}")
        return False
    
    if stdout != "":
        print(f"FAIL: Expected no stdout, got: {stdout}")
        return False
    
    if "File not found" not in stderr:
        print(f"FAIL: Expected error message about file not found, got: {stderr}")
        return False
    
    print("PASS: Missing file test")
    return True


def test_only_brace():
    """Test queen_log with a file containing only '}'."""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.jsonl', delete=False) as f:
        f.write('}\n')
        temp_file = f.name
    
    try:
        exit_code, stdout, stderr = run_queen_log(temp_file)
        
        if exit_code != 2:
            print(f"FAIL: Expected exit code 2, got {exit_code}")
            return False
        
        if stdout != "":
            print(f"FAIL: Expected no stdout, got: {stdout}")
            return False
        
        if "No JSON lines found" not in stderr:
            print(f"FAIL: Expected error message about no JSON lines, got: {stderr}")
            return False
        
        print("PASS: Only brace test")
        return True
    
    finally:
        os.unlink(temp_file)


def main():
    """Run all tests."""
    print("Running queen_log tests...")
    
    tests = [
        test_fixture_file,
        test_missing_file,
        test_only_brace,
    ]
    
    passed = 0
    for test in tests:
        if test():
            passed += 1
    
    print(f"\nResults: {passed}/{len(tests)} tests passed")
    
    if passed == len(tests):
        print("All tests passed!")
        return 0
    else:
        print("Some tests failed!")
        return 1


if __name__ == '__main__':
    sys.exit(main())