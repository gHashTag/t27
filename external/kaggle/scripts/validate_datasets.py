#!/usr/bin/env python3
"""
Validate all MC dataset CSV files.

Checks format, answer distribution, question uniqueness, and produces summary stats.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from mc_generator_utils import QuestionValidator

DATA_DIR = Path(__file__).parent.parent / "data"

EXPECTED_FILES = {
    "tefb_mc.csv": {"min_rows": 200, "track": "tefb"},
    "tscp_mc.csv": {"min_rows": 200, "track": "tscp"},
    "thlp_mc_new.csv": {"min_rows": 200, "track": "thlp"},
    "ttm_mc.csv": {"min_rows": 200, "track": "ttm"},
}


def main():
    all_valid = True

    for filename, config in EXPECTED_FILES.items():
        csv_path = DATA_DIR / filename
        print(f"\n--- Validating {filename} ---")

        if not csv_path.exists():
            print(f"  MISSING: {csv_path} not found")
            all_valid = False
            continue

        result = QuestionValidator.validate_dataset(csv_path)

        print(f"  Valid: {result['valid']}")
        print(f"  Total questions: {result['stats']['total']}")

        if result['stats']['total'] < config['min_rows']:
            print(f"  WARNING: Expected at least {config['min_rows']} rows")
            all_valid = False

        for answer, count in sorted(result['stats']['by_answer'].items()):
            total = result['stats']['total']
            pct = count / total * 100 if total > 0 else 0
            print(f"  {answer}: {count} ({pct:.1f}%)")

        if result['errors']:
            for err in result['errors'][:5]:
                print(f"  ERROR: {err}")
            if len(result['errors']) > 5:
                print(f"  ... and {len(result['errors']) - 5} more errors")

        if not result['valid']:
            all_valid = False

    print(f"\n{'='*60}")
    if all_valid:
        print("ALL DATASETS VALID")
        return 0
    else:
        print("SOME DATASETS FAILED VALIDATION")
        return 1


if __name__ == "__main__":
    sys.exit(main())
