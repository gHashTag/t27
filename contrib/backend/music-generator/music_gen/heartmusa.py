# contrib/backend/music-generator/music_gen/heartmusa.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""HeartMuLa model wrapper.

HeartMuLa is a family of open-source foundational music models with 4B parameters.
Released January 2026, Apache 2.0 licensed.

GitHub: https://github.com/HeartMuLa/heartlib
HuggingFace: https://huggingface.co/HeartMuLa

Components:
- HeartCLAP: Audio-text alignment model
- HeartCodec: High-quality music codec (12.5 kHz, low frame rate)
- HeartTranscriptor: Text-to-music transcription optimized for real music
- HeartMuLa: LLM-driven song generation with controllable conditions

Note: HeartMuLa excels at instrumental music generation but vocals are
still an active area of development.
"""

import torch
import numpy as np
from pathlib import Path
from typing import Optional, List, Dict, Any
import logging

logger = logging.getLogger(__name__)


class HeartMuLaGenerator:
    """Wrapper for HeartMuLa music generation model.

    Generates full songs up to 10 minutes in duration with controllable
    parameters like style, lyrics, tempo, key.

    Attributes:
        model_path: Path to HeartMuLa checkpoint or HF model ID
        device: Target device (cpu or cuda)
        sample_rate: Audio sample rate in Hz
        _model: Loaded HeartMuLa model
        _processor: HeartMuLa text processor
        _model_loaded: Whether model is currently loaded

    Example:
        >>> gen = HeartMuLaGenerator(device="cuda")
        >>> gen.generate(
        ...     prompt="140 BPM trap beat",
        ...     duration=60,
        ...     style="dark trap"
        ... )
    """
    def __init__(
        self,
        model_path: str = "HeartMuLa/HeartMuLa-mini",
        device: str = "cpu",
        sample_rate: int = 48000,
    ):
        """Initialize HeartMuLa generator.

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
        self._processor = None
        self._model_loaded = False
        self._text_encoder = None
        self._music_decoder = None

    def load_model(self) -> None:
        """Load HeartMuLa model from HuggingFace.

        Raises:
            ImportError: If required dependencies are not installed
            RuntimeError: If model loading fails

        Complexity: O(model_size) - 4B parameters
        """
        if self._model_loaded:
            return

        logger.info(f"Loading HeartMuLa model from {self.model_path} on {self.device}...")

        try:
            from heartmusalib import HeartMuLa

            # Load model
            self._model = HeartMuLa.from_pretrained(
                self.model_path,
                device_map={"": self.device},
            )

            self._model_loaded = True
            logger.info("HeartMuLa model loaded successfully")

        except ImportError as e:
            logger.error(f"Failed to import heartmusalib: {e}")
            raise ImportError(
                "heartmusalib required for HeartMuLa. "
                "Install: pip install heartmusalib"
            )
        except Exception as e:
            logger.error(f"Failed to load HeartMuLa model: {e}")
            raise

    def generate(
        self,
        prompt: str,
        duration: float = 60.0,
        style: Optional[str] = None,
        key: Optional[str] = None,
        tempo: Optional[float] = None,
        lyrics: Optional[str] = None,
        num_waves: int = 1,
        seed: Optional[int] = None,
        **kwargs,
    ) -> Dict[str, Any]:
        """Generate music from text prompt.

        Args:
            prompt: Text description of desired music
            duration: Song duration in seconds (up to 600s for HeartMuLa)
            style: Music style (trap, phonk, hiphop, etc.)
            key: Musical key (C, Dm, etc.)
            tempo: Tempo in BPM
            lyrics: Lyrics for the song
            num_waves: Number of variations to generate
            seed: Random seed for reproducibility
            **kwargs: Additional parameters for HeartMuLa

        Returns:
            Dictionary with generated audio and metadata

        Raises:
            RuntimeError: If model is not loaded

        Complexity: O(duration * sample_rate) - proportional to audio length

        Example:
            >>> gen = HeartMuLaGenerator()
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

        # Add optional parameters
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
            f"Generating HeartMuLa: '{prompt}' "
            f"({duration}s, style={style}, tempo={tempo})"
        )

        try:
            with torch.no_grad():
                result = self._model.generate(
                    **gen_params,
                    num_waves=num_waves,
                )

            # Extract audio
            if hasattr(result, "audios"):
                audio = result.audios[0]
                if isinstance(audio, torch.Tensor):
                    audio = audio.cpu().numpy()
            else:
                audio = result[0]
                if isinstance(audio, torch.Tensor):
                    audio = audio.cpu().numpy()
                else:
                    audio = np.array(audio)

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
            logger.error(f"HeartMuLa generation failed: {e}")
            raise

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
            HeartMuLa is still improving vocal synthesis.
            Instrumental music generation is more reliable.
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

        logger.info("HeartMuLa model unloaded from memory")

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
        }


def create_heartmusa_generator(
    model_path: str = "HeartMuLa/HeartMuLa-mini",
    device: str = "cpu",
    sample_rate: int = 48000,
) -> HeartMuLaGenerator:
    """Factory function to create HeartMuLa generator.

    Args:
        model_path: HF model ID or local path
        device: Target device (cpu, cuda)
        sample_rate: Audio sample rate in Hz

    Returns:
        Initialized HeartMuLaGenerator instance

    Complexity: O(1)
    """
    return HeartMuLaGenerator(
        model_path=model_path,
        device=device,
        sample_rate=sample_rate,
    )
