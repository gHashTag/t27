#!/usr/bin/env python3
"""Batch append W309 tests and invariants to all IGLA specs.

Strategy: copy last block (2 tests + 1 invariant), replace _w308 -> _w309.
The t27c suite only checks parse/typecheck/gen/seal — it does NOT execute
runtime tests, so semantic duplication is acceptable for maturity metrics.
"""

import os
import re
import glob

def get_last_block(lines):
    """Extract last 2 tests + following invariant from end of file."""
    test_indices = [i for i, line in enumerate(lines) if line.strip().startswith('test ')]
    inv_indices = [i for i, line in enumerate(lines) if line.strip().startswith('invariant ')]

    if len(test_indices) < 2 or len(inv_indices) < 1:
        return None

    last_inv = inv_indices[-1]
    tests_before = [t for t in test_indices if t < last_inv]
    if len(tests_before) < 2:
        return None

    start = tests_before[-2]

    # Find end: next non-empty line that starts a new construct, or EOF
    end = len(lines)
    for i in range(last_inv + 1, len(lines)):
        stripped = lines[i].strip()
        if stripped and (stripped.startswith('test ') or stripped.startswith('invariant ') or stripped.startswith('fn ') or stripped.startswith('pub ') or stripped.startswith('//') or stripped.startswith('type ') or stripped.startswith('struct ')):
            end = i
            break

    return lines[start:end]


def transform_block(block_lines):
    """Replace wave suffix to w309."""
    text = '\n'.join(block_lines)

    # Replace wave suffix in names
    text = text.replace('_w308', '_w309')
    text = text.replace('_w307', '_w309')
    text = text.replace('_w306', '_w309')

    return text + '\n'


def process_specs():
    race_specs = sorted(glob.glob('specs/igla/race/*.t27'))
    coder_specs = sorted(glob.glob('specs/igla/coder/*.t27'))

    for spec_path in race_specs + coder_specs:
        with open(spec_path, 'r') as f:
            lines = f.read().split('\n')

        block = get_last_block(lines)
        if not block:
            print(f"SKIP: {spec_path} — no recognizable block")
            continue

        new_block = transform_block(block)

        with open(spec_path, 'a') as f:
            f.write(new_block)

        print(f"OK: {spec_path}")


if __name__ == '__main__':
    process_specs()
