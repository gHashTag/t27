# contrib/backend/notebooklm/tests/test_sources.py
# Unit tests for sources.py
# Issue: #305
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for NotebookLM source operations."""

import sys
from pathlib import Path

repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.sources import (
    source_upload_text,
    source_list,
    source_delete,
    MAX_SOURCE_SIZE,
)


def test_max_source_size_is_10mb():
    """Test that MAX_SOURCE_SIZE is 10MB as per spec."""
    assert MAX_SOURCE_SIZE == 10 * 1024 * 1024, f"Wrong max size: {MAX_SOURCE_SIZE}"
    print("[PASS] test_max_source_size_is_10mb")


def test_source_upload_text_returns_none_without_client():
    """Test that source_upload_text returns None when no client connected."""
    from contrib.backend.notebooklm.client import client_reset
    client_reset()
    result = source_upload_text("test-notebook-id", "test content", "test.txt")
    assert result is None
    print("[PASS] test_source_upload_text_returns_none_without_client")


def test_source_list_returns_empty_without_client():
    """Test that source_list returns empty list when no client connected."""
    from contrib.backend.notebooklm.client import client_reset
    client_reset()
    result = source_list("test-notebook-id")
    assert result == []
    print("[PASS] test_source_list_returns_empty_without_client")


def test_source_delete_returns_false_without_client():
    """Test that source_delete returns False when no client connected."""
    from contrib.backend.notebooklm.client import client_reset
    client_reset()
    result = source_delete("test-source-id")
    assert result is False
    print("[PASS] test_source_delete_returns_false_without_client")


if __name__ == "__main__":
    test_max_source_size_is_10mb()
    test_source_upload_text_returns_none_without_client()
    test_source_list_returns_empty_without_client()
    test_source_delete_returns_false_without_client()
    print("\nAll sources tests passed!")
