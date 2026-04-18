# contrib/backend/github/issues.py
# GitHub Issue Management
# phi^2 + 1/phi^2 = 3 | TRINITY

"""GitHub Issue operations.

Provides CRUD operations for GitHub issues.
"""

from typing import List, Optional, Dict
from dataclasses import dataclass
from datetime import datetime


@dataclass
class GitHubIssue:
    """GitHub issue data model.

    Attributes:
        id: Issue number
        title: Issue title
        body: Issue body content
        state: Issue state (open/in_progress/closed)
        labels: List of labels
        html_url: Issue URL
        created_at: Creation timestamp
        updated_at: Last update timestamp
    """

    id: int
    title: str
    body: str
    state: str
    labels: List[str]
    html_url: str
    created_at: Optional[datetime]
    updated_at: Optional[datetime]


class GitHubIssuesAPI:
    """GitHub Issue API operations.

    Uses gh CLI for all operations.
    """

    def __init__(self, gh_client):
        """Initialize with gh client.

        Args:
            gh_client: GitHubClient instance

        Complexity: O(1)
        """
        self.gh = gh_client

    def issue_create(
        self,
        title: str,
        body: Optional[str] = None,
        labels: Optional[List[str]] = None,
    ) -> GitHubIssue:
        """Create a new GitHub issue.

        Args:
            title: Issue title
            body: Issue body content
            labels: List of labels to apply

        Returns:
            Created GitHubIssue

        Complexity: O(1) (gh CLI call)

        Raises:
            RuntimeError: If gh CLI fails
        """
        cmd = ["issue", "create", "--title", title, "--body", body or ""]

        # Add labels if provided
        if labels:
            for label in labels:
                cmd.extend(["--label", label])

        result = self.gh._run(cmd)

        return GitHubIssue(
            id=int(result.get("number", 0)),
            title=result.get("title", ""),
            body=result.get("body", ""),
            state="open",
            labels=labels or [],
            html_url=result.get("url", ""),
            created_at=datetime.fromisoformat(result.get("createdAt", ""))
            if "createdAt" in result else None,
            updated_at=datetime.fromisoformat(result.get("updatedAt", ""))
            if "updatedAt" in result else None,
        )

    def issue_update(
        self,
        issue_id: int,
        title: Optional[str] = None,
        body: Optional[str] = None,
        state: Optional[str] = None,
    ) -> GitHubIssue:
        """Update an existing GitHub issue.

        Args:
            issue_id: Issue number to update
            title: New title (optional)
            body: New body (optional)
            state: New state (open/in_progress/closed)

        Returns:
            Updated GitHubIssue

        Complexity: O(1) (gh CLI call)

        Raises:
            RuntimeError: If gh CLI fails
        """
        cmd = ["issue", "edit", str(issue_id)]

        if title:
            cmd.extend(["--title", title])

        if body:
            cmd.extend(["--body", body])

        if state:
            cmd.extend(["--state", state])

        result = self.gh._run(cmd)

        return GitHubIssue(
            id=issue_id,
            title=result.get("title", ""),
            body=result.get("body", ""),
            state=result.get("state", ""),
            labels=[],  # Labels not returned by edit
            html_url=result.get("url", ""),
            updated_at=datetime.fromisoformat(result.get("updatedAt", ""))
            if "updatedAt" in result else None,
        )

    def issue_get(self, issue_id: int) -> Optional[GitHubIssue]:
        """Get a GitHub issue by ID.

        Args:
            issue_id: Issue number

        Returns:
            GitHubIssue if found, None otherwise

        Complexity: O(1) (gh CLI call)
        """
        result = self.gh._run(["issue", "view", str(issue_id)])

        if not result.get("id"):
            return None

        return GitHubIssue(
            id=issue_id,
            title=result.get("title", ""),
            body=result.get("body", ""),
            state=result.get("state", ""),
            labels=[label.get("name", "") for label in result.get("labels", [])],
            html_url=result.get("url", ""),
            updated_at=datetime.fromisoformat(result.get("updatedAt", ""))
            if "updatedAt" in result else None,
        )

    def issue_list(
        self,
        state: Optional[str] = None,
        labels: Optional[List[str]] = None,
        limit: Optional[int] = None,
    ) -> List[GitHubIssue]:
        """List GitHub issues.

        Args:
            state: Filter by state (open/closed/all)
            labels: Filter by labels
            limit: Maximum number of results

        Returns:
            List of GitHubIssue

        Complexity: O(n) (gh CLI call)
        """
        cmd = ["issue", "list", "--json"]

        if state:
            cmd.extend(["--state", state])

        if labels:
            for label in labels:
                cmd.extend(["--label", label])

        if limit:
            cmd.extend(["--limit", str(limit)])

        result = self.gh._run(cmd)

        issues = []
        for item in result:
            issues.append(
                GitHubIssue(
                    id=int(item.get("number", 0)),
                    title=item.get("title", ""),
                    body=item.get("body", ""),
                    state=item.get("state", ""),
                    labels=[label.get("name", "") for label in item.get("labels", [])],
                    html_url=item.get("url", ""),
                    created_at=datetime.fromisoformat(item.get("createdAt", ""))
                    if "createdAt" in item else None,
                    updated_at=datetime.fromisoformat(item.get("updatedAt", ""))
                    if "updatedAt" in item else None,
                )
            )

        return issues

    def issue_find_similar(
        self,
        query: str,
        threshold: float = 0.7,
    ) -> List[GitHubIssue]:
        """Find similar issues based on query.

        Uses GitHub search API via gh CLI.

        Args:
            query: Search query string
            threshold: Similarity threshold (0-0 to 1.0)

        Returns:
            List of similar GitHubIssue, sorted by relevance

        Complexity: O(n * m) where n = search results, m = labels per issue

        Note:
            This is a simplified similarity based on GitHub search ranking.
            Future improvement: Use semantic embedding comparison.
        """
        cmd = ["search", "issues", query, "--limit", "20", "--json"]

        result = self.gh._run(cmd)

        issues = []
        for item in result:
            # Simple similarity: check if query appears in title or body
            title_lower = item.get("title", "").lower()
            body_lower = item.get("body", "").lower()
            query_lower = query.lower()

            similarity = 0.0

            if query_lower in title_lower:
                similarity += 0.5

            if query_lower in body_lower:
                similarity += 0.3

            # Add bonus for matching labels
            labels = item.get("labels", [])
            for label in labels:
                if query_lower in label.get("name", "").lower():
                    similarity += 0.1

            if similarity >= threshold:
                issues.append(
                    GitHubIssue(
                        id=int(item.get("number", 0)),
                        title=item.get("title", ""),
                        body=item.get("body", ""),
                        state=item.get("state", ""),
                        labels=[label.get("name", "") for label in labels],
                        html_url=item.get("url", ""),
                        similarity=similarity,
                    )
                )

        # Sort by similarity descending
        issues.sort(key=lambda x: x.similarity, reverse=True)

        return issues[:5]  # Return top 5
