"""measure.py: Functions for measuring Zig test output.

This module provides helper functions to parse the output of `zig test` and
compute statistics about the test suite, such as the number of lines in
function bodies, etc.
"""

import re
from typing import List, Set, Tuple


def silent(output: str) -> int:
    """Return the number of deleted-receiver calls in Zig test output.

    A deleted-receiver call is detected by the pattern `).name(` where a call
    result is immediately followed by a method call. This matches cases like
    `remainder.abs().compare_abs(b)` -> `compare_abs(b)`.

    Note: This detector does NOT catch plain-identifier receivers like
    `x.abs()` -> `abs()` because the pattern requires a closing parenthesis
    before the dot.

    Args:
        output: The raw output string from `zig test` (or any Zig code snippet).

    Returns:
        The count of deleted-receiver calls found.
    """
    # Pattern: a closing parenthesis, then a dot, then an identifier, then an opening parenthesis.
    # This matches `).name(`.
    pattern = re.compile(r'\)\.[a-zA-Z_][a-zA-Z0-9_]*\(')
    matches = pattern.findall(output)
    return len(matches)


def truncated_bodies(output: str) -> int:
    """Return the number of truncated function bodies in Zig test output.

    A truncated body is indicated by the word "truncated" (case-insensitive) in the
    error output. This function counts occurrences of that word.

    Args:
        output: The raw output string from `zig test`.

    Returns:
        The count of truncated bodies.
    """
    # Count occurrences of the word "truncated" (case-insensitive)
    pattern = re.compile(r'\btruncated\b', re.IGNORECASE)
    matches = pattern.findall(output)
    return len(matches)


def parse_zig_test_output(output: str) -> Tuple[Set[str], Set[str], int]:
    """Parse Zig test output to extract failing files, phantom imports, and truncated count.

    Args:
        output: The raw output string from `zig test`.

    Returns:
        A tuple (failing_files, phantom_imports, truncated_count) where:
        - failing_files: Set of relative paths to files that contain failing tests.
        - phantom_imports: Set of relative paths to files that are referenced in
          error messages but do not actually exist (phantom imports).
        - truncated_count: Number of truncated function bodies encountered.

    Note: This function was extracted from `measure()` to make it testable
    without running a real `zig test` subprocess.
    """
    failing_files: Set[str] = set()
    phantom_imports: Set[str] = set()
    truncated_count = 0

    # Pattern for phantom imports: "error: file not found: 'filename'"
    phantom_pattern = re.compile(r"^error: file not found: '([^']+)'")

    # Pattern for truncated bodies: "function body truncated"
    truncated_pattern = re.compile(r'function body truncated', re.IGNORECASE)

    # Pattern for real failing files: "path/to/file.zig:line:col: error:"
    # or "path/to/file.zig:line: error:" (column optional)
    # We capture the file path before the first colon.
    failing_pattern = re.compile(r'^([^:]+):\d+(?::\d+)?:\s*error:')

    for line in output.splitlines():
        # Check for phantom imports first
        match = phantom_pattern.search(line)
        if match:
            filepath = match.group(1)
            phantom_imports.add(filepath)
            continue

        # Check for truncated bodies
        if truncated_pattern.search(line):
            truncated_count += 1
            continue

        # Check for failing files
        match = failing_pattern.search(line)
        if match:
            filepath = match.group(1)
            failing_files.add(filepath)
            continue

    return failing_files, phantom_imports, truncated_count


def measure(output: str) -> dict:
    """Measure Zig test output and return a dictionary of metrics.

    This function uses `parse_zig_test_output` to extract data and then
    computes additional metrics using `silent()` and `truncated_bodies()`.

    Args:
        output: The raw output string from `zig test`.

    Returns:
        A dictionary with keys such as 'silent', 'truncated_bodies',
        'failing_files', 'phantom_imports', etc.
    """
    failing_files, phantom_imports, truncated_count = parse_zig_test_output(output)
    silent_count = silent(output)
    # Note: truncated_bodies() might be computed from output as well?
    # For now, we use the truncated_count from parse_zig_test_output.
    return {
        'silent': silent_count,
        'truncated_bodies': truncated_count,
        'failing_files': failing_files,
        'phantom_imports': phantom_imports,
    }