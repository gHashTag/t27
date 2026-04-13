# contrib/backend/music-generator/voice_clone/preprocessing.py
# Audio preprocessing for voice cloning
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Audio preprocessing utilities for voice cloning.

Provides noise reduction, segmentation, and normalization functions
to prepare voice samples for RVC training and inference.
"""

import numpy as np
import librosa
import soundfile as sf
from pathlib import Path
from typing import Tuple, List, Optional
import logging

logger = logging.getLogger(__name__)


def preprocess_audio(
    audio_path: Path,
    output_dir: Optional[Path] = None,
    noise_reduce: bool = True,
    normalize: bool = True,
    sample_rate: int = 48000,
) -> np.ndarray:
    """Load and preprocess audio file for voice cloning.

    Args:
        audio_path: Path to input audio file
        output_dir: Optional directory to save processed audio
        noise_reduce: Whether to apply noise reduction
        normalize: Whether to normalize audio levels
        sample_rate: Target sample rate in Hz

    Returns:
        Preprocessed audio as numpy array

    Raises:
        FileNotFoundError: If audio file does not exist
        RuntimeError: If audio loading fails

    Complexity: O(n) where n is number of samples

    Example:
        >>> audio = preprocess_audio(Path("voice.wav"))
        >>> print(audio.shape)
    """
    if not audio_path.exists():
        raise FileNotFoundError(f"Audio file not found: {audio_path}")

    logger.info(f"Loading audio from {audio_path}")

    # Load audio with librosa
    audio, sr = librosa.load(str(audio_path), sr=sample_rate, mono=True)

    # Apply noise reduction if requested
    if noise_reduce:
        audio = reduce_noise(audio, sr)
        logger.info("Applied noise reduction")

    # Normalize audio if requested
    if normalize:
        audio = normalize_audio(audio)
        logger.info("Normalized audio levels")

    # Save processed audio if output directory specified
    if output_dir:
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"processed_{audio_path.name}"
        sf.write(str(output_path), audio, sample_rate)
        logger.info(f"Saved processed audio to {output_path}")

    return audio


def reduce_noise(audio: np.ndarray, sample_rate: int) -> np.ndarray:
    """Reduce background noise using spectral subtraction.

    Args:
        audio: Input audio array
        sample_rate: Audio sample rate in Hz

    Returns:
        Noise-reduced audio array

    Complexity: O(n log n) due to FFT operations

    Example:
        >>> clean_audio = reduce_noise(noisy_audio, 48000)
    """
    # Compute STFT
    stft = librosa.stft(audio)
    magnitude = np.abs(stft)
    phase = np.angle(stft)

    # Estimate noise from first 0.5 seconds (assuming silence)
    noise_frames = int(0.5 * sample_rate / 512)  # Assuming 512 hop
    if noise_frames < magnitude.shape[1]:
        noise_spectrum = np.mean(magnitude[:, :noise_frames], axis=1, keepdims=True)
    else:
        noise_spectrum = np.mean(magnitude, axis=1, keepdims=True)

    # Spectral subtraction with over-subtraction factor
    alpha = 2.0  # Over-subtraction factor
    beta = 0.01  # Spectral floor

    enhanced_magnitude = magnitude - alpha * noise_spectrum
    enhanced_magnitude = np.maximum(enhanced_magnitude, beta * magnitude)

    # Reconstruct signal
    enhanced_stft = enhanced_magnitude * np.exp(1j * phase)
    enhanced_audio = librosa.istft(enhanced_stft)

    return enhanced_audio


def segment_by_vocal_activity(
    audio: np.ndarray,
    sample_rate: int,
    min_segment_length: float = 2.0,
    max_silence_duration: float = 0.5,
) -> List[Tuple[int, int]]:
    """Segment audio by vocal activity detection.

    Args:
        audio: Input audio array
        sample_rate: Audio sample rate in Hz
        min_segment_length: Minimum segment length in seconds
        max_silence_duration: Maximum silence duration in seconds

    Returns:
        List of (start_sample, end_sample) tuples for speech segments

    Complexity: O(n) where n is number of samples

    Example:
        >>> segments = segment_by_vocal_activity(audio, 48000)
        >>> print(f"Found {len(segments)} speech segments")
    """
    # Compute short-time energy
    frame_length = 2048
    hop_length = 512

    frames = librosa.util.frame(audio, frame_length=frame_length, hop_length=hop_length)
    energy = np.sum(frames ** 2, axis=0)

    # Normalize energy
    energy = energy / (np.max(energy) + 1e-8)

    # Detect speech frames (simple energy-based VAD)
    speech_threshold = 0.01
    speech_frames = energy > speech_threshold

    # Convert to sample indices
    frame_indices = np.where(speech_frames)[0]

    if len(frame_indices) == 0:
        return []

    # Group consecutive speech frames
    segments = []
    start_idx = frame_indices[0]
    prev_idx = start_idx

    for idx in frame_indices[1:]:
        if idx - prev_idx > int(max_silence_duration * sample_rate / hop_length):
            # End of segment
            end_idx = prev_idx
            segments.append((start_idx, end_idx))
            start_idx = idx
        prev_idx = idx

    # Add final segment
    segments.append((start_idx, prev_idx))

    # Convert frame indices to sample indices and filter by min length
    min_frames = int(min_segment_length * sample_rate / hop_length)
    sample_segments = [
        (start * hop_length, (end + 1) * hop_length)
        for start, end in segments
        if end - start + 1 >= min_frames
    ]

    logger.info(f"Found {len(sample_segments)} speech segments")
    return sample_segments


def normalize_audio(audio: np.ndarray, target_dbfs: float = -20.0) -> np.ndarray:
    """Normalize audio to target dBFS level.

    Args:
        audio: Input audio array
        target_dbfs: Target level in dBFS (default: -20.0)

    Returns:
        Normalized audio array

    Complexity: O(n) where n is number of samples

    Example:
        >>> normalized = normalize_audio(audio, target_dbfs=-18.0)
    """
    # Avoid division by zero
    if np.max(np.abs(audio)) < 1e-8:
        return audio

    # Calculate current level
    rms = np.sqrt(np.mean(audio ** 2))

    if rms == 0:
        return audio

    # Calculate current dBFS
    current_dbfs = 20 * np.log10(rms)

    # Calculate gain
    gain = target_dbfs - current_dbfs
    gain_linear = 10 ** (gain / 20)

    # Apply gain with limiting to prevent clipping
    normalized = audio * gain_linear

    # Soft clip if necessary
    max_val = np.max(np.abs(normalized))
    if max_val > 0.99:
        normalized = normalized / max_val * 0.99

    return normalized


def extract_features(
    audio: np.ndarray,
    sample_rate: int,
    n_mfcc: int = 13,
) -> np.ndarray:
    """Extract MFCC features from audio for voice analysis.

    Args:
        audio: Input audio array
        sample_rate: Audio sample rate in Hz
        n_mfcc: Number of MFCC coefficients to extract

    Returns:
        MFCC feature array [n_mfcc, n_frames]

    Complexity: O(n log n) due to FFT operations

    Example:
        >>> features = extract_features(audio, 48000)
        >>> print(features.shape)
    """
    mfcc = librosa.feature.mfcc(
        y=audio,
        sr=sample_rate,
        n_mfcc=n_mfcc,
        n_fft=2048,
        hop_length=512,
    )
    return mfcc


def detect_silence(
    audio: np.ndarray,
    sample_rate: int,
    silence_threshold: float = 0.01,
    min_silence_duration: float = 0.3,
) -> List[Tuple[int, int]]:
    """Detect silence regions in audio.

    Args:
        audio: Input audio array
        sample_rate: Audio sample rate in Hz
        silence_threshold: Energy threshold for silence detection
        min_silence_duration: Minimum silence duration in seconds

    Returns:
        List of (start_sample, end_sample) tuples for silence regions

    Complexity: O(n) where n is number of samples

    Example:
        >>> silence_regions = detect_silence(audio, 48000)
    """
    frame_length = 2048
    hop_length = 512

    frames = librosa.util.frame(audio, frame_length=frame_length, hop_length=hop_length)
    energy = np.sum(frames ** 2, axis=0)

    # Normalize energy
    max_energy = np.max(energy)
    if max_energy > 0:
        energy = energy / max_energy

    # Detect silent frames
    silent_frames = energy < silence_threshold

    # Find contiguous silent regions
    silent_indices = np.where(silent_frames)[0]

    if len(silent_indices) == 0:
        return []

    silence_regions = []
    start_idx = silent_indices[0]
    prev_idx = start_idx
    min_frames = int(min_silence_duration * sample_rate / hop_length)

    for idx in silent_indices[1:]:
        if idx - prev_idx > 1:
            # End of silence region
            if prev_idx - start_idx + 1 >= min_frames:
                silence_regions.append((start_idx * hop_length, (prev_idx + 1) * hop_length))
            start_idx = idx
        prev_idx = idx

    # Add final region
    if prev_idx - start_idx + 1 >= min_frames:
        silence_regions.append((start_idx * hop_length, (prev_idx + 1) * hop_length))

    return silence_regions
