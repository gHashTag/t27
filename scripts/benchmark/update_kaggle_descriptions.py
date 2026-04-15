#!/usr/bin/env python3
"""
Update Kaggle dataset descriptions to include t27 repo link.
Source: specs/benchmarks/trinity_cognitive_probe_runner.t27

Adds source repository reference to all 5 Trinity Cognitive Probe datasets.

Usage:
    pip install kaggle
    python scripts/benchmark/update_kaggle_descriptions.py [--dry-run]
"""

import json
import os
import subprocess
import tempfile
from pathlib import Path

KAGGLE_USER = "playra"

REPO_SECTION = """
## Source Repository

Original specifications and data pipeline: [github.com/gHashTag/t27](https://github.com/gHashTag/t27)

Trinity Cognitive Probes are part of the T27 (TRI-27) spec-first ternary architecture.
For methodology, paper citations, and contribution guidelines, see the repo.

## Hackathon

Part of [Google DeepMind x Kaggle: Measuring Progress Toward AGI](https://www.kaggle.com/competitions/llm-cognitive-capabilities)

## Citation

```bibtex
@misc{trinity_cognitive_probes_2026,
  title={Trinity Cognitive Probes: Ternary Hyperdimensional Computing AGI Benchmark},
  author={gHashTag},
  year={2026},
  url={https://github.com/gHashTag/t27}
}
```
"""

DATASETS = {
    "thlp": {
        "slug": "trinity-cognitive-probes-thlp-mc",
        "title": "Trinity Hippocampal Learning Probe - MC Format",
        "about": (
            "19,681 multiple-choice questions testing pattern learning, "
            "belief update, and rule induction for AGI assessment. "
            "Part of Trinity Cognitive Probes for the Google DeepMind x Kaggle "
            "Measuring Progress Toward AGI hackathon."
        ),
        "readme": """# Trinity Hippocampal Learning Probe (THLP) - Multiple Choice Format v2

**Track 1**: Tests pattern learning, belief update, rule induction.

## Format
- `id`: Unique question identifier
- `question_type`: Type of learning task
- `question`: Learning scenario
- `choices`: 4 options (A, B, C, D)
- `answer`: Correct letter (A-D)

## Contents
- 19,681 MC questions (v2 - cleaned and validated)

## Brain Zone
- **Hippocampus** - Pattern completion, error-driven learning
- **Entorhinal Cortex** - Grid cells for spatial reasoning
""",
    },
    "ttm": {
        "slug": "trinity-cognitive-probes-tmp-mc",
        "title": "Trinity Metacognition Probe - MC Format",
        "about": (
            "4,931 multiple-choice questions testing confidence calibration, "
            "error detection, and meta-learning for AGI assessment. "
            "Part of Trinity Cognitive Probes for the Google DeepMind x Kaggle "
            "Measuring Progress Toward AGI hackathon."
        ),
        "readme": """# Trinity Metacognition Probe (TTM) - Multiple Choice Format v5

**Track 2**: Tests confidence calibration, error detection, meta-learning.

## Format
- `id`: Unique question identifier
- `question_type`: Type of metacognitive task
- `question`: Metacognitive question or scenario
- `choices`: 4 options (A, B, C, D)
- `answer`: Correct letter (A-D)

## Contents
- 4,931 MC questions (v5 - cleaned and validated)

## Brain Zone
- **PCC** (Posterior Cingulate Cortex) - Metacognitive monitoring
- **dlPFC** (Dorsolateral PFC) - Executive control
""",
    },
    "tagp": {
        "slug": "trinity-cognitive-probes-tagp-mc",
        "title": "Trinity Attentional Gateway Probe - MC Format",
        "about": (
            "17,601 multiple-choice questions testing selective filtering, "
            "sustained attention, and attention shifting for AGI assessment. "
            "Part of Trinity Cognitive Probes for the Google DeepMind x Kaggle "
            "Measuring Progress Toward AGI hackathon."
        ),
        "readme": """# Trinity Attentional Gateway Probe (TAGP) - Multiple Choice Format

**Track 3**: Tests selective filtering, sustained attention, attention shifting.

## Format
- `id`: Unique question identifier
- `question_type`: Type of attention task
- `question`: Attention scenario
- `choices`: 4 options (A, B, C, D)
- `answer`: Correct letter (A-D)

## Contents
- 17,601 MC questions

## Brain Zone
- **Parietal Cortex** - Spatial attention
- **FEF** (Frontal Eye Fields) - Attention control
""",
    },
    "tefb": {
        "slug": "trinity-cognitive-probes-tefb-mc",
        "title": "Trinity Executive Function Battery - MC Format",
        "about": (
            "21,081 multiple-choice questions testing multi-step planning, "
            "working memory, and cognitive flexibility for AGI assessment. "
            "Part of Trinity Cognitive Probes for the Google DeepMind x Kaggle "
            "Measuring Progress Toward AGI hackathon."
        ),
        "readme": """# Trinity Executive Function Battery (TEFB) - Multiple Choice Format v2

**Track 4**: Tests multi-step planning, working memory, cognitive flexibility.

## Format
- `id`: Unique question identifier
- `question_type`: Type of executive function task
- `question`: Executive function scenario with context
- `choices`: 4 options (A, B, C, D)
- `answer`: Correct letter (A-D)

## Contents
- 21,081 MC questions (v2 - cleaned and validated)

## Brain Zone
- **dlPFC** (Dorsolateral PFC) - Working memory, planning
- **ACC** (Anterior Cingulate Cortex) - Conflict monitoring
- **OFC** (Orbitofrontal Cortex) - Value-based decision
""",
    },
    "tscp": {
        "slug": "trinity-cognitive-probes-tscp-mc",
        "title": "Trinity Social Cognition Probe - MC Format",
        "about": (
            "2,839 multiple-choice questions testing Theory of Mind, "
            "pragmatic inference, and social norms for AGI assessment. "
            "Part of Trinity Cognitive Probes for the Google DeepMind x Kaggle "
            "Measuring Progress Toward AGI hackathon."
        ),
        "readme": """# Trinity Social Cognition Probe (TSCP) - Multiple Choice Format v5

**Track 5**: Tests Theory of Mind, pragmatic inference, social norms.

## Format
- `id`: Unique question identifier
- `question_type`: Type of social cognition task
- `question`: Social scenario
- `choices`: 4 options (A, B, C, D)
- `answer`: Correct letter (A-D)

## Contents
- 2,839 MC questions (v5 - cleaned and validated)

## Brain Zone
- **TPJ** (Temporo-Parietal Junction) - Theory of Mind
- **mPFC** (Medial Prefrontal Cortex) - Self-other distinction
""",
    },
}


