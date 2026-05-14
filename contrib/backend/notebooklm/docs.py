# contrib/backend/notebooklm/docs.py
# NotebookLM ↔ GitHub Documentation Extension
# phi^2 + 1/phi^2 = 3 | TRINITY

"""NotebookLM extension for GitHub Documentation sync.

Provides bidirectional sync between documentation files and NotebookLM.
"""

from typing import Optional, List, Dict
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path


@dataclass
class NotebookLMDocLink:
    """Link between documentation file and NotebookLM source.

    Attributes:
        doc_path: str
        notebooklm_source_id: str
        created_at: Timestamp
    """

    doc_path: str
    notebooklm_source_id: str
    created_at: datetime


def doc_upload_notebooklm(
    notebooklm_client,
    doc_path: str,
    title: str,
) -> Optional[str]:
    """Upload documentation to NotebookLM.

    Args:
        notebooklm_client: NotebookLM client instance
        doc_path: Path to documentation file
        title: Document title

    Returns:
        NotebookLM source ID if successful, None otherwise

    Complexity: O(n) where n = doc size
    """
    try:
        from contrib.backend.notebooklm.sources import source_upload_text

        # Read file
        with open(doc_path, "r", encoding="utf-8") as f:
            file_content = f.read()

        # Build NotebookLM content with metadata
        content = f"""# {title}

## Path
{doc_path}

## Type
Documentation

## Content
{file_content}

---
Uploaded from t27 SSOT GitHub Bridge
"""

        # Upload
        source_id = source_upload_text(
            notebooklm_client=notebooklm_client,
            content=content,
            title=title,
        )

        if source_id:
            print(f"Uploaded documentation {doc_path} to NotebookLM: {source_id}")
        else:
            print(f"Failed to upload {doc_path}")

        return source_id

    except Exception as e:
        print(f"Error uploading {doc_path}: {e}")
        return None


def doc_sync_all(
    notebooklm_client,
    repo_root: str = ".",
    pattern: str = "*.md",
) -> Dict[str, int]:
    """Sync all documentation files matching pattern.

    Args:
        notebooklm_client: NotebookLM client instance
        repo_root: Repository root path
        pattern: File pattern to match (e.g., "*.md", "*.tex")

    Returns:
        Dict with "synced", "failed" counts

    Complexity: O(n) where n = docs count
    """
    repo_path = Path(repo_root)
    synced = 0
    failed = 0

    # Find all matching files
    docs = list(repo_path.glob(pattern))

    for doc in docs:
        if not doc.is_file():
            continue

        title = f"[{doc.suffix[1:]}] {doc.stem}"

        if doc_upload_notebooklm(notebooklm_client, str(doc), title):
            synced += 1
        else:
            failed += 1

    print(f"Doc sync complete: {synced} synced, {failed} failed")

    return {"synced": synced, "failed": failed}
