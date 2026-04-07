# contrib/backend/notebooklm/tests/test_client.py
# Unit tests for client.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for client lifecycle management."""

import sys
from pathlib import Path

# Add project root to path
repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.client import (
    client_get_current,
    client_is_authenticated,
    client_close,
    client_reset,
)


def test_client_get_current_initially_none():
    """Test that client_get_current() returns None initially."""
    client = client_get_current()
    assert client is None, "Client should be None initially"
    print("[PASS] test_client_get_current_initially_none")


def test_client_is_authenticated_initially_false():
    """Test that client_is_authenticated() returns False initially."""
    is_auth = client_is_authenticated()
    assert is_auth is False, "Should not be authenticated initially"
    print("[PASS] test_client_is_authenticated_initially_false")


def test_client_close_without_client():
    """Test that client_close() returns False when no client."""
    result = client_close()
    assert result is False, "Should return False when no client"
    print("[PASS] test_client_close_without_client")


def test_client_reset():
    """Test that client_reset() clears state."""
    # This is mostly for testing - should not raise error
    client_reset()
    assert client_get_current() is None
    assert client_is_authenticated() is False
    print("[PASS] test_client_reset")


def test_client_state_integrity():
    """Test that client state remains consistent."""
    # Reset first
    client_reset()

    # Verify initial state
    assert client_get_current() is None
    assert client_is_authenticated() is False

    # Reset again - should still be consistent
    client_reset()
    assert client_get_current() is None
    assert client_is_authenticated() is False

    print("[PASS] test_client_state_integrity")


if __name__ == "__main__":
    test_client_get_current_initially_none()
    test_client_is_authenticated_initially_false()
    test_client_close_without_client()
    test_client_reset()
    test_client_state_integrity()
    print("\nAll client tests passed!")
