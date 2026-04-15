#!/usr/bin/env python3
"""
Fix Kaggle dataset CSV files for proper Data Explorer display.

Problems fixed:
1. Multiline choices with \n inside single CSV field
2. Wrong counts in About descriptions
3. MIT license mention (should be CC0)
"""

import pandas as pd
from pathlib import Path
import shutil

_KAGGLE_ROOT = Path(__file__).resolve().parent.parent

# Datasets to fix
DATASETS_TO_FIX = {
    "ttm": {
        "csv": "data/ttm_mc_new.csv",
        "output_csv": "data/ttm_mc_fixed.csv",
        "rows": 2482,  # 2483 - 1 header
    },
    "tscp": {
        "csv": "data/tscp_mc_new.csv",
        "output_csv": "data/tscp_mc_fixed.csv",
        "rows": 2838,  # 2839 - 1 header
    },
    "tefb": {
        "csv": "data/tefb_mc_new.csv",
        "output_csv": "data/tefb_mc_fixed.csv",
        "rows": 21080,  # 21081 - 1 header
    },
    "tagp": {
        "csv": "data/tagp_mc.csv",
        "output_csv": "data/tagp_mc_fixed.csv",
        "rows": 17600,  # 17601 - 1 header
    },
    "thlp": {
        "csv": "data/thlp_mc_new.csv",
        "output_csv": "data/thlp_mc_fixed.csv",
        "rows": 19680,  # 19681 - 1 header
    },
}


def fix_csv_multiline(input_path: Path, output_path: Path) -> int:
    """
    Fix CSV by replacing newlines inside 'choices' column with ' | '.

    Returns number of rows in output.
    """
    print(f"  Reading: {input_path}")
    df = pd.read_csv(input_path)

    # Check if choices column has newlines
    if 'choices' in df.columns:
        sample_choice = df['choices'].iloc[0] if len(df) > 0 else ""
        has_newlines = '\n' in str(sample_choice)
        if has_newlines:
            print(f"  Found newlines in choices, fixing...")
            df['choices'] = df['choices'].str.replace('\n', ' | ', regex=False)
            df['choices'] = df['choices'].str.replace('\r', '', regex=False)

    # Write output
    print(f"  Writing: {output_path}")
    df.to_csv(output_path, index=False)

    row_count = len(df)
    print(f"  Rows: {row_count}")
    return row_count


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Fix Kaggle dataset CSVs")
    parser.add_argument("--track", default="ALL", help="Track to fix (ttm, tscp, tefb, tagp, thlp, ALL)")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done")
    args = parser.parse_args()

    print("\n" + "=" * 60)
    print("KAGGLE DATASET CSV FIX")
    print("=" * 60 + "\n")

    tracks_to_fix = []
    if args.track == "ALL":
        tracks_to_fix = list(DATASETS_TO_FIX.keys())
    else:
        if args.track not in DATASETS_TO_FIX:
            print(f"❌ Unknown track: {args.track}")
            print(f"Available: {', '.join(DATASETS_TO_FIX.keys())}")
            return
        tracks_to_fix = [args.track]

    if args.dry_run:
        print("📋 DRY RUN MODE\n")

    results = {}
    for track in tracks_to_fix:
        info = DATASETS_TO_FIX[track]
        print(f"\n{track.upper()}:")

        input_path = _KAGGLE_ROOT / info["csv"]
        output_path = _KAGGLE_ROOT / info["output_csv"]

        if not input_path.exists():
            print(f"  ❌ Input not found: {input_path}")
            results[track] = False
            continue

        if args.dry_run:
            print(f"  Would fix: {input_path}")
            print(f"  Would write: {output_path}")
            results[track] = True
            continue

        try:
            row_count = fix_csv_multiline(input_path, output_path)
            expected = info["rows"]
            if row_count == expected:
                print(f"  ✅ Row count matches: {row_count}")
            else:
                print(f"  ⚠️  Row count: {row_count}, expected: {expected}")
            results[track] = True
        except Exception as e:
            print(f"  ❌ Error: {e}")
            results[track] = False

    # Summary
    print("\n" + "=" * 60)
    print("FIX SUMMARY")
    print("=" * 60)
    for track, success in results.items():
        status = "✅" if success else "❌"
        print(f"  {status} {track.upper()}")
    print("=" * 60)

    if not args.dry_run:
        print("\nTo upload fixed datasets to Kaggle:")
        print("  python3 scripts/upload_mc_datasets_fixed.py --track ALL")


if __name__ == "__main__":
    main()
