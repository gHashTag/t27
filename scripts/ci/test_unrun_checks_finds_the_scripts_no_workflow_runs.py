#!/usr/bin/env python3
"""Test script for unrun_checks tool"""

import os
import subprocess
import sys
import tempfile
from pathlib import Path


def test_unrun_checks_on_fixture():
    """Test the tool on the fixture package.json"""
    script_dir = Path(__file__).parent.parent / 'tri_loop'
    fixture_dir = Path(__file__).parent / 'fixtures' / 'unrun_checks'
    
    # Run the tool on the fixture
    result = subprocess.run([
        sys.executable, 
        str(script_dir / 'unrun_checks.py'),
        '--package', str(fixture_dir / 'package.json'),
        '--workflows', str(fixture_dir / '.github')
    ], capture_output=True, text=True)
    
    
    
    # Check expected output
    expected_lines = [
        "check:a-extra",
        "check:c",
        "unrun: 2 of 4 check:* scripts",
        "NOT ESTABLISHED: A script run by some other route (a Makefile, a shell script, a person) is still listed as unrun."
    ]
    
    actual_lines = result.stdout.strip().split('\n')
    
    # Check the first three lines match exactly
    if len(actual_lines) < 4:
        return False, f"Expected at least 4 lines, got {len(actual_lines)}"
    
    if actual_lines[0] != "check:a-extra":
        return False, f"Expected first line 'check:a-extra', got '{actual_lines[0]}'"
    if actual_lines[1] != "check:c":
        return False, f"Expected second line 'check:c', got '{actual_lines[1]}'"
    if actual_lines[2] != "unrun: 2 of 4 check:* scripts":
        return False, f"Expected third line 'unrun: 2 of 4 check:* scripts', got '{actual_lines[2]}'"
    
    # Check that NOT ESTABLISHED line is the fourth line
    if actual_lines[3] != "NOT ESTABLISHED: A script run by some other route (a Makefile, a shell script, a person) is still listed as unrun.":
        return False, f"Expected fourth line to be NOT ESTABLISHED message, got '{actual_lines[3]}'"
    
    # Check exit code is 1 (there are unrun scripts)
    if result.returncode != 1:
        return False, f"Expected exit code 1, got {result.returncode}"
    
    return True, "Success"


def test_missing_package_file():
    """Test the tool with a missing package file"""
    script_dir = Path(__file__).parent.parent / 'tri_loop'
    
    # Run the tool with a non-existent package file
    result = subprocess.run([
        sys.executable,
        str(script_dir / 'unrun_checks.py'),
        '--package', '/nonexistent/package.json'
    ], capture_output=True, text=True)
    
    # Check exit code is 2 (file not found)
    if result.returncode != 2:
        return False, f"Expected exit code 2 for missing package file, got {result.returncode}"
    
    # Check that no count is printed (should not contain "unrun:")
    if "unrun:" in result.stdout:
        return False, "Should not print count when package file is missing"
    
    return True, "Success"


def main():
    """Run all tests"""
    print("Running unrun_checks tests...")
    
    # Test 1: Run on fixture
    success1, message1 = test_unrun_checks_on_fixture()
    print(f"Test 1 (fixture): {'PASS' if success1 else 'FAIL'} - {message1}")
    
    # Test 2: Missing package file
    success2, message2 = test_missing_package_file()
    print(f"Test 2 (missing file): {'PASS' if success2 else 'FAIL'} - {message2}")
    
    if success1 and success2:
        print("All tests passed!")
        return 0
    else:
        print("Some tests failed!")
        return 1


if __name__ == '__main__':
    sys.exit(main())