# contrib/backend/notebooklm/tests/test_queries.py
# Unit tests for queries.py
# Issue: #305
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for NotebookLM query operations."""

import sys
from pathlib import Path

repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.queries import notebook_query


def test_notebook_query_returns_none_without_client():
    """Test that notebook_query returns None when no client connected."""
    from contrib.backend.notebooklm.client import client_reset
    client_reset()
    result = notebook_query("test-notebook-id", "test query")
    assert result is None
    print("[PASS] test_notebook_query_returns_none_without_client")


def test_notebook_query_accepts_empty_query():
    """Test that notebook_query handles empty query string."""
    from contrib.backend.notebooklm.client import client_reset
    client_reset()
    result = notebook_query("test-notebook-id", "")
    assert result is None
    print("[PASS] test_notebook_query_accepts_empty_query")


if __name__ == "__main__":
    test_notebook_query_returns_none_without_client()
    test_notebook_query_accepts_empty_query()
    print("\nAll queries tests passed!")
