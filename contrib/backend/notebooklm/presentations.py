#!/usr/bin/env python3.10
"""NotebookLM presentations module.

Generates AI presentations and audio overviews (podcast-style)
for enriched notebooks.

phi^2 + 1/phi^2 = 3 | TRINITY
"""

import argparse
import asyncio
import json
import logging
import re
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional, Any

try:
    from notebooklm import (
        NotebookLMClient,
        AudioFormat,
        AudioLength,
        SlideDeckFormat,
        SlideDeckLength,
        GenerationStatus,
        Artifact,
        ArtifactDownloadError,
        ArtifactNotReadyError,
    )
except ImportError:
    print("Error: notebooklm-py not installed. Run: pip install notebooklm-py")
    exit(1)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S"
)
logger = logging.getLogger(__name__)

# Paths
NOTEBOOKLM_DIR = Path(__file__).parent
METADATA_PATH = NOTEBOOKLM_DIR / "enrichment_metadata.json"
PRESENTATIONS_PATH = NOTEBOOKLM_DIR / "presentations.json"


@dataclass
class PresentationResult:
    """Result of generating presentation/audio for a notebook."""
    notebook_id: str
    notebook_title: str
    topic: Optional[str] = None
    presentation_task_id: Optional[str] = None
    presentation_download_url: Optional[str] = None
    audio_task_id: Optional[str] = None
    audio_download_url: Optional[str] = None
    success: bool = True
    errors: list[str] = field(default_factory=list)


