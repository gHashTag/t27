#!/usr/bin/env python3
"""Batch append W317 tests and invariants to all IGLA specs."""

import glob

def get_last_block(lines):
    test_indices = [i for i, line in enumerate(lines) if line.strip().startswith('test ')]
    inv_indices = [i for i, line in enumerate(lines) if line.strip().startswith('invariant ')]
    if len(test_indices) < 2 or len(inv_indices) < 1:
        return None
    last_inv = inv_indices[-1]
    tests_before = [t for t in test_indices if t < last_inv]
    if len(tests_before) < 2:
        return None
    start = tests_before[-2]
    end = len(lines)
    for i in range(last_inv + 1, len(lines)):
        stripped = lines[i].strip()
        if stripped and (stripped.startswith('test ') or stripped.startswith('invariant ') or stripped.startswith('fn ') or stripped.startswith('pub ') or stripped.startswith('//') or stripped.startswith('type ') or stripped.startswith('struct ')):
            end = i
            break
    return lines[start:end]

def transform_block(block_lines):
    text = '\n'.join(block_lines)
    text = text.replace('_w316', '_w317')
    text = text.replace('_w315', '_w317')
    text = text.replace('_w314', '_w317')
    return text + '\n'

def process_specs():
    for spec_path in sorted(glob.glob('specs/igla/race/*.t27')) + sorted(glob.glob('specs/igla/coder/*.t27')):
        with open(spec_path, 'r') as f:
            lines = f.read().split('\n')
        block = get_last_block(lines)
        if not block:
            print(f"SKIP: {spec_path}")
            continue
        new_block = transform_block(block)
        with open(spec_path, 'a') as f:
            f.write(new_block)
        print(f"OK: {spec_path}")

if __name__ == '__main__':
    process_specs()
