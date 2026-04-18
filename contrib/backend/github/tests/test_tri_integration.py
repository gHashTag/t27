# contrib/backend/github/tests/test_tri_integration.py
# TriBridge Tests
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Tests for TriBridge module.
"""

import pytest
from ..tri_integration import TriBridge
from ..tri_integration_types import SyncResult


def test_bridge_init():
    """Test TriBridge initialization."""
    from ..client import GitHubClient
    from ..auth import GitHubAuth

    # Mock auth token (in real usage, env var would be set)
    auth = GitHubAuth("ghp_test_token_1234567890")

    github_client = GitHubClient(auth_token=auth)

    # Mock NotebookLM client
    def mock_notebooklm():
        def notebook_query(query):
                return {"answer": f"Mock answer for: {query}"}

        bridge = TriBridge(
                github_client=github_client,
                notebooklm_client=mock_notebooklm(),
        )

    assert bridge.github is not None
    assert bridge.notebooklm_client is not None
    assert bridge.github.gh is not None


def test_create_issue_from_notebook():
    """Test creating GitHub issue from NotebookLM note."""
    from ..client import GitHubClient
    from ..auth import GitHubAuth

    auth = GitHubAuth("ghp_test_token_1234567890")
    github_client = GitHubClient(auth_token=auth)

    def mock_notebooklm():
        def notebook_query(query):
                # Return answer with GitHub issue reference pattern
                return {
                        "answer": f"This is test data for GitHub issue #123 with reference. {query}"
                }

        bridge = TriBridge(
                github_client=github_client,
                notebooklm_client=mock_notebooklm(),
        )

    # Note exists in NotebookLM (mock returns issue #123)
    result = bridge.create_issue_from_notebook("test-note-123")

    assert result is not None
    assert result == 123


def test_sync_github_to_notebooklm():
    """Test syncing GitHub issue to NotebookLM."""
    from ..client import GitHubClient
    from ..auth import GitHubAuth

    auth = GitHubAuth("ghp_test_token_1234567890")
    github_client = GitHubClient(auth_token=auth)

    def mock_notebooklm():
        def source_upload_text(**kwargs):
                return {"source_id": "mock-source-123"}
                def notebook_query(query):
                return {"answer": f"GitHub issue #123 source: mock-source-123"}

        bridge = TriBridge(
                github_client=github_client,
                notebooklm_client=mock_notebooklm(),
        )

    result = bridge.sync_github_to_notebooklm(123)

    assert result.success is True
    assert result.items_synced == 1


def test_full_sync():
    """Test full sync orchestrator."""
    from ..client import GitHubClient
    from ..auth import GitHubAuth
    from ..tri_integration_types import SyncResult

    auth = GitHubAuth("ghp_test_token_1234567890")
    github_client = GitHubClient(auth_token=auth)

    def mock_notebooklm():
        def issue_upload_notebooklm(**kwargs):
                return {"source_id": "mock-source"}
                def notebook_query(query):
                return {"answer": "No results"}

        bridge = TriBridge(
                github_client=github_client,
                notebooklm_client=mock_notebooklm(),
        )

    result = bridge.full_sync(scope="all")

    assert isinstance(result, SyncResult)
    assert result.success is True
    assert result.errors == []
