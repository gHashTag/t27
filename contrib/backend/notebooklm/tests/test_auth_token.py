# contrib/backend/notebooklm/tests/test_auth_token.py
# Unit tests for auth_token.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for authentication token management."""

import sys
from pathlib import Path
from datetime import datetime, timedelta

# Add project root to path
repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.auth_token import (
    AuthTokens,
    token_load,
    token_save,
    token_is_valid,
    token_clear,
    TOKEN_PATH,
)


def test_auth_tokens_dataclass():
    """Test that AuthTokens is a proper dataclass."""
    now = datetime.now()
    tokens = AuthTokens(
        access_token="access123",
        refresh_token="refresh456",
        expires_at=now,
        token_type="bearer",
    )

    assert tokens.access_token == "access123"
    assert tokens.refresh_token == "refresh456"
    assert tokens.expires_at == now
    assert tokens.token_type == "bearer"
    print("[PASS] test_auth_tokens_dataclass")


def test_auth_tokens_to_dict():
    """Test that AuthTokens.to_dict() converts datetime to ISO string."""
    now = datetime.now()
    tokens = AuthTokens(
        access_token="access123",
        refresh_token="refresh456",
        expires_at=now,
        token_type="bearer",
    )

    as_dict = tokens.to_dict()

    assert isinstance(as_dict, dict)
    assert as_dict["access_token"] == "access123"
    assert as_dict["expires_at"] == now.isoformat()
    print("[PASS] test_auth_tokens_to_dict")


def test_token_is_valid_future_expires():
    """Test token_is_valid with future expiry."""
    future = datetime.now() + timedelta(hours=1)
    tokens = AuthTokens(
        access_token="access",
        refresh_token="refresh",
        expires_at=future,
        token_type="bearer",
    )

    assert token_is_valid(tokens) == True, "Future token should be valid"
    print("[PASS] test_token_is_valid_future_expires")


def test_token_is_valid_expired():
    """Test token_is_valid with expired token."""
    past = datetime.now() - timedelta(hours=1)
    tokens = AuthTokens(
        access_token="access",
        refresh_token="refresh",
        expires_at=past,
        token_type="bearer",
    )

    assert token_is_valid(tokens) == False, "Expired token should be invalid"
    print("[PASS] test_token_is_valid_expired")


def test_token_is_valid_none():
    """Test token_is_valid with None."""
    assert token_is_valid(None) == False, "None token should be invalid"
    print("[PASS] test_token_is_valid_none")


def test_token_is_expired_buffer():
    """Test token.is_expired() with buffer."""
    now = datetime.now()
    tokens = AuthTokens(
        access_token="access",
        refresh_token="refresh",
        expires_at=now + timedelta(seconds=100),
        token_type="bearer",
    )

    # Should be expired with 300s buffer
    assert tokens.is_expired(buffer_seconds=300) == True
    # Should not be expired with 50s buffer
    assert tokens.is_expired(buffer_seconds=50) == False
    print("[PASS] test_token_is_expired_buffer")


def test_token_save_and_load():
    """Test token_save and token_load round-trip."""
    future = datetime.now() + timedelta(hours=1)
    tokens = AuthTokens(
        access_token="access123",
        refresh_token="refresh456",
        expires_at=future,
        token_type="bearer",
    )

    # Save
    assert token_save(tokens) == True, "Token save should succeed"

    # Load
    loaded = token_load()
    assert loaded is not None, "Token load should return tokens"
    assert loaded.access_token == "access123"
    assert loaded.refresh_token == "refresh456"
    assert loaded.token_type == "bearer"

    # Clean up
    token_clear()
    print("[PASS] test_token_save_and_load")


def test_token_load_not_found():
    """Test token_load when file doesn't exist."""
    # Clear any existing token
    token_clear()

    result = token_load()
    assert result is None, "Token load should return None when file doesn't exist"
    print("[PASS] test_token_load_not_found")


def test_token_clear():
    """Test token_clear."""
    future = datetime.now() + timedelta(hours=1)
    tokens = AuthTokens(
        access_token="access",
        refresh_token="refresh",
        expires_at=future,
        token_type="bearer",
    )

    token_save(tokens)
    assert TOKEN_PATH.exists()

    token_clear()
    assert not TOKEN_PATH.exists(), "Token file should be deleted"
    print("[PASS] test_token_clear")


if __name__ == "__main__":
    test_auth_tokens_dataclass()
    test_auth_tokens_to_dict()
    test_token_is_valid_future_expires()
    test_token_is_valid_expired()
    test_token_is_valid_none()
    test_token_is_expired_buffer()
    test_token_save_and_load()
    test_token_load_not_found()
    test_token_clear()
    print("\nAll auth_token tests passed!")
