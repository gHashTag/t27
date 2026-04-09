#!/usr/bin/env python3.10
"""NotebookLM enrichment module.

Adds contextual sources (YouTube videos, podcasts, docs) to notebooks
based on issue topics and labels.

phi^2 + 1/phi^2 = 3 | TRINITY
"""

import argparse
import asyncio
import json
import logging
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional, Any

import yaml

# Import notebooklm-py
try:
    from notebooklm import NotebookLMClient, Source, Notebook
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

# Path to content registry
REGISTRY_PATH = Path(__file__).parent / "content_registry.yaml"
# Path to enrichment metadata
METADATA_PATH = Path(__file__).parent / "enrichment_metadata.json"


@dataclass
class EnrichmentResult:
    """Result of enriching a single notebook."""
    notebook_id: str
    notebook_title: str
    topic: Optional[str] = None
    sources_added: int = 0
    sources_skipped: int = 0
    errors: list[str] = field(default_factory=list)
    success: bool = True


@dataclass
class Topic:
    """Content topic with associated sources."""
    name: str
    description: str
    youtube: list[str] = field(default_factory=list)
    podcasts: list[str] = field(default_factory=list)
    docs: list[str] = field(default_factory=list)

    @property
    def all_sources(self) -> list[str]:
        """Get all sources for this topic."""
        return self.youtube + self.podcasts + self.docs


class ContentRegistry:
    """Loads and provides access to topic → content mappings."""

    def __init__(self, path: Path = REGISTRY_PATH):
        self.path = path
        self.topics: dict[str, Topic] = {}
        self.label_mappings: list[dict[str, str]] = []
        self.keyword_mappings: list[dict[str, Any]] = []
        self._load()

    def _load(self) -> None:
        """Load registry from YAML file."""
        if not self.path.exists():
            logger.warning(f"Registry file not found: {self.path}")
            return

        try:
            with open(self.path, "r") as f:
                data = yaml.safe_load(f)

            # Load topics
            for topic_id, topic_data in data.get("topics", {}).items():
                self.topics[topic_id] = Topic(
                    name=topic_data.get("name", topic_id),
                    description=topic_data.get("description", ""),
                    youtube=topic_data.get("youtube", []),
                    podcasts=topic_data.get("podcasts", []),
                    docs=topic_data.get("docs", []),
                )

            # Load label mappings
            self.label_mappings = data.get("label_mappings", [])

            # Load keyword mappings
            self.keyword_mappings = data.get("keyword_mappings", [])

            logger.info(f"Loaded {len(self.topics)} topics from registry")
        except Exception as e:
            logger.error(f"Failed to load registry: {e}")

    def find_topic_by_labels(self, labels: list[str]) -> Optional[str]:
        """Find topic based on issue labels."""
        for label in labels:
            label_lower = label.lower()
            for mapping in self.label_mappings:
                if label_lower.startswith(mapping["label"].lower()):
                    topic = mapping["topic"]
                    logger.debug(f"Matched label '{label}' -> topic '{topic}'")
                    return topic
        return None

    def find_topic_by_keywords(self, text: str) -> Optional[str]:
        """Find topic based on keywords in text."""
        text_lower = text.lower()
        for mapping in self.keyword_mappings:
            for keyword in mapping["keywords"]:
                if keyword.lower() in text_lower:
                    topic = mapping["topic"]
                    logger.debug(f"Matched keyword '{keyword}' -> topic '{topic}'")
                    return topic
        return None

    def get_topic(self, topic_id: str) -> Optional[Topic]:
        """Get topic by ID."""
        return self.topics.get(topic_id)

    def get_all_topic_ids(self) -> list[str]:
        """Get all available topic IDs."""
        return list(self.topics.keys())


