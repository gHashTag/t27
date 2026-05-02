#!/usr/bin/env python3.10
"""
Generate bilingual audio (EN + RU) for notebooks using official NotebookLM SDK.
phi^2 + 1/phi^2 = 3 | TRINITY
"""

import argparse
import sys
from notebooklm import NotebookLMClient

# Language templates
LANG_TEMPLATES = {
    "en": "Complete technical overview of {title}",
    "ru": "Полный технический обзор: {title}",
}

def generate_audio_for_notebook(client, notebook_id, title):
    """Generate EN and RU audio for a notebook."""
    print(f"[EN] {title}...")
    en_status = client.artifacts.generate_audio(
        notebook_id,
        instructions=LANG_TEMPLATES["en"].format(title=title),
        language_code="en",
    )
    print(f"[EN] ✓ done")

    print(f"[RU] {title}...")
    ru_status = client.artifacts.generate_audio(
        notebook_id,
        instructions=LANG_TEMPLATES["ru"].format(title=title),
        language_code="ru",
    )
    print(f"[RU] ✓ done")
    return True


def main():
    parser = argparse.ArgumentParser(
        description="Generate bilingual audio for notebooks with sources"
    )
    parser.add_argument(
        "--notebook",
        type=str,
        help="Single notebook ID to process"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Process all notebooks with sources"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Dry run - list without generating"
    )

    args = parser.parse_args()

    # Check auth (uses built-in storage management)
    print("Authenticating with NotebookLM...")
    try:
        client = NotebookLMClient.from_storage()
        print("✓ Authenticated")

        # List notebooks
        notebooks = client.notebooks.list()

        # Filter to notebooks with sources
        with_sources = [nb for nb in notebooks
                          if hasattr(nb, "source_count") and nb.source_count > 0]

        print(f"\nFound {len(with_sources)} notebooks with sources")

        if args.dry_run:
                print()
                print("DRY RUN MODE - No audio will be generated")
                print("=" * 60)
                for i, nb in enumerate(with_sources, 1):
                    print(f"{i}. {nb.id}")
                    print(f"   Title: {nb.title}")
                    print(f"   Sources: {nb.source_count}")
                print(f"\nTotal: {len(with_sources)} notebooks")
                return

        # Process notebooks
        if args.notebook:
                notebooks = [nb for nb in with_sources if nb.id == args.notebook]
        elif args.all:
                notebooks = with_sources
        else:
                print("ERROR: Specify --notebook or --all")
                return

        print(f"\nProcessing {len(notebooks)} notebooks...")

        # Generate audio
        success = 0
        failed = 0
        for i, nb in enumerate(notebooks, 1):
                print(f"[{i}/{len(notebooks)}] {nb.id}")
                if generate_audio_for_notebook(client, nb.id, nb.title):
                    success += 1
                else:
                    failed += 1

        # Summary
        print()
        print("=" * 60)
        print("SUMMARY")
        print("=" * 60)
        print(f"Processed: {len(notebooks)}")
        print(f"Success: {success}")
        print(f"Failed: {failed}")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"[ERROR] {e}")
        sys.exit(1)
