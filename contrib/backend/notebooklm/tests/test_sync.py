# contrib/backend/notebooklm/tests/test_sync.py
# Tests for Unified Sync Orchestrator
# phi^2 + 1/phi^2 = 3 | TRINITY

"""E2E tests for UnifiedSyncOrchestrator.

Tests sync operations between GitHub and NotebookLM.
These tests require valid GitHub tokens and NotebookLM cookies.
"""

import pytest
from unittest.mock import Mock, MagicMock, patch
from datetime import datetime

from contrib.backend.github.tri_integration_types import SyncResult, Episode, EpisodeType


class TestUnifiedSyncOrchestrator:
    """Test UnifiedSyncOrchestrator sync operations."""

    @pytest.fixture
    def mock_github_issues(self):
        """Mock GitHub issues client."""
        client = Mock()
        client.issue_list = Mock(return_value=[
            Mock(id=1, title="Test Issue", state="open", number=1)
        ])
        return client

    @pytest.fixture
    def mock_github_prs(self):
        """Mock GitHub PRs client."""
        client = Mock()
        client.pr_list = Mock(return_value=[
            Mock(id=2, title="Test PR", state="open", number=2, merged_at=None)
        ])
        return client

    @pytest.fixture
    def mock_github_docs(self):
        """Mock GitHub docs client."""
        client = Mock()
        client.doc_list = Mock(return_value=[
            Mock(id=3, title="Test Doc", path="docs/test.md")
        ])
        return client

    @pytest.fixture
    def mock_notebooklm_issue(self):
        """Mock NotebookLM issue sync function."""
        return Mock(return_value="source-id-1")

    @pytest.fixture
    def mock_notebooklm_pr(self):
        """Mock NotebookLM PR sync function."""
        return Mock(return_value="source-id-2")

    @pytest.fixture
    def mock_notebooklm_doc(self):
        """Mock NotebookLM doc sync function."""
        return Mock(return_value="source-id-3")

    @pytest.fixture
    def orchestrator(self, mock_github_issues, mock_github_prs, mock_github_docs,
                     mock_notebooklm_issue, mock_notebooklm_pr, mock_notebooklm_doc):
        """Create UnifiedSyncOrchestrator with mocks."""
        from contrib.backend.notebooklm.sync import UnifiedSyncOrchestrator

        return UnifiedSyncOrchestrator(
            github_issues=mock_github_issues,
            github_prs=mock_github_prs,
            github_docs=mock_github_docs,
            notebooklm_issue=mock_notebooklm_issue,
            notebooklm_pr=mock_notebooklm_pr,
            notebooklm_doc=mock_notebooklm_doc,
        )

    def test_sync_issues(self, orchestrator, mock_github_issues, mock_notebooklm_issue):
        """Test GitHub Issues sync."""
        result = orchestrator.sync_issues()

        assert result.success is True
        assert result.items_synced == 1
        assert len(result.errors) == 0
        mock_github_issues.issue_list.assert_called_once_with(state="open", limit=5)
        mock_notebooklm_issue.assert_called_once()

    def test_sync_prs(self, orchestrator, mock_github_prs, mock_notebooklm_pr):
        """Test GitHub PRs sync."""
        result = orchestrator.sync_prs()

        assert result.success is True
        assert result.items_synced == 1
        assert len(result.errors) == 0
        mock_github_prs.pr_list.assert_called_once_with(state="open", limit=5)
        mock_notebooklm_pr.assert_called_once()

    def test_sync_docs(self, orchestrator, mock_github_docs, mock_notebooklm_doc):
        """Test GitHub Documentation sync."""
        result = orchestrator.sync_docs()

        assert result.success is True
        assert result.items_synced == 1
        assert len(result.errors) == 0
        mock_github_docs.doc_list.assert_called_once()
        mock_notebooklm_doc.assert_called_once()

    def test_full_sync(self, orchestrator):
        """Test full sync across all entities."""
        result = orchestrator.full_sync()

        assert result.success is True
        assert result.items_synced == 3  # issues + prs + docs
        assert len(result.errors) == 0
        assert result.duration_ms > 0

    def test_sync_with_errors(self, orchestrator, mock_notebooklm_issue):
        """Test sync with errors."""
        # Make sync fail
        mock_notebooklm_issue.side_effect = Exception("Sync failed")

        result = orchestrator.sync_issues()

        assert result.success is False
        assert result.items_synced == 0
        assert len(result.errors) > 0

    def test_sync_result_type(self, orchestrator):
        """Test SyncResult type validation."""
        result = orchestrator.sync_issues()

        assert isinstance(result, SyncResult)
        assert isinstance(result.success, bool)
        assert isinstance(result.items_synced, int)
        assert isinstance(result.errors, list)
        assert isinstance(result.duration_ms, int)


class TestEpisodeType:
    """Test EpisodeType enumeration."""

    def test_episode_type_values(self):
        """Test EpisodeType has correct values."""
        assert EpisodeType.ISSUE.value == "issue"
        assert EpisodeType.PR.value == "pr"
        assert EpisodeType.DOC.value == "doc"

    def test_episode_type_members(self):
        """Test EpisodeType has all expected members."""
        assert hasattr(EpisodeType, "ISSUE")
        assert hasattr(EpisodeType, "PR")
        assert hasattr(EpisodeType, "DOC")


class TestEpisode:
    """Test Episode dataclass."""

    def test_episode_creation(self):
        """Test Episode can be created."""
        now = datetime.now()
        episode = Episode(
            type=EpisodeType.ISSUE,
            github_id=1,
            github_type="issue",
            title="Test Issue",
            notebooklm_id=None,
            notebooklm_type=None,
            created_at=now,
            updated_at=None,
            status="pending",
        )

        assert episode.github_id == 1
        assert episode.type == EpisodeType.ISSUE
        assert episode.title == "Test Issue"
        assert episode.status == "pending"

    def test_episode_with_optional_fields(self):
        """Test Episode with optional fields."""
        episode = Episode(
            type=EpisodeType.PR,
            github_id=2,
            github_type="pr",
            title="Test PR",
            notebooklm_id="source-id-2",
            notebooklm_type="source",
            created_at=datetime.now(),
            updated_at=datetime.now(),
            status="synced",
        )

        assert episode.updated_at is not None
        assert episode.notebooklm_id == "source-id-2"
