# contrib/backend/music-generator/music_gen/bark.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Bark text-to-audio model wrapper from Suno AI.

Bark is a transformer-based text-to-audio model that can generate:
- Highly realistic multilingual speech
- Music and sound effects
- Non-verbal communications (laughing, sighing, crying)

MIT License - Fully open source.
"""

import torch
import numpy as np
from pathlib import Path
from typing import Optional
import logging

logger = logging.getLogger(__name__)


class BarkGenerator:
    """Wrapper for Bark text-to-audio model.

    Generates audio from text using Suno AI's Bark model.
    Can produce speech, music, and sound effects.

    Attributes:
        model_size: Size of Bark model ("small" or "large")
        device: Target device (cpu or cuda)
        sample_rate: Audio sample rate in Hz (default: 24000)
        _pipeline: Loaded transformers pipeline
        _model_loaded: Whether model is currently loaded

    Example:
        >>> gen = BarkGenerator(model_size="small", device="cpu")
        >>> audio = gen.generate("140 BPM dark trap beat")
        >>> gen.save(audio, "output.wav")
    """

    def __init__(
        self,
        model_size: str = "small",
        device: str = "cpu",
        sample_rate: int = 24000,
    ):
        """Initialize Bark generator.

        Args:
            model_size: Size of model ("small" or "large")
            device: Target device (cpu, cuda)
            sample_rate: Audio sample rate in Hz (Bark uses 24kHz)

        Complexity: O(1) initialization, O(model_size) for loading
        """
        if model_size not in ["small", "large"]:
            raise ValueError(f"model_size must be 'small' or 'large', got {model_size}")

        self.model_size = model_size
        self.device = device
        self.sample_rate = sample_rate
        self._pipeline = None
        self._model_loaded = False

        # Model IDs for HuggingFace
        self._model_id = f"suno/bark{'' if model_size == 'large' else '-small'}"

    def load_model(self) -> None:
        """Load Bark model from HuggingFace.

        Raises:
            ImportError: If transformers is not installed
            RuntimeError: If model loading fails

        Complexity: O(model_size) - small is ~80M params, large ~300M
        """
        if self._model_loaded:
            return

        logger.info(f"Loading Bark model ({self.model_size}) from {self._model_id} on {self.device}...")

        try:
            from transformers import pipeline

            self._pipeline = pipeline(
                "text-to-audio",
                model=self._model_id,
                device="cpu",  # Force CPU to avoid MPS channel limit on Mac
                dtype=torch.float32,
            )

            self._model_loaded = True
            logger.info(f"Bark model ({self.model_size}) loaded on CPU")

        except ImportError as e:
            logger.error(f"Failed to import transformers: {e}")
            raise ImportError(
                "transformers required for Bark. Install: pip install transformers scipy"
            )
        except Exception as e:
            logger.error(f"Failed to load Bark model: {e}")
            raise

    def generate(
        self,
        prompt: str,
        seed: Optional[int] = None,
        do_sample: bool = True,
        temperature: float = 0.7,
        **forward_params,
    ) -> np.ndarray:
        """Generate audio from text prompt.

        Args:
            prompt: Text description of desired audio/music
            seed: Random seed for reproducibility
            do_sample: Whether to use sampling (True for creative, False for deterministic)
            temperature: Sampling temperature (higher = more random, lower = more deterministic)
            **forward_params: Additional parameters for the model

        Returns:
            Generated audio as numpy array (mono)

        Raises:
            RuntimeError: If model is not loaded

        Complexity: O(n) where n is number of tokens generated

        Example:
            >>> gen = BarkGenerator()
            >>> gen.load_model()
            >>> audio = gen.generate("A drum beat", seed=42)
        """
        if not self._model_loaded:
            self.load_model()

        logger.info(f"Generating Bark audio: '{prompt}'")

        # Set seed for reproducibility
        generator = None
        if seed is not None:
            torch.manual_seed(seed)
            generator = torch.Generator(self.device).manual_seed(seed)

        try:
            # Generate using Bark pipeline
            # Note: Bark pipeline doesn't accept do_sample directly in newer transformers
            result = self._pipeline(
                prompt,
                **forward_params,
            )

            # Extract audio from result
            if isinstance(result, dict):
                audio = result.get("audio", None)
                if audio is None and "outputs" in result:
                    audio = result["outputs"]
            else:
                audio = result

            # Convert to numpy if tensor
            if isinstance(audio, torch.Tensor):
                audio = audio.cpu().numpy()

            return audio

        except Exception as e:
            logger.error(f"Bark generation failed: {e}")
            raise

    def generate_music(
        self,
        prompt: str,
        style_hints: Optional[str] = None,
        seed: Optional[int] = None,
        **kwargs,
    ) -> np.ndarray:
        """Generate music with style hints.

        Bark can generate music, but it's optimized for speech.
        Use style hints to guide toward music generation.

        Args:
            prompt: Description of music style
            style_hints: Additional musical context (BPM, genre, instruments)
            seed: Random seed
            **kwargs: Additional generation parameters

        Returns:
            Generated audio

        Example:
            >>> gen = BarkGenerator()
            >>> audio = gen.generate_music(
            ...     "trap beat",
            ...     style_hints="140 BPM with heavy bass"
            ... )
        """
        # Build enhanced prompt
        if style_hints:
            full_prompt = f"[music] {prompt}. {style_hints}."
        else:
            full_prompt = f"[music] {prompt}."

        return self.generate(full_prompt, seed=seed, **kwargs)

    def generate_vocal(
        self,
        text: str,
        voice_preset: Optional[str] = None,
        seed: Optional[int] = None,
    ) -> np.ndarray:
        """Generate speech/vocal from text.

        Bark excels at realistic speech generation.

        Args:
            text: Text to speak
            voice_preset: Voice characteristics (e.g., "male", "female", "deep")
            seed: Random seed

        Returns:
            Generated speech audio

        Example:
            >>> gen = BarkGenerator()
            >>> audio = gen.generate_vocal(
            ...     "Check out this new track"
            ... )
        """
        # Add vocal hints
        if voice_preset:
            prompt = f"[speech] {text}. Voice: {voice_preset}."
        else:
            prompt = f"[speech] {text}."

        return self.generate(prompt, seed=seed)

    def save(self, audio: np.ndarray, output_path: Path) -> None:
        """Save generated audio to file.

        Args:
            audio: Audio array (mono)
            output_path: Path to save output file (.wav recommended)

        Raises:
            ImportError: If soundfile or scipy is not installed
        """
        try:
            import soundfile as sf
        except ImportError:
            raise ImportError(
                "soundfile required for saving audio. Install: pip install soundfile"
            )

        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Ensure 1D array for mono
        if audio.ndim > 1:
            audio = audio.squeeze()

        sf.write(str(output_path), audio, self.sample_rate)
        logger.info(f"Audio saved to {output_path}")

    def unload_model(self) -> None:
        """Unload model from memory to free resources.

        Complexity: O(1)
        """
        if self._pipeline is not None:
            del self._pipeline
            self._pipeline = None

        self._model_loaded = False

        if self.device == "cuda":
            torch.cuda.empty_cache()

        logger.info("Bark model unloaded from memory")

    def get_model_info(self) -> dict:
        """Get information about loaded model.

        Returns:
            Dictionary containing model information

        Complexity: O(1)
        """
        return {
            "model_id": self._model_id,
            "model_size": self.model_size,
            "device": self.device,
            "sample_rate": self.sample_rate,
            "loaded": self._model_loaded,
            "license": "MIT",
        }


def create_bark_generator(
    model_size: str = "small",
    device: str = "cpu",
    sample_rate: int = 24000,
) -> BarkGenerator:
    """Factory function to create Bark generator.

    Args:
        model_size: Size of model ("small" or "large")
        device: Target device (cpu, cuda)
        sample_rate: Audio sample rate in Hz

    Returns:
        Initialized BarkGenerator instance

    Complexity: O(1)
    """
    return BarkGenerator(
        model_size=model_size,
        device=device,
        sample_rate=sample_rate,
    )
