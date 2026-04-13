#!/usr/bin/env python3
# contrib/backend/notebooklm/create_notebook.py
# Create NotebookLM notebook for GitHub issues
# phi^2 + 1/phi^2 = 3 | TRINITY

import asyncio
import sys
import json
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent))


async def create_notebook_for_issue(title: str, issue_number: str) -> str:
    """Create a NotebookLM notebook for a GitHub issue.

    Args:
        title: Notebook title
        issue_number: GitHub issue number

    Returns:
        Notebook ID

    Raises:
        RuntimeError: If notebook creation fails
    """
    try:
        from notebooklm import NotebookLMClient

        async with await NotebookLMClient.from_storage() as client:
            notebook = await client.notebooks.create(title)
            return notebook.id

    except Exception as e:
        print(f"Error creating notebook: {e}", file=sys.stderr)
        raise RuntimeError(f"Failed to create notebook: {e}") from e


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Create NotebookLM notebook for GitHub issue"
    )
    parser.add_argument("--title", required=True, help="Notebook title")
    parser.add_argument("--issue", required=True, help="GitHub issue number")
    parser.add_argument("--output", help="Output notebook metadata to file")

    args = parser.parse_args()

    try:
        # Create notebook
        notebook_id = asyncio.run(create_notebook_for_issue(args.title, args.issue))

        # Output notebook ID
        print(notebook_id)

        # Optionally write metadata file
        if args.output:
            metadata = {
                "notebook_id": notebook_id,
                "title": args.title,
                "issue_number": args.issue,
                "created_at": asyncio.get_event_loop().time(),
            }
            Path(args.output).write_text(json.dumps(metadata, indent=2))

        return 0

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