def update_dataset(track: str, info: dict, dry_run: bool = False):
    """Update a single dataset's metadata on Kaggle."""
    slug = info["slug"]
    print(f"\n{'='*60}")
    print(f"Updating: {track.upper()} - {slug}")
    print(f"{'='*60}")

    with tempfile.TemporaryDirectory() as tmpdir:
        # Create dataset-metadata.json
        metadata = {
            "title": info["title"],
            "id": f"{KAGGLE_USER}/{slug}",
            "licenses": [{"name": "CC0-1.0"}],
            "description": info["about"],
        }

        meta_path = os.path.join(tmpdir, "dataset-metadata.json")
        with open(meta_path, "w") as f:
            json.dump(metadata, f, indent=2)

        # Create README with repo link
        readme_text = info["readme"] + REPO_SECTION
        readme_path = os.path.join(tmpdir, "README.md")
        with open(readme_path, "w") as f:
            f.write(readme_text)

        print(f"  Metadata: {meta_path}")
        print(f"  README: {readme_path}")
        print(f"  About: {info['about'][:80]}...")
        print(f"  License: CC0-1.0")

        if dry_run:
            print(f"  [DRY RUN] Would update {KAGGLE_USER}/{slug}")
            return True

        # Upload metadata update
        result = subprocess.run(
            [
                "kaggle",
                "datasets",
                "metadata",
                "-p",
                tmpdir,
                f"{KAGGLE_USER}/{slug}",
            ],
            capture_output=True,
            text=True,
        )

        if result.returncode == 0:
            print(f"  OK: Metadata updated")
        else:
            print(f"  WARN: Metadata update: {result.stderr.strip()}")

        return True


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Update Kaggle dataset descriptions with t27 repo link"
    )
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "--track", default="ALL", help="Track to update (thlp, ttm, tagp, tefb, tscp, ALL)"
    )
    args = parser.parse_args()

    print("=" * 60)
    print("KAGGLE DATASET DESCRIPTION UPDATER")
    print("Adding github.com/gHashTag/t27 references")
    print("=" * 60)

    if args.dry_run:
        print("[DRY RUN MODE]")

    tracks = (
        list(DATASETS.keys()) if args.track == "ALL" else [args.track]
    )

    for track in tracks:
        if track not in DATASETS:
            print(f"Unknown track: {track}")
            continue
        update_dataset(track, DATASETS[track], args.dry_run)

    print("\nDone.")


if __name__ == "__main__":
    main()
