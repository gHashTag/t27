# contrib/backend/github/auth.py
# GitHub Authentication
# phi^2 + 1/phi^2 = 3 | TRINITY

"""GitHub authentication using GH_TOKEN environment variable.

Reuses Agent Runner's GH_TOKEN for authentication.
"""

import os
from typing import Optional


class GitHubAuth:
    """GitHub authentication manager.

    Manages GitHub token validation and client initialization.
    """

    TOKEN_ENV_VAR = "GH_TOKEN"

    @staticmethod
    def token_load() -> Optional[str]:
        """Load GitHub token from environment.

        Returns:
            Token string if valid and set, None otherwise.

        Complexity: O(1)
        """
        token = os.getenv(GitHubAuth.TOKEN_ENV_VAR)

        # Basic validation
        if not token:
            return None

        if not token.startswith(("ghp_", "github_pat_")):
            raise ValueError(
                f"Invalid token format. "
                f"GH_TOKEN must start with 'ghp_' or 'github_pat_'"
            )

        return token

    @staticmethod
    def token_validate(token: str) -> bool:
        """Validate GitHub token format.

        Args:
            token: Token string to validate

        Returns:
            True if token has valid format, False otherwise.

        Complexity: O(1)
        """
        if not token:
            return False

        # Must be a valid PAT format
        return token.startswith(("ghp_", "github_pat_"))

    @staticmethod
    def get_client():
        """Get authenticated GitHub client.

        Returns:
            GitHubClient instance if token is valid.

        Raises:
            ValueError: If GH_TOKEN is not set or invalid.

        Complexity: O(1)
        """
        token = GitHubAuth.token_load()

        if not token:
            raise ValueError(
                "GH_TOKEN environment variable is required. "
                "Set it with: export GH_TOKEN=<token>"
            )

        if not GitHubAuth.token_validate(token):
            raise ValueError(
                "Invalid GH_TOKEN format. "
                "Must start with 'ghp_' or 'github_pat_'"
            )

        from .client import GitHubClient
        return GitHubClient(auth_token=token)