class EnrichmentMetadata:
    """Tracks enrichment metadata for notebooks."""

    def __init__(self, path: Path = METADATA_PATH):
        self.path = path
        self.data: dict[str, dict[str, Any]] = {}
        self._load()

    def _load(self) -> None:
        """Load metadata from JSON file."""
        if self.path.exists():
            try:
                with open(self.path, "r") as f:
                    self.data = json.load(f)
            except Exception as e:
                logger.error(f"Failed to load metadata: {e}")

    def _save(self) -> None:
        """Save metadata to JSON file."""
        try:
            with open(self.path, "w") as f:
                json.dump(self.data, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save metadata: {e}")

    def get_notebook_metadata(self, notebook_id: str) -> Optional[dict[str, Any]]:
        """Get metadata for a specific notebook."""
        return self.data.get(notebook_id)

    def set_notebook_metadata(self, notebook_id: str, metadata: dict[str, Any]) -> None:
        """Set metadata for a specific notebook."""
        self.data[notebook_id] = metadata
        self._save()

    def get_enriched_sources(self, notebook_id: str) -> set[str]:
        """Get set of enriched source URLs for a notebook."""
        meta = self.get_notebook_metadata(notebook_id)
        if meta:
            return set(meta.get("enriched_sources", []))
        return set()

    def add_enriched_source(self, notebook_id: str, url: str) -> None:
        """Mark a source as enriched for a notebook."""
        meta = self.get_notebook_metadata(notebook_id) or {
            "enriched_at": datetime.utcnow().isoformat(),
            "topic": None,
            "enriched_sources": [],
        }
        if "enriched_sources" not in meta:
            meta["enriched_sources"] = []
        if url not in meta["enriched_sources"]:
            meta["enriched_sources"].append(url)
        self.set_notebook_metadata(notebook_id, meta)


class NotebookEnricher:
    """Enriches notebooks with contextual content."""

    def __init__(self, client: NotebookLMClient):
        self.client = client
        self.registry = ContentRegistry()
        self.metadata = EnrichmentMetadata()

    async def get_existing_sources(self, notebook_id: str) -> dict[str, Source]:
        """Get all existing sources for a notebook."""
        try:
            sources = await self.client.sources.list(notebook_id)
            return {s.title or s.url or "": s for s in sources}
        except Exception as e:
            logger.error(f"Failed to list sources: {e}")
            return {}

    async def add_source(self, notebook_id: str, url: str) -> bool:
        """Add a source to a notebook."""
        try:
            source = await self.client.sources.add_url(notebook_id, url)
            logger.info(f"  Added source: {source.title or url}")
            return True
        except Exception as e:
            logger.warning(f"  Failed to add {url}: {e}")
            return False

    async def enrich_notebook(
        self,
        notebook: Notebook,
        labels: list[str] = None,
        force: bool = False
    ) -> EnrichmentResult:
        """Enrich a single notebook with contextual sources.

        Args:
            notebook: The notebook to enrich.
            labels: Issue labels for topic matching.
            force: Re-add sources even if already enriched.
        """
        result = EnrichmentResult(
            notebook_id=notebook.id,
            notebook_title=notebook.title or "Untitled",
        )

        # Find matching topic
        topic_id = None

        # Try labels first
        if labels:
            topic_id = self.registry.find_topic_by_labels(labels)

        # Fall back to title keywords
        if not topic_id:
            topic_id = self.registry.find_topic_by_keywords(notebook.title or "")

        if not topic_id:
            logger.info(f"No topic match for '{notebook.title}' - skipping")
            result.success = False
            return result

        topic = self.registry.get_topic(topic_id)
        if not topic:
            logger.warning(f"Topic '{topic_id}' not found in registry")
            result.success = False
            return result

        result.topic = topic_id
        logger.info(f"Enriching '{notebook.title}' with topic: {topic.name}")

        # Get existing sources
        existing_sources = await self.get_existing_sources(notebook.id)
        existing_urls = {s.url for s in existing_sources.values() if s.url}

        # Get already enriched sources from metadata
        enriched_urls = self.metadata.get_enriched_sources(notebook.id)

        # Add sources
        for url in topic.all_sources:
            # Skip if already added or previously enriched
            if url in existing_urls:
                logger.debug(f"  Skipping (already exists): {url}")
                result.sources_skipped += 1
                continue

            if not force and url in enriched_urls:
                logger.debug(f"  Skipping (previously enriched): {url}")
                result.sources_skipped += 1
                continue

            if await self.add_source(notebook.id, url):
                result.sources_added += 1
                self.metadata.add_enriched_source(notebook.id, url)
                # Small delay to avoid rate limiting
                await asyncio.sleep(0.5)
            else:
                result.errors.append(f"Failed to add: {url}")

        # Update metadata
        meta = self.metadata.get_notebook_metadata(notebook.id) or {}
        meta.update({
            "topic": topic_id,
            "topic_name": topic.name,
            "notebook_title": notebook.title,
            "last_enriched": datetime.utcnow().isoformat(),
        })
        self.metadata.set_notebook_metadata(notebook.id, meta)

        result.success = result.sources_added > 0 or len(result.errors) == 0
        return result

    async def find_notebook_by_issue_number(self, issue_number: int) -> Optional[Notebook]:
        """Find a notebook by issue number in title."""
        notebooks = await self.client.notebooks.list()
        pattern = re.compile(rf"#?{issue_number}\b", re.IGNORECASE)

        for nb in notebooks:
            if pattern.search(nb.title or ""):
                return nb
        return None

    async def enrich_all(
        self,
        issue_data: dict[int, dict[str, Any]] = None,
        force: bool = False
    ) -> dict[str, Any]:
        """Enrich all notebooks.

        Args:
            issue_data: Mapping of issue_number -> {title, labels}
            force: Re-add sources even if already enriched.
        """
        results = []

        notebooks = await self.client.notebooks.list()
        logger.info(f"Found {len(notebooks)} notebooks")

        for nb in notebooks:
            # Try to extract issue number from title
            labels = []
            if issue_data:
                for issue_num, data in issue_data.items():
                    if re.search(rf"#?{issue_num}\b", nb.title or ""):
                        labels = data.get("labels", [])
                        break

            result = await self.enrich_notebook(nb, labels, force)
            results.append(result)

        # Summary
        total = len(results)
        successful = sum(1 for r in results if r.success)
        total_added = sum(r.sources_added for r in results)
        total_skipped = sum(r.sources_skipped for r in results)
        total_errors = sum(len(r.errors) for r in results)

        summary = {
            "total_notebooks": total,
            "successful": successful,
            "failed": total - successful,
            "sources_added": total_added,
            "sources_skipped": total_skipped,
            "total_errors": total_errors,
            "results": [
                {
                    "notebook_id": r.notebook_id,
                    "notebook_title": r.notebook_title,
                    "topic": r.topic,
                    "sources_added": r.sources_added,
                    "sources_skipped": r.sources_skipped,
                    "success": r.success,
                    "errors": r.errors,
                }
                for r in results
            ],
        }

        return summary


async def main():
    parser = argparse.ArgumentParser(
        description="Enrich NotebookLM notebooks with contextual content"
    )
    parser.add_argument(
        "--issue",
        type=int,
        help="Enrich a specific notebook by issue number"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Enrich all notebooks"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Re-add sources even if previously enriched"
    )
    parser.add_argument(
        "--list-topics",
        action="store_true",
        help="List available topics and exit"
    )
    parser.add_argument(
        "--export-dashboard",
        type=Path,
        help="Export dashboard data to JSON file"
    )

    args = parser.parse_args()

    # List topics mode
    if args.list_topics:
        registry = ContentRegistry()
        print("\nAvailable Topics:")
        print("=" * 60)
        for topic_id, topic in registry.topics.items():
            print(f"\n{topic_id}: {topic.name}")
            print(f"  {topic.description}")
            print(f"  Sources: {len(topic.all_sources)} ({len(topic.youtube)} YT, {len(topic.podcasts)} podcasts, {len(topic.docs)} docs)")
        print()
        return

    # Need at least --issue or --all
    if not args.issue and not args.all:
        parser.print_help()
        sys.exit(1)

    async with await NotebookLMClient.from_storage() as client:
        enricher = NotebookEnricher(client)

        if args.issue:
            notebook = await enricher.find_notebook_by_issue_number(args.issue)
            if not notebook:
                print(f"Error: Could not find notebook for issue #{args.issue}")
                sys.exit(1)

            result = await enricher.enrich_notebook(notebook, force=args.force)

            print(f"\n{'='*60}")
            print(f"Enriched: {result.notebook_title}")
            print(f"Topic: {result.topic or 'None'}")
            print(f"Sources added: {result.sources_added}")
            print(f"Sources skipped: {result.sources_skipped}")
            if result.errors:
                print(f"Errors: {len(result.errors)}")
                for err in result.errors:
                    print(f"  - {err}")
            print(f"{'='*60}\n")

        elif args.all:
            summary = await enricher.enrich_all(force=args.force)

            print(f"\n{'='*60}")
            print(f"Enrichment Summary")
            print(f"{'='*60}")
            print(f"Total notebooks: {summary['total_notebooks']}")
            print(f"Successful: {summary['successful']}")
            print(f"Failed: {summary['failed']}")
            print(f"Sources added: {summary['sources_added']}")
            print(f"Sources skipped: {summary['sources_skipped']}")
            print(f"Total errors: {summary['total_errors']}")
            print(f"{'='*60}\n")

            # Export dashboard data if requested
            if args.export_dashboard:
                with open(args.export_dashboard, "w") as f:
                    json.dump(summary, f, indent=2)
                print(f"Dashboard data exported to: {args.export_dashboard}")


if __name__ == "__main__":
    asyncio.run(main())
