#!/usr/bin/env python3
"""
Deduplicate TTM and TSCP MC datasets and prepare v5 versions.

Removes duplicate questions (based on question text) and regenerates IDs.
Tracks: TTM (target: 600 unique), TSCP (target: 500 unique)
"""

import pandas as pd
from pathlib import Path
from typing import Tuple


def deduplicate_dataset(
    input_path: Path,
    output_path: Path,
    name: str
) -> Tuple[pd.DataFrame, int]:
    """
    Deduplicate a dataset by removing duplicate questions.

    Args:
        input_path: Path to input CSV
        output_path: Path to deduplicated output CSV
        name: Dataset name for logging

    Returns:
        Tuple of (deduplicated dataframe, number of duplicates removed)
    """
    df = pd.read_csv(input_path)
    original = len(df)

    # Drop duplicates, keeping first occurrence
    df_deduped = df.drop_duplicates(subset=['question'], keep='first')
    unique = len(df_deduped)
    removed = original - unique

    # Regenerate IDs after deduplication
    track = name.split('_')[0]  # ttm or tscp
    df_deduped['id'] = [f"{track}_v5_{i:04d}" for i in range(len(df_deduped))]

    # Ensure all question_type are 'mc'
    if not df_deduped['question_type'].eq('mc').all():
        raise ValueError(f"Dataset {name} has non-MC question types")

    # Write output
    df_deduped.to_csv(output_path, index=False)

    print(f"{name}: {original} → {unique} unique (removed {removed} dupes)")
    return df_deduped, removed


def main():
    """Process both TTM and TSCP datasets."""
    print("=" * 60)
    print("Dataset Deduplication for v5")
    print("=" * 60)

    data_dir = Path(__file__).parent.parent / "data"

    # Process TTM
    ttm_input = data_dir / "ttm_mc_new.csv"
    ttm_output = data_dir / "ttm_mc_v5_deduped.csv"
    ttm_df, ttm_removed = deduplicate_dataset(ttm_input, ttm_output, "TTM")

    # Process TSCP
    tscp_input = data_dir / "tscp_mc_new.csv"
    tscp_output = data_dir / "tscp_mc_v5_deduped.csv"
    tscp_df, tscp_removed = deduplicate_dataset(tscp_input, tscp_output, "TSCP")

    print("\n" + "=" * 60)
    print("Deduplication Complete")
    print("=" * 60)
    print(f"\nTTM: {ttm_df['question'].nunique()} unique questions remain")
    print(f"TSCP: {tscp_df['question'].nunique()} unique questions remain")
    print(f"\nNext: Run modified generators to add {600 - ttm_df['question'].nunique()} new TTM questions")
    print(f"      and {500 - tscp_df['question'].nunique()} new TSCP questions")


if __name__ == "__main__":
    main()
