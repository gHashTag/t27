#!/usr/bin/env python3
# ACE-Step Wrapper for t27
# phi^2 + 1/phi^2 = 3 | TRINITY

"""ACE-Step local AI music generation integration.

ACE-Step (April 2026) — Best local lyrics-to-song model.
- 3.5B parameters, Suno-quality output
- Full song generation with vocals
- Multi-language support including Russian
- Local execution, no API limits

Installation:
    pip install ace-step

CLI Usage:
    ace-step generate --tags "dark phonk, russian rap" \
                     --lyrics track.txt \
                     --output track.wav \
                     --duration 120

Architecture integration (PHI LOOP):
    track.md (lyrics) → M-agent (ACE-Step) → gen/music/track.wav → seal → commit
"""

import subprocess
import hashlib
from pathlib import Path
from typing import Optional, Dict, List
import json
import logging
import re

logger = logging.getLogger(__name__)


class ACEStepGenerator:
    """Wrapper for ACE-Step local music generation.

    Generates complete songs with vocals from lyrics text.
    """

    # REF track styles for trixphi-album
    REF_STYLES = {
        "trap": {
            "tags": "dark trap, russian rap, heavy 808 bass, rolling hi-hats, aggressive drums, 140 bpm, C# minor",
            "bpm": 140,
            "key": "C#",
            "duration": 120,
        },
        "phonk": {
            "tags": "drift phonk, distorted bass, bells, aggressive drums, cowbell, russian rap, 140 bpm, E minor",
            "bpm": 140,
            "key": "E",
            "duration": 120,
        },
        "wavephonk": {
            "tags": "wave phonk, distorted bass, ethereal pads, bells, heavy compression, russian rap, 150 bpm, E minor",
            "bpm": 150,
            "key": "E",
            "duration": 120,
        },
        "lofi": {
            "tags": "lo-fi hip-hop, chill, vinyl crackle, soft piano, russian vocals, 85 bpm, C major",
            "bpm": 85,
            "key": "C",
            "duration": 150,
        },
    }

    def __init__(self, ace_step_path: Optional[str] = None):
        """Initialize ACE-Step generator.

        Args:
            ace_step_path: Path to ace-step CLI (auto-detected if None)
        """
        self.ace_step_path = ace_step_path or self._find_ace_step()
        self.available = self._check_available()

    def _find_ace_step(self) -> Optional[str]:
        """Find ace-step executable."""
        try:
            result = subprocess.run(
                ["which", "ace-step"],
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                return result.stdout.strip()
        except Exception:
            pass

        # Try python module
        try:
            result = subprocess.run(
                ["python3", "-c", "import ace_step; print('OK')"],
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                return "python3"
        except Exception:
            pass

        return None

    def _check_available(self) -> bool:
        """Check if ACE-Step is available."""
        if self.ace_step_path is None:
            return False
        return True

    def generate(
        self,
        lyrics: str,
        style: str = "trap",
        output: Path = None,
        duration: Optional[int] = None,
        custom_tags: Optional[str] = None,
    ) -> Dict:
        """Generate music with vocals from lyrics.

        Args:
            lyrics: Song lyrics text
            style: Music style (trap, phonk, wavephonk, lofi)
            output: Output WAV file path
            duration: Duration in seconds (uses style default if None)
            custom_tags: Custom style tags (overrides style)

        Returns:
            Dictionary with generation result

        Example:
            >>> gen = ACEStepGenerator()
            >>> result = gen.generate(
            ...     lyrics="фи в квадрате...",
            ...     style="trap",
            ...     output="track.wav"
            ... )
        """
        if not self.available:
            return {
                "success": False,
                "error": "ACE-Step not installed. Run: pip install ace-step",
            }

        # Get style configuration
        if style not in self.REF_STYLES:
            return {
                "success": False,
                "error": f"Unknown style: {style}. Available: {list(self.REF_STYLES.keys())}",
            }

        style_config = self.REF_STYLES[style]
        tags = custom_tags or style_config["tags"]
        duration = duration or style_config["duration"]

        # Set output path
        if output is None:
            output = Path(f"gen/music/track_{style}.wav")
        output = Path(output)
        output.parent.mkdir(parents=True, exist_ok=True)

        # Write lyrics to temp file
        lyrics_file = Path("/tmp/ace_step_lyrics.txt")
        lyrics_file.write_text(lyrics, encoding="utf-8")

        logger.info(f"Generating {style} track with ACE-Step...")
        logger.info(f"Tags: {tags[:80]}...")
        logger.info(f"Duration: {duration}s")
        logger.info(f"Output: {output}")

        # Build command
        cmd = [
            self.ace_step_path,
            "-m", "ace_step",
            "generate",
            "--tags", tags,
            "--lyrics", str(lyrics_file),
            "--output", str(output),
            "--duration", str(duration),
        ]

        # For python module invocation
        if self.ace_step_path == "python3":
            cmd = [
                "python3", "-c",
                f"""
import ace_step
ace_step.generate(
    tags="{tags}",
    lyrics_path="{lyrics_file}",
    output_path="{output}",
    duration={duration},
)
"""
            ]

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=600,  # 10 minutes max
            )

            if result.returncode == 0 and output.exists():
                file_size = output.stat().st_size / (1024 * 1024)

                return {
                    "success": True,
                    "output_path": str(output),
                    "style": style,
                    "duration": duration,
                    "file_size_mb": file_size,
                    "stderr": result.stderr,
                }
            else:
                return {
                    "success": False,
                    "error": result.stderr or "Generation failed",
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                }

        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "error": "Generation timeout (10 minutes)",
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
            }

    def generate_from_markdown(
        self,
        markdown_path: Path,
        style: str = "trap",
        output: Path = None,
    ) -> Dict:
        """Generate track from trixphi markdown file.

        Extracts lyrics from markdown file and generates music.

        Args:
            markdown_path: Path to track markdown file (e.g., 01-*.md)
            style: Music style
            output: Output path

        Returns:
            Generation result dictionary
        """
        markdown_path = Path(markdown_path)

        if not markdown_path.exists():
            return {
                "success": False,
                "error": f"Markdown file not found: {markdown_path}",
            }

        # Read markdown and extract lyrics
        content = markdown_path.read_text(encoding="utf-8")

        # Extract lyrics (usually after ## Lyrics or in code block)
        lyrics = self._extract_lyrics(content)

        if not lyrics:
            # Use filename/title as default
            title = self._extract_title(content)
            lyrics = f"{title}\n\n" + content[:500]  # First 500 chars as lyrics

        # Set output path based on markdown filename
        if output is None:
            track_num = markdown_path.stem.split("-")[0].zfill(2)
            output = Path(f"gen/music/{track_num}-{style}.wav")

        return self.generate(
            lyrics=lyrics,
            style=style,
            output=output,
        )

    def _extract_lyrics(self, markdown_content: str) -> Optional[str]:
        """Extract lyrics from markdown content."""
        # Try to find ## Lyrics section
        lyrics_match = re.search(
            r"##\s*(?:Lyrics|Текст|Lyrics:|Текст:)\s*\n+(.*?)(?:\n##|\n\n|\Z)",
            markdown_content,
            re.DOTALL | re.IGNORECASE,
        )

        if lyrics_match:
            return lyrics_match.group(1).strip()

        # Try code block
        code_match = re.search(r"```(?:lyrics|text)?\n(.*?)```", markdown_content, re.DOTALL)
        if code_match:
            return code_match.group(1).strip()

        return None

    def _extract_title(self, markdown_content: str) -> str:
        """Extract title from markdown."""
        title_match = re.search(r"#\s+(.+)", markdown_content)
        if title_match:
            return title_match.group(1).strip()

        # Try first line
        first_line = markdown_content.split("\n")[0]
        return first_line or "TRIXPHI Track"

    def seal_file(self, file_path: Path) -> Dict:
        """Create SHA-256 seal for generated file.

        Args:
            file_path: Path to generated WAV file

        Returns:
            Dictionary with seal information
        """
        file_path = Path(file_path)

        if not file_path.exists():
            return {"success": False, "error": "File not found"}

        # Calculate SHA-256
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)

        seal = sha256_hash.hexdigest()

        # Write seal file
        seal_path = file_path.with_suffix(".wav.seal")
        seal_path.write_text(
            json.dumps({
                "sha256": seal,
                "file": str(file_path),
                "generator": "ace-step",
                "phi_identity": "phi^2 + 1/phi^2 = 3",
            }, indent=2)
        )

        return {
            "success": True,
            "seal": seal,
            "seal_path": str(seal_path),
        }

    def batch_generate(
        self,
        markdown_dir: Path,
        style: str = "trap",
        output_dir: Path = None,
    ) -> List[Dict]:
        """Generate tracks from all markdown files in directory.

        Args:
            markdown_dir: Directory containing track markdown files
            style: Music style
            output_dir: Output directory for WAV files

        Returns:
            List of generation results
        """
        markdown_dir = Path(markdown_dir)

        if output_dir is None:
            output_dir = Path("gen/music")
        output_dir.mkdir(parents=True, exist_ok=True)

        # Find all markdown files
        md_files = sorted(markdown_dir.glob("*.md"))

        results = []
        for md_file in md_files:
            track_num = md_file.stem.split("-")[0] if "-" in md_file.stem else "unknown"

            output = output_dir / f"{track_num}-{style}.wav"

            logger.info(f"\n{'='*60}")
            logger.info(f"Processing: {md_file.name}")
            logger.info(f"{'='*60}")

            result = self.generate_from_markdown(md_file, style, output)
            results.append(result)

            # Create seal if successful
            if result.get("success"):
                self.seal_file(Path(result["output_path"]))

        return results


