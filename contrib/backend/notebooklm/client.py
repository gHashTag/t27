# contrib/backend/notebooklm/client.py
# Client management for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""NotebookLM client lifecycle management."""

from typing import Optional, Dict, Any

from .config import NotebookLMConfig


# Global client state for singleton pattern
_client_state: Dict[str, Any] = {
    "client": None,
    "config": None,
    "authenticated": False,
}


def client_new(config: NotebookLMConfig) -> Dict[str, Any]:
    """Create a new NotebookLM client.

    Args:
        config: NotebookLMConfig

    Returns:
        Dict with keys: 'client', 'authenticated', 'error'
    """
    client = authenticate_with_cookies(config)

    _client_state.update({
        "client": client,
        "config": config,
        "authenticated": client is not None,
    })

    return {
        "client": client,
        "authenticated": client is not None,
        "error": None if client is not None else "Authentication failed",
    }


def client_authenticate(config: NotebookLMConfig) -> bool:
    """Authenticate client with NotebookLM.

    Args:
        config: NotebookLMConfig

    Returns:
        True if successful, False otherwise
    """
    result = client_new(config)
    _client_state["authenticated"] = result["authenticated"]
    return result["authenticated"]


def client_is_authenticated() -> bool:
    """Check if client is authenticated.

    Returns:
        True if authenticated, False otherwise
    """
    return _client_state.get("authenticated", False)


def client_close() -> bool:
    """Close the current client connection.

    Returns:
        True if successful, False otherwise
    """
    client = _client_state.get("client")
    if client is None:
        return False

    result = client_close_sync(client)

    _client_state.update({
        "client": None,
        "authenticated": False,
    })

    return result


def client_get_current() -> Optional[Any]:
    """Get the current client instance.

    Returns:
        NotebookLMClient or None
    """
    return _client_state.get("client")


def client_reset() -> None:
    """Reset client state (for testing)."""
    _client_state.update({
        "client": None,
        "config": None,
        "authenticated": False,
    })
