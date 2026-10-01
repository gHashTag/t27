"""Measure Zig test output for code metrics."""

import re
from typing import List, Tuple, Set


def parse_zig_test_output(lines: List[str]) -> Tuple[Set[str], Set[str], int]:
    """Parse Zig test output lines.

    Returns:
        Tuple of (failing_files, phantom_imports, truncated_body_count)
    """
    import re
    failing_files = set()
    phantom_imports = set()
    truncated_body_count = 0

    # Patterns for phantom imports:
    # 1. cannot find file 'filename' or unable to open file 'filename'
    phantom_pattern1 = re.compile(r"(?:cannot find file|unable to open file)\s+'([^']+)'")
    # 2. error: filename: No such file or directory
    phantom_pattern2 = re.compile(r"error:\s+([^:]+):\s+No such file or directory")

    # Pattern for failing file: file_path:line:col: at start of line (general)
    failing_file_pattern = re.compile(r'^([^:]+):\d+:\d+:')

    # Patterns for truncated body
    truncated_pattern = re.compile(r'\(\s*function\s+body\s+truncated\s*\)|\(\s*body\s+truncated\s*\)')

    for line in lines:
        # Check for phantom imports pattern 1
        m = phantom_pattern1.search(line)
        if m:
            phantom_imports.add(m.group(1))
            continue
        # Check for phantom imports pattern 2
        m = phantom_pattern2.search(line)
        if m:
            phantom_imports.add(m.group(1))
            continue

        # Check for failing file (starts with file_path:line:column:)
        m = failing_file_pattern.match(line)
        if m:
            # Ignore lines about unable to open input file (treated as phantom, not failing)
            if "unable to open input file" not in line:
                failing_files.add(m.group(1))
            # Note: we don't continue because a line could also indicate truncated body? unlikely

        # Check for truncated body
        if truncated_pattern.search(line):
            truncated_body_count += 1

    return failing_files, phantom_imports, truncated_body_count


def silent(lines: List[str]) -> int:
    """Count deleted receivers in Zig test output.

    Only matches patterns like `).name(` (call result receiver).
    Does NOT match plain-identifier receivers like `x.abs()`.
    """
    import re
    pattern = re.compile(r'\)\.\s*\w+\s*\(')
    count = 0
    for line in lines:
        # Find all non-overlapping matches
        matches = pattern.findall(line)
        count += len(matches)
    return count


def truncated_bodies(lines: List[str]) -> int:
    """Count truncated function bodies in Zig test output."""
    count = 0
    # TODO: Implement
    return count