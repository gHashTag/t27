# contrib/backend/notebooklm/tests/test_notebooks.py
# Unit tests for notebooks.py
# Issue: #305
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for NotebookLM notebook operations."""

import sys
from pathlib import Path

repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.notebooks import (
    notebook_find_by_name,
    Notebook,
)


def test_notebook_creation():
    """Test Notebook dataclass creation."""
    nb = Notebook(
        id="test-id",
        title="test-notebook",
        created_at="2026-04-30",
        updated_at="2026-04-30",
        source_count=5,
    )
    assert nb.id == "test-id"
    assert nb.title == "test-notebook"
    assert nb.source_count == 5
    print("[PASS] test_notebook_creation")


def test_notebook_find_by_name_returns_none_when_no_client():
    """Test that notebook_find_by_name returns None when no client is connected."""
    from contrib.backend.notebooklm.client import client_reset
    client_reset()
    result = notebook_find_by_name("nonexistent")
    assert result is None
    print("[PASS] test_notebook_find_by_name_returns_none_when_no_client")


def test_notebook_to_dict():
    """Test Notebook serialization."""
    nb = Notebook(
        id="abc",
        title="Test Notebook",
        created_at="2026-04-30T00:00:00Z",
        updated_at="2026-04-30T00:00:00Z",
        source_count=0,
    )
    d = nb.__dict__
    assert isinstance(d, dict)
    assert d["title"] == "Test Notebook"
    assert d["source_count"] == 0
    print("[PASS] test_notebook_to_dict")


if __name__ == "__main__":
    test_notebook_creation()
    test_notebook_find_by_name_returns_none_when_no_client()
    test_notebook_to_dict()
    print("\nAll notebook tests passed!")
