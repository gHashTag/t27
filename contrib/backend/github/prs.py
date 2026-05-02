# contrib/backend/github/prs.py
# GitHub PR Management
# phi^2 + 1/phi^2 = 3 | TRINITY

"""GitHub Pull Request operations.

Provides PR creation, merge, and status tracking.
"""

from typing import List, Optional
from dataclasses import dataclass
from datetime import datetime


@dataclass
class GitHubPR:
    """GitHub PR data model.

    Attributes:
        id: PR number
        title: PR title
        body: PR body content
        state: PR state (open/merged/closed)
        issue_id: Linked issue number
        html_url: PR URL
        created_at: Creation timestamp
        merged_at: Merge timestamp (if merged)
    """

    id: int
    title: str
    body: str
    state: str
    issue_id: int
    html_url: str
    created_at: Optional[datetime]
    merged_at: Optional[datetime]


class GitHubPRsAPI:
    """GitHub PR API operations.

    Uses gh CLI for all operations.
    """

    def __init__(self, gh_client):
        """Initialize with gh client.

        Args:
            gh_client: GitHubClient instance

        Complexity: O(1)
        """
        self.gh = gh_client

    def pr_create(
        self,
        title: str,
        body: Optional[str] = None,
        issue_id: Optional[int] = None,
        base: Optional[str] = None,
        head: Optional[str] = None,
    ) -> GitHubPR:
        """Create a new GitHub PR.

        Args:
            title: PR title
            body: PR body content
            issue_id: Linked issue number (references in body)
            base: Base branch (default: master)
            head: Head branch

        Returns:
            Created GitHubPR

        Complexity: O(1) (gh CLI call)

        Raises:
            RuntimeError: If gh CLI fails
        """
        cmd = ["pr", "create", "--title", title]

        if body:
            cmd.extend(["--body", body])

        # Add issue reference if provided
        if issue_id:
            cmd.extend(["--issue", str(issue_id)])

        if base:
            cmd.extend(["--base", base])

        if head:
            cmd.extend(["--head", head])

        # Default to draft
        cmd.extend(["--draft"])

        result = self.gh._run(cmd)

        return GitHubPR(
            id=int(result.get("number", 0)),
            title=result.get("title", ""),
            body=result.get("body", ""),
            state="open",
            issue_id=issue_id or 0,
            html_url=result.get("url", ""),
            created_at=datetime.fromisoformat(result.get("createdAt", ""))
            if "createdAt" in result else None,
            merged_at=None,
        )

    def pr_merge(self, pr_id: int) -> bool:
        """Merge a GitHub PR.

        Args:
            pr_id: PR number to merge

        Returns:
            True if merged successfully

        Complexity: O(1) (gh CLI call)

        Raises:
            RuntimeError: If gh CLI fails
        """
        result = self.gh._run(["pr", "merge", str(pr_id), "--merge"])

        return result.get("mergedAt", None) is not None

    def pr_close(self, pr_id: int) -> bool:
        """Close a GitHub PR without merging.

        Args:
            pr_id: PR number to close

        Returns:
            True if closed successfully

        Complexity: O(1) (gh CLI call)

        Raises:
            RuntimeError: If gh CLI fails
        """
        result = self.gh._run(["pr", "close", str(pr_id)])

        return result.get("closedAt", None) is not None

    def pr_get(self, pr_id: int) -> Optional[GitHubPR]:
        """Get a GitHub PR by ID.

        Args:
            pr_id: PR number

        Returns:
            GitHubPR if found, None otherwise

        Complexity: O(1) (gh CLI call)
        """
        result = self.gh._run(["pr", "view", str(pr_id), "--json"])

        if not result.get("number"):
            return None

        pr_data = result.get("state", {}).get("mergedBy", {}).get("title", "")

        return GitHubPR(
            id=pr_id,
            title=result.get("title", ""),
            body=result.get("body", ""),
            state=result.get("state", {}).get("name", ""),
            issue_id=0,  # Not directly available
            html_url=result.get("url", ""),
            created_at=datetime.fromisoformat(result.get("createdAt", ""))
            if "createdAt" in result else None,
            merged_at=datetime.fromisoformat(result.get("mergedAt", ""))
            if "mergedAt" in result else None,
        )

    def pr_get_status(self, pr_id: int) -> Optional[dict]:
        """Get detailed PR status.

        Args:
            pr_id: PR number

        Returns:
            Status dict with state, reviews, checks, etc.

        Complexity: O(1) (gh CLI call)
        """
        result = self.gh._run(["pr", "view", str(pr_id), "--json"])

        state = result.get("state", {}).get("name", "")
        reviews = result.get("reviews", {}).get("totalCount", 0)
        checks = result.get("statusCheckRollup", [])  # Simplified

        return {
            "state": state,
            "reviews": reviews,
            "checks": checks,
        }
