# contrib/backend/music-generator/lightweight/mixer.py
# Simple audio mixer (no external dependencies)
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Simple audio mixer using only numpy.

Provides basic mixing and mastering functionality without
requiring external audio processing libraries.
"""

import numpy as np
from pathlib import Path
from typing import Optional, Tuple
import logging

logger = logging.getLogger(__name__)


class SimpleMixer:
    """Simple audio mixer for combining tracks.

    Provides mixing, ducking, level matching, and basic mastering.

    Attributes:
        sample_rate: Audio sample rate in Hz
        vocal_level_db: Default vocal level in dB
        limiter_threshold: Limiter threshold in dB TP

    Example:
        >>> mixer = SimpleMixer(sample_rate=48000)
        >>> mix = mixer.mix(vocals, instrumental, vocal_level_db=-4)
        >>> mastered = mixer.master(mix)
    """

    def __init__(
        self,
        sample_rate: int = 48000,
        vocal_level_db: float = -4.0,
        limiter_threshold: float = -0.3,
    ):
        """Initialize mixer.

        Args:
            sample_rate: Audio sample rate in Hz
            vocal_level_db: Default vocal level in dB
            limiter_threshold: Limiter threshold in dB TP

        Complexity: O(1)
        """
        self.sample_rate = sample_rate
        self.vocal_level_db = vocal_level_db
        self.limiter_threshold = limiter_threshold

    def mix(
        self,
        vocals: np.ndarray,
        instrumental: np.ndarray,
        vocal_level_db: Optional[float] = None,
        ducking: bool = True,
        ducking_db: float = -3.0,
    ) -> np.ndarray:
        """Mix vocals with instrumental.

        Args:
            vocals: Vocal audio [2, samples] or [samples]
            instrumental: Instrumental audio [2, samples] or [samples]
            vocal_level_db: Vocal level in dB (uses default if None)
            ducking: Apply ducking to instrumental
            ducking_db: Ducking amount in dB

        Returns:
            Mixed audio [2, samples]

        Complexity: O(n)

        Example:
            >>> mix = mixer.mix(vocals, instrumental, vocal_level_db=-2)
        """
        if vocal_level_db is None:
            vocal_level_db = self.vocal_level_db

        # Ensure stereo
        vocals = self._ensure_stereo(vocals)
        instrumental = self._ensure_stereo(instrumental)

        # Match lengths
        vocals, instrumental = self._match_lengths(vocals, instrumental)

        # Apply vocal gain
        vocal_gain = 10 ** (vocal_level_db / 20)
        vocals = vocals * vocal_gain

        # Apply ducking if requested
        if ducking:
            instrumental = self._apply_ducking(instrumental, vocals, ducking_db)

        # Mix
        mix = vocals + instrumental

        logger.debug(f"Mixed vocals at {vocal_level_db}dB")
        return mix

    def master(
        self,
        audio: np.ndarray,
        target_lufs: float = -14.0,
        stereo_width: float = 1.0,
    ) -> np.ndarray:
        """Master the mix.

        Args:
            audio: Input audio [2, samples] or [samples]
            target_lufs: Target LUFS level
            stereo_width: Stereo width (0.5 to 2.0)

        Returns:
            Mastered audio

        Complexity: O(n)

        Example:
            >>> mastered = mixer.master(mix, target_lufs=-14)
        """
        # Ensure stereo
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)

        # Apply stereo width
        if stereo_width != 1.0:
            audio = self._adjust_stereo_width(audio, stereo_width)

        # Auto-level
        audio = self._auto_level(audio, target_lufs)

        # Limiter
        audio = self._limit(audio, self.limiter_threshold)

        return audio

    def _ensure_stereo(self, audio: np.ndarray) -> np.ndarray:
        """Ensure audio is stereo."""
        if audio.ndim == 1:
            return np.vstack([audio, audio])
        elif audio.ndim == 2:
            if audio.shape[0] == 2:
                return audio
            elif audio.shape[1] == 2:
                return audio.T
            else:
                # Take first 2 channels
                return audio[:2, :]
        return audio

    def _match_lengths(
        self,
        audio1: np.ndarray,
        audio2: np.ndarray,
    ) -> Tuple[np.ndarray, np.ndarray]:
        """Match lengths of two audio arrays."""
        max_length = max(audio1.shape[1], audio2.shape[1])

        if audio1.shape[1] < max_length:
            padding = np.zeros((2, max_length - audio1.shape[1]))
            audio1 = np.hstack([audio1, padding])

        if audio2.shape[1] < max_length:
            padding = np.zeros((2, max_length - audio2.shape[1]))
            audio2 = np.hstack([audio2, padding])

        return audio1, audio2

    def _apply_ducking(
        self,
        instrumental: np.ndarray,
        vocals: np.ndarray,
        ducking_db: float,
    ) -> np.ndarray:
        """Apply sidechain ducking."""
        # Calculate vocal envelope
        vocal_envelope = np.abs(vocals).mean(axis=0)

        # Smooth envelope
        window_size = int(0.05 * self.sample_rate)
        if window_size > 1:
            vocal_envelope = np.convolve(
                vocal_envelope,
                np.ones(window_size) / window_size,
                mode='same',
            )

        # Normalize
        if vocal_envelope.max() > 0:
            vocal_envelope = vocal_envelope / vocal_envelope.max()

        # Calculate ducking gain
        ducking_gain = 10 ** (ducking_db / 20)
        ducking_envelope = 1.0 - (vocal_envelope * (1.0 - ducking_gain))

        return instrumental * ducking_envelope[np.newaxis, :]

    def _auto_level(self, audio: np.ndarray, target_lufs: float) -> np.ndarray:
        """Auto-level to target LUFS."""
        rms = np.sqrt(np.mean(audio ** 2))

        if rms == 0:
            return audio

        # Calculate required gain
        target_rms = 10 ** ((target_lufs + 20) / 20) * 0.1
        gain = target_rms / rms

        # Limit gain
        gain = max(gain, -6.0)  # -6dB minimum
        gain = min(gain, 6.0)   # +6dB maximum

        gain_linear = 10 ** (gain / 20)
        leveled = audio * gain_linear

        return leveled

    def _adjust_stereo_width(self, audio: np.ndarray, width: float) -> np.ndarray:
        """Adjust stereo width."""
        mid = (audio[0] + audio[1]) / 2.0
        side = (audio[0] - audio[1]) / 2.0

        side = side * width

        left = mid + side
        right = mid - side

        return np.vstack([left, right])

    def _limit(self, audio: np.ndarray, threshold_db: float) -> np.ndarray:
        """Apply true peak limiting."""
        threshold_linear = 10 ** (threshold_db / 20)
        peak = np.max(np.abs(audio))

        if peak > threshold_linear:
            gain = threshold_linear / peak
            audio = audio * gain

        return audio

    def analyze(self, audio: np.ndarray) -> dict:
        """Analyze audio levels.

        Args:
            audio: Input audio

        Returns:
            Dictionary with level metrics

        Complexity: O(n)

        Example:
            >>> info = mixer.analyze(audio)
            >>> print(f"Peak: {info['peak_db']} dB")
        """
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)

        peak_linear = np.max(np.abs(audio))
        peak_db = 20 * np.log10(peak_linear) if peak_linear > 0 else -np.inf

        rms_linear = np.sqrt(np.mean(audio ** 2))
        rms_db = 20 * np.log10(rms_linear) if rms_linear > 0 else -np.inf

        lufs_approx = rms_db + 6

        return {
            "peak_db": peak_db,
            "rms_db": rms_db,
            "lufs_approx": lufs_approx,
            "dynamic_range": peak_db - rms_db if peak_db > -np.inf else 0,
        }

    def save(
        self,
        audio: np.ndarray,
        path: Path,
    ) -> None:
        """Save audio to file.

        Args:
            audio: Audio [2, samples]
            path: Output path

        Complexity: O(n)
        """
        path.parent.mkdir(parents=True, exist_ok=True)

        try:
            import soundfile as sf
            sf.write(str(path), audio.T, self.sample_rate)
        except ImportError:
            logger.warning("soundfile not available, save skipped")


def create_simple_mixer(
    sample_rate: int = 48000,
    vocal_level_db: float = -4.0,
) -> SimpleMixer:
    """Factory function to create mixer.

    Args:
        sample_rate: Audio sample rate in Hz
        vocal_level_db: Default vocal level in dB

    Returns:
        SimpleMixer instance

    Complexity: O(1)
    """
    return SimpleMixer(
        sample_rate=sample_rate,
        vocal_level_db=vocal_level_db,
    )
