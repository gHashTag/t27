#!/usr/bin/env python3.10
"""NotebookLM sync module — Continuous synchronization point.

This module provides unified synchronization between t27 repo and NotebookLM notebooks.
Called by: pre-commit hooks, GitHub Actions, or manual CLI.

phi^2 + 1/phi^2 = 3 | TRINITY
"""

import argparse
import asyncio
import json
import logging
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional, Any

import yaml

try:
    from notebooklm import NotebookLMClient, Source
except ImportError:
    print("Error: notebooklm-py not installed. Run: pip install notebooklm-py")
    sys.exit(1)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S"
)
logger = logging.getLogger(__name__)

# Paths
REPO_ROOT = Path(__file__).parent.parent.parent.parent.parent
NOTEBOOKLM_DIR = Path(__file__).parent
METADATA_PATH = NOTEBOOKLM_DIR / "enrichment_metadata.json"
SYNC_STATE_PATH = NOTEBOOKLM_DIR / "sync_state.json"
ACTIVITY_MD_PATH = REPO_ROOT / "activity.md"


@dataclass
class SyncEvent:
    """A sync event to be recorded."""
    issue_number: int
    event_type: str  # push, comment, pr, merge, spec_change
    timestamp: str
    details: dict[str, Any] = field(default_factory=dict)
    source_added: bool = False
    notebook_id: Optional[str] = None


@dataclass
class SyncState:
    """Persistent sync state tracking last sync timestamps."""
    last_issue_sync: dict[int, str] = field(default_factory=dict)
    last_repo_sync: str = ""


