# contrib/backend/github
# GitHub API integration for t27 SSOT (Issues + PRs + Docs → NotebookLM)
# phi^2 + 1/phi^2 = 3 | TRINITY

"""GitHub backend for autonomous issue/PR/documentation management.

Provides:
- Issue operations: create, update, list, close
- PR operations: create, merge, close, status
- Documentation operations: upload, sync, query
- Comment operations: list, create, react
- Authentication: GH_TOKEN-based auth
- Bridge: /tri skill ↔ GitHub ↔ NotebookLM

Usage:
    from contrib.backend.github import TriBridge, GitHubClient

    client = GitHubClient()
    issues = client.issues.list(labels="phi-loop")

    bridge = TriBridge()
    source_id = bridge.sync_github_to_notebooklm(issue_id=128)
"""

__all__ = [
    # Client
    "GitHubClient",
    "GitHubAuth",
    # Modules
    "issues",
    "prs",
    "docs",
    "comments",
    # Bridge
    "TriBridge",
    # Types
    "GitHubIssue",
    "GitHubPR",
    "GitHubDoc",
]

from .client import GitHubClient
from .auth import GitHubAuth
from .tri_integration import TriBridge

# Version
__version__ = "1.0.0"
