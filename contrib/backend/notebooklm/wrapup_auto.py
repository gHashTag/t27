# contrib/backend/notebooklm/wrapup_auto.py
# Wrap-up automation for NotebookLM integration
# Ring-071 - RAG-Backed Semantic Memory
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Wrap-up automation: read args, find/create issue-specific notebook, upload markdown.

Each GitHub issue gets its own notebook in NotebookLM:
  Issue #343 "Restore phi-loop-ci.yml" -> Notebook: "t27 #343 — Restore phi-loop-ci.yml"

Each /tri wrapup adds a new source to the issue's notebook, preserving full session history.
"""

import argparse
import sys
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Optional

# Delay import of notebooklm until needed (allows --dry-run without installation)
NOTEBOOKLM_AVAILABLE = None  # Will be checked when needed


def check_notebooklm() -> bool:
    """Check if notebooklm-py is installed."""
    global NOTEBOOKLM_AVAILABLE
    if NOTEBOOKLM_AVAILABLE is not None:
        return NOTEBOOKLM_AVAILABLE
    try:
        import importlib
        importlib.import_module("notebooklm")
        NOTEBOOKLM_AVAILABLE = True
        return True
    except ImportError:
        NOTEBOOKLM_AVAILABLE = False
        return False


def require_notebooklm() -> None:
    """Raise error if notebooklm-py not installed."""
    if not check_notebooklm():
        print("Error: notebooklm-py not installed", file=sys.stderr)
        print(f"Install with: python -m venv {VENV_PATH} && {VENV_PATH}/bin/pip install notebooklm-py", file=sys.stderr)
        sys.exit(1)


DEFAULT_NOTEBOOK = "t27-QUEEN-BRAIN"
VENV_PATH = ".trinity/notebooklm-venv"
ISSUE_BINDING_PATH = ".trinity/state/issue-binding.json"
NOTEBOOK_PREFIX = "t27 #"


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


def get_issue_info() -> Optional[tuple[str, str]]:
    """Get current issue number and title from .trinity/state/issue-binding.json.

    Returns:
        Tuple of (issue_number, issue_title) or None if not found
    """
    import json

    try:
        with open(ISSUE_BINDING_PATH, "r") as f:
            binding = json.load(f)

        # Extract issue number from issue_id (handles "INFRA", "350", etc.)
        issue_id = binding.get("issue_id", "")
        title = binding.get("title", "")

        # If issue_id is a number, use it directly
        if issue_id and issue_id.isdigit():
            return (issue_id, title)

        # If issue_id is a string like "INFRA", try to get from GitHub API
        if issue_id:
            try:
                result = subprocess.run(
                    ["gh", "issue", "view", issue_id, "--json", "title,number"],
                    capture_output=True,
                    text=True,
                    check=True
                )
                data = json.loads(result.stdout)
                return (str(data["number"]), data["title"])
            except (subprocess.CalledProcessError, json.JSONDecodeError, KeyError):
                pass

        return None

    except (FileNotFoundError, json.JSONDecodeError):
        return None


def get_notebook_name_for_issue(issue_number: str, issue_title: str) -> str:
    """Generate notebook name for an issue.

    Args:
        issue_number: GitHub issue number
        issue_title: Issue title

    Returns:
        Notebook name in format "t27 #NNN — title"
    """
    return f"{NOTEBOOK_PREFIX}{issue_number} — {issue_title}"


async def find_or_create_notebook(
    client,
    title: str,
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
    client,
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
            "notebook_name": notebook_title,
            "source_id": source.id,
            "uploaded_at": datetime.now().isoformat(),
        }

        print(f"Uploaded wrap-up: source_id={source.id}")
        return result

    except Exception as e:
        print(f"Error uploading source: {e}", file=sys.stderr)
        return None


def format_markdown(
    summary: str,
    decisions: str,
    files_modified: list[str],
    next_steps: str,
    session_id: str,
    issue_number: Optional[str] = None,
    issue_title: Optional[str] = None,
) -> str:
    """Format wrap-up summary as Markdown for NotebookLM.

    Args:
        summary: Session summary text
        decisions: Key decisions made
        files_modified: Files that were changed
        next_steps: Next steps to take
        session_id: Session identifier
        issue_number: Optional GitHub issue number
        issue_title: Optional GitHub issue title

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
    ]

    if issue_number:
        lines.append(f"**Issue:** #{issue_number}")
        if issue_title:
            lines.append(f"**Issue Title:** {issue_title}")

    lines.extend([
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
    ])

    return "\n".join(lines)


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
    parser.add_argument("--notebook", help="Target notebook name (default: auto-detect from issue)")
    parser.add_argument("--dry-run", action="store_true", help="Print markdown without uploading")
    parser.add_argument("--issue", help="GitHub issue number (overrides .trinity/state/issue-binding.json)")

    args = parser.parse_args()

    # Only require notebooklm-py for actual upload
    if not args.dry_run:
        require_notebooklm()

    # Default session_id to git commit
    session_id = args.session_id or get_git_commit(short=True)

    # Determine notebook name
    notebook_name = args.notebook
    issue_number = None
    issue_title = None

    if not notebook_name:
        # Try --issue argument first
        if args.issue:
            issue_number = args.issue
            try:
                result = subprocess.run(
                    ["gh", "issue", "view", issue_number, "--json", "title"],
                    capture_output=True,
                    text=True,
                    check=True
                )
                import json
                issue_title = json.loads(result.stdout).get("title", "")
                notebook_name = get_notebook_name_for_issue(issue_number, issue_title)
            except Exception as e:
                print(f"Warning: Could not fetch issue {issue_number}: {e}", file=sys.stderr)
                notebook_name = DEFAULT_NOTEBOOK
        else:
            # Try to read from .trinity/state/issue-binding.json
            issue_info = get_issue_info()
            if issue_info:
                issue_number, issue_title = issue_info
                notebook_name = get_notebook_name_for_issue(issue_number, issue_title)
                print(f"Auto-detected issue: #{issue_number} — {issue_title}")
            else:
                notebook_name = DEFAULT_NOTEBOOK

    # Parse files list
    files_modified = [f.strip() for f in args.files.split(",") if f.strip()]

    # Format markdown with issue info if available
    markdown = format_markdown(
        summary=args.summary,
        decisions=args.decisions,
        files_modified=files_modified,
        next_steps=args.steps,
        session_id=session_id,
        issue_number=issue_number,
        issue_title=issue_title,
    )

    if args.dry_run:
        print(f"--- Markdown Preview ---")
        print(f"Target Notebook: {notebook_name}")
        print()
        print(markdown)
        print("--- End Preview ---")
        return 0

    # Run async upload
    import asyncio

    async def upload():
        try:
            from notebooklm import NotebookLMClient
            client = await NotebookLMClient.from_storage()
            result = await wrapup_run(
                client=client,
                summary=args.summary,
                decisions=args.decisions,
                files_modified=files_modified,
                next_steps=args.steps,
                session_id=session_id,
                notebook_title=notebook_name,
            )
            if result:
                print(f"✅ Uploaded to: {notebook_name}")
                return 0
            return 1
        except Exception as e:
            print(f"Error: {e}", file=sys.stderr)
            return 1

    return asyncio.run(upload())


if __name__ == "__main__":
    sys.exit(main())
