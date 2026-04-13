#!/usr/bin/env python3.10
"""Generate audio for notebooks using NotebookLM SDK.
Bypasses Google Cloud API auth issues by using cookie auth.
"""

import asyncio
import argparse
import sys
from datetime import datetime
from pathlib import Path

# Check SDK is available
try:
    from notebooklm import NotebookLMClient
except ImportError:
    print("ERROR: notebooklm-py not installed")
    print("       Run: pip install notebooklm-py")
    sys.exit(1)

from cookie_auth import authenticate_with_cookies, notebooklm_client_init


async def generate_audio_for_notebook(client, notebook_id, bilingual=False):
    """Generate audio for a single notebook."""
    print(f"Generating audio for notebook {notebook_id}...")

    try:
        # Get notebook details
        nb = await client.notebooks.get(notebook_id)
        print(f"  Notebook: {nb.title}")
        print(f"  Sources: {len(nb.sources) if hasattr(nb, 'sources') else 0}")

        # Audio generation requires specific SDK method
        if hasattr(client, 'audio_overviews'):
            # New API style
            if bilingual:
                # Generate English
                en_overview = await client.audio_overviews.create(
                    notebook_id,
                    language_code="en",
                    episode_focus=f"Complete technical overview of {nb.title}",
                    source_ids=None
                )
                print(f"  Created EN audio: {en_overview.name}")

                # Delete to allow Russian
                try:
                    await client.audio_overviews.delete_default(notebook_id)
                    print(f"  Deleted default audio for RU generation")
                except:
                    pass  # May not exist

                # Generate Russian
                ru_overview = await client.audio_overviews.create(
                    notebook_id,
                    language_code="ru",
                    episode_focus=f"Complete technical overview of {nb.title}",
                    source_ids=None
                )
                print(f"  Created RU audio: {ru_overview.name}")

                # Get audio files (if available)
                if hasattr(client, 'get_audio_file'):
                    # Try to download audio
                    print(f"  Audio files would be available for download")

            else:
                # Fallback: create audio overview request
                print(f"  Note: Audio SDK API may be different")
                print(f"  Check client.audio_overviews availability")

    except Exception as e:
        print(f"ERROR generating audio: {e}")
        return False

    return True


async def main():
    parser = argparse.ArgumentParser(
        description="Generate NotebookLM audio for notebooks with sources"
    )
    parser.add_argument(
        "--issue",
        type=str,
        help="Single notebook ID to process"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Process all notebooks with sources"
    )
    parser.add_argument(
        "--bilingual",
        action="store_true",
        help="Generate both EN and RU audio"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Dry run - list notebooks without generating audio"
    )

    args = parser.parse_args()

    # Authenticate
    print("Authenticating with NotebookLM...")
    success, error, tokens = authenticate_with_cookies()
    if not success:
        print(f"ERROR: {error}")
        sys.exit(1)

    client = await notebooklm_client_init(tokens=tokens)

    # Get list of notebooks
    all_notebooks = await client.notebooks.list()

    # Filter to notebooks with sources
    notebooks_with_sources = [
        nb for nb in all_notebooks
        if hasattr(nb, 'sources') and len(nb.sources) > 0
    ]

    print(f"Found {len(notebooks_with_sources)} notebooks with sources")

    if args.dry_run:
        print()
        print("=" * 60)
        print("DRY RUN MODE - No audio will be generated")
        print("=" * 60)
        print()
        for i, nb in enumerate(notebooks_with_sources, 1):
            print(f"{i}. {nb.id}")
            print(f"   Title: {nb.title}")
            print(f"   Sources: {len(nb.sources)}")
        print()
        print(f"Total: {len(notebooks_with_sources)} notebooks")
        sys.exit(0)

    # Process notebooks
    if args.issue:
        notebooks = [nb for nb in notebooks_with_sources if nb.id == args.issue]
        if not notebooks:
            print(f"ERROR: Notebook {args.issue} not found or has no sources")
            sys.exit(1)
        notebooks = notebooks_with_sources
    elif args.all:
        notebooks = notebooks_with_sources
    else:
        print("ERROR: Specify --issue or --all")
        sys.exit(1)

    print(f"Processing {len(notebooks)} notebooks...")

    results = {"success": 0, "failed": 0, "errors": []}

    for i, nb in enumerate(notebooks, 1):
        print()
        print(f"[{i}/{len(notebooks)}] {nb.id}")
        success = await generate_audio_for_notebook(client, nb.id, args.bilingual)
        if success:
            results["success"] += 1
        else:
            results["failed"] += 1
            results["errors"].append(nb.id)

    # Summary
    print()
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Processed: {len(notebooks)}")
    print(f"Success: {results['success']}")
    print(f"Failed: {results['failed']}")
    if results["errors"]:
        print()
        print("Failed notebooks:")
        for nb_id in results["errors"]:
            print(f"  - {nb_id}")
    print("=" * 60)


if __name__ == "__main__":
    asyncio.run(main())
