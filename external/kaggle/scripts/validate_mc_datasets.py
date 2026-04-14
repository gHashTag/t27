#!/usr/bin/env python3
"""
Validate generated MC datasets using the validation utilities.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from mc_generator_utils import QuestionValidator

# Datasets to validate
DATASETS = [
    "external/kaggle/data/thlp_mc_new.csv",
    "external/kaggle/data/ttm_mc_new.csv",
    "external/kaggle/data/ttm_mc_adversarial.csv",
    "external/kaggle/data/tscp_mc_new.csv",
    "external/kaggle/data/tscp_mc_adversarial.csv",
    "external/kaggle/data/tefb_mc_new.csv",
]

def main():
    print(f"{'='*60}")
    print("MC Dataset Validation")
    print(f"{'='*60}\n")

    all_valid = True

    for dataset in DATASETS:
        path = Path(dataset)
        if not path.exists():
            print(f"❌ {dataset}: File not found")
            all_valid = False
            continue

        print(f"Validating: {dataset}")
        results = QuestionValidator.validate_dataset(path)

        if results["valid"]:
            print(f"  ✅ Valid")
        else:
            print(f"  ❌ Invalid")
            all_valid = False
            for error in results["errors"][:5]:  # Show first 5 errors
                print(f"    - {error}")
            if len(results["errors"]) > 5:
                print(f"    ... and {len(results['errors']) - 5} more errors")

        print(f"  Total: {results['stats']['total']} questions")
        print(f"  Avg question length: {results['stats']['avg_question_length']:.0f} chars")
        print()

    print(f"{'='*60}")
    if all_valid:
        print("✅ All datasets passed validation")
    else:
        print("❌ Some datasets failed validation")
    print(f"{'='*60}")

    return 0 if all_valid else 1


if __name__ == "__main__":
    sys.exit(main())