class PresentationGenerator:
    """Generates presentations and audio overviews for notebooks."""

    def __init__(self, client: NotebookLMClient):
        self.client = client
        self.metadata = self._load_metadata()
        self.presentations = self._load_presentations()

    def _load_metadata(self) -> dict[str, Any]:
        """Load enrichment metadata."""
        if not METADATA_PATH.exists():
            return {}
        try:
            with open(METADATA_PATH, "r") as f:
                return json.load(f)
        except Exception as e:
            logger.error(f"Failed to load metadata: {e}")
            return {}

    def _load_presentations(self) -> dict[str, dict[str, Any]]:
        """Load existing presentations state."""
        if not PRESENTATIONS_PATH.exists():
            return {}
        try:
            with open(PRESENTATIONS_PATH, "r") as f:
                return json.load(f)
        except Exception as e:
            logger.warning(f"Failed to load presentations: {e}")
            return {}

    def _save_presentations(self) -> None:
        """Save presentations state."""
        try:
            with open(PRESENTATIONS_PATH, "w") as f:
                json.dump(self.presentations, f, indent=2)
        except Exception as e:
            logger.error(f"Failed to save presentations: {e}")

    def _get_enriched_notebooks(self) -> list[tuple[str, dict]]:
        """Get all enriched notebooks from metadata."""
        enriched = []
        for notebook_id, meta in self.metadata.items():
            if meta.get("enriched_sources") and len(meta["enriched_sources"]) > 0:
                enriched.append((notebook_id, meta))
        return enriched

    async def wait_for_artifact(
        self, notebook_id: str, task_id: str, artifact_type: str
    ) -> Optional[Artifact]:
        """Wait for artifact generation and return download info.

        Args:
            notebook_id: Notebook ID.
            task_id: Artifact generation task ID.
            artifact_type: Type of artifact ("presentation" or "audio").

        Returns:
            Artifact object if successful, None otherwise.
        """
        max_attempts = 60  # 5 minutes max wait
        attempt = 0

        while attempt < max_attempts:
            try:
                artifact = await self.client.artifacts.get(notebook_id, task_id)

                if artifact and artifact.status == "ready":
                    logger.info(f"{artifact_type} ready: {artifact.title}")
                    return artifact
                elif artifact and artifact.status == "failed":
                    logger.warning(f"{artifact_type} generation failed: {artifact.title}")
                    return None

                attempt += 1
                await asyncio.sleep(5)

            except ArtifactNotReadyError:
                # Still processing, continue waiting
                attempt += 1
                await asyncio.sleep(5)
            except Exception as e:
                logger.error(f"Error checking artifact status: {e}")
                return None

        logger.warning(f"{artifact_type} generation timeout for {task_id}")
        return None

    async def generate_for_notebook(
        self, notebook_id: str, meta: dict[str, Any]
    ) -> PresentationResult:
        """Generate presentation and audio for a notebook.

        Args:
            notebook_id: Notebook ID.
            meta: Notebook metadata.

        Returns:
            PresentationResult with task IDs and download URLs.
        """
        result = PresentationResult(
            notebook_id=notebook_id,
            notebook_title=meta.get("notebook_title", "Untitled"),
            topic=meta.get("topic"),
        )

        logger.info(f"Generating for: {result.notebook_title}")

        # Generate Audio Overview (Podcast style)
        try:
            audio_instructions = (
                "Create an engaging podcast-style audio overview. "
                "Use two hosts discussing the content in a conversational tone. "
                "Highlight key insights and technical details. "
                "The hosts should have distinct voices and personalities."
            )
            audio_status = await self.client.artifacts.generate_audio(
                notebook_id=notebook_id,
                instructions=audio_instructions,
                audio_length=AudioLength.DEFAULT,
                language="en",
            )

            result.audio_task_id = audio_status.task_id
            logger.info(f"  Audio task: {audio_status.task_id}")

            # Wait for audio to be ready
            audio_artifact = await self.wait_for_artifact(
                notebook_id, audio_status.task_id, "Audio"
            )

            if audio_artifact:
                result.audio_download_url = audio_artifact.download_url

        except Exception as e:
            logger.warning(f"  Audio generation failed: {e}")
            result.errors.append(f"Audio: {e}")
            result.success = False

        # Generate Slide Deck Presentation
        try:
            slide_instructions = (
                "Create a professional presentation summarizing the notebook content. "
                "Include: title slide, overview, key technical details, "
                "conclusions, and references. Use clear structure and "
                "supporting visuals in the description."
            )
            slide_status = await self.client.artifacts.generate_slide_deck(
                notebook_id=notebook_id,
                instructions=slide_instructions,
                slide_format=SlideDeckFormat.PRESENTER_SLIDES,
                slide_length=SlideDeckLength.DEFAULT,
                language="en",
            )

            result.presentation_task_id = slide_status.task_id
            logger.info(f"  Presentation task: {slide_status.task_id}")

            # Wait for presentation to be ready
            slide_artifact = await self.wait_for_artifact(
                notebook_id, slide_status.task_id, "Presentation"
            )

            if slide_artifact:
                result.presentation_download_url = slide_artifact.download_url

        except Exception as e:
            logger.warning(f"  Presentation generation failed: {e}")
            result.errors.append(f"Presentation: {e}")
            result.success = False

        # Update state
        if result.audio_task_id or result.presentation_task_id:
            self.presentations[notebook_id] = {
                "notebook_title": result.notebook_title,
                "topic": result.topic,
                "generated_at": datetime.utcnow().isoformat(),
                "audio_task_id": result.audio_task_id,
                "presentation_task_id": result.presentation_task_id,
                "audio_url": result.audio_download_url,
                "presentation_url": result.presentation_download_url,
                "success": result.success and (
                    result.audio_download_url is not None
                    or result.presentation_download_url is not None
                ),
            }
            self._save_presentations()

        return result

    async def generate_all(
        self, limit: int = 0
    ) -> dict[str, Any]:
        """Generate presentations and audio for all enriched notebooks.

        Args:
            limit: Maximum notebooks to process (0 = all).

        Returns:
            Summary dictionary.
        """
        enriched = self._get_enriched_notebooks()

        if limit > 0:
            enriched = enriched[:limit]

        logger.info(f"Processing {len(enriched)} enriched notebooks")

        results = []

        for i, (notebook_id, meta) in enumerate(enriched, 1):
            logger.info(f"[{i}/{len(enriched)}] Processing notebook...")

            # Skip if already processed
            if notebook_id in self.presentations:
                logger.info(f"  Skipping (already processed)")
                existing = self.presentations[notebook_id]
                results.append({
                    "notebook_id": notebook_id,
                    "notebook_title": meta.get("notebook_title"),
                    "topic": meta.get("topic"),
                    "skipped": True,
                    "reason": "already_processed",
                })
                continue

            result = await self.generate_for_notebook(notebook_id, meta)
            results.append({
                "notebook_id": result.notebook_id,
                "notebook_title": result.notebook_title,
                "topic": result.topic,
                "audio_task_id": result.audio_task_id,
                "presentation_task_id": result.presentation_task_id,
                "audio_url": result.audio_download_url,
                "presentation_url": result.presentation_download_url,
                "success": result.success,
                "errors": result.errors,
            })

            # Delay between generations to avoid rate limiting
            await asyncio.sleep(2)

        # Summary
        total = len(results)
        new_generated = sum(1 for r in results if not r.get("skipped", False))
        successful = sum(1 for r in results if r.get("success", False))
        failed = total - successful

        summary = {
            "total_processed": total,
            "skipped": total - new_generated,
            "new_generated": new_generated,
            "successful": successful,
            "failed": failed,
            "generated_at": datetime.utcnow().isoformat(),
            "results": results,
        }

        return summary

    async def regenerate_for_notebook(self, notebook_id: str) -> PresentationResult:
        """Regenerate presentation and audio for a specific notebook.

        Args:
            notebook_id: Notebook ID.

        Returns:
            PresentationResult.
        """
        # Remove existing entry to force regeneration
        if notebook_id in self.presentations:
            del self.presentations[notebook_id]
            self._save_presentations()

        meta = self.metadata.get(notebook_id)
        if not meta:
            logger.error(f"No metadata found for notebook: {notebook_id}")
            return PresentationResult(
                notebook_id=notebook_id,
                notebook_title="Unknown",
                success=False,
                errors=["No metadata found"],
            )

        return await self.generate_for_notebook(notebook_id, meta)


