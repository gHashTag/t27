# contrib/backend/github/comments.py
# GitHub Comment Management
# phi^2 + 1/phi^2 = 3 | TRINITY

"""GitHub Comment operations.

Provides comment listing, creation, and reactions for issues and PRs.
"""

from typing import List, Optional
from datetime import datetime


@dataclass
class GitHubComment:
    """GitHub comment data model.

    Attributes:
        id: Comment ID
        body: Comment content
        author: Comment author username
        created_at: Creation timestamp
        issue_id: Issue number (if on issue)
        pr_id: PR number (if on PR)
    """

    id: int
    body: str
    author: str
    created_at: Optional[datetime]
    issue_id: Optional[int]
    pr_id: Optional[int]


class GitHubCommentsAPI:
    """GitHub Comment API operations.

    Uses gh CLI for all operations.
    """

    def __init__(self, gh_client):
        """Initialize with gh client.

        Args:
            gh_client: GitHubClient instance

        Complexity: O(1)
        """
        self.gh = gh_client

    def comment_list(
        self,
        issue_id: Optional[int] = None,
        pr_id: Optional[int] = None,
        limit: Optional[int] = None,
    ) -> List[GitHubComment]:
        """List comments for an issue or PR.

        Args:
            issue_id: Issue number (exclusive with pr_id)
            pr_id: PR number (exclusive with issue_id)
            limit: Maximum number of comments

        Returns:
            List of GitHubComment

        Complexity: O(1) (gh CLI call)

        Raises:
            RuntimeError: If gh CLI fails
        """
        cmd = ["api", "rest", "repos/issues/comments"]

        # Add target identifier
        if issue_id:
            cmd.extend([str(issue_id)])
        elif pr_id:
            cmd.extend([f"pulls/{pr_id}/comments"])

        if limit:
            cmd.extend(["--limit", str(limit)])

        result = self.gh._run(cmd)

        comments = []
        for item in result:
            comments.append(
                        GitHubComment(
                            id=int(item.get("id", 0)),
                            body=item.get("body", ""),
                            author=item.get("author", {}).get("login", ""),
                            created_at=datetime.fromisoformat(item.get("createdAt", ""))
                            if "createdAt" in item else None,
                            issue_id=issue_id,
                            pr_id=pr_id,
                        )
                    )

        return comments

    def comment_create(
        self,
        body: str,
        issue_id: Optional[int] = None,
        pr_id: Optional[int] = None,
    ) -> GitHubComment:
        """Create a comment on an issue or PR.

        Args:
            body: Comment content
            issue_id: Issue number (exclusive with pr_id)
            pr_id: PR number (exclusive with issue_id)

        Returns:
            Created GitHubComment

        Complexity: O(1) (gh CLI call)

        Raises:
            RuntimeError: If gh CLI fails
        """
        if not (issue_id or pr_id):
            raise ValueError("Either issue_id or pr_id must be specified")

        # Build command
        if issue_id:
            cmd = ["issue", "comment", str(issue_id), "--body", body]
        else:
            cmd = ["pr", "comment", str(pr_id), "--body", body]

        result = self.gh._run(cmd)

        return GitHubComment(
            id=int(result.get("id", 0)),
            body=result.get("body", ""),
            author=result.get("author", {}).get("login", ""),
            created_at=datetime.fromisoformat(result.get("createdAt", ""))
            if "createdAt" in result else None,
            issue_id=issue_id,
            pr_id=pr_id,
        )

    def comment_reaction(
        self,
        comment_id: int,
        reaction: str = "eyes",
    ) -> bool:
        """Add reaction to a comment.

        Args:
            comment_id: Comment ID
            reaction: Reaction emoji (eyes, thumbsup, etc.)

        Returns:
            True if reaction added

        Complexity: O(1) (gh CLI call)

        Raises:
            RuntimeError: If gh CLI fails
        """
        cmd = ["api", "rest", "repos/comments/reactions", str(comment_id), "--add", reaction]

        result = self.gh._run(cmd)

        return result.get("addedAt", None) is not None
