# contrib/backend/notebooklm/notebooks.py
# Notebook CRUD operations for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

from typing import Optional, List, Dict, Any
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime

from .config import NotebookLMConfig, config_from_env
from .client import client_new, _update_client_state, client_get_current
from .auth_token import token_load, token_save, token_is_valid, AuthTokens

# Global cache for notebooks (in-memory)
_notebook_cache: Dict[str, Notebook] = {}


def _clear_cache() -> None:
    """Clear notebook cache.

    Complexity: O(n) where n is cache size
    """
    _notebook_cache.clear()


# ============================================================================
# 1. Data Structures
# ============================================================================

@dataclass
class Notebook:
    """Notebook data structure.

    Attributes:
        id: Notebook ID
        title: Notebook title
        created_at: Creation timestamp
        updated_at: Last update timestamp
        source_count: Number of sources
    """
    id: str
    title: str
    created_at: datetime
    updated_at: datetime
    source_count: int = 0


# ============================================================================
# 2. Helper Functions
# ============================================================================

def _run_sync(coro):
    """Run async coroutine synchronously.

    Args:
        coro: Async coroutine to run

    Returns:
        Result of coroutine or None on error
    """
    import asyncio
    try:
        loop = asyncio.get_event_loop()
        if loop.is_running():
            import concurrent.futures
            import threading

            result = [None]
            exception = [None]

            def run_in_new_loop():
                new_loop = asyncio.new_event_loop()
                asyncio.set_event_loop(new_loop)
                try:
                    result[0] = new_loop.run_until_complete(coro)
                except Exception as e:
                    exception[0] = e
                finally:
                    new_loop.close()

            thread = threading.Thread(target=run_in_new_loop)
            thread.start()
            thread.join(timeout=60)

            if exception[0]:
                raise exception[0]
            return result[0]
        else:
            return loop.run_until_complete(coro)
    except RuntimeError:
        return asyncio.run(coro)


# ============================================================================
# 3. Notebook CRUD Functions
# ============================================================================

def notebook_create(title: str, config: Optional[NotebookLMConfig] = None) -> Dict[str, Any]:
    """Create a new NotebookLM notebook.

    Args:
        title: Title for the notebook

    Returns:
        Dict with keys: 'success', 'notebook', 'error'

    Complexity: O(1)
    """
    if config is None:
        config = config_from_env()

    # Validate notebook name
    if not title or len(title.strip()) == 0:
        return {
            "success": False,
            "notebook": None,
            "error": "Notebook name cannot be empty",
        }

    # Check cache first
    cache_key = f"nb_{title}"
    if cache_key in _notebook_cache:
        # Return cached notebook without re-fetching
        return {
            "success": True,
            "notebook": _notebook_cache[cache_key],
        }

    # Initialize client
    client = client_new(config)
    if client is None:
        return {
            "success": False,
            "notebook": None,
            "error": "Failed to initialize client",
        }

    from notebooklm import Notebook

    async def _create():
        try:
            nb = await Notebook()
            result = await nb.notebooks.create(title)

            # Update cache
            new_nb = Notebook(
                id=str(result.id),
                title=result.title,
                created_at=result.created_at,
                updated_at=result.updated_at,
                source_count=len(result.sources) if hasattr(result, "sources") else 0,
            )
            _notebook_cache[cache_key] = new_nb
            _clear_cache()  # Clear old cache on success

            return {
                "success": True,
                "notebook": new_nb,
            }
        except Exception as e:
            return {
                "success": False,
                "notebook": None,
                "error": str(e),
            }

    return _run_sync(_create)


def notebook_list(config: Optional[NotebookLMConfig] = None) -> Dict[str, Any]:
    """List all NotebookLM notebooks.

    Args:
        config: Configuration (uses defaults if None)

    Returns:
        List of Notebook data dicts

    Complexity: O(1)
    """
    if config is None:
        config = config_from_env()

    # Initialize client
    client = client_new(config)
    if client is None:
        return []

    from notebooklm import Notebook

    async def _list():
        try:
            result = await client.notebooks.list()

            # Convert to Notebook objects
            notebooks = []
            for nb in result:
                notebooks.append(Notebook(
                    id=str(nb.id),
                    title=nb.title,
                    created_at=nb.created_at,
                    updated_at=nb.updated_at,
                    source_count=len(nb.sources) if hasattr(nb, "sources") else 0,
                ))

            return notebooks
        except Exception:
            return []

    return _run_sync(_list)


def notebook_get(notebook_id: str, config: Optional[NotebookLMConfig] = None) -> Optional[Notebook]:
    """Get a specific notebook by ID.

    Args:
        notebook_id: Notebook ID
        config: Configuration (uses defaults if None)

    Returns:
        Notebook object or None if not found

    Complexity: O(1)
    """
    if config is None:
        config = config_from_env()

    # Check cache first
    if notebook_id in _notebook_cache:
        return _notebook_cache[notebook_id]

    # Initialize client
    client = client_new(config)
    if client is None:
        return None

    from notebooklm import Notebook

    async def _get():
        try:
            result = await client.notebooks.get(notebook_id)
            return Notebook(
                id=str(result.id),
                title=result.title,
                created_at=result.created_at,
                updated_at=result.updated_at,
                source_count=len(result.sources) if hasattr(result, "sources") else 0,
            )
        except Exception:
            return None

    return _run_sync(_get)


def notebook_find_by_name(name: str, config: Optional[NotebookLMConfig] = None) -> Optional[Notebook]:
    """Find a notebook by title.

    Args:
        name: Notebook title to search for
        config: Configuration (uses defaults if None)

    Returns:
        Notebook object or None if not found

    Complexity: O(n) where n is number of notebooks
    """
    if config is None:
        config = config_from_env()

    # List all notebooks to search
    all_notebooks = notebook_list(config)

    # Find matching notebook (case-insensitive)
    for notebook in all_notebooks:
        if notebook["title"].lower() == name.lower():
            return notebook

    return None


def notebook_delete(notebook_id: str, config: Optional[NotebookLMConfig] = None) -> bool:
    """Delete a notebook.

    Args:
        notebook_id: Notebook ID to delete
        config: Configuration (uses defaults if None)

    Returns:
        True if successful, False otherwise

    Complexity: O(1)
    """
    if config is None:
        config = config_from_env()

    # Remove from cache
    if notebook_id in _notebook_cache:
        del _notebook_cache[notebook_id]

    # Initialize client
    client = client_new(config)
    if client is None:
        return False

    from notebooklm import Notebook

    async def _delete():
        try:
            # Use notebooks.delete (synchronous)
            await client.notebooks.delete(notebook_id)

            # Update cache
            if notebook_id in _notebook_cache:
                del _notebook_cache[notebook_id]

            return True
        except Exception:
            return False

    return _run_sync(_delete)
