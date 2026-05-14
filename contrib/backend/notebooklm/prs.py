# contrib/backend/notebooklm/prs.py
# NotebookLM ↔ GitHub Pull Requests Extension
# phi^2 + 1/phi^2 = 3 | TRINITY

"""NotebookLM extension for GitHub Pull Request management.

Provides bidirectional sync between GitHub PRs and NotebookLM notes.
"""

from typing import Optional, List, Dict
from dataclasses import dataclass
from datetime import datetime


@dataclass
class NotebookLMPRLink:
    """Link between GitHub PR and NotebookLM note.

    Attributes:
        github_pr_id: int
        notebooklm_source_id: str
        created_at: Timestamp
    """

    github_pr_id: int
    notebooklm_source_id: str
    created_at: datetime


def pr_upload_notebooklm(
    notebooklm_client,
    github_pr_id: int,
    title: str,
    state: str = "open",
    merged: bool = False,
) -> Optional[str]:
    """Upload GitHub PR to NotebookLM as source.

    Args:
        notebooklm_client: NotebookLM client instance
        github_pr_id: GitHub PR number
        title: PR title
        state: PR state
        merged: Whether PR was merged

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

    # Build PR content
    merged_text = "This PR was merged" if merged else "This PR is open"

    content = f"""# GitHub Pull Request #{github_pr_id}

## Title
{title}

## State
{state}

## Merged
{merged_text}

## Created
{datetime.now().strftime("%Y-%m-%d")}

## Labels
phi-loop, notebooklm

---

Full PR details available in GitHub repository.
"""

    # Upload as text source
    try:
        source_id = source_upload_text(
            notebooklm_client=notebooklm_client,
            content=content,
            title=f"[GitHub PR #{github_pr_id}] {title}",
        )

        if source_id:
            print(f"Uploaded GitHub PR #{github_pr_id} to NotebookLM: {source_id}")
            return source_id
        else:
            print("Failed to upload to NotebookLM")
            return None

    except Exception as e:
        print(f"Error uploading PR #{github_pr_id}: {e}")
        return None
