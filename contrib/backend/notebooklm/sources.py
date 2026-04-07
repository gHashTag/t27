# contrib/backend/notebooklm/sources.py
# Source operations for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Source operations: upload text, upload file, list, delete."""

import asyncio
import os
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional, List, Dict, Any

try:
    from notebooklm import NotebookLMClient
    NOTEBOOKLM_AVAILABLE = True
except ImportError:
    NOTEBOOKLM_AVAILABLE = False

from .client import client_get_current

# Constants
MAX_SOURCE_SIZE = 10 * 1024 * 1024  # 10MB


@dataclass
class Source:
    """Source data structure.

    Attributes:
        id: Source ID
        notebook_id: Parent notebook ID
        title: Source title
        source_type: Type (text, url, file, youtube)
        status: Processing status
        created_at: Creation timestamp
    """

    id: str
    notebook_id: str
    title: str
    source_type: str
    status: str
    created_at: str

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


def source_upload_text(notebook_id: str, title: str, content: str) -> Optional[Dict[str, Any]]:
    """Upload text content as a source.

    Args:
        notebook_id: Target notebook ID
        title: Source title
        content: Text content to upload

    Returns:
        Dict with source data or None if failed
    """
    if len(content.encode('utf-8')) > MAX_SOURCE_SIZE:
        print(f"Error: Content exceeds max size of {MAX_SOURCE_SIZE} bytes")
        return None

    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return None

    async def _upload() -> Optional[Dict[str, Any]]:
        try:
            src = await client.sources.add_text(notebook_id, content, title)
            return Source(
                id=src.id,
                notebook_id=notebook_id,
                title=title,
                source_type="text",
                status="ready",
                created_at=str(src.created_at) if hasattr(src, "created_at") else "",
            ).to_dict()
        except Exception as e:
            print(f"Error uploading text: {e}")
            return None

    return _run_async(_upload())


def source_upload_file(notebook_id: str, file_path: str) -> Optional[Dict[str, Any]]:
    """Upload a file as a source.

    Args:
        notebook_id: Target notebook ID
        file_path: Path to file to upload

    Returns:
        Dict with source data or None if failed
    """
    path = Path(file_path)
    if not path.exists():
        print(f"Error: File not found: {file_path}")
        return None

    file_size = path.stat().st_size
    if file_size > MAX_SOURCE_SIZE:
        print(f"Error: File exceeds max size of {MAX_SOURCE_SIZE} bytes")
        return None

    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return None

    async def _upload() -> Optional[Dict[str, Any]]:
        try:
            src = await client.sources.add_file(notebook_id, str(path))
            return Source(
                id=src.id,
                notebook_id=notebook_id,
                title=path.name,
                source_type="file",
                status="processing",
                created_at=str(src.created_at) if hasattr(src, "created_at") else "",
            ).to_dict()
        except Exception as e:
            print(f"Error uploading file: {e}")
            return None

    return _run_async(_upload())


def source_list(notebook_id: str) -> List[Dict[str, Any]]:
    """List all sources in a notebook.

    Args:
        notebook_id: Notebook ID

    Returns:
        List of source data dicts
    """
    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return []

    async def _list() -> List[Dict[str, Any]]:
        try:
            sources = await client.sources.list(notebook_id)
            return [
                Source(
                    id=src.id,
                    notebook_id=notebook_id,
                    title=src.title,
                    source_type=src.type if hasattr(src, "type") else "unknown",
                    status=src.status if hasattr(src, "status") else "unknown",
                    created_at=str(src.created_at) if hasattr(src, "created_at") else "",
                ).to_dict()
                for src in sources
            ]
        except Exception as e:
            print(f"Error listing sources: {e}")
            return []

    return _run_async(_list())


def source_delete(source_id: str) -> bool:
    """Delete a source.

    Args:
        source_id: Source ID to delete

    Returns:
        True if successful, False otherwise
    """
    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return False

    async def _delete() -> bool:
        try:
            # NotebookLM SDK may not have direct source delete
            # This is a placeholder for when the API supports it
            print(f"Warning: source_delete not fully implemented (ID: {source_id})")
            return False
        except Exception as e:
            print(f"Error deleting source: {e}")
            return False

    return _run_async(_delete())
