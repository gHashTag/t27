# contrib/backend/music-generator/pipeline.py
# Music generation pipeline orchestration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Music generation pipeline orchestration.

Provides CLI and programmatic interface for the complete
music generation workflow from lyrics to final mastered track.
"""

import argparse
import logging
import sys
from pathlib import Path
from typing import Optional, Dict, Any, List
import numpy as np
from tqdm import tqdm

try:
    from .config import config_from_env, MusicGenConfig
    from .music_gen.musicgen import MusicGenWrapper
    from .voice_clone.rvc import RVCCloner
    from .vocal_synth.synthesizer import VocalSynthesizer, VocalStyle
    from .effects.processor import VocalProcessor, InstrumentalProcessor
    from .mixing.auto_mixer import AutoMixer
except ImportError:
    # Fallback to absolute imports for direct execution/testing
    from config import config_from_env, MusicGenConfig
    from music_gen.musicgen import MusicGenWrapper
    from voice_clone.rvc import RVCCloner
    from vocal_synth.synthesizer import VocalSynthesizer, VocalStyle
    from effects.processor import VocalProcessor, InstrumentalProcessor
    from mixing.auto_mixer import AutoMixer

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


class MusicPipeline:
    """Complete music generation pipeline.

    Orchestrates music generation, vocal synthesis, effects,
    mixing, and mastering into a cohesive workflow.

    Attributes:
        config: Configuration object
        musicgen: Music generation model
        rvc: Voice cloning model (optional)
        synthesizer: Vocal synthesizer
        vocal_processor: Vocal effect processor
        instrumental_processor: Instrumental effect processor
        mixer: Auto mixer and masterer

    Example:
        >>> pipeline = MusicPipeline()
        >>> result = pipeline.generate(
        ...     lyrics="Riding through the dark",
        ...     style="phonk",
        ...     output_path="output.wav"
        ... )
    """

    def __init__(
        self,
        config: Optional[MusicGenConfig] = None,
        device: str = "cpu",
    ):
        """Initialize music pipeline.

        Args:
            config: Configuration object (uses defaults if None)
            device: Target device (cpu, cuda)

        Complexity: O(1) initialization
        """
        self.config = config or config_from_env()
        self.device = device or self.config.device

        # Initialize components (lazy loading)
        self.musicgen: Optional[MusicGenWrapper] = None
        self.rvc: Optional[RVCCloner] = None
        self.synthesizer: Optional[VocalSynthesizer] = None
        self.vocal_processor: Optional[VocalProcessor] = None
        self.instrumental_processor: Optional[InstrumentalProcessor] = None
        self.mixer: Optional[AutoMixer] = None

        logger.info(f"MusicPipeline initialized with device={self.device}")

    def generate(
        self,
        lyrics: str,
        style: str = "phonk",
        output_path: Optional[Path] = None,
        voice_sample: Optional[Path] = None,
        duration: int = 60,
        vocal_style: str = "aggressive",
        instrumental_intensity: float = 1.0,
        vocal_level_db: float = -4.0,
        skip_vocals: bool = False,
        skip_effects: bool = False,
        skip_mastering: bool = False,
        progress_callback: Optional[callable] = None,
    ) -> Dict[str, Any]:
        """Generate complete music track.

        Args:
            lyrics: Lyrics for vocal track
            style: Musical style (phonk, trap, hip_hop, drill, lofi)
            output_path: Path to save final track
            voice_sample: Optional voice sample for cloning
            duration: Track duration in seconds
            vocal_style: Vocal delivery style (aggressive, ethereal, eerie, smooth, choppy)
            instrumental_intensity: Effect intensity for instrumental (0.0 to 2.0)
            vocal_level_db: Vocal level in mix (dB)
            skip_vocals: Skip vocal generation (instrumental only)
            skip_effects: Skip effect processing
            skip_mastering: Skip mastering chain
            progress_callback: Optional callback for progress updates

        Returns:
            Dictionary with generated audio arrays and metadata

        Raises:
            ValueError: If invalid style or parameters

        Complexity: O(duration * sample_rate)

        Example:
            >>> result = pipeline.generate(
            ...     lyrics="Shadows creeping in the night",
            ...     style="phonk",
            ...     output_path="track.wav"
            ... )
        """
        logger.info(f"Starting music generation: style={style}, duration={duration}s")

        result: Dict[str, Any] = {
            "instrumental": None,
            "vocals": None,
            "mix": None,
            "mastered": None,
            "metadata": {
                "style": style,
                "duration": duration,
                "vocal_level_db": vocal_level_db,
            },
        }

        try:
            # Phase 1: Generate instrumental
            if progress_callback:
                progress_callback("Generating instrumental...", 10)

            instrumental = self._generate_instrumental(style, duration)
            result["instrumental"] = instrumental
            logger.info("Instrumental generated")

            # Phase 2: Process instrumental effects
            if not skip_effects:
                if progress_callback:
                    progress_callback("Processing instrumental effects...", 30)

                instrumental = self._process_instrumental(instrumental, style, instrumental_intensity)
                result["instrumental"] = instrumental
                logger.info("Instrumental effects applied")

            # Phase 3: Generate vocals (if not skipped)
            if not skip_vocals and lyrics:
                if progress_callback:
                    progress_callback("Generating vocals...", 50)

                vocals = self._generate_vocals(
                    lyrics,
                    style,
                    duration,
                    vocal_style,
                    voice_sample,
                )
                result["vocals"] = vocals
                logger.info("Vocals generated")

                # Phase 4: Process vocal effects
                if not skip_effects:
                    if progress_callback:
                        progress_callback("Processing vocal effects...", 70)

                    vocals = self._process_vocals(vocals, vocal_style)
                    result["vocals"] = vocals
                    logger.info("Vocal effects applied")

                # Phase 5: Mix vocals with instrumental
                if progress_callback:
                    progress_callback("Mixing tracks...", 80)

                mix = self._mix(vocals, instrumental, vocal_level_db)
                result["mix"] = mix
                logger.info("Tracks mixed")
            else:
                # Instrumental only
                result["mix"] = instrumental

            # Phase 6: Master final track
            if not skip_mastering:
                if progress_callback:
                    progress_callback("Mastering final track...", 90)

                mastered = self._master(result["mix"])
                result["mastered"] = mastered
                logger.info("Track mastered")
            else:
                result["mastered"] = result["mix"]

            # Phase 7: Save output
            if output_path:
                if progress_callback:
                    progress_callback("Saving output...", 95)

                self._save_audio(result["mastered"], output_path)
                result["output_path"] = str(output_path)
                logger.info(f"Final track saved to {output_path}")

            if progress_callback:
                progress_callback("Complete!", 100)

            logger.info("Music generation complete")
            return result

        except Exception as e:
            logger.error(f"Pipeline failed: {e}")
            raise

    def _get_musicgen(self) -> MusicGenWrapper:
        """Get or create MusicGen wrapper.

        Returns:
            MusicGenWrapper instance

        Complexity: O(1) or O(model_size) on first call
        """
        if self.musicgen is None:
            self.musicgen = MusicGenWrapper(
                model_size=self.config.model_size,
                device=self.device,
                sample_rate=self.config.sample_rate,
            )
        return self.musicgen

    def _get_synthesizer(self) -> VocalSynthesizer:
        """Get or create vocal synthesizer.

        Returns:
            VocalSynthesizer instance

        Complexity: O(1) or O(model_size) on first call
        """
        if self.synthesizer is None:
            self.synthesizer = VocalSynthesizer(
                sample_rate=self.config.sample_rate,
                device=self.device,
            )
        return self.synthesizer

    def _get_vocal_processor(self) -> VocalProcessor:
        """Get or create vocal processor.

        Returns:
            VocalProcessor instance

        Complexity: O(1)
        """
        if self.vocal_processor is None:
            self.vocal_processor = VocalProcessor(
                sample_rate=self.config.sample_rate,
            )
        return self.vocal_processor

    def _get_instrumental_processor(self) -> InstrumentalProcessor:
        """Get or create instrumental processor.

        Returns:
            InstrumentalProcessor instance

        Complexity: O(1)
        """
        if self.instrumental_processor is None:
            self.instrumental_processor = InstrumentalProcessor(
                sample_rate=self.config.sample_rate,
            )
        return self.instrumental_processor

    def _get_mixer(self) -> AutoMixer:
        """Get or create auto mixer.

        Returns:
            AutoMixer instance

        Complexity: O(1)
        """
        if self.mixer is None:
            self.mixer = AutoMixer(
                sample_rate=self.config.sample_rate,
                vocal_level_db=self.config.vocal_level_db,
                limiter_threshold=self.config.limiter_threshold,
            )
        return self.mixer

    def _generate_instrumental(self, style: str, duration: int) -> np.ndarray:
        """Generate instrumental track.

        Args:
            style: Musical style
            duration: Duration in seconds

        Returns:
            Generated instrumental audio

        Complexity: O(duration * sample_rate)
        """
        musicgen = self._get_musicgen()

        # Get genre-specific prompt
        genre_config = self.config.genres.get(style, {})
        prompt = genre_config.get("prompt_template", f"{style} beat")

        return musicgen.generate(
            prompt=prompt,
            duration=duration,
            genre=style,
        )

    def _process_instrumental(
        self,
        audio: np.ndarray,
        style: str,
        intensity: float,
    ) -> np.ndarray:
        """Process instrumental with style-specific effects.

        Args:
            audio: Input instrumental audio
            style: Musical style
            intensity: Effect intensity

        Returns:
            Processed instrumental audio

        Complexity: O(n) where n is number of samples
        """
        processor = self._get_instrumental_processor()

        style_methods = {
            "phonk": processor.process_phonk,
            "trap": processor.process_trap,
            "drill": processor.process_drill,
            "lofi": processor.process_lofi,
            "hip_hop": processor.process_trap,  # Use trap processing for hip-hop
        }

        method = style_methods.get(style, processor.process_phonk)

        if style == "lofi":
            return method(audio, warmth=intensity)
        elif style == "trap":
            return method(audio, sub_bass_boost=intensity)
        elif style == "drill":
            return method(audio, darkness=intensity)
        else:
            return method(audio, intensity=intensity)

    def _generate_vocals(
        self,
        lyrics: str,
        style: str,
        duration: int,
        vocal_style: str,
        voice_sample: Optional[Path],
    ) -> np.ndarray:
        """Generate vocal track.

        Args:
            lyrics: Lyrics text
            style: Musical style (affects tempo)
            duration: Duration in seconds
            vocal_style: Vocal delivery style
            voice_sample: Optional voice sample for cloning

        Returns:
            Generated vocal audio

        Complexity: O(duration * sample_rate)
        """
        synthesizer = self._get_synthesizer()

        # Map vocal style string to enum
        style_map = {
            "aggressive": VocalStyle.AGGRESSIVE,
            "eerie": VocalStyle.EERIE,
            "ethereal": VocalStyle.ETHEREAL,
            "smooth": VocalStyle.SMOOTH,
            "choppy": VocalStyle.CHOPPY,
        }

        vstyle = style_map.get(vocal_style.lower(), VocalStyle.AGGRESSIVE)

        # Generate vocals
        vocals = synthesizer.generate_vocal(
            lyrics=lyrics,
            style=vstyle,
            duration=duration,
        )

        # Apply voice cloning if sample provided
        if voice_sample and voice_sample.exists():
            vocals = self._apply_voice_clone(vocals, voice_sample)

        return vocals

    def _apply_voice_clone(
        self,
        vocals: np.ndarray,
        voice_sample: Path,
    ) -> np.ndarray:
        """Apply voice cloning to vocals.

        Args:
            vocals: Input vocal audio
            voice_sample: Voice sample for cloning

        Returns:
            Voice-cloned audio

        Note:
            This is a placeholder. Real implementation would use RVC.
        """
        # Placeholder - return original vocals
        logger.info(f"Voice cloning requested but not fully implemented")
        return vocals

    def _process_vocals(self, vocals: np.ndarray, vocal_style: str) -> np.ndarray:
        """Process vocals with style-specific effects.

        Args:
            vocals: Input vocal audio
            vocal_style: Vocal delivery style

        Returns:
            Processed vocal audio

        Complexity: O(n) where n is number of samples
        """
        processor = self._get_vocal_processor()

        if vocal_style == "aggressive":
            return processor.process_verses(vocals)
        elif vocal_style == "ethereal":
            return processor.process_chorus(vocals, ethereal=1.0)
        elif vocal_style == "eerie":
            return processor.process_chorus(vocals, ethereal=0.7)
        elif vocal_style == "smooth":
            return processor.clean_vocal(vocals)
        else:
            return processor.process_verses(vocals)

    def _mix(
        self,
        vocals: np.ndarray,
        instrumental: np.ndarray,
        vocal_level_db: float,
    ) -> np.ndarray:
        """Mix vocals with instrumental.

        Args:
            vocals: Vocal audio array
            instrumental: Instrumental audio array
            vocal_level_db: Vocal level in dB

        Returns:
            Mixed audio array

        Complexity: O(n) where n is number of samples
        """
        mixer = self._get_mixer()
        return mixer.mix(vocals, instrumental, vocal_level_db=vocal_level_db)

    def _master(self, mix: np.ndarray) -> np.ndarray:
        """Master the mix.

        Args:
            mix: Mixed audio array

        Returns:
            Mastered audio array

        Complexity: O(n) where n is number of samples
        """
        mixer = self._get_mixer()
        return mixer.master(mix)

    def _save_audio(self, audio: np.ndarray, path: Path) -> None:
        """Save audio to file.

        Args:
            audio: Audio array [channels, samples]
            path: Output file path

        Complexity: O(n) where n is number of samples
        """
        import soundfile as sf

        path.parent.mkdir(parents=True, exist_ok=True)

        # Convert to [samples, channels] for soundfile
        audio_transposed = audio.T

        sf.write(
            str(path),
            audio_transposed,
            self.config.sample_rate,
            format="WAV",
            subtype="PCM_24",
        )


def create_pipeline(
    config: Optional[MusicGenConfig] = None,
    device: str = "cpu",
) -> MusicPipeline:
    """Factory function to create music pipeline.

    Args:
        config: Configuration object
        device: Target device (cpu, cuda)

    Returns:
        Initialized MusicPipeline instance

    Complexity: O(1)
    """
    return MusicPipeline(config=config, device=device)


def main() -> int:
    """CLI entry point for music generation.

    Returns:
        Exit code (0 for success, non-zero for error)

    Example:
        $ python pipeline.py --lyrics "My lyrics" --style phonk --output track.wav
    """
    parser = argparse.ArgumentParser(
        description="T27 Music Generator - AI music generation pipeline",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate phonk track with vocals
  python pipeline.py --lyrics "Shadows creeping" --style phonk --output track.wav

  # Generate instrumental only
  python pipeline.py --style trap --duration 30 --output beat.wav --skip-vocals

  # Generate with custom voice sample
  python pipeline.py --lyrics "My flow" --voice-sample my_voice.wav --style hip-hop
        """,
    )

    # Input arguments
    parser.add_argument(
        "--lyrics",
        type=str,
        help="Lyrics for vocal track",
    )
    parser.add_argument(
        "--voice-sample",
        type=Path,
        help="Voice sample for cloning",
    )
    parser.add_argument(
        "--lyrics-file",
        type=Path,
        help="Read lyrics from file",
    )

    # Style arguments
    parser.add_argument(
        "--style",
        type=str,
        choices=["phonk", "trap", "hip_hop", "drill", "lofi"],
        default="phonk",
        help="Musical style (default: phonk)",
    )
    parser.add_argument(
        "--vocal-style",
        type=str,
        choices=["aggressive", "eerie", "ethereal", "smooth", "choppy"],
        default="aggressive",
        help="Vocal delivery style (default: aggressive)",
    )

    # Output arguments
    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        default=Path("output.wav"),
        help="Output file path (default: output.wav)",
    )

    # Technical arguments
    parser.add_argument(
        "--duration",
        type=int,
        default=60,
        help="Track duration in seconds (default: 60)",
    )
    parser.add_argument(
        "--vocal-level",
        type=float,
        default=-4.0,
        help="Vocal level in dB (default: -4.0)",
    )
    parser.add_argument(
        "--intensity",
        type=float,
        default=1.0,
        help="Effect intensity (0.0 to 2.0, default: 1.0)",
    )
    parser.add_argument(
        "--device",
        type=str,
        choices=["cpu", "cuda"],
        default="cpu",
        help="Device to use (default: cpu)",
    )

    # Skip options
    parser.add_argument(
        "--skip-vocals",
        action="store_true",
        help="Generate instrumental only",
    )
    parser.add_argument(
        "--skip-effects",
        action="store_true",
        help="Skip effect processing",
    )
    parser.add_argument(
        "--skip-mastering",
        action="store_true",
        help="Skip mastering chain",
    )

    # Other options
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Verbose output",
    )
    parser.add_argument(
        "--quiet", "-q",
        action="store_true",
        help="Quiet output (errors only)",
    )

    args = parser.parse_args()

    # Setup logging level
    if args.quiet:
        logging.getLogger().setLevel(logging.ERROR)
    elif args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    # Read lyrics from file if specified
    lyrics = args.lyrics
    if args.lyrics_file:
        if lyrics:
            logger.warning("Both --lyrics and --lyrics-file specified, using file content")
        lyrics = args.lyrics_file.read_text()

    # Validate inputs
    if not args.skip_vocals and not lyrics:
        parser.error("--lyrics or --lyrics-file required unless --skip-vocals is specified")

    if args.voice_sample and not args.voice_sample.exists():
        parser.error(f"Voice sample file not found: {args.voice_sample}")

    # Create pipeline
    try:
        pipeline = create_pipeline(device=args.device)

        # Progress callback
        def progress(message: str, percent: int) -> None:
            if not args.quiet:
                print(f"[{percent:3d}%] {message}")

        # Generate
        result = pipeline.generate(
            lyrics=lyrics or "",
            style=args.style,
            output_path=args.output,
            voice_sample=args.voice_sample,
            duration=args.duration,
            vocal_style=args.vocal_style,
            instrumental_intensity=args.intensity,
            vocal_level_db=args.vocal_level,
            skip_vocals=args.skip_vocals,
            skip_effects=args.skip_effects,
            skip_mastering=args.skip_mastering,
            progress_callback=progress,
        )

        if not args.quiet:
            print(f"\nSuccess! Track saved to: {args.output}")
            if "mix" in result:
                levels = pipeline._get_mixer().analyze_levels(result["mastered"])
                print(f"Mastered levels: LUFS ~{levels['lufs_approx']:.1f}, Peak {levels['peak_db']:.1f}dB")

        return 0

    except KeyboardInterrupt:
        logger.info("Interrupted by user")
        return 130
    except Exception as e:
        logger.error(f"Error: {e}")
        if args.verbose:
            import traceback
            traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
