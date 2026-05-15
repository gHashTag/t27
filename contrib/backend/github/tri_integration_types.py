# contrib/backend/github/tri_integration_types.py
# Type definitions for TriBridge
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Shared type definitions for TriBridge module.
"""

from typing import Optional, Dict, List
from dataclasses import dataclass
from enum import Enum
from datetime import datetime


@dataclass
class TriBridgeConfig:
    """Configuration for TriBridge.

    Attributes:
        github_client: GitHubClient instance
        notebooklm_client: Optional callable for NotebookLM operations
        repo_root: Path to t27 repository
    """

    github_client: "GitHubClient"  # Avoid forward reference
    notebooklm_client: Optional[callable]
    repo_root: str


@dataclass
class SyncResult:
    """Result of sync operation.

    Attributes:
        success: bool
        items_synced: int
        errors: List[str]
        duration_ms: int
    """

    success: bool
    items_synced: int
    errors: List[str]
    duration_ms: int


@dataclass
class UnifiedSearchResult:
    """Result of unified search across GitHub + NotebookLM.

    Attributes:
        github_issues: List[Dict]
        github_prs: List[Dict]
        notebooklm_notes: List[Dict]
        combined_results: List[Dict]
    """

    github_issues: List[Dict]
    github_prs: List[Dict]
    notebooklm_notes: List[Dict]
    combined_results: List[Dict]


@dataclass
class Episode:
    """Episode data model.

    Attributes:
        type: EpisodeType
        github_id: int
        github_type: str  # "issue" or "pr" or "doc"
        title: str
        notebooklm_id: Optional[str]
        notebooklm_type: Optional[str]  # "source" or "note"
        created_at: datetime
        updated_at: Optional[datetime]
        status: str  # "pending", "synced", "conflict"
    """

    type: EpisodeType
    github_id: int
    github_type: str
    title: str
    notebooklm_id: Optional[str]
    notebooklm_type: Optional[str]
    created_at: datetime
    updated_at: Optional[datetime]
    status: str


class EpisodeType(Enum):
    """Episode type enumeration."""
    ISSUE = "issue"
    PR = "pr"
    DOC = "doc"
