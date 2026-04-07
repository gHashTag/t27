#!/usr/bin/env python3
# scripts/wrapup/format-summary.py
# Format session wrap-up summary as Markdown
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Format session wrap-up summary as Markdown for NotebookLM."""

import sys
import argparse
from pathlib import Path

# Add project root to path
repo_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.wrapup import (
    wrapup_format_summary,
    wrapup_format_markdown,
    wrapup_create_and_upload,
)


def main():
    """Parse arguments and format/summary."""
    parser = argparse.ArgumentParser(
        description="Format session wrap-up summary as Markdown"
    )
    parser.add_argument("--notebook-id", help="Target NotebookLM notebook ID")
    parser.add_argument("--summary", default="No summary provided", help="Session summary")
    parser.add_argument("--decisions", default="No key decisions", help="Key decisions")
    parser.add_argument("--files", default="No files changed", help="Files changed")
    parser.add_argument("--next", default="No next steps", help="Next steps")
    parser.add_argument("--upload", action="store_true", help="Upload to NotebookLM")

    args = parser.parse_args()

    # Extract session context
    from contrib.backend.notebooklm.session import session_extract_from_current_dir
    session = session_extract_from_current_dir()

    if session is None:
        print("Error: Not in a t27 repository", file=sys.stderr)
        sys.exit(1)

    # Format summary
    wrapup = wrapup_format_summary(
        session,
        args.summary,
        args.decisions,
        args.files,
        args.next,
    )

    # Output
    if args.upload:
        if not args.notebook_id:
            print("Error: --notebook-id required for upload", file=sys.stderr)
            sys.exit(1)

        result = wrapup_upload(args.notebook_id, wrapup)
        if result:
            print(f"Uploaded: {result.get('id', 'unknown')}")
        else:
            print("Upload failed", file=sys.stderr)
            sys.exit(1)
    else:
        print(wrapup_format_markdown(wrapup))


if __name__ == "__main__":
    main()
