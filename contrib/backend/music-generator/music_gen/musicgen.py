# contrib/backend/music-generator/music_gen/musicgen.py
# MusicGen wrapper for music generation
# phi^2 + 1/phi^2 = 3 | TRINITY

"""MusicGen integration for AI music generation.

Provides CPU-optimized wrapper around Meta's MusicGen model for
generating instrumental tracks in various genres.
"""

import torch
import numpy as np
from pathlib import Path
from typing import Optional, Dict, Any
import logging
from tqdm import tqdm

logger = logging.getLogger(__name__)


class MusicGenWrapper:
    """Wrapper for MusicGen model with CPU optimization.

    Attributes:
        model: Loaded MusicGen model
        config: Configuration dictionary
        device: Target device (cpu or cuda)
        sample_rate: Audio sample rate in Hz
        model_size: Model size identifier (small, medium, large)

    Example:
        >>> wrapper = MusicGenWrapper(model_size="small", device="cpu")
        >>> wrapper.generate("dark phonk beat", duration=30, output_path="output.wav")
    """

    def __init__(
        self,
        model_size: str = "small",
        device: str = "cpu",
        sample_rate: int = 48000,
        config: Optional[Dict[str, Any]] = None,
    ):
        """Initialize MusicGen wrapper.

        Args:
            model_size: Model size (small, medium, large)
            device: Target device (cpu, cuda)
            sample_rate: Audio sample rate in Hz
            config: Optional configuration dictionary

        Complexity: O(1) initialization, O(model_size) for loading
        """
        self.model_size = model_size
        self.device = device
        self.sample_rate = sample_rate
        self.config = config or {}
        self.model = None
        self._model_loaded = False

    def load_model(self) -> None:
        """Load MusicGen model onto configured device.

        Raises:
            ImportError: If audiocraft is not installed
            RuntimeError: If model loading fails

        Complexity: O(model_size) - depends on model parameter count
        """
        if self._model_loaded:
            return

        try:
            from audiocraft.models import MusicGen
            from audiocraft.data.audio import audio_write
        except ImportError as e:
            raise ImportError(
                "audiocraft is required for MusicGen. "
                "Install with: pip install audiocraft"
            ) from e

        logger.info(f"Loading MusicGen {self.model_size} model on {self.device}...")

        model_name = self._get_model_name()
        self.model = MusicGen.get_pretrained(model_name)
        self.model.set_generation_params(
            use_sampling=True,
            top_k=250,
            top_p=0.0,
            temperature=1.0,
            duration=30.0,
            cfg_coef=3.0,
        )

        if self.device == "cpu":
            self.model.cpu()
            logger.info("Model loaded on CPU (optimized for inference)")
        else:
            self.model.to(self.device)
            logger.info(f"Model loaded on {self.device}")

        self._model_loaded = True
        self._audio_write = audio_write

    def _get_model_name(self) -> str:
        """Get audiocraft model name from size identifier.

        Returns:
            Model name string for audiocraft

        Complexity: O(1)
        """
        model_map = {
            "small": "facebook/musicgen-small",
            "medium": "facebook/musicgen-medium",
            "large": "facebook/musicgen-large",
        }
        return model_map.get(self.model_size, "facebook/musicgen-small")

    def generate(
        self,
        prompt: str,
        duration: int = 30,
        bpm: Optional[int] = None,
        genre: Optional[str] = None,
        output_path: Optional[Path] = None,
    ) -> np.ndarray:
        """Generate music from text prompt.

        Args:
            prompt: Text description of desired music
            duration: Duration in seconds (default: 30)
            bpm: Optional BPM for tempo guidance
            genre: Optional genre for prompt enhancement
            output_path: Optional path to save generated audio

        Returns:
            Generated audio as numpy array (shape: [channels, samples])

        Raises:
            RuntimeError: If model is not loaded

        Complexity: O(duration * sample_rate) - proportional to audio length

        Example:
            >>> wrapper = MusicGenWrapper(model_size="small")
            >>> audio = wrapper.generate("dark phonk", duration=30)
            >>> print(audio.shape)  # (2, 1440000) for stereo 48kHz
        """
        if not self._model_loaded:
            self.load_model()

        # Enhance prompt with genre if specified
        enhanced_prompt = self._enhance_prompt(prompt, genre)
        logger.info(f"Generating: {enhanced_prompt}")

        # Generate in chunks for long durations
        audio = self._generate_chunked(enhanced_prompt, duration)

        # Save to file if path provided
        if output_path:
            self._save_audio(audio, output_path)
            logger.info(f"Saved to {output_path}")

        return audio

    def _enhance_prompt(self, prompt: str, genre: Optional[str]) -> str:
        """Enhance prompt with genre-specific characteristics.

        Args:
            prompt: Base prompt
            genre: Optional genre identifier

        Returns:
            Enhanced prompt string

        Complexity: O(1)
        """
        if not genre:
            return prompt

        from .prompts import enhance_prompt_for_genre
        return enhance_prompt_for_genre(prompt, genre)

    def _generate_chunked(self, prompt: str, duration: int) -> np.ndarray:
        """Generate audio in chunks for long durations.

        Args:
            prompt: Text prompt
            duration: Total duration in seconds

        Returns:
            Concatenated audio array

        Complexity: O(duration * sample_rate)
        """
        chunk_duration = 30  # MusicGen max is typically 30s
        num_chunks = max(1, (duration + chunk_duration - 1) // chunk_duration)

        if num_chunks == 1:
            return self._generate_single(prompt, duration)

        logger.info(f"Generating {duration}s in {num_chunks} chunks...")
        audio_chunks = []

        for i in tqdm(range(num_chunks), desc="Generating chunks"):
            chunk_dur = min(chunk_duration, duration - i * chunk_duration)
            chunk = self._generate_single(prompt, chunk_dur)
            audio_chunks.append(chunk)

        # Concatenate chunks with crossfade
        return self._crossfade_chunks(audio_chunks, crossfade_ms=500)

    def _generate_single(self, prompt: str, duration: int) -> np.ndarray:
        """Generate a single chunk of audio.

        Args:
            prompt: Text prompt
            duration: Duration in seconds

        Returns:
            Generated audio array

        Complexity: O(duration * sample_rate)
        """
        self.model.set_generation_params(duration=duration)

        with torch.no_grad():
            wav = self.model.generate([prompt])

        # Convert to numpy: [batch, channels, samples] -> [channels, samples]
        return wav[0].cpu().numpy()

    def _crossfade_chunks(self, chunks: list, crossfade_ms: int = 500) -> np.ndarray:
        """Crossfade audio chunks for smooth concatenation.

        Args:
            chunks: List of audio arrays
            crossfade_ms: Crossfade duration in milliseconds

        Returns:
            Crossfaded audio array

        Complexity: O(total_samples)
        """
        if len(chunks) == 1:
            return chunks[0]

        crossfade_samples = int(crossfade_ms * self.sample_rate / 1000)
        result = chunks[0]

        for i, chunk in enumerate(chunks[1:], start=1):
            # Fade out end of previous, fade in start of current
            fade_out = np.linspace(1, 0, crossfade_samples)
            fade_in = np.linspace(0, 1, crossfade_samples)

            # Apply crossfade
            result_end = result[:, -crossfade_samples:]
            chunk_start = chunk[:, :crossfade_samples]

            crossfaded = result_end * fade_out + chunk_start * fade_in

            # Concatenate
            result = np.concatenate([
                result[:, :-crossfade_samples],
                crossfaded,
                chunk[:, crossfade_samples:],
            ], axis=1)

        return result

    def _save_audio(self, audio: np.ndarray, path: Path) -> None:
        """Save audio array to file.

        Args:
            audio: Audio array [channels, samples]
            path: Output file path

        Complexity: O(audio_size)
        """
        import soundfile as sf

        # Ensure directory exists
        path.parent.mkdir(parents=True, exist_ok=True)

        # Convert to [samples, channels] for soundfile
        audio_transposed = audio.T

        sf.write(
            str(path),
            audio_transposed,
            self.sample_rate,
            format="WAV",
            subtype="PCM_16",
        )

    def unload_model(self) -> None:
        """Unload model from memory to free resources.

        Complexity: O(1)
        """
        if self.model is not None:
            del self.model
            self.model = None
            self._model_loaded = False

            if self.device == "cuda":
                torch.cuda.empty_cache()

            logger.info("Model unloaded from memory")


def create_musicgen_wrapper(
    model_size: str = "small",
    device: str = "cpu",
    sample_rate: int = 48000,
) -> MusicGenWrapper:
    """Factory function to create MusicGen wrapper.

    Args:
        model_size: Model size (small, medium, large)
        device: Target device (cpu, cuda)
        sample_rate: Audio sample rate in Hz

    Returns:
        Initialized MusicGenWrapper instance

    Complexity: O(1)
    """
    return MusicGenWrapper(
        model_size=model_size,
        device=device,
        sample_rate=sample_rate,
    )
