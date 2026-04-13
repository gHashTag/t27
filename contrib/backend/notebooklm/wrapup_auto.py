# contrib/backend/notebooklm/wrapup_auto.py
# Wrap-up automation for NotebookLM integration
# Ring-071 - RAG-Backed Semantic Memory
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Wrap-up automation: read args, find/create notebook, upload markdown."""

import argparse
import sys
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Optional

try:
    from notebooklm import NotebookLMClient
    NOTEBOOKLM_AVAILABLE = True
except ImportError:
    NOTEBOOKLM_AVAILABLE = False


DEFAULT_NOTEBOOK = "t27-QUEEN-BRAIN"
VENV_PATH = ".trinity/notebooklm-venv"


def get_git_branch() -> str:
    """Get current git branch name."""
    try:
        return subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True,
            text=True,
            check=True
        ).stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def get_git_commit(short: bool = True) -> str:
    """Get current git commit hash."""
    try:
        cmd = ["git", "rev-parse", "--short", "HEAD"] if short else ["git", "rev-parse", "HEAD"]
        return subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True
        ).stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def format_markdown(
    summary: str,
    decisions: str,
    files_modified: list[str],
    next_steps: str,
    session_id: str,
) -> str:
    """Format wrap-up summary as Markdown for NotebookLM.

    Args:
        summary: Session summary text
        decisions: Key decisions made
        files_modified: Files that were changed
        next_steps: Next steps to take
        session_id: Session identifier

    Returns:
        Markdown formatted string
    """
    lines = [
        "# Session Wrap-up",
        "",
        f"**Session ID:** {session_id}",
        f"**Branch:** {get_git_branch()}",
        f"**Commit:** {get_git_commit(short=True)}",
        f"**Date:** {datetime.now().isoformat()}",
        "",
        "## Summary",
        "",
        summary,
        "",
        "## Key Decisions",
        "",
        decisions,
        "",
        "## Files Modified",
        "",
        *files_modified,
        "",
        "## Next Steps",
        "",
        next_steps,
    ]

    return "\n".join(lines)


async def find_or_create_notebook(
    client: NotebookLMClient,
    title: str = DEFAULT_NOTEBOOK,
) -> Optional[str]:
    """Find notebook by name or create it.

    Args:
        client: Authenticated NotebookLM client
        title: Notebook title to find/create

    Returns:
        Notebook ID or None if failed
    """
    try:
        # List all notebooks
        notebooks = await client.notebooks.list()

        # Search for existing notebook
        for nb in notebooks:
            if nb.title == title:
                print(f"Found existing notebook: {title} ({nb.id})")
                return nb.id

        # Create new notebook
        print(f"Creating new notebook: {title}")
        new_nb = await client.notebooks.create(title)
        print(f"Created notebook: {title} ({new_nb.id})")
        return new_nb.id

    except Exception as e:
        print(f"Error finding/creating notebook: {e}", file=sys.stderr)
        return None


async def wrapup_run(
    client: NotebookLMClient,
    summary: str,
    decisions: str,
    files_modified: list[str],
    next_steps: str,
    session_id: str,
    notebook_title: str = DEFAULT_NOTEBOOK,
) -> Optional[dict]:
    """Find/create notebook, format markdown, upload source.

    Args:
        client: Authenticated NotebookLM client
        summary: Session summary text
        decisions: Key decisions made
        files_modified: Files that were changed
        next_steps: Next steps to take
        session_id: Session identifier
        notebook_title: Target notebook title

    Returns:
        Dict with notebook_id, source_id, uploaded_at or None if failed
    """
    import asyncio

    # Find or create notebook
    notebook_id = await find_or_create_notebook(client, notebook_title)
    if not notebook_id:
        return None

    # Format markdown
    markdown = format_markdown(summary, decisions, files_modified, next_steps, session_id)

    # Create title
    title = f"Session {session_id}"

    # Upload as text source
    try:
        notebook = await client.notebooks.get(notebook_id)
        source = await notebook.sources.create_text(title, markdown)

        result = {
            "notebook_id": notebook_id,
            "source_id": source.id,
            "uploaded_at": datetime.now().isoformat(),
        }

        print(f"Uploaded wrap-up: source_id={source.id}")
        return result

    except Exception as e:
        print(f"Error uploading source: {e}", file=sys.stderr)
        return None


def main() -> int:
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Upload session wrap-up to NotebookLM"
    )
    parser.add_argument("--summary", required=True, help="Session summary")
    parser.add_argument("--decisions", default="", help="Key decisions made")
    parser.add_argument("--files", default="", help="Files modified (comma-separated)")
    parser.add_argument("--steps", default="", help="Next steps")
    parser.add_argument("--session-id", help="Session ID (defaults to git commit)")
    parser.add_argument("--notebook", default=DEFAULT_NOTEBOOK, help="Target notebook name")
    parser.add_argument("--dry-run", action="store_true", help="Print markdown without uploading")

    args = parser.parse_args()

    if not NOTEBOOKLM_AVAILABLE:
        print("Error: notebooklm-py not installed", file=sys.stderr)
        print(f"Install with: python -m venv {VENV_PATH} && {VENV_PATH}/bin/pip install notebooklm-py", file=sys.stderr)
        return 1

    # Default session_id to git commit
    session_id = args.session_id or get_git_commit(short=True)

    # Parse files list
    files_modified = [f.strip() for f in args.files.split(",") if f.strip()]

    # Format markdown
    markdown = format_markdown(
        summary=args.summary,
        decisions=args.decisions,
        files_modified=files_modified,
        next_steps=args.steps,
        session_id=session_id,
    )

    if args.dry_run:
        print("--- Markdown Preview ---")
        print(markdown)
        print("--- End Preview ---")
        return 0

    # Run async upload
    import asyncio

    async def upload():
        try:
            client = await NotebookLMClient.from_storage()
            result = await wrapup_run(
                client=client,
                summary=args.summary,
                decisions=args.decisions,
                files_modified=files_modified,
                next_steps=args.steps,
                session_id=session_id,
                notebook_title=args.notebook,
            )
            if result:
                print(f"Success: {result}")
                return 0
            return 1
        except Exception as e:
            print(f"Error: {e}", file=sys.stderr)
            return 1

    return asyncio.run(upload())


if __name__ == "__main__":
    sys.exit(main())
