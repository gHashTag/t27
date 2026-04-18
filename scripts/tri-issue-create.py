#!/usr/bin/env python3
# scripts/tri-issue-create.py
# Wrapper for GitHub issue creation with NotebookLM sync
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Create GitHub issue with automatic NotebookLM sync."""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path

# Add backend to path
sys.path.insert(0, str(Path(__file__).parent.parent / "contrib" / "backend"))

from notebooklm import notebook_create, source_upload_text, client_new, client_authenticate


def main():
    parser = argparse.ArgumentParser(
        description="Create GitHub issue with NotebookLM sync"
    )
    parser.add_argument("--title", required=True, help="Issue title")
    parser.add_argument("--body", required=True, help="Issue description")
    parser.add_argument("--labels", default="phi-loop", help="Comma-separated labels")
    parser.add_argument("--issue", help="Existing issue ID to link")
    parser.add_argument("--dry-run", action="store_true", help="Print only, no action")

    args = parser.parse_args()

    # Create issue content
    content = f"""# {args.title}

{args.body}

## Labels
{args.labels}

## Metadata
- Created via tri-ssot bridge
- Synced to NotebookLM
"""

    if args.dry_run:
        print(f"[DRY-RUN] Would create issue: {args.title}")
        print(f"[DRY-RUN] Labels: {args.labels}")
        print(f"[DRY-RUN] Body length: {len(args.body)} chars")
        return 0

    # Initialize NotebookLM client
    try:
        client = client_new()
        if not client_is_authenticated(client):
            client = client_authenticate(client)

        # Get or create notebook
        notebook = notebook_create(client, "t27-GH-SSOT")
        notebook_id = notebook.id

        # Upload as source
        source_id = source_upload_text(
            notebooklm_client=client,
            notebook_id=notebook_id,
            content=content,
            title=f"[GH Issue] {args.title}",
        )

        if source_id:
            print(f"✓ Uploaded to NotebookLM: source_id={source_id}")
            print(f"  Notebook: {notebook_id}")
            print(f"  Title: {args.title}")
            return 0
        else:
            print("✗ Failed to upload to NotebookLM")
            return 1

    except Exception as e:
        print(f"✗ Error: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
