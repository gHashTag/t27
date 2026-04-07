# contrib/backend/notebooklm/wrapup.py
# Wrap-up summary formatting and upload for NotebookLM
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Wrap-up summary formatting and upload to NotebookLM."""

from dataclasses import dataclass, asdict
from typing import Dict, Any, Optional
from datetime import datetime

from .session import SessionContext
from .sources import source_upload_text


@dataclass
class WrapupSummary:
    """Wrap-up summary data structure.

    Attributes:
        session: Session context
        summary: Session summary text
        key_decisions: Key decisions made
        files_changed: Files that were changed
        next_steps: Next steps to take
        created_at: Creation timestamp
    """

    session: SessionContext
    summary: str
    key_decisions: str
    files_changed: str
    next_steps: str
    created_at: str

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)


def wrapup_format_summary(
    session: Dict[str, Any],
    summary: str,
    decisions: str,
    files: str,
    steps: str,
) -> Dict[str, Any]:
    """Format a wrap-up summary from session data.

    Args:
        session: Session context dict
        summary: Session summary text
        decisions: Key decisions made
        files: Files that were changed
        steps: Next steps to take

    Returns:
        WrapupSummary dict
    """
    wrapup = WrapupSummary(
        session=session,
        summary=summary,
        key_decisions=decisions,
        files_changed=files,
        next_steps=steps,
        created_at=datetime.now().isoformat(),
    )

    return wrapup.to_dict()


def wrapup_format_markdown(wrapup: Dict[str, Any]) -> str:
    """Format a wrap-up summary as Markdown for NotebookLM.

    Args:
        wrapup: WrapupSummary dict

    Returns:
        Markdown formatted string
    """
    session = wrapup.get("session", {})

    lines = [
        "# Session Wrap-up",
        "",
        f"**Session ID:** {session.get('session_id', 'unknown')}",
        f"**Branch:** {session.get('branch', 'unknown')}",
        f"**Skill:** {session.get('skill_id', 'unknown')}",
        f"**Issue:** {session.get('issue_number', 0)}",
        f"**Date:** {wrapup.get('created_at', datetime.now().isoformat())}",
        "",
        "## Summary",
        "",
        wrapup.get('summary', 'No summary provided.'),
        "",
        "## Key Decisions",
        "",
        wrapup.get('key_decisions', 'No key decisions recorded.'),
        "",
        "## Files Changed",
        "",
        wrapup.get('files_changed', 'No files changed.'),
        "",
        "## Next Steps",
        "",
        wrapup.get('next_steps', 'No next steps defined.'),
    ]

    return "\n".join(lines)


def wrapup_upload(notebook_id: str, wrapup: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Upload a wrap-up summary to NotebookLM.

    Args:
        notebook_id: Target notebook ID
        wrapup: WrapupSummary dict

    Returns:
        Source data dict or None if upload fails
    """
    # Format as Markdown
    markdown = wrapup_format_markdown(wrapup)

    # Create title
    session = wrapup.get("session", {})
    title = f"Session {session.get('session_id', 'unknown')} - {session.get('skill_id', 'unknown')}"

    # Upload as text source
    return source_upload_text(notebook_id, title, markdown)


def wrapup_create_and_upload(
    notebook_id: str,
    summary: str,
    decisions: str,
    files: str,
    steps: str,
) -> Optional[Dict[str, Any]]:
    """Extract current session context, format wrap-up, and upload.

    Args:
        notebook_id: Target notebook ID
        summary: Session summary text
        decisions: Key decisions made
        files: Files that were changed
        steps: Next steps to take

    Returns:
        Source data dict or None if upload fails
    """
    from .session import session_extract_from_current_dir

    # Extract session context
    session = session_extract_from_current_dir()
    if session is None:
        print("Error: Could not extract session context")
        return None

    # Format wrap-up
    wrapup = wrapup_format_summary(session, summary, decisions, files, steps)

    # Upload to NotebookLM
    return wrapup_upload(notebook_id, wrapup)
