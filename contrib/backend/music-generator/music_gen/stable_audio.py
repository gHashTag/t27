# contrib/backend/music-generator/music_gen/stable_audio.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Stable Audio Open 1.0 wrapper from Stability AI.

Provides interface for high-quality text-to-music generation using
Stable Audio Open 1.0 model. CPU-optimized with GPU support.
"""

import torch
import numpy as np
from pathlib import Path
from typing import Optional, List, Dict, Any
import logging

logger = logging.getLogger(__name__)


class StableAudioGenerator:
    """Wrapper for Stable Audio Open 1.0 model.

    Generates high-quality stereo audio from text prompts using
    Stability AI's Stable Audio Open 1.0 model.

    Attributes:
        model_path: Path to model or HuggingFace model ID
        device: Target device (cpu or cuda)
        sample_rate: Audio sample rate in Hz (default: 44100)
        max_duration: Maximum generation duration in seconds (up to 47s)
        _model: Loaded diffusion model
        _config: Model configuration
        _model_loaded: Whether model is currently loaded

    Example:
        >>> gen = StableAudioGenerator(device="cpu")
        >>> audio = gen.generate("140 BPM dark trap beat", duration=30)
        >>> gen.save(audio, "output.wav")
    """

    def __init__(
        self,
        model_path: str = "stabilityai/stable-audio-open-1.0",
        device: str = "cpu",
        sample_rate: int = 44100,
        max_duration: float = 45.0,
    ):
        """Initialize Stable Audio generator.

        Args:
            model_path: HuggingFace model ID or local path
            device: Target device (cpu, cuda)
            sample_rate: Audio sample rate in Hz
            max_duration: Maximum generation duration in seconds (up to 47s)

        Complexity: O(1) initialization, O(model_size) for loading
        """
        self.model_path = model_path
        self.device = device
        self.sample_rate = sample_rate
        self.max_duration = max_duration
        self._model = None
        self._config = None
        self._model_loaded = False
        self._pipeline = None

    def load_model(self) -> None:
        """Load Stable Audio model and required dependencies.

        Raises:
            ImportError: If required dependencies are not installed
            RuntimeError: If model loading fails

        Complexity: O(model_size) - depends on model parameter count
        """
        if self._model_loaded:
            return

        logger.info(f"Loading Stable Audio model from {self.model_path} on {self.device}...")

        try:
            # Try using diffusers API first (easier, more compatible)
            try:
                from diffusers import StableAudioPipeline

                self._pipeline = StableAudioPipeline.from_pretrained(
                    self.model_path,
                    torch_dtype=torch.float32 if self.device == "cpu" else torch.float16,
                )
                self._pipeline = self._pipeline.to(self.device)
                self._model_loaded = True
                logger.info("Stable Audio loaded via diffusers")
                return

            except ImportError:
                logger.info("diffusers not available, trying stable-audio-tools...")
                pass

            # Fallback: Use stable-audio-tools directly
            from stable_audio_tools import get_pretrained_model
            from stable_audio_tools.inference.generation import generate_diffusion_cond

            self._model, self._config = get_pretrained_model(self.model_path)
            self._model = self._model.to(self.device)
            self._model_loaded = True
            logger.info("Stable Audio loaded via stable-audio-tools")

        except ImportError as e:
            logger.error(f"Failed to import Stable Audio dependencies: {e}")
            logger.info("Install with: pip install stable-audio-tools or pip install diffusers")
            raise ImportError(
                "Stable Audio dependencies not found. "
                "Install: pip install stable-audio-tools diffusers soundfile torchaudio"
            )
        except Exception as e:
            logger.error(f"Failed to load Stable Audio model: {e}")
            raise

    def generate(
        self,
        prompt: str,
        duration: float = 30.0,
        negative_prompt: Optional[str] = None,
        num_inference_steps: int = 200,
        cfg_scale: float = 7.0,
        seed: Optional[int] = None,
        num_waveforms: int = 1,
    ) -> np.ndarray:
        """Generate audio from text prompt.

        Args:
            prompt: Text description of desired audio/music
            duration: Output duration in seconds (max: max_duration)
            negative_prompt: What to avoid in generation
            num_inference_steps: Diffusion steps (higher = better quality, slower)
            cfg_scale: Classifier-free guidance scale (higher = more prompt adherence)
            seed: Random seed for reproducibility
            num_waveforms: Number of variants to generate

        Returns:
            Generated audio as numpy array (stereo if available)

        Raises:
            RuntimeError: If model is not loaded

        Complexity: O(n * steps) where n is number of samples

        Example:
            >>> gen = StableAudioGenerator()
            >>> gen.load_model()
            >>> audio = gen.generate(
            ...     "140 BPM dark trap beat with heavy 808 bass",
            ...     duration=30
            ... )
        """
        if not self._model_loaded:
            self.load_model()

        if duration > self.max_duration:
            logger.warning(
                f"Requested duration {duration}s exceeds max {self.max_duration}s. "
                f"Using {self.max_duration}s."
            )
            duration = self.max_duration

        logger.info(
            f"Generating audio: '{prompt}' ({duration}s, "
            f"{num_inference_steps} steps, cfg={cfg_scale})"
        )

        # Set seed for reproducibility
        generator = None
        if seed is not None:
            generator = torch.Generator(self.device).manual_seed(seed)

        # Try diffusers pipeline first
        if self._pipeline is not None:
            return self._generate_with_diffusers(
                prompt,
                duration,
                negative_prompt,
                num_inference_steps,
                cfg_scale,
                generator,
                num_waveforms,
            )
        else:
            return self._generate_with_tools(
                prompt,
                duration,
                negative_prompt,
                num_inference_steps,
                cfg_scale,
                seed,
                num_waveforms,
            )

    def _generate_with_diffusers(
        self,
        prompt: str,
        duration: float,
        negative_prompt: Optional[str],
        num_inference_steps: int,
        cfg_scale: float,
        generator: Optional[torch.Generator],
        num_waveforms: int,
    ) -> np.ndarray:
        """Generate using diffusers pipeline.

        Complexity: O(n * steps) diffusion sampling
        """
        try:
            output = self._pipeline(
                prompt,
                negative_prompt=negative_prompt or "Low quality, distorted, noisy.",
                num_inference_steps=num_inference_steps,
                audio_end_in_s=duration,
                num_waveforms_per_prompt=num_waveforms,
                generator=generator,
                guidance_scale=cfg_scale,
            )

            # Extract audio from output
            if hasattr(output, "audios"):
                audio = output.audios[0].cpu().numpy()
            elif hasattr(output, "audio"):
                audio = output.audio.cpu().numpy()
            else:
                audio = output[0].cpu().numpy()

            # Ensure stereo
            if audio.ndim == 1:
                audio = np.stack([audio, audio])

            return audio

        except Exception as e:
            logger.error(f"Diffusers generation failed: {e}")
            raise

    def _generate_with_tools(
        self,
        prompt: str,
        duration: float,
        negative_prompt: Optional[str],
        num_inference_steps: int,
        cfg_scale: float,
        seed: Optional[int],
        num_waveforms: int,
    ) -> np.ndarray:
        """Generate using stable-audio-tools.

        Complexity: O(n * steps) diffusion sampling
        """
        try:
            from stable_audio_tools.inference.generation import generate_diffusion_cond
            import torchaudio
            from einops import rearrange

            # Prepare conditioning
            conditioning = [{
                "prompt": prompt,
                "seconds_start": 0,
                "seconds_total": duration,
            }]

            sample_size = self._config.get("sample_size", 81920)

            # Set seed
            torch.manual_seed(seed) if seed is not None else None

            # Generate
            output = generate_diffusion_cond(
                self._model,
                steps=num_inference_steps,
                cfg_scale=cfg_scale,
                conditioning=conditioning,
                sample_size=sample_size,
                sigma_min=0.3,
                sigma_max=500,
                sampler_type="dpmpp-3m-sde",
                device=self.device,
            )

            # Process output
            if isinstance(output, dict) and "audio" in output:
                audio = output["audio"].cpu().numpy()
            elif isinstance(output, torch.Tensor):
                audio = output.cpu().numpy()
            else:
                audio = np.array(output)

            # Ensure stereo
            if audio.ndim == 1:
                audio = np.stack([audio, audio])

            return audio

        except Exception as e:
            logger.error(f"stable-audio-tools generation failed: {e}")
            raise

    def generate_with_voice(
        self,
        prompt: str,
        voice_audio: np.ndarray,
        duration: float = 30.0,
        voice_strength: float = 0.5,
        **kwargs,
    ) -> np.ndarray:
        """Generate audio with voice conditioning (experimental).

        Note: Stable Audio Open 1.0 does not support realistic vocal generation.
        This is a placeholder for future model updates.

        Args:
            prompt: Text description of desired music
            voice_audio: Voice sample array
            duration: Output duration in seconds
            voice_strength: How much voice influences generation (0.0-1.0)
            **kwargs: Additional generation parameters

        Returns:
            Generated audio (voice not actually used yet)
        """
        logger.warning(
            "Voice conditioning not supported by Stable Audio Open 1.0. "
            "Generating without voice. Future versions may support this."
        )

        # Generate music without voice for now
        music = self.generate(prompt, duration, **kwargs)

        return music

    def save(self, audio: np.ndarray, output_path: Path) -> None:
        """Save generated audio to file.

        Args:
            audio: Audio array (mono or stereo)
            output_path: Path to save output file (.wav recommended)

        Raises:
            ImportError: If soundfile is not installed
        """
        try:
            import soundfile as sf
        except ImportError:
            raise ImportError(
                "soundfile required for saving audio. "
                "Install: pip install soundfile"
            )

        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Ensure proper shape for stereo
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)
        elif audio.ndim == 2 and audio.shape[0] > audio.shape[1]:
            audio = audio.T

        sf.write(str(output_path), audio.T, self.sample_rate)
        logger.info(f"Audio saved to {output_path}")

    def unload_model(self) -> None:
        """Unload model from memory to free resources.

        Complexity: O(1)
        """
        if self._pipeline is not None:
            del self._pipeline
            self._pipeline = None

        if self._model is not None:
            del self._model
            self._model = None

        self._config = None
        self._model_loaded = False

        if self.device == "cuda":
            torch.cuda.empty_cache()

        logger.info("Stable Audio model unloaded from memory")

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
            "max_duration": self.max_duration,
            "loaded": self._model_loaded,
            "pipeline_type": "diffusers" if self._pipeline else "stable-audio-tools",
        }


def create_stable_audio_generator(
    model_path: str = "stabilityai/stable-audio-open-1.0",
    device: str = "cpu",
    sample_rate: int = 44100,
    max_duration: float = 45.0,
) -> StableAudioGenerator:
    """Factory function to create Stable Audio generator.

    Args:
        model_path: HuggingFace model ID or local path
        device: Target device (cpu, cuda)
        sample_rate: Audio sample rate in Hz
        max_duration: Maximum generation duration in seconds

    Returns:
        Initialized StableAudioGenerator instance

    Complexity: O(1)
    """
    return StableAudioGenerator(
        model_path=model_path,
        device=device,
        sample_rate=sample_rate,
        max_duration=max_duration,
    )
