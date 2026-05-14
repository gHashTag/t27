#!/usr/bin/env python3
# scripts/tri-doc-sync.py
# Wrapper for documentation sync to NotebookLM
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Sync documentation files to NotebookLM."""

import argparse
import sys
from pathlib import Path

# Add backend to path
sys.path.insert(0, str(Path(__file__).parent.parent / "contrib" / "backend"))

from notebooklm import client_new, notebook_create
from notebooklm.docs import doc_sync_all, doc_upload_notebooklm


def main():
    parser = argparse.ArgumentParser(
        description="Sync documentation to NotebookLM"
    )
    parser.add_argument("--file", help="Single file to upload")
    parser.add_argument("--title", help="Title for single file upload")
    parser.add_argument("--pattern", default="*.md", help="File pattern for batch sync")
    parser.add_argument("--repo-root", default=".", help="Repository root path")
    parser.add_argument("--dry-run", action="store_true", help="Print only, no action")

    args = parser.parse_args()

    if args.dry_run:
        if args.file:
            print(f"[DRY-RUN] Would upload: {args.file}")
        else:
            print(f"[DRY-RUN] Would sync pattern: {args.pattern} in {args.repo_root}")
        return 0

    try:
        notebooklm_client = client_new()
        notebook = notebook_create(notebooklm_client, "t27-GH-SSOT")

        if args.file:
            # Upload single file
            if not args.title:
                args.title = Path(args.file).stem

            source_id = doc_upload_notebooklm(
                notebooklm_client=notebooklm_client,
                doc_path=args.file,
                title=args.title,
            )

            if source_id:
                print(f"✓ Uploaded: {args.file}")
                return 0
            else:
                print(f"✗ Failed: {args.file}")
                return 1
        else:
            # Batch sync
            result = doc_sync_all(
                notebooklm_client=notebooklm_client,
                repo_root=args.repo_root,
                pattern=args.pattern,
            )
            print(f"✓ Synced: {result['synced']} files")
            if result['failed'] > 0:
                print(f"✗ Failed: {result['failed']} files")
            return 0

    except Exception as e:
        print(f"✗ Error: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
