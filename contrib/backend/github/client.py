# contrib/backend/github/client.py
# GitHub Client (gh CLI wrapper)
# phi^2 + 1/phi^2 = 3 | TRINITY

"""GitHub client singleton using gh CLI.

Provides process execution with token-based authentication.
"""

import subprocess
from typing import Optional, List


class GitHubClient:
    """GitHub client singleton.

    Wraps gh CLI commands with subprocess management.
    """

    _instance: Optional["GitHubClient"] = None

    @classmethod
    def get_instance(cls, auth_token: Optional[str] = None) -> "GitHubClient":
        """Get singleton instance.

        Args:
            auth_token: GitHub auth token (uses env var if not provided)

        Returns:
            GitHubClient instance

        Complexity: O(1)
        """
        if cls._instance is not None and auth_token is None:
                return cls._instance

        # Load token from env if not provided
        if auth_token is None:
                from .auth import GitHubAuth
                auth_token = GitHubAuth.token_load()

        cls._instance = cls.__new(auth_token)
        return cls._instance

    @staticmethod
    def __new(auth_token: str) -> "GitHubClient":
        """Create new GitHubClient instance.

        Args:
            auth_token: GitHub auth token

        Returns:
            New GitHubClient instance

        Complexity: O(1)
        """
        return GitHubClient(auth_token=auth_token)

    def __init__(self, auth_token: str):
        """Initialize GitHub client.

        Args:
            auth_token: GitHub auth token

        Complexity: O(1)
        """
        self.auth_token = auth_token
        self._check_gh_cli()

    def _check_gh_cli(self) -> None:
        """Check if gh CLI is available.

        Complexity: O(1)

        Raises:
            RuntimeError: If gh not found
        """
        try:
                subprocess.run(
                        ["gh", "--version"],
                        check=True,
                        capture_output=True,
                        text=True,
                )
                print("gh CLI available")
        except FileNotFoundError:
                raise RuntimeError(
                        "gh CLI not found. Install from: https://cli.github.com/"
                )

    def _run(self, cmd: List[str]) -> dict:
        """Run gh CLI command.

        Args:
            cmd: Command arguments as list (e.g., ["issue", "create", "--title", "bug"])

        Returns:
            Parsed JSON response as dict

        Complexity: O(n) where n = command length + output size

        Raises:
            RuntimeError: If gh CLI fails
        """
        try:
            # Add auth token for authenticated commands
            full_cmd = cmd.copy()
            if self.auth_token and not any(
                item in cmd for item in ["auth", "login", "--version"]
            ):
                full_cmd.extend(["--with-token", self.auth_token])

            result = subprocess.run(
                ["gh"] + full_cmd,
                check=True,
                capture_output=True,
                text=True,
            )

            # Parse JSON output (gh returns JSON when --json flag is used)
            # For commands without --json, gh returns text
            if "--json" in cmd:
                import json
                return json.loads(result.stdout)

            # Return simple dict for non-JSON output
            return {"stdout": result.stdout, "stderr": result.stderr}

        except subprocess.CalledProcessError as e:
            error = e.stderr.strip() if e.stderr else e.stdout.strip()
            raise RuntimeError(f"gh CLI error: {error}") from e

    def close(self) -> None:
        """Close gh CLI connection.

        Note: Subprocess-based clients don't need explicit closing.

        Complexity: O(1)
        """
        pass  # No-op for subprocess wrapper