def create_ace_step_generator(ace_step_path: Optional[str] = None) -> ACEStepGenerator:
    """Factory function to create ACE-Step generator.

    Args:
        ace_step_path: Path to ace-step CLI

    Returns:
        ACEStepGenerator instance
    """
    return ACEStepGenerator(ace_step_path=ace_step_path)


def main():
    """CLI for ACE-Step integration."""
    import argparse

    parser = argparse.ArgumentParser(
        description="ACE-Step Music Generator — t27 Integration"
    )
    parser.add_argument(
        "--mode",
        choices=["generate", "batch", "check"],
        default="generate",
        help="Operation mode"
    )
    parser.add_argument(
        "--lyrics",
        help="Lyrics text or path to lyrics file"
    )
    parser.add_argument(
        "--markdown",
        type=Path,
        help="Path to track markdown file"
    )
    parser.add_argument(
        "--style",
        choices=list(ACEStepGenerator.REF_STYLES.keys()),
        default="trap",
        help="Music style"
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Output WAV file path"
    )
    parser.add_argument(
        "--markdown-dir",
        type=Path,
        help="Directory with markdown files (for batch mode)"
    )
    parser.add_argument(
        "--duration",
        type=int,
        help="Duration in seconds (overrides style default)"
    )
    parser.add_argument(
        "--tags",
        help="Custom style tags (overrides style)"
    )
    parser.add_argument(
        "--seal",
        action="store_true",
        help="Create SHA-256 seal for output file"
    )

    args = parser.parse_args()

    print("╔════════════════════════════════════════════════════════════╗")
    print("║       ACE-Step Music Generator                            ║")
    print("║  phi^2 + 1/phi^2 = 3 | TRINITY                            ║")
    print("╚════════════════════════════════════════════════════════════╝")
    print()

    gen = create_ace_step_generator()

    if args.mode == "check":
        if gen.available:
            print("✅ ACE-Step is available")
            print(f"   Path: {gen.ace_step_path}")
            return 0
        else:
            print("❌ ACE-Step not found")
            print()
            print("Install with:")
            print("  pip install ace-step")
            return 1

    if args.mode == "generate":
        if args.markdown:
            # Generate from markdown file
            result = gen.generate_from_markdown(
                markdown_path=args.markdown,
                style=args.style,
                output=args.output,
            )
        elif args.lyrics:
            # Generate from lyrics text or file
            lyrics_path = Path(args.lyrics)
            if lyrics_path.exists():
                lyrics = lyrics_path.read_text(encoding="utf-8")
            else:
                lyrics = args.lyrics

            result = gen.generate(
                lyrics=lyrics,
                style=args.style,
                output=args.output,
                custom_tags=args.tags,
                duration=args.duration,
            )
        else:
            print("❌ Error: --lyrics or --markdown required")
            return 1

        if result["success"]:
            print()
            print("✅ Generation successful!")
            print(f"   Output: {result['output_path']}")
            print(f"   Style: {result['style']}")
            print(f"   Size: {result.get('file_size_mb', 0):.1f} MB")

            if args.seal:
                seal_result = gen.seal_file(Path(result["output_path"]))
                if seal_result["success"]:
                    print(f"   Seal: {seal_result['seal'][:16]}...")
                    print(f"   Seal file: {seal_result['seal_path']}")

            return 0
        else:
            print()
            print(f"❌ Generation failed: {result.get('error', 'Unknown')}")
            return 1

    elif args.mode == "batch":
        if not args.markdown_dir:
            print("❌ Error: --markdown-dir required for batch mode")
            return 1

        results = gen.batch_generate(
            markdown_dir=args.markdown_dir,
            style=args.style,
        )

        success_count = sum(1 for r in results if r.get("success"))
        print()
        print(f"Batch complete: {success_count}/{len(results)} successful")

        return 0 if success_count > 0 else 1


if __name__ == "__main__":
    import sys
    sys.exit(main())
