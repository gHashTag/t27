# contrib/backend/notebooklm/notebooks.py
# Notebook operations for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Notebook operations: create, list, get, find, delete."""

import asyncio
from dataclasses import dataclass, asdict
from typing import Optional, List, Dict, Any
from datetime import datetime

try:
    from notebooklm import NotebookLMClient
    NOTEBOOKLM_AVAILABLE = True
except ImportError:
    NOTEBOOKLM_AVAILABLE = False

from .client import client_get_current


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
    created_at: str
    updated_at: str
    source_count: int

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)


def _run_async(coro):
    """Run async coroutine synchronously."""
    try:
        return asyncio.run(coro)
    except RuntimeError as e:
        if "This event loop" in str(e):
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            try:
                return loop.run_until_complete(coro)
            finally:
                loop.close()
        raise


def notebook_create(title: str) -> Optional[Dict[str, Any]]:
    """Create a new notebook.

    Args:
        title: Title for the notebook

    Returns:
        Dict with notebook data or None if failed
    """
    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return None

    async def _create() -> Optional[Dict[str, Any]]:
        try:
            nb = await client.notebooks.create(title)
            return Notebook(
                id=nb.id,
                title=nb.title,
                created_at=str(nb.created_at),
                updated_at=str(nb.updated_at),
                source_count=len(nb.sources) if hasattr(nb, "sources") else 0,
            ).to_dict()
        except Exception as e:
            print(f"Error creating notebook: {e}")
            return None

    return _run_async(_create())


def notebook_list() -> List[Dict[str, Any]]:
    """List all notebooks.

    Returns:
        List of notebook data dicts
    """
    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return []

    async def _list() -> List[Dict[str, Any]]:
        try:
            notebooks = await client.notebooks.list()
            return [
                Notebook(
                    id=nb.id,
                    title=nb.title,
                    created_at=str(nb.created_at),
                    updated_at=str(nb.updated_at),
                    source_count=len(nb.sources) if hasattr(nb, "sources") else 0,
                ).to_dict()
                for nb in notebooks
            ]
        except Exception as e:
            print(f"Error listing notebooks: {e}")
            return []

    return _run_async(_list())


def notebook_get(notebook_id: str) -> Optional[Dict[str, Any]]:
    """Get a specific notebook.

    Args:
        notebook_id: Notebook ID

    Returns:
        Dict with notebook data or None if not found
    """
    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return None

    async def _get() -> Optional[Dict[str, Any]]:
        try:
            nb = await client.notebooks.get(notebook_id)
            return Notebook(
                id=nb.id,
                title=nb.title,
                created_at=str(nb.created_at),
                updated_at=str(nb.updated_at),
                source_count=len(nb.sources) if hasattr(nb, "sources") else 0,
            ).to_dict()
        except Exception as e:
            print(f"Error getting notebook: {e}")
            return None

    return _run_async(_get())


def notebook_find_by_name(name: str) -> Optional[Dict[str, Any]]:
    """Find a notebook by title.

    Args:
        name: Notebook title to search for

    Returns:
        Dict with notebook data or None if not found
    """
    notebooks = notebook_list()

    for nb in notebooks:
        if nb["title"] == name:
            return nb

    return None


def notebook_delete(notebook_id: str) -> bool:
    """Delete a notebook.

    Args:
        notebook_id: Notebook ID to delete

    Returns:
        True if successful, False otherwise
    """
    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return False

    async def _delete() -> bool:
        try:
            await client.notebooks.delete(notebook_id)
            return True
        except Exception as e:
            print(f"Error deleting notebook: {e}")
            return False

    return _run_async(_delete())
