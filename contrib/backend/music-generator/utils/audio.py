# contrib/backend/music-generator/utils/audio.py
# Audio utility functions
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Audio loading, saving, and conversion utilities."""

import numpy as np
import librosa
import soundfile as sf
from pathlib import Path
from typing import Tuple, Optional, Dict, Any
import logging

logger = logging.getLogger(__name__)


def load_audio(
    path: Path,
    sample_rate: Optional[int] = None,
    mono: bool = True,
) -> Tuple[np.ndarray, int]:
    """Load audio file.

    Args:
        path: Path to audio file
        sample_rate: Target sample rate (None = use original)
        mono: Convert to mono if True

    Returns:
        Tuple of (audio_array, sample_rate)

    Raises:
        FileNotFoundError: If file does not exist
        RuntimeError: If audio loading fails

    Complexity: O(n) where n is number of samples

    Example:
        >>> audio, sr = load_audio(Path("track.wav"), sample_rate=48000)
        >>> print(f"Loaded {len(audio)} samples at {sr}Hz")
    """
    if not path.exists():
        raise FileNotFoundError(f"Audio file not found: {path}")

    try:
        audio, sr = librosa.load(
            str(path),
            sr=sample_rate,
            mono=mono,
        )
        return audio, sr
    except Exception as e:
        raise RuntimeError(f"Failed to load audio: {e}") from e


def save_audio(
    audio: np.ndarray,
    path: Path,
    sample_rate: int = 48000,
    format: str = "WAV",
    subtype: str = "PCM_24",
) -> None:
    """Save audio to file.

    Args:
        audio: Audio array (mono [n] or stereo [n, 2])
        path: Output file path
        sample_rate: Sample rate in Hz
        format: Audio format (WAV, FLAC, etc.)
        subtype: Audio subtype (PCM_16, PCM_24, FLOAT, etc.)

    Raises:
        ValueError: If audio shape is invalid
        RuntimeError: If saving fails

    Complexity: O(n) where n is number of samples

    Example:
        >>> save_audio(audio, Path("output.wav"), sample_rate=48000)
    """
    path.parent.mkdir(parents=True, exist_ok=True)

    # Ensure correct shape for soundfile
    if audio.ndim == 1:
        # Mono - soundfile expects [samples]
        audio_to_save = audio
    elif audio.ndim == 2:
        if audio.shape[0] == 2:
            # Stereo [channels, samples] -> transpose to [samples, channels]
            audio_to_save = audio.T
        elif audio.shape[1] == 2:
            # Already [samples, channels]
            audio_to_save = audio
        else:
            raise ValueError(f"Unsupported audio shape: {audio.shape}")
    else:
        raise ValueError(f"Unsupported audio dimensions: {audio.ndim}")

    try:
        sf.write(
            str(path),
            audio_to_save,
            sample_rate,
            format=format,
            subtype=subtype,
        )
        logger.debug(f"Saved audio to {path}")
    except Exception as e:
        raise RuntimeError(f"Failed to save audio: {e}") from e


def get_audio_info(path: Path) -> Dict[str, Any]:
    """Get information about audio file.

    Args:
        path: Path to audio file

    Returns:
        Dictionary with audio information

    Raises:
        FileNotFoundError: If file does not exist

    Complexity: O(1)

    Example:
        >>> info = get_audio_info(Path("track.wav"))
        >>> print(f"Duration: {info['duration']}s")
    """
    if not path.exists():
        raise FileNotFoundError(f"Audio file not found: {path}")

    try:
        info = sf.info(str(path))
        return {
            "samplerate": info.samplerate,
            "frames": info.frames,
            "channels": info.channels,
            "duration": info.duration,
            "format": info.format,
            "subtype": info.subtype,
        }
    except Exception as e:
        raise RuntimeError(f"Failed to get audio info: {e}") from e


def convert_format(
    input_path: Path,
    output_path: Path,
    target_sample_rate: Optional[int] = None,
    target_format: str = "WAV",
    target_subtype: str = "PCM_24",
) -> None:
    """Convert audio file to different format.

    Args:
        input_path: Input file path
        output_path: Output file path
        target_sample_rate: Target sample rate (None = keep original)
        target_format: Target format
        target_subtype: Target subtype

    Raises:
        FileNotFoundError: If input file does not exist
        RuntimeError: If conversion fails

    Complexity: O(n) where n is number of samples

    Example:
        >>> convert_format(
        ...     Path("input.mp3"),
        ...     Path("output.wav"),
        ...     target_sample_rate=48000
        ... )
    """
    audio, sr = load_audio(input_path, sample_rate=target_sample_rate, mono=False)

    save_audio(
        audio,
        output_path,
        sample_rate=sr,
        format=target_format,
        subtype=target_subtype,
    )

    logger.info(f"Converted {input_path} to {output_path}")


def ensure_sample_rate(
    audio: np.ndarray,
    current_sr: int,
    target_sr: int,
) -> Tuple[np.ndarray, int]:
    """Ensure audio is at target sample rate.

    Args:
        audio: Input audio array
        current_sr: Current sample rate
        target_sr: Target sample rate

    Returns:
        Tuple of (resampled_audio, target_sr)

    Complexity: O(n) where n is number of samples

    Example:
        >>> audio_48k, sr = ensure_sample_rate(audio_44k, 44100, 48000)
    """
    if current_sr == target_sr:
        return audio, target_sr

    resampled = librosa.resample(audio, orig_sr=current_sr, target_sr=target_sr)
    return resampled, target_sr


def split_channels(audio: np.ndarray) -> Tuple[np.ndarray, np.ndarray]:
    """Split stereo audio into left and right channels.

    Args:
        audio: Stereo audio array [n, 2] or [2, n]

    Returns:
        Tuple of (left_channel, right_channel)

    Raises:
        ValueError: If audio is not stereo

    Complexity: O(n)

    Example:
        >>> left, right = split_channels(stereo_audio)
    """
    if audio.ndim != 2:
        raise ValueError("Audio must be stereo")

    if audio.shape[0] == 2:
        left = audio[0]
        right = audio[1]
    elif audio.shape[1] == 2:
        left = audio[:, 0]
        right = audio[:, 1]
    else:
        raise ValueError(f"Audio must have 2 channels, got shape {audio.shape}")

    return left, right


def merge_channels(left: np.ndarray, right: np.ndarray) -> np.ndarray:
    """Merge left and right channels into stereo.

    Args:
        left: Left channel audio
        right: Right channel audio

    Returns:
        Stereo audio array [2, n]

    Raises:
        ValueError: If channels have different lengths

    Complexity: O(n)

    Example:
        >>> stereo = merge_channels(left, right)
    """
    if len(left) != len(right):
        raise ValueError("Channels must have the same length")

    return np.vstack([left, right])


def calculate_rms(audio: np.ndarray) -> float:
    """Calculate RMS level of audio.

    Args:
        audio: Audio array

    Returns:
        RMS level

    Complexity: O(n)

    Example:
        >>> rms = calculate_rms(audio)
        >>> rms_db = 20 * np.log10(rms)
    """
    return np.sqrt(np.mean(audio ** 2))


def calculate_peak(audio: np.ndarray) -> float:
    """Calculate peak level of audio.

    Args:
        audio: Audio array

    Returns:
        Peak level (linear)

    Complexity: O(n)

    Example:
        >>> peak = calculate_peak(audio)
        >>> peak_db = 20 * np.log10(peak)
    """
    return np.max(np.abs(audio))


def fade_in(audio: np.ndarray, duration: float, sample_rate: int) -> np.ndarray:
    """Apply fade-in to audio.

    Args:
        audio: Input audio array
        duration: Fade duration in seconds
        sample_rate: Sample rate in Hz

    Returns:
        Audio with fade-in applied

    Complexity: O(n)

    Example:
        >>> faded = fade_in(audio, duration=0.5, sample_rate=48000)
    """
    fade_samples = int(duration * sample_rate)
    fade_samples = min(fade_samples, len(audio))

    fade_curve = np.linspace(0, 1, fade_samples)
    result = audio.copy()
    result[:fade_samples] = audio[:fade_samples] * fade_curve

    return result


def fade_out(audio: np.ndarray, duration: float, sample_rate: int) -> np.ndarray:
    """Apply fade-out to audio.

    Args:
        audio: Input audio array
        duration: Fade duration in seconds
        sample_rate: Sample rate in Hz

    Returns:
        Audio with fade-out applied

    Complexity: O(n)

    Example:
        >>> faded = fade_out(audio, duration=1.0, sample_rate=48000)
    """
    fade_samples = int(duration * sample_rate)
    fade_samples = min(fade_samples, len(audio))

    fade_curve = np.linspace(1, 0, fade_samples)
    result = audio.copy()
    result[-fade_samples:] = audio[-fade_samples:] * fade_curve

    return result