def extract_issue_number(title: str) -> Optional[int]:
    """Extract issue number from notebook title."""
    match = re.search(r"#?(\d+)", title)
    return int(match.group(1)) if match else None


async def main():
    parser = argparse.ArgumentParser(
        description="Generate presentations and audio overviews for enriched notebooks"
    )
    parser.add_argument(
        "--issue",
        type=int,
        help="Generate for specific issue number"
    )
    parser.add_argument(
        "--notebook-id",
        type=str,
        help="Generate for specific notebook ID"
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Generate for all enriched notebooks"
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=0,
        help="Limit number of notebooks to process (0 = all)"
    )
    parser.add_argument(
        "--regenerate",
        action="store_true",
        help="Regenerate existing presentations"
    )

    args = parser.parse_args()

    async with await NotebookLMClient.from_storage() as client:
        generator = PresentationGenerator(client)

        if args.notebook_id:
            # Generate for specific notebook ID
            meta = generator.metadata.get(args.notebook_id)
            if not meta:
                print(f"Error: No metadata for notebook {args.notebook_id}")
                return

            if args.regenerate and args.notebook_id in generator.presentations:
                del generator.presentations[args.notebook_id]

            result = await generator.generate_for_notebook(args.notebook_id, meta)

            print(f"\n{'='*60}")
            print(f"Presentation Generation Result")
            print(f"{'='*60}")
            print(f"Notebook: {result.notebook_title}")
            print(f"Topic: {result.topic or 'None'}")
            print(f"\nAudio Task ID: {result.audio_task_id or 'None'}")
            print(f"Audio URL: {result.audio_download_url or 'None'}")
            print(f"\nPresentation Task ID: {result.presentation_task_id or 'None'}")
            print(f"Presentation URL: {result.presentation_download_url or 'None'}")
            if result.errors:
                print(f"\nErrors: {len(result.errors)}")
                for err in result.errors:
                    print(f"  - {err}")
            print(f"{'='*60}\n")

        elif args.issue:
            # Find notebook by issue number
            found_id = None
            for nb_id, meta in generator.metadata.items():
                title = meta.get("notebook_title", "")
                if extract_issue_number(title) == args.issue:
                    found_id = nb_id
                    break

            if not found_id:
                print(f"Error: No enriched notebook found for issue #{args.issue}")
                return

            meta = generator.metadata[found_id]
            if args.regenerate and found_id in generator.presentations:
                del generator.presentations[found_id]

            result = await generator.generate_for_notebook(found_id, meta)

            print(f"\n{'='*60}")
            print(f"Presentation Generation Result")
            print(f"{'='*60}")
            print(f"Notebook: {result.notebook_title}")
            print(f"Topic: {result.topic or 'None'}")
            print(f"\nAudio Task ID: {result.audio_task_id or 'None'}")
            print(f"Audio URL: {result.audio_download_url or 'None'}")
            print(f"\nPresentation Task ID: {result.presentation_task_id or 'None'}")
            print(f"Presentation URL: {result.presentation_download_url or 'None'}")
            if result.errors:
                print(f"\nErrors: {len(result.errors)}")
                for err in result.errors:
                    print(f"  - {err}")
            print(f"{'='*60}\n")

        elif args.all:
            # Generate for all notebooks
            summary = await generator.generate_all(limit=args.limit)

            print(f"\n{'='*60}")
            print(f"Presentation Generation Summary")
            print(f"{'='*60}")
            print(f"Total notebooks: {summary['total_processed']}")
            print(f"Skipped (already done): {summary['skipped']}")
            print(f"New generated: {summary['new_generated']}")
            print(f"Successful: {summary['successful']}")
            print(f"Failed: {summary['failed']}")
            print(f"Generated at: {summary['generated_at']}")
            print(f"{'='*60}\n")

        else:
            parser.print_help()


if __name__ == "__main__":
    asyncio.run(main())
