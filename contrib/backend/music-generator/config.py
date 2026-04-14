# contrib/backend/music-generator/config.py
# Configuration for Music Generator
# phi^2 + 1/phi^2 = 3 | TRINITY

from dataclasses import dataclass, field
from typing import Optional, Dict, Any
from pathlib import Path
import os


@dataclass
class MusicGenConfig:
    """Configuration for Music Generator.

    Attributes:
        model_path: Path to MusicGen model directory
        output_path: Path to output directory for generated audio
        temp_path: Path to temporary files directory
        model_size: MusicGen model size (small, medium, large)
        sample_rate: Audio sample rate in Hz
        chunk_duration: Chunk duration in seconds for long generation
        device: Device to use (cpu, cuda)
        vocal_level_db: Vocal level in dB relative to instrumental
        limiter_threshold: Limiter threshold in dB TP
        cache_models: Whether to cache loaded models
        batch_size: Batch size for generation
        genres: Genre-specific configuration
    """

    model_path: Path
    output_path: Path
    temp_path: Path
    model_size: str
    sample_rate: int
    chunk_duration: int
    device: str
    vocal_level_db: float
    limiter_threshold: float
    cache_models: bool
    batch_size: int
    genres: Dict[str, Dict[str, Any]] = field(default_factory=dict)

    @classmethod
    def from_env(cls) -> "MusicGenConfig":
        """Create configuration from environment variables.

        Environment Variables:
            MUSICGEN_MODEL_PATH: Path to model directory (default: ./models)
            MUSICGEN_OUTPUT_PATH: Output directory (default: ./outputs)
            MUSICGEN_TEMP_PATH: Temp directory (default: ./temp)
            MUSICGEN_MODEL_SIZE: Model size (default: small)
            MUSICGEN_SAMPLE_RATE: Sample rate in Hz (default: 48000)
            MUSICGEN_CHUNK_DURATION: Chunk duration in seconds (default: 30)
            MUSICGEN_DEVICE: Device to use (default: cpu)
            MUSICGEN_VOCAL_LEVEL_DB: Vocal level in dB (default: -4.0)
            MUSICGEN_LIMITER_THRESHOLD: Limiter threshold in dB (default: -0.3)
            MUSICGEN_CACHE_MODELS: Cache models (default: true)
            MUSICGEN_BATCH_SIZE: Batch size (default: 1)

        Returns:
            MusicGenConfig with defaults for unset values

        Complexity: O(1)
        """
        home = Path.home()

        model_path_env = os.getenv("MUSICGEN_MODEL_PATH")
        output_path_env = os.getenv("MUSICGEN_OUTPUT_PATH")
        temp_path_env = os.getenv("MUSICGEN_TEMP_PATH")
        model_size_env = os.getenv("MUSICGEN_MODEL_SIZE")
        sample_rate_env = os.getenv("MUSICGEN_SAMPLE_RATE")
        chunk_duration_env = os.getenv("MUSICGEN_CHUNK_DURATION")
        device_env = os.getenv("MUSICGEN_DEVICE")
        vocal_level_env = os.getenv("MUSICGEN_VOCAL_LEVEL_DB")
        limiter_threshold_env = os.getenv("MUSICGEN_LIMITER_THRESHOLD")
        cache_models_env = os.getenv("MUSICGEN_CACHE_MODELS")
        batch_size_env = os.getenv("MUSICGEN_BATCH_SIZE")

        model_path = Path(model_path_env) if model_path_env else Path.cwd() / "models"
        output_path = Path(output_path_env) if output_path_env else Path.cwd() / "outputs"
        temp_path = Path(temp_path_env) if temp_path_env else Path.cwd() / "temp"
        model_size = model_size_env if model_size_env else "small"
        sample_rate = int(sample_rate_env) if sample_rate_env else 48000
        chunk_duration = int(chunk_duration_env) if chunk_duration_env else 30
        device = device_env if device_env else "cpu"
        vocal_level_db = float(vocal_level_env) if vocal_level_env else -4.0
        limiter_threshold = float(limiter_threshold_env) if limiter_threshold_env else -0.3
        cache_models = cache_models_env.lower() == "true" if cache_models_env else True
        batch_size = int(batch_size_env) if batch_size_env else 1

        return cls(
            model_path=model_path,
            output_path=output_path,
            temp_path=temp_path,
            model_size=model_size,
            sample_rate=sample_rate,
            chunk_duration=chunk_duration,
            device=device,
            vocal_level_db=vocal_level_db,
            limiter_threshold=limiter_threshold,
            cache_models=cache_models,
            batch_size=batch_size,
            genres={
                "phonk": {
                    "prompt_template": "dark drift phonk, aggressive double-time cowbell, heavy 808 bass",
                    "bpm": 140,
                    "key": "minor",
                },
                "trap": {
                    "prompt_template": "hard trap, rolling hi-hats, 808 slides, dark atmosphere",
                    "bpm": 140,
                    "key": "minor",
                },
                "hip_hop": {
                    "prompt_template": "boom bap hip-hop, dusty drums, soul sample, classic feel",
                    "bpm": 90,
                    "key": "major",
                },
                "drill": {
                    "prompt_template": "UK drill, sliding 808s, dark melody, fast hi-hats",
                    "bpm": 140,
                    "key": "minor",
                },
                "lofi": {
                    "prompt_template": "lofi hip-hop, chill beats, vinyl crackle, nostalgic",
                    "bpm": 80,
                    "key": "major",
                },
            },
        )


def config_from_env() -> MusicGenConfig:
    """Create configuration from environment variables.

    Returns:
        MusicGenConfig with defaults for unset values

    Complexity: O(1)
    """
    return MusicGenConfig.from_env()


# Default configuration
DEFAULT_CONFIG = config_from_env()
