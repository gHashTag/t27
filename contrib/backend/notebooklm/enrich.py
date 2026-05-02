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
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional, Any
from urllib.parse import urlparse, parse_qs

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
# Transcript extraction constants
MAX_TRANSCRIPT_SIZE = 10 * 1024 * 1024  # 10MB limit
YOUTUBE_DOMAINS = {"youtube.com", "www.youtube.com", "m.youtube.com", "music.youtube.com", "youtu.be"}


@dataclass
class EnrichmentResult:
    """Result of enriching a single notebook."""
    notebook_id: str
    notebook_title: str
    topic: Optional[str] = None
    sources_added: int = 0
    sources_skipped: int = 0
    transcripts_added: int = 0
    transcripts_failed: int = 0
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
            "enriched_transcripts": [],
        }
        if "enriched_sources" not in meta:
            meta["enriched_sources"] = []
        if url not in meta["enriched_sources"]:
            meta["enriched_sources"].append(url)
        self.set_notebook_metadata(notebook_id, meta)

    def get_enriched_transcripts(self, notebook_id: str) -> set[str]:
        """Get set of enriched transcript video URLs for a notebook."""
        meta = self.get_notebook_metadata(notebook_id)
        if meta:
            return set(meta.get("enriched_transcripts", []))
        return set()

    def add_enriched_transcript(self, notebook_id: str, video_url: str) -> None:
        """Mark a video transcript as enriched for a notebook."""
        meta = self.get_notebook_metadata(notebook_id) or {
            "enriched_at": datetime.utcnow().isoformat(),
            "topic": None,
            "enriched_sources": [],
            "enriched_transcripts": [],
        }
        if "enriched_transcripts" not in meta:
            meta["enriched_transcripts"] = []
        if video_url not in meta["enriched_transcripts"]:
            meta["enriched_transcripts"].append(video_url)
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

    def _is_youtube_url(self, url: str) -> bool:
        """Check if URL is a YouTube URL."""
        parsed = urlparse(url.strip())
        hostname = (parsed.hostname or "").lower()
        return hostname in YOUTUBE_DOMAINS

    def _extract_youtube_video_id(self, url: str) -> Optional[str]:
        """Extract YouTube video ID from URL (mirrors SDK pattern)."""
        try:
            parsed = urlparse(url.strip())
            hostname = (parsed.hostname or "").lower()

            # youtu.be short URLs
            if hostname == "youtu.be":
                return parsed.path.lstrip("/").split("/")[0]

            # youtube.com path-based formats
            path_segments = parsed.path.lstrip("/").split("/")
            if len(path_segments) >= 2 and path_segments[0] in ("shorts", "embed", "live", "v"):
                return path_segments[1]

            # Query param ?v=VIDEO_ID
            if parsed.query:
                query_params = parse_qs(parsed.query)
                if "v" in query_params:
                    return query_params["v"][0]
        except Exception:
            pass
        return None

    def _srt_to_text(self, srt_content: str) -> str:
        """Convert SRT subtitle format to plain text."""
        lines = []
        in_text_block = False

        for line in srt_content.split("\n"):
            line = line.strip()

            # Skip timestamps (contain -->)
            if "-->" in line or re.match(r"^\d+$", line):
                in_text_block = False
                continue

            # Skip empty lines and line numbers
            if not line or (not in_text_block and line.isdigit()):
                if not line.isdigit():
                    in_text_block = True
                continue

            lines.append(line)

        return "\n".join(lines)

    async def _extract_transcript(self, video_url: str) -> tuple[Optional[str], Optional[str], Optional[str]]:
        """Extract YouTube transcript using yt-dlp.

        Returns:
            (video_title, transcript_content, error_message)
        """
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_dir_path = Path(temp_dir)

            # Check if yt-dlp is available
            try:
                subprocess.run(
                    ["yt-dlp", "--version"],
                    capture_output=True,
                    check=True,
                    timeout=10
                )
            except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
                return None, None, "yt-dlp not found or not working. Install with: brew install yt-dlp"

            # yt-dlp command to download subtitles without video
            cmd = [
                "yt-dlp",
                "--no-update",  # Suppress update warning
                "--skip-download",
                "--write-subs",
                "--write-auto-subs",
                "--sub-langs", "en",
                "--sub-format", "srt",
                "--output", str(temp_dir_path / "%(title)s.%(ext)s"),
                video_url
            ]

            try:
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
            except subprocess.TimeoutExpired:
                return None, None, f"Transcript extraction timed out for {video_url}"

            if result.returncode != 0:
                stderr = result.stderr.lower()
                if "video unavailable" in stderr or "video has been removed" in stderr:
                    return None, None, f"Video unavailable: {video_url}"
                elif "video has no subtitles" in stderr or "subtitles" in stderr:
                    return None, None, f"No subtitles available: {video_url}"
                else:
                    return None, None, f"yt-dlp error: {result.stderr.strip()}"

            # Find subtitle file
            subtitle_files = list(temp_dir_path.glob("*.srt"))
            if not subtitle_files:
                return None, None, f"No subtitle files generated: {video_url}"

            subtitle_path = subtitle_files[0]

            try:
                with open(subtitle_path, "r", encoding="utf-8") as f:
                    content = f.read()

                transcript = self._srt_to_text(content)

                if not transcript.strip():
                    return None, None, f"Empty transcript: {video_url}"

                # Check size limit
                if len(transcript) > MAX_TRANSCRIPT_SIZE:
                    return None, None, f"Transcript too large ({len(transcript)} bytes): {video_url}"

                title = subtitle_path.stem
                return title, transcript, None
            except UnicodeDecodeError:
                return None, None, f"Failed to decode subtitle (encoding issue): {video_url}"
            except Exception as e:
                return None, None, f"Failed to read subtitle: {e}"

    async def add_source(self, notebook_id: str, url: str) -> bool:
        """Add a source to a notebook with YouTube transcript fallback."""
        # Try direct URL upload first
        try:
            source = await self.client.sources.add_url(notebook_id, url)
            logger.info(f"  Added source: {source.title or url}")
            return True
        except Exception as e:
            error_msg = str(e).lower()

            # Check if YouTube URL upload failed
            if self._is_youtube_url(url) and any(
                pattern in error_msg
                for pattern in ["youtube", "blocked", "not allowed", "video"]
            ):
                logger.info(f"  YouTube URL blocked, trying transcript extraction...")
                title, transcript, error = await self._extract_transcript(url)

                if error:
                    logger.warning(f"  Transcript extraction failed: {error}")
                    return False

                # Upload transcript via add_text
                try:
                    formatted_title = f"🎥 {title} (transcript)"
                    await self.client.sources.add_text(notebook_id, formatted_title, transcript)
                    logger.info(f"  Added transcript: {formatted_title}")
                    return True
                except Exception as te:
                    logger.warning(f"  Failed to add transcript: {te}")
                    return False

            # Non-YouTube error
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

        # Get already enriched sources AND transcripts from metadata
        enriched_urls = self.metadata.get_enriched_sources(notebook.id)
        enriched_transcripts = self.metadata.get_enriched_transcripts(notebook.id)

        # Add sources
        for url in topic.all_sources:
            # Skip if already added
            if url in existing_urls:
                logger.debug(f"  Skipping (already exists): {url}")
                result.sources_skipped += 1
                continue

            # Special handling for YouTube URLs
            if self._is_youtube_url(url):
                # Skip if transcript already added
                if not force and url in enriched_transcripts:
                    logger.debug(f"  Skipping (transcript already added): {url}")
                    result.sources_skipped += 1
                    continue

                # Try add_source which handles fallback
                success = await self.add_source(notebook.id, url)
                if success:
                    # Track in transcripts metadata
                    result.transcripts_added += 1
                    self.metadata.add_enriched_transcript(notebook.id, url)
                    await asyncio.sleep(0.5)
                else:
                    result.transcripts_failed += 1
                    result.errors.append(f"Failed to add transcript: {url}")
            else:
                # Non-YouTube sources
                # Skip if previously enriched
                if not force and url in enriched_urls:
                    logger.debug(f"  Skipping (previously enriched): {url}")
                    result.sources_skipped += 1
                    continue

                if await self.add_source(notebook.id, url):
                    result.sources_added += 1
                    self.metadata.add_enriched_source(notebook.id, url)
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
        total_transcripts_added = sum(r.transcripts_added for r in results)
        total_transcripts_failed = sum(r.transcripts_failed for r in results)
        total_errors = sum(len(r.errors) for r in results)

        summary = {
            "total_notebooks": total,
            "successful": successful,
            "failed": total - successful,
            "sources_added": total_added,
            "sources_skipped": total_skipped,
            "transcripts_added": total_transcripts_added,
            "transcripts_failed": total_transcripts_failed,
            "total_errors": total_errors,
            "results": [
                {
                    "notebook_id": r.notebook_id,
                    "notebook_title": r.notebook_title,
                    "topic": r.topic,
                    "sources_added": r.sources_added,
                    "sources_skipped": r.sources_skipped,
                    "transcripts_added": r.transcripts_added,
                    "transcripts_failed": r.transcripts_failed,
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
            if result.transcripts_added > 0:
                print(f"Transcripts added: {result.transcripts_added}")
            if result.transcripts_failed > 0:
                print(f"Transcripts failed: {result.transcripts_failed}")
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
            if summary.get('transcripts_added', 0) > 0:
                print(f"Transcripts added: {summary['transcripts_added']}")
            if summary.get('transcripts_failed', 0) > 0:
                print(f"Transcripts failed: {summary['transcripts_failed']}")
            print(f"Total errors: {summary['total_errors']}")
            print(f"{'='*60}\n")

            # Export dashboard data if requested
            if args.export_dashboard:
                with open(args.export_dashboard, "w") as f:
                    json.dump(summary, f, indent=2)
                print(f"Dashboard data exported to: {args.export_dashboard}")


if __name__ == "__main__":
    asyncio.run(main())
