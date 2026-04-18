# contrib/backend/notebooklm/issues.py
# NotebookLM ↔ GitHub Issues Extension
# phi^2 + 1/phi^2 = 3 | TRINITY

"""NotebookLM extension for GitHub Issue management.

Provides bidirectional sync between GitHub issues and NotebookLM sources.
"""

from typing import Optional, List, Dict
from dataclasses import dataclass
from datetime import datetime


@dataclass
class NotebookLMIssueLink:
    """Link between GitHub issue and NotebookLM source.

    Attributes:
        github_issue_id: int
        notebooklm_source_id: str
        created_at: Timestamp
    """

    github_issue_id: int
    notebooklm_source_id: str
    created_at: datetime


def issue_upload_notebooklm(
    notebooklm_client,
    github_issue_id: int,
    title: str,
    state: str = "open",
) -> Optional[str]:
    """Upload GitHub issue to NotebookLM as source.

    Args:
        notebooklm_client: NotebookLM client instance
        github_issue_id: GitHub issue number
        title: Issue title
        state: Issue state

    Returns:
        NotebookLM source ID if successful, None otherwise

    Complexity: O(1) query + O(1) upload
    """
    # Import source upload function
    try:
        from contrib.backend.notebooklm.sources import source_upload_text
    except ImportError:
        print("source_upload_text not available - upload disabled")
        return None

    # Build issue content
    content = f"""# GitHub Issue #{github_issue_id}

## Title
{title}

## State
{state}

## Created
{datetime.now().strftime("%Y-%m-%d")}

## Labels
phi-loop, notebooklm

---

Full issue content and discussion available in GitHub repository.
"""

    # Upload as text source
    try:
        source_id = source_upload_text(
            notebooklm_client=notebooklm_client,
            content=content,
            title=f"[GitHub Issue #{github_issue_id}] {title}",
        )

        if source_id:
            print(f"Uploaded GitHub issue #{github_issue_id} to NotebookLM: {source_id}")
            return source_id
        else:
            print("Failed to upload to NotebookLM")
            return None

    except Exception as e:
        print(f"Error uploading issue #{github_issue_id}: {e}")
        return None
