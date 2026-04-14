#!/usr/bin/env python3
"""
UNIFIED SEARCH ALL — Run ALL discovery methods in parallel
Combines: v6.5 ABSOLUTE, Chimera Search, and all formula combinations
"""

import subprocess
import time
import sys
from datetime import datetime
from pathlib import Path

def run_method(name, script, args):
    """Run a search method"""
    print(f"\n{'='*70}")
    print(f"  {name}")
    print(f"{'='*70}")
    start = time.time()
    result = subprocess.run(
        [sys.executable, script] + args,
        capture_output=True,
        text=True
    )
    elapsed = time.time() - start
    return {
        "method": name,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "returncode": result.returncode,
        "elapsed": elapsed
    }

def main():
    print("="*70)
    print("  UNIFIED SEARCH ALL — ALL METHODS RUNNING")
    print("="*70)
    print(f"  Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print()

    # Run all methods
    results = []

    # Method 1: v6.5 ABSOLUTE (CPU)
    results.append(run_method(
        "v6.5 ABSOLUTE (NumPy + 8-core)",
        "/Users/playra/t27/scripts/ultra_engine_v65_absolute.py",
        []
    ))

    # Method 2: Chimera Search (max_pow=7, threshold=0.01)
    results.append(run_method(
        "Chimera Search (Rust, 3375 basis)",
        "/Users/playra/t27/target/release/t27c",
        ["formula", "chimera-search", "--max-pow", "7", "--threshold", "0.01"]
    ))

    # Method 3: Chimera Search (max_pow=7, threshold=0.03)
    results.append(run_method(
        "Chimera Search (Rust, 3375 basis, wider)",
        "/Users/playra/t27/target/release/t27c",
        ["formula", "chimera-search", "--max-pow", "7", "--threshold", "0.03"]
    ))

    # Print summary
    print("\n" + "="*70)
    print("  UNIFIED SEARCH SUMMARY")
    print("="*70)

    total_formulas = 0
    for r in results:
        print(f"\n  {r['method']}")
        print(f"  Elapsed: {r['elapsed']:.2f}s")
        if r['returncode'] == 0:
            # Extract formula count from output
            lines = r['stdout'].split('\n')
            for line in lines:
                if 'Total formulas found:' in line:
                    count = int(line.split(':')[-1].strip())
                    total_formulas += count
                    print(f"  Formulas found: {count}")
                    break
        else:
            print(f"  ERROR: {r['stderr'][:200]}")

    print(f"\n  TOTAL FORMULAS FROM ALL METHODS: {total_formulas}")
    print(f"  Total elapsed: {sum(r['elapsed'] for r in results):.2f}s")

    # Save unified output
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = f"/tmp/unified_discovery_{timestamp}.txt"

    with open(output_file, "w") as f:
        f.write("# UNIFIED SEARCH ALL — ALL METHODS\n")
        f.write(f"# Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")

        for r in results:
            f.write(f"\n## {r['method']}\n")
            f.write(f"# Elapsed: {r['elapsed']:.2f}s\n")
            f.write(r['stdout'])

        f.write(f"\n## SUMMARY\n")
        f.write(f"Total formulas from all methods: {total_formulas}\n")

    print(f"\nResults saved to: {output_file}")

if __name__ == "__main__":
    main()
