#!/usr/bin/env python3
"""tri queen-log -- read a window of Queen engine logs and print a health card

Usage: tri queen-log --file PATH
       tri queen-log --file -  # read from stdin

Reads JSON lines from Railway's --json output and prints a health card showing
what the Queen chose, how many bees she dispatched, and any anomalies.
"""

import argparse
import json
import sys
from collections import defaultdict


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--file', required=True, 
                       help='File path to read (or "-" for stdin)')
    args = parser.parse_args()

    # Read input
    if args.file == '-':
        lines = sys.stdin.readlines()
    else:
        try:
            with open(args.file, 'r') as f:
                lines = f.readlines()
        except FileNotFoundError:
            print(f"ERROR: File not found: {args.file}", file=sys.stderr)
            sys.exit(2)
        except Exception as e:
            print(f"ERROR: Could not read file {args.file}: {e}", file=sys.stderr)
            sys.exit(2)

    if not lines:
        print("ERROR: No lines read from input", file=sys.stderr)
        sys.exit(2)

    # Parse JSON lines and collect data
    json_lines = []
    not_json_count = 0
    json_objects = []
    total_lines = 0

    for line in lines:
        line = line.strip()
        if not line:
            continue
        
        total_lines += 1
        try:
            obj = json.loads(line)
            json_objects.append(obj)
            json_lines.append(line)
        except json.JSONDecodeError:
            not_json_count += 1

    if not json_objects:
        print("ERROR: No JSON lines found in input", file=sys.stderr)
        sys.exit(2)

    # Extract data from JSON objects
    timestamps = []
    chosen = None
    candidates = None
    dispatch_count = 0
    review_verdicts = None
    diff_read_failures = set()
    diff_read_total_count = 0
    ring_00_disagreements = 0

    for obj in json_objects:
        if 'timestamp' in obj:
            timestamps.append(obj['timestamp'])
        
        if obj.get('message') == 'Queen tick decided':
            chosen = obj.get('chosen')
            candidates = obj.get('candidates')
        
        elif obj.get('message') == 'Queen dispatch':
            dispatch_count += 1
        
        elif obj.get('message') == 'Queen could not read the diff of finished work':
            diff_read_total_count += 1
            if 'issue' in obj:
                diff_read_failures.add(obj['issue'])
        
        elif obj.get('message') == 'Queen reviewed her own work':
            if 'verdicts' in obj:
                review_verdicts = obj['verdicts']
        
        elif obj.get('message') == 'RING-00 DISAGREES WITH THE SWIFT POLICY':
            ring_00_disagreements += 1

    # Prepare output
    earliest = min(timestamps) if timestamps else 'unknown'
    latest = max(timestamps) if timestamps else 'unknown'
    
    # Format chosen/candidates
    if chosen is not None:
        chosen_str = str(chosen)
    else:
        chosen_str = 'none'
    
    if candidates is not None:
        candidates_str = str(candidates)
    else:
        candidates_str = 'none'

    # Count review verdicts
    verdict_counts = defaultdict(int)
    if review_verdicts:
        for verdict in review_verdicts:
            if ':' in verdict:
                outcome = verdict.split(':', 1)[1]
                verdict_counts[outcome] += 1

    # Format diff-read failures
    if diff_read_failures:
        sorted_issues = sorted(diff_read_failures)
        issues_str = ','.join(f'#{issue}' for issue in sorted_issues)
    else:
        issues_str = 'none'

    # Print the seven card lines
    print(f"lines: {total_lines} ({not_json_count} not JSON)")
    print(f"window: {earliest} .. {latest}")
    print(f"last tick: chosen={chosen_str} candidates={candidates_str}")
    print(f"dispatches: {dispatch_count}")
    
    accept = verdict_counts.get('accept', 0)
    send = verdict_counts.get('send', 0)
    wait = verdict_counts.get('wait', 0)
    escalate = verdict_counts.get('escalate', 0)
    empty = verdict_counts.get('empty', 0)
    print(f"last review: accept={accept} send={send} wait={wait} escalate={escalate} empty={empty}")
    
    print(f"diff-read failures: {diff_read_total_count} on {len(diff_read_failures)} issues ({issues_str})")
    print(f"ring-00 disagreements: {ring_00_disagreements}")
    
    print("NOT ESTABLISHED: This card describes only the window it was given, not the engine's state now.")

    sys.exit(0)


if __name__ == '__main__':
    main()