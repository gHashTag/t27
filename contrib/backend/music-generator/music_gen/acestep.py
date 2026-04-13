# contrib/backend/music-generator/music_gen/acestep.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""ACE-Step model wrapper.

ACE-Step is an open-source foundational model for music generation.
Released by ACE Studio, Apache 2.0 licensed.

Website: https://acestep.io
GitHub: https://github.com/ace-step/ace-step

Key Features:
- Generates music from text in 50+ languages
- Generates up to 10 minutes of music in ~20 seconds on A100 GPU
- Supports voice cloning, text editing, remixes
- 15x faster than LLM-based music generators
- Works on consumer hardware (self-hosted)
"""

import torch
import numpy as np
from pathlib import Path
from typing import Optional, Dict, Any
import logging

logger = logging.getLogger(__name__)


class ACEStepGenerator:
    """Wrapper for ACE-Step music generation model.

    Generates full songs with controllable parameters including
    style, vocals, lyrics, tempo, and key.

    Attributes:
        model_path: Path to ACE-Step checkpoint
        device: Target device (cpu or cuda)
        sample_rate: Audio sample rate in Hz
        _model: Loaded ACE-Step model
        _model_loaded: Whether model is currently loaded

    Example:
        >>> gen = ACEStepGenerator(device="cuda")
        >>> gen.generate(
        ...     "140 BPM dark trap beat",
        ...     duration=60,
        ... )
    """
    def __init__(
        self,
        model_path: str = "ace-step/ace-step-1.5",
        device: str = "cpu",
        sample_rate: int = 48000,
    ):
        """Initialize ACE-Step generator.

        Args:
            model_path: HF model ID or local path
            device: Target device (cpu, cuda)
            sample_rate: Audio sample rate in Hz

        Complexity: O(1) initialization, O(model_size) for loading
        """
        self.model_path = model_path
        self.device = device
        self.sample_rate = sample_rate
        self._model = None
        self._model_loaded = False

    def load_model(self) -> None:
        """Load ACE-Step model from HuggingFace or local.

        Raises:
            ImportError: If required dependencies are not installed
            RuntimeError: If model loading fails

        Complexity: O(model_size) - model parameter count
        """
        if self._model_loaded:
            return

        logger.info(f"Loading ACE-Step model from {self.model_path} on {self.device}...")

        try:
            # Try loading from transformers if available
            from transformers import AutoModel, AutoTokenizer
            from transformers import GenerationConfig

            logger.info("Loading via transformers...")

            self._model = AutoModel.from_pretrained(
                self.model_path,
                trust_remote_code=True,
                device_map={"": self.device},
            )

            self._model_loaded = True
            logger.info("ACE-Step model loaded successfully")

        except ImportError as e:
            logger.error(f"Failed to import transformers: {e}")
            raise ImportError(
                "transformers required for ACE-Step. Install: pip install transformers"
            )
        except Exception as e:
            logger.error(f"Failed to load ACE-Step model: {e}")
            raise

    def generate(
        self,
        prompt: str,
        duration: float = 60.0,
        style: Optional[str] = None,
        key: Optional[str] = None,
        tempo: Optional[float] = None,
        lyrics: Optional[str] = None,
        seed: Optional[int] = None,
        **kwargs,
    ) -> Dict[str, Any]:
        """Generate music from text prompt.

        Args:
            prompt: Text description of desired music
            duration: Song duration in seconds (up to 600s)
            style: Music style (trap, phonk, hiphop, etc.)
            key: Musical key (C, Dm, etc.)
            tempo: Tempo in BPM
            lyrics: Lyrics for song
            seed: Random seed for reproducibility
            **kwargs: Additional model parameters

        Returns:
            Dictionary with generated audio and metadata

        Raises:
            RuntimeError: If model is not loaded

        Complexity: O(duration * sample_rate) - proportional to audio length

        Example:
            >>> gen = ACEStepGenerator()
            >>> gen.load_model()
            >>> result = gen.generate(
            ...     "140 BPM dark trap beat",
            ...     duration=30,
            ...     style="trap"
            ... )
        """
        if not self._model_loaded:
            self.load_model()

        # Build generation parameters
        gen_params = {
            "text": prompt,
            "duration": duration,
        }

        if style:
            gen_params["genre"] = style
        if key:
            gen_params["key"] = key
        if tempo:
            gen_params["tempo"] = tempo
        if lyrics:
            gen_params["lyrics"] = lyrics
        if seed is not None:
            gen_params["seed"] = seed

        logger.info(
            f"Generating ACE-Step: '{prompt}' "
            f"({duration}s, style={style})"
        )

        try:
            with torch.no_grad():
                # Generate using model
                result = self._model.generate(
                    **gen_params,
                    **kwargs,
                )

            # Extract audio
            audio = self._extract_audio(result)

            # Return as dictionary
            output = {
                "audio": audio,
                "sample_rate": self.sample_rate,
                "duration": duration,
                "style": style,
                "key": key,
                "tempo": tempo,
                "prompt": prompt,
            }

            return output

        except Exception as e:
            logger.error(f"ACE-Step generation failed: {e}")
            raise

    def _extract_audio(self, result: Any) -> np.ndarray:
        """Extract audio tensor from model result.

        Args:
            result: Model output (format varies by model)

        Returns:
            Audio numpy array
        """
        if isinstance(result, dict):
            if "audio" in result:
                audio = result["audio"]
            elif "samples" in result:
                audio = result["samples"]
            else:
                audio = result
        elif isinstance(result, torch.Tensor):
            audio = result
        elif isinstance(result, (list, tuple)):
            audio = result[0] if result else np.array(result)
        else:
            audio = np.array(result)

        if isinstance(audio, torch.Tensor):
            audio = audio.cpu().numpy()

        return audio

    def generate_with_lyrics(
        self,
        lyrics: str,
        style: str = "pop",
        duration: float = 120.0,
        **kwargs,
    ) -> Dict[str, Any]:
        """Generate a full song with lyrics.

        Args:
            lyrics: Song lyrics text
            style: Music genre
            duration: Song duration in seconds
            **kwargs: Additional generation parameters

        Returns:
            Dictionary with generated audio and metadata

        Note:
            ACE-Step supports lyrics generation and vocal synthesis.
        """
        prompt = f"{style} song with lyrics: {lyrics}"
        return self.generate(
            prompt=prompt,
            style=style,
            duration=duration,
            **kwargs,
        )

    def save(self, audio: np.ndarray, output_path: Path) -> None:
        """Save generated audio to file.

        Args:
            audio: Audio array (channels, samples)
            output_path: Path to save output file (.wav recommended)

        Raises:
            ImportError: If soundfile is not installed
        """
        try:
            import soundfile as sf
        except ImportError:
            raise ImportError(
                "soundfile required for saving audio. Install: pip install soundfile"
            )

        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Ensure proper shape for stereo
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)
        elif audio.ndim == 2 and audio.shape[0] > audio.shape[1]:
            audio = audio.T

        sf.write(str(output_path), audio, self.sample_rate)
        logger.info(f"Audio saved to {output_path}")

    def unload_model(self) -> None:
        """Unload model from memory to free resources.

        Complexity: O(1)
        """
        if self._model is not None:
            del self._model
            self._model = None

        self._model_loaded = False

        if self.device == "cuda":
            torch.cuda.empty_cache()

        logger.info("ACE-Step model unloaded from memory")

    def get_model_info(self) -> Dict[str, Any]:
        """Get information about loaded model.

        Returns:
            Dictionary containing model information

        Complexity: O(1)
        """
        return {
            "model_path": self.model_path,
            "device": self.device,
            "sample_rate": self.sample_rate,
            "loaded": self._model_loaded,
            "license": "Apache 2.0",
            "max_duration": 600,  # Up to 10 minutes
            "features": "text-to-music, lyrics, voice cloning, remixes",
        }


def create_acestep_generator(
    model_path: str = "ace-step/ace-step-1.5",
    device: str = "cpu",
    sample_rate: int = 48000,
) -> ACEStepGenerator:
    """Factory function to create ACE-Step generator.

    Args:
        model_path: HF model ID or local path
        device: Target device (cpu, cuda)
        sample_rate: Audio sample rate in Hz

    Returns:
        Initialized ACE-StepGenerator instance

    Complexity: O(1)
    """
    return ACEStepGenerator(
        model_path=model_path,
        device=device,
        sample_rate=sample_rate,
    )
