# contrib/backend/github/docs.py
# GitHub Documentation Management
# phi^2 + 1/phi^2 = 3 | TRINITY

"""GitHub documentation operations.

Provides document upload to NotebookLM, sync, and query.
"""

from typing import List, Optional
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path


@dataclass
class GitHubDoc:
    """GitHub documentation data model.

    Attributes:
        path: Document file path
        title: Document title
        type: Document type (paper/spec/whitepaper/readme)
        created_at: Upload timestamp
    """

    path: str
    title: str
    doc_type: str
    created_at: Optional[datetime]


class GitHubDocsAPI:
    """GitHub Documentation API operations.

    Uses file system operations for docs.
    """

    def __init__(self, repo_root: str = "."):
        """Initialize with repository root.

        Args:
            repo_root: Path to t27 repository

        Complexity: O(1)
        """
        self.repo_root = Path(repo_root)

    def _get_docs_dir(self) -> Path:
        """Get documentation directory path.

        Returns:
            Path to docs/ directory

        Complexity: O(1)
        """
        return self.repo_root / "docs"

    def doc_list(self) -> List[GitHubDoc]:
        """List all documentation files.

        Returns:
            List of GitHubDoc

        Complexity: O(n) where n = docs count

        Raises:
            IOError: If directory doesn't exist
        """
        docs_dir = self._get_docs_dir()

        if not docs_dir.exists():
            raise IOError(f"Documentation directory not found: {docs_dir}")

        docs = []

        # Common doc patterns
        for pattern in [
                "WHITEPAPER/*.md",
                "WHITEPAPER/*.tex",
                "WHITEPAPER/*.bib",
                "specs/**/*.t27",
                "neurips/**/*.tex",
                "neurips/**/*.bib",
                "README.md",
                "*.md",
        ]:
            for file in docs_dir.glob(pattern):
                docs.append(
                        GitHubDoc(
                                path=str(file.relative_to(self.repo_root)),
                                title=file.stem,
                                doc_type=self._detect_doc_type(file),
                                created_at=datetime.fromtimestamp(file.stat().st_mtime),
                        )
                )

        # Sort by type, then date
        docs.sort(key=lambda x: (x.doc_type, x.created_at), reverse=True)

        return docs

    def _detect_doc_type(self, file_path: Path) -> str:
        """Detect document type from file path.

        Args:
            file_path: Path to file

        Returns:
            Document type string

        Complexity: O(1)
        """
        path_str = str(file_path)

        if "WHITEPAPER" in path_str:
                if ".tex" in path_str or ".bib" in path_str:
                        return "paper"
                else:
                        return "whitepaper"

        elif "neurips" in path_str:
                if ".tex" in path_str or ".bib" in path_str:
                        return "neurips"
                else:
                        return "spec"

        elif "specs/" in path_str:
                return "spec"

        elif ".md" in path_str:
                if "README" in path_str:
                        return "readme"
                else:
                        return "doc"

        else:
                return "unknown"

    def doc_sync(self, notebooklm_client) -> bool:
        """Sync all documentation to NotebookLM.

        Args:
            notebooklm_client: NotebookLM client instance

        Returns:
            True if sync successful

        Complexity: O(n) where n = docs count

        Raises:
            RuntimeError: If sync fails
        """
        from contrib.backend.notebooklm.sources import source_upload_text

        docs = self.doc_list()

        for doc in docs:
                try:
                        # Read file content
                        with open(self.repo_root / doc.path, "r") as f:
                                content = f.read()

                        # Upload to NotebookLM
                        source_upload_text(
                                notebooklm_client=notebooklm_client,
                                content=content,
                                title=f"[{doc.doc_type.upper()}] {doc.title}",
                        )

                        print(f"Synced: {doc.path}")

                except Exception as e:
                        print(f"Error syncing {doc.path}: {e}")
                        return False

        return True

    def doc_find_similar(
        self,
        query: str,
        limit: int = 5,
    ) -> List[GitHubDoc]:
        """Find similar documentation based on query.

        Args:
            query: Search query string
            limit: Maximum number of results

        Returns:
            List of similar GitHubDoc

        Complexity: O(n) where n = docs count

        Note:
            This is a simple keyword matching.
            Future improvement: Use semantic embedding comparison.
        """
        docs = self.doc_list()
        query_lower = query.lower()

        # Simple similarity: check if query appears in path or title
        scored = []

        for doc in docs:
                similarity = 0.0
                path_lower = doc.path.lower()
                title_lower = doc.title.lower()

                if query_lower in path_lower:
                        similarity += 0.5

                if query_lower in title_lower:
                        similarity += 0.3

                if similarity > 0:
                        scored.append((similarity, doc))

        # Sort by similarity descending
        scored.sort(key=lambda x: x[0], reverse=True)

        return [doc for _, doc in scored[:limit]]
