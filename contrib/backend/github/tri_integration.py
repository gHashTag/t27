# contrib/backend/github/tri_integration.py
# TriBridge: /tri skill ↔ GitHub ↔ NotebookLM
# phi^2 + 1/phi^2 = 3 | TRINITY

"""TriBridge: Connects /tri skill to GitHub and NotebookLM.

Provides:
- Issue operations (create, update, list, close)
- PR operations (create, merge, close, status)
- Documentation sync (upload to NotebookLM)
- Unified search across all entities
- Episode management for tracking work across systems
"""

from typing import List, Optional, Dict, Callable
from pathlib import Path
from datetime import datetime
import asyncio
import concurrent.futures

from .client import GitHubClient
from .issues import GitHubIssuesAPI
from .prs import GitHubPRsAPI
from .docs import GitHubDocsAPI
from .tri_integration_types import (
    TriBridgeConfig,
    SyncResult,
    UnifiedSearchResult,
    Episode,
    EpisodeType,
)

# Import NotebookLM functions (may fail if not available)
try:
    from contrib.backend.notebooklm import (
        source_upload_text,
        notebook_query,
    )
    NOTEBOOKLM_AVAILABLE = True
except ImportError:
    NOTEBOOKLM_AVAILABLE = False


class TriBridge:
    """Bridge between /tri skill and GitHub + NotebookLM.

    Enables unified autonomous work with:
    - GitHub Issues (tasks, priorities)
    - GitHub PRs (changes, review)
    - Documentation (papers, specs)
    - NotebookLM (semantic memory, RAG)
    """

    def __init__(
        self,
        github_client: GitHubClient,
        notebooklm_client: Optional[Callable] = None,
        repo_root: str = ".",
    ):
        """Initialize TriBridge.

        Args:
            github_client: GitHubClient instance
            notebooklm_client: Optional callable for NotebookLM operations
            repo_root: Path to t27 repository

        Complexity: O(1)
        """
        self.github = github_client.issues
        self.prs = GitHubPRsAPI(github_client)
        self.docs = GitHubDocsAPI(github_client, repo_root)
        self.notebooklm = notebooklm_client
        self.repo_root = Path(repo_root)

    def create_issue_from_notebook(
        self,
        notebooklm_id: str,
    ) -> Optional[int]:
        """Create GitHub issue from NotebookLM note.

        Args:
            notebooklm_id: NotebookLM source ID

        Returns:
            GitHub issue ID if created, None on error

        Complexity: O(1) query + O(1) gh CLI call
        """
        if not NOTEBOOKLM_AVAILABLE:
                print("NotebookLM not available - cannot create from note")
                return None

        # Query NotebookLM for note content
        from contrib.backend.notebooklm.queries import notebook_query
        result = notebook_query(notebooklm_id)

        if not result.get("answer"):
                print(f"Note {notebooklm_id} not found in NotebookLM")
                return None

        # Extract key information from Note
        answer = result["answer"]
        lines = [line.strip() for line in answer.split("\n") if line.strip()]

        # Extract title (first non-empty line)
        title = lines[0] if lines else "From NotebookLM Note"

        # Extract description (rest of content)
        description = "\n".join(lines[1:5]) if len(lines) > 1 else ""

        # Check if similar issue exists
        similar_issues = self.github.issue_find_similar(
                query=title,
                threshold=0.7
        )

        if similar_issues:
                # Update existing similar issue
                similar_issue = similar_issues[0]
                self.github.issue_update(
                        issue_id=similar_issue.id,
                        body=f"Context from NotebookLM ({notebooklm_id}):\n\n{description}",
                        state="in_progress",
                )
                print(f"Updated existing issue #{similar_issue.id} with NotebookLM context")
                return similar_issue.id
        else:
                # Create new issue
                issue = self.github.issue_create(
                        title=title,
                        body=f"From NotebookLM ({notebooklm_id}):\n\n{description}",
                        labels=["phi-loop", "notebooklm"],
                )
                print(f"Created new issue #{issue.id}")
                return issue.id

    def create_pr_from_notebook(
        self,
        notebooklm_id: str,
    ) -> Optional[int]:
        """Create GitHub PR from NotebookLM note.

        Args:
            notebooklm_id: NotebookLM source ID

        Returns:
            GitHub PR ID if created, None on error

        Complexity: O(1) query + O(1) gh CLI call
        """
        if not NOTEBOOKLM_AVAILABLE:
                print("NotebookLM not available - cannot create PR from note")
                return None

        from contrib.backend.notebooklm.queries import notebook_query
        result = notebook_query(notebooklm_id)

        if not result.get("answer"):
                print(f"Note {notebooklm_id} not found in NotebookLM")
                return None

        # Extract information
        answer = result["answer"]
        lines = [line.strip() for line in answer.split("\n") if line.strip()]

        title = lines[0] if lines else "From NotebookLM Note"
        description = "\n".join(lines[1:5]) if len(lines) > 1 else ""

        # Find related issues for PR body
        related_issues = self.github.issue_find_similar(
                query="PR",
                threshold=0.7,
        )

        # Build PR body with references
        body = f"From NotebookLM ({notebooklm_id}):\n\n{description}"

        if related_issues:
                body += "\n\nRelated issues:\n"
                for issue in related_issues[:3]:
                        body += f"- Issue #{issue.id}: {issue.title}\n"

        # Create PR (without issue reference for now)
        pr = self.prs.pr_create(
                title=title,
                body=body,
        )

        if pr:
                print(f"Created PR #{pr.id}")
                return pr.id
        else:
                print("Failed to create PR")
                return None

    def sync_github_to_notebooklm(
        self,
        issue_id: int,
    ) -> Optional[str]:
        """Sync GitHub issue to NotebookLM (upload note as source).

        Args:
            issue_id: GitHub issue number

        Returns:
            NotebookLM source ID if synced, None on error

        Complexity: O(1) issue get + O(1) upload
        """
        if not NOTEBOOKLM_AVAILABLE:
                print("NotebookLM not available - cannot sync")
                return None

        # Get GitHub issue details
        issue = self.github.issue_get(issue_id)

        if not issue:
                print(f"Issue #{issue_id} not found")
                return None

        # Upload to NotebookLM
        try:
                from contrib.backend.notebooklm.sources import source_upload_text

                content = f"""# GitHub Issue #{issue.id}

## Title
{issue.title}

## State
{issue.state}

## Created
{issue.created_at.strftime("%Y-%m-%d")}

## Labels
{", ".join(issue.labels)}

---

[Issue body content truncated for NotebookLM]
"""

                source_id = source_upload_text(
                        notebooklm_client=self.notebooklm,
                        content=content,
                        title=f"[GitHub Issue #{issue_id}] {issue.title}",
                )

                print(f"Uploaded issue #{issue_id} to NotebookLM: {source_id}")
                return source_id

        except Exception as e:
                print(f"Error uploading to NotebookLM: {e}")
                return None

    def sync_notebooklm_to_github(
        self,
        source_id: str,
    ) -> bool:
        """Sync NotebookLM source back to GitHub (add comment with link).

        Args:
            source_id: NotebookLM source ID

        Returns:
            True if synced, False on error

        Complexity: O(1) comment create
        """
        if not NOTEBOOKLM_AVAILABLE:
                print("NotebookLM not available - cannot sync")
                return False

        # Get source from NotebookLM
        from contrib.backend.notebooklm.queries import notebook_query
        result = notebook_query(source_id)

        if not result.get("answer"):
                print(f"Source {source_id} not found in NotebookLM")
                return False

        # Extract GitHub issue ID from NotebookLM source title
        answer = result["answer"]
        # Parse: "[GitHub Issue #123] Title" pattern
        import re

        match = re.search(r"\[GitHub Issue #(\d+)\]", answer)
        if not match:
                print(f"Cannot parse GitHub issue ID from NotebookLM source")
                return False

        issue_id = int(match.group(1))

        # Add comment to GitHub issue
        comment_body = f"Linked from NotebookLM source: {source_id}"

        self.github.comments.comment_create(
                issue_id=issue_id,
                body=comment_body,
        )

        print(f"Added comment to issue #{issue_id}")
        return True

    def full_sync(self, scope: str = "all") -> SyncResult:
        """Perform full sync across all entities.

        Args:
            scope: Sync scope - "all", "issues", "prs", "docs"

        Returns:
            SyncResult with statistics

        Complexity: O(n) where n = total entities

        Raises:
            RuntimeError: If critical errors occur
        """
        start_time = datetime.now()
        items_synced = 0
        errors = []

        # Sync based on scope
        tasks = []

        if scope in ("all", "issues"):
                # Sync issues to NotebookLM
                issues = self.github.issue_list(state="open")

                for issue in issues[:10]:  # Limit to first 10 for initial sync
                        tasks.append((
                                self.sync_github_to_notebooklm,
                                {"issue_id": issue.id},
                        ))

                if scope in ("all", "prs"):
                        # Sync PRs to NotebookLM
                        prs = self.prs.pr_list(state="open")

                        for pr in prs[:5]:
                                tasks.append((
                                        self.create_pr_from_notebook,
                                        {"notebooklm_id": f"pr-{pr.id}"},
                                ))

                if scope in ("all", "docs"):
                        # Sync docs to NotebookLM
                        docs = self.docs.doc_list()

                        for doc in docs[:5]:
                                try:
                                        from contrib.backend.notebooklm.sources import source_upload_text

                                        with open(self.repo_root / doc.path, "r") as f:
                                                content = f.read()

                                        # Upload to NotebookLM
                                        source_id = source_upload_text(
                                                notebooklm_client=self.notebooklm,
                                                content=content,
                                                title=f"[Doc] {doc.title}",
                                        )

                                        tasks.append((source_upload_text, {}))
                                        items_synced += 1

                                except Exception as e:
                                        errors.append(f"Failed to sync {doc.path}: {e}")

        # Execute tasks in parallel
        results = []
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
                futures = {
                        executor.submit(task[0], *task[1])
                        for task in tasks
                }

                for future in concurrent.futures.as_completed(futures.values()):
                        if future.exception():
                                errors.append(str(future.exception()))
                        else:
                                results.append(future.result())

        duration_ms = int((datetime.now() - start_time).total_seconds() * 1000)

        return SyncResult(
                success=len(errors) == 0,
                items_synced=items_synced,
                errors=errors,
                duration_ms=duration_ms,
        )

    def unified_search(self, query: str, limit: int = 10) -> UnifiedSearchResult:
        """Unified search across GitHub Issues, PRs, and NotebookLM.

        Args:
            query: Search query string
            limit: Maximum results per entity type

        Returns:
            UnifiedSearchResult with combined results

        Complexity: O(1) + O(n) where n = total entities searched
        """
        results = UnifiedSearchResult()

        # Search GitHub Issues
        if NOTEBOOKLM_AVAILABLE:
                from contrib.backend.notebooklm.queries import notebook_query

                # Search in NotebookLM first
                notebooklm_result = notebook_query(query)
                if notebooklm_result.get("answer"):
                        results.notebooklm_notes = [{
                                "type": "notebooklm",
                                "id": "query-result",
                                "title": query,
                                "content": notebooklm_result["answer"][:200],
                                "relevance": 1.0,
                        }]

        # Search GitHub Issues
        github_issues = self.github.issue_find_similar(query=query, threshold=0.7)
        results.github_issues = [
                        {
                                "type": "github",
                                "id": f"issue-{issue.id}",
                                "title": issue.title,
                                "content": issue.title[:200],
                                "relevance": issue.similarity if hasattr(issue, "similarity") else 0.8,
                        }
                        for issue in github_issues[:limit]
        ]

        # Search GitHub PRs
        github_prs = self.prs.pr_list(state="open")
        results.github_prs = [
                        {
                                "type": "github",
                                "id": f"pr-{pr.id}",
                                "title": pr.title,
                                "content": pr.title[:200],
                                "relevance": 0.7,
                        }
                        for pr in github_prs[:limit]
        ]

        # Search NotebookLM docs
        if NOTEBOOKLM_AVAILABLE:
                docs_results = notebook_query(f"{query} documentation")
                if docs_results.get("answer"):
                        results.notebooklm_notes = [
                                {
                                        "type": "notebooklm",
                                        "id": f"doc-{i}",
                                        "title": query,
                                        "content": docs_results["answer"][:200],
                                        "relevance": 0.8,
                                }
                                for i, content in enumerate(
                                        docs_results["answer"].split("\n\n")[2:limit]
                                )
                        ]

        # Combine and sort by relevance
        all_results = (
                results.notebooklm_notes or []
        ) + results.github_issues + results.github_prs + (results.notebooklm_notes or [])

        all_results.sort(key=lambda x: x["relevance"], reverse=True)

        results.combined_results = all_results[:limit]

        return results


def create_bridge(config: TriBridgeConfig) -> TriBridge:
    """Factory function to create TriBridge instance.

    Args:
        config: TriBridge configuration

    Returns:
        TriBridge instance

    Complexity: O(1)
        """
    from .client import GitHubClient

    github_client = GitHubClient.get_instance()

    return TriBridge(
                github_client=github_client,
                notebooklm_client=config.notebooklm_client,
                repo_root=config.repo_root,
        )