class NotebookSyncer:
    """Main sync orchestrator for NotebookLM."""

    def __init__(self):
        self.state = self._load_sync_state()
        self.registry = ContentRegistry()

    def _load_sync_state(self) -> SyncState:
        """Load persistent sync state."""
        if SYNC_STATE_PATH.exists():
            try:
                with open(SYNC_STATE_PATH, "r") as f:
                    data = json.load(f)
                    return SyncState(
                        last_issue_sync=data.get("last_issue_sync", {}),
                        last_repo_sync=data.get("last_repo_sync", ""),
                    )
            except Exception as e:
                logger.warning(f"Failed to load sync state: {e}")
        return SyncState()

    def _save_sync_state(self) -> None:
        """Save persistent sync state."""
        try:
            with open(SYNC_STATE_PATH, "w") as f:
                json.dump({
                    "last_issue_sync": self.state.last_issue_sync,
                    "last_repo_sync": self.state.last_repo_sync,
                }, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save sync state: {e}")

    def extract_issue_number(self, branch_name: str = None, commit_msg: str = None) -> Optional[int]:
        """Extract issue number from branch name or commit message."""
        # Try branch name first: feature/issue-357-something -> 357
        if branch_name:
            match = re.search(r"(?:issue-|#|fix|feature)[-_]?(\d+)", branch_name, re.IGNORECASE)
            if match:
                return int(match.group(1))

        # Try commit message: "Closes #357" or "#357: fix something"
        if commit_msg:
            match = re.search(r"(?:Closes|Fixes|Resolves)?\s*#(\d+)", commit_msg, re.IGNORECASE)
            if match:
                return int(match.group(1))
            match = re.search(r"#(\d+):", commit_msg)
            if match:
                return int(match.group(1))

        return None

    async def get_notebook_for_issue(self, issue_number: int) -> Optional[str]:
        """Find notebook ID for a given issue number."""
        if not METADATA_PATH.exists():
            return None

        try:
            with open(METADATA_PATH, "r") as f:
                data = json.load(f)
        except Exception:
            return None

        # Search by notebook title containing issue number
        for nb_id, meta in data.items():
            title = meta.get("notebook_title", "")
            if re.search(rf"#?{issue_number}\b", title, re.IGNORECASE):
                return nb_id

        return None

    async def create_notebook_for_issue(
        self, issue_number: int, title: str, labels: list[str] = None
    ) -> str:
        """Create a new notebook for an issue."""
        async with await NotebookLMClient.from_storage() as client:
            notebook_title = f"#{issue_number}: {title}"
            notebook_description = (
                f"GitHub Issue #{issue_number}\n"
                f"Labels: {', '.join(labels) if labels else 'None'}\n"
                f"Auto-synced: {datetime.utcnow().isoformat()}"
            )

            notebook = await client.notebooks.create(
                title=notebook_title,
                description=notebook_description,
            )

            logger.info(f"Created notebook: {notebook.id} for issue #{issue_number}")
            return notebook.id

    async def add_source(self, notebook_id: str, title: str, content: str) -> bool:
        """Add or update a text source to a notebook."""
        try:
            async with await NotebookLMClient.from_storage() as client:
                # Check if source with same title exists
                sources = await client.sources.list(notebook_id)
                existing = None
                for s in sources:
                    if s.title == title:
                        existing = s
                        break

                if existing:
                    # Update existing source
                    logger.info(f"Updating source: {title}")
                    # Delete and recreate (notebooklm-py doesn't have update_text)
                    await client.sources.delete(notebook_id, existing.id)
                    await client.sources.add_text(notebook_id, title, content)
                else:
                    # Add new source
                    logger.info(f"Adding source: {title}")
                    await client.sources.add_text(notebook_id, title, content)

                return True
        except Exception as e:
            logger.error(f"Failed to add source {title}: {e}")
            return False

    async def sync_activity_md(self) -> None:
        """Sync activity.md to all notebooks."""
        if not ACTIVITY_MD_PATH.exists():
            logger.warning("activity.md not found")
            return

        # For now, add to all enriched notebooks
        # In production, would be smarter about which notebook
        with open(ACTIVITY_MD_PATH, "r") as f:
            content = f.read()

        async with await NotebookLMClient.from_storage() as client:
            notebooks = await client.notebooks.list()

            # Add to first enriched notebook
            for nb in notebooks[:1]:
                title = "activity.md"
                source_name = f"activity_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.md"

                try:
                    await client.sources.add_text(nb.id, source_name, content)
                    logger.info(f"Added activity.md to notebook: {nb.id}")
                    break
                except Exception as e:
                    logger.warning(f"Failed to add activity.md: {e}")

    async def sync_commits(self, issue_number: int) -> None:
        """Sync recent commits for an issue."""
        # Get commits since last sync
        last_sync = self.state.last_issue_sync.get(issue_number)

        # Get recent commits (simplified - in production use git log with proper dates)
        try:
            result = subprocess.run(
                ["git", "log", "--all", f"--since={last_sync}", "--pretty=format:%s"],
                capture_output=True,
                text=True,
                cwd=REPO_ROOT,
            )

            commits = result.stdout.strip().split("\n") if result.stdout.strip() else []

            if commits:
                notebook_id = await self.get_notebook_for_issue(issue_number)
                if not notebook_id:
                    logger.info(f"No notebook found for issue #{issue_number}")
                    return

                commit_content = "\n\n".join(
                    [f"- {c[:80]}..." if len(c) > 80 else c for c in commits]
                )
                await self.add_source(
                    notebook_id,
                    f"Commits since {last_sync or 'beginning'}",
                    commit_content,
                )

                self.state.last_issue_sync[issue_number] = datetime.utcnow().isoformat()
                self._save_sync_state()

        except Exception as e:
            logger.error(f"Failed to sync commits: {e}")

    async def sync_spec_change(self) -> None:
        """Sync changed .t27 spec files."""
        try:
            # Get changed .t27 files
            result = subprocess.run(
                ["git", "diff", "--name-only", "HEAD~1", "HEAD"],
                capture_output=True,
                text=True,
                cwd=REPO_ROOT,
            )

            changed_files = [f for f in result.stdout.strip().split("\n") if f.endswith(".t27")]

            if not changed_files:
                return

            spec_content = ""
            for spec_file in changed_files:
                spec_path = REPO_ROOT / spec_file
                if spec_path.exists():
                    with open(spec_path, "r") as f:
                        spec_content += f"\n\n## {spec_file}\n\n"
                        spec_content += f.read()[:1000]  # Limit content size
                        if len(f.read()) > 1000:
                            spec_content += "\n... (truncated)"

            if spec_content:
                # Add to first notebook
                async with await NotebookLMClient.from_storage() as client:
                    notebooks = await client.notebooks.list()
                    for nb in notebooks[:1]:
                        await self.add_source(nb.id, "Spec Changes", spec_content)
                        break

        except Exception as e:
            logger.error(f"Failed to sync spec changes: {e}")

    async def handle_push_event(self, branch: str = None) -> None:
        """Handle push event from git."""
        # Get commit info
        try:
            result = subprocess.run(
                ["git", "log", "-1", "--pretty=format:%s|%b"],
                capture_output=True,
                text=True,
                cwd=REPO_ROOT,
            )

            if not result.stdout:
                return

            commit_msg, branch_name = result.stdout.strip().split("|")
            branch_name = branch or branch or "main"

            issue_number = self.extract_issue_number(branch_name, commit_msg)

            if issue_number:
                logger.info(f"Push for issue #{issue_number}")
                await self.sync_commits(issue_number)
            else:
                # General push - sync activity.md every 3rd push
                self.state.last_repo_sync = datetime.utcnow().isoformat()
                self._save_sync_state()
                # await self.sync_activity_md()

        except Exception as e:
            logger.error(f"Failed to handle push: {e}")

    async def handle_issue_comment(self, issue_number: int) -> None:
        """Handle new comment on an issue."""
        notebook_id = await self.get_notebook_for_issue(issue_number)

        if not notebook_id:
            logger.info(f"No notebook for issue #{issue_number}")
            return

        # In production, would fetch actual comment
        comment = f"New comment added on issue #{issue_number}\nSynced: {datetime.utcnow().isoformat()}"
        await self.add_source(notebook_id, "Comment", comment)

        self.state.last_issue_sync[issue_number] = datetime.utcnow().isoformat()
        self._save_sync_state()

    async def handle_pr_event(self, pr_number: int) -> None:
        """Handle PR event (opened, updated, merged)."""
        # In production, would fetch PR diff and comments
        notebook_id = await self.get_notebook_for_issue(pr_number)

        if notebook_id:
            diff = f"PR #{pr_number} diff and discussion\nSynced: {datetime.utcnow().isoformat()}"
            await self.add_source(notebook_id, f"PR #{pr_number}", diff)

    async def handle_merge_event(self, branch: str) -> None:
        """Handle merge event."""
        issue_number = self.extract_issue_number(branch)
        if not issue_number:
            logger.info(f"No issue number in branch: {branch}")
            return

        notebook_id = await self.get_notebook_for_issue(issue_number)
        if notebook_id:
            summary = f"Branch merged: {branch}\nIssue #{issue_number} completed\n{datetime.utcnow().isoformat()}"
            await self.add_source(notebook_id, "Merge Summary", summary)

        # Update dashboard metadata timestamp
        if METADATA_PATH.exists():
            with open(METADATA_PATH, "r") as f:
                data = json.load(f)
            for nb_id, meta in data.items():
                if meta.get("notebook_title", "").find(str(issue_number)) != -1:
                    meta["last_merged"] = datetime.utcnow().isoformat()
            with open(METADATA_PATH, "w") as f:
                json.dump(data, f, indent=2)


class ContentRegistry:
    """Load topic mappings from YAML."""

    def __init__(self):
        self.path = NOTEBOOKLM_DIR / "content_registry.yaml"
        self.topics: dict[str, dict] = {}
        self._load()

    def _load(self):
        if not self.path.exists():
            return

        try:
            with open(self.path, "r") as f:
                data = yaml.safe_load(f)
            self.topics = data.get("topics", {})
        except Exception as e:
            logger.warning(f"Failed to load content registry: {e}")


async def main():
    parser = argparse.ArgumentParser(
        description="NotebookLM continuous sync — Keep notebooks in sync with repo changes"
    )
    parser.add_argument(
        "--issue",
        type=int,
        help="Sync specific issue"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Sync all enriched notebooks"
    )
    parser.add_argument(
        "--auto",
        action="store_true",
        help="Watch mode — continuous sync on file changes"
    )
    parser.add_argument(
        "--event",
        choices=["push", "comment", "pr", "merge", "spec_change"],
        help="Sync event type"
    )
    parser.add_argument(
        "--trigger",
        help="Event trigger value (from GitHub Actions)"
    )
    parser.add_argument(
        "--activity",
        action="store_true",
        help="Sync activity.md to notebooks"
    )

    args = parser.parse_args()

    syncer = NotebookSyncer()

    if args.auto:
        logger.info("Starting watch mode for continuous sync...")
        logger.info("Press Ctrl+C to stop")
        try:
            # Simple watch loop
            while True:
                await asyncio.sleep(10)
                # Check for changes via git status
                result = subprocess.run(
                    ["git", "status", "--short"],
                    capture_output=True,
                    text=True,
                    cwd=REPO_ROOT,
                )
                if result.stdout.strip():
                    logger.info("Detected changes, syncing...")
                    await syncer.handle_push_event()
        except KeyboardInterrupt:
            logger.info("Watch mode stopped")
        return

    if args.activity:
        await syncer.sync_activity_md()
        return

    if args.event == "push":
        await syncer.handle_push_event()
    elif args.event == "spec_change":
        await syncer.sync_spec_change()
    elif args.issue:
        issue_num = args.issue
        if args.trigger == "comment":
            await syncer.handle_issue_comment(issue_num)
        elif args.trigger == "merge":
            await syncer.handle_merge_event(f"issue-{issue_num}")
        else:
            await syncer.sync_commits(issue_num)
    elif args.all:
        logger.info("Syncing all enriched notebooks with activity.md...")
        await syncer.sync_activity_md()
    else:
        parser.print_help()


if __name__ == "__main__":
    asyncio.run(main())
