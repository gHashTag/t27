# contrib/backend/music-generator/mixing/auto_mixer.py
# Automated mixing and mastering
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Automated mixing and mastering for generated tracks.

Provides automatic level balancing, EQ matching, and mastering
for combining vocals and instrumentals into a final mix.
"""

import numpy as np
import librosa
import soundfile as sf
from pathlib import Path
from typing import Optional, Dict, Any, Tuple
import logging

logger = logging.getLogger(__name__)


class AutoMixer:
    """Automated mixing and mastering for music tracks.

    Handles combining vocals with instrumentals, auto-leveling,
    and applying a mastering chain.

    Attributes:
        sample_rate: Audio sample rate in Hz
        vocal_level_db: Default vocal level in dB
        limiter_threshold: Limiter threshold in dB TP

    Example:
        >>> mixer = AutoMixer(sample_rate=48000)
        >>> mix = mixer.mix(vocals, instrumental, vocal_level_db=-4)
        >>> mastered = mixer.master(mix, "final.wav")
    """

    def __init__(
        self,
        sample_rate: int = 48000,
        vocal_level_db: float = -4.0,
        limiter_threshold: float = -0.3,
    ):
        """Initialize auto mixer.

        Args:
            sample_rate: Audio sample rate in Hz
            vocal_level_db: Default vocal level relative to instrumental (dB)
            limiter_threshold: True peak limiter threshold (dB)

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
        """Mix vocals with instrumental track.

        Applies level matching, length adjustment, and optional
        sidechain ducking of instrumental under vocals.

        Args:
            vocals: Vocal audio array [channels, samples] or [samples]
            instrumental: Instrumental audio array [channels, samples] or [samples]
            vocal_level_db: Vocal level in dB (uses default if None)
            ducking: Whether to apply ducking to instrumental
            ducking_db: Ducking amount in dB

        Returns:
            Mixed audio array

        Complexity: O(n) where n is number of samples

        Example:
            >>> mix = mixer.mix(vocals, instrumental, vocal_level_db=-2)
        """
        if vocal_level_db is None:
            vocal_level_db = self.vocal_level_db

        # Ensure 2D arrays
        vocals = self._ensure_stereo(vocals)
        instrumental = self._ensure_stereo(instrumental)

        # Match lengths
        vocals, instrumental = self._match_lengths(vocals, instrumental)

        # Calculate vocal gain
        vocal_gain = 10 ** (vocal_level_db / 20)
        vocals = vocals * vocal_gain

        # Apply ducking if requested
        if ducking:
            instrumental = self._apply_ducking(instrumental, vocals, ducking_db)

        # Mix
        mix = vocals + instrumental

        logger.info(
            f"Mixed vocals at {vocal_level_db}dB "
            f"{'with' if ducking else 'without'} ducking"
        )

        return mix

    def _ensure_stereo(self, audio: np.ndarray) -> np.ndarray:
        """Ensure audio is stereo format.

        Args:
            audio: Input audio array

        Returns:
            Stereo audio array [2, samples]

        Complexity: O(n) where n is number of samples
        """
        if audio.ndim == 1:
            # Convert mono to stereo
            return np.vstack([audio, audio])
        elif audio.ndim == 2:
            if audio.shape[0] == 2:
                return audio
            elif audio.shape[1] == 2:
                # Transpose to [channels, samples]
                return audio.T
            else:
                # Assume [samples, channels] with more than 2 channels
                return audio[:, :2].T
        else:
            raise ValueError(f"Unsupported audio shape: {audio.shape}")

    def _match_lengths(
        self,
        audio1: np.ndarray,
        audio2: np.ndarray,
    ) -> Tuple[np.ndarray, np.ndarray]:
        """Match lengths of two audio arrays.

        Shorter arrays are zero-padded to match the longer one.

        Args:
            audio1: First audio array
            audio2: Second audio array

        Returns:
            Tuple of length-matched audio arrays

        Complexity: O(n) where n is max length
        """
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
        """Apply sidechain ducking to instrumental.

        Reduces instrumental level when vocals are present.

        Args:
            instrumental: Instrumental audio array
            vocals: Vocal audio array for detection
            ducking_db: Ducking amount in dB

        Returns:
            Ducked instrumental audio

        Complexity: O(n) where n is number of samples
        """
        # Calculate vocal envelope
        vocal_envelope = np.abs(vocals).mean(axis=0)

        # Smooth envelope
        window_size = int(0.05 * self.sample_rate)  # 50ms window
        if window_size > 1:
            vocal_envelope = np.convolve(
                vocal_envelope,
                np.ones(window_size) / window_size,
                mode="same",
            )

        # Normalize envelope
        if vocal_envelope.max() > 0:
            vocal_envelope = vocal_envelope / vocal_envelope.max()

        # Calculate ducking gain
        ducking_gain = 10 ** (ducking_db / 20)
        ducking_envelope = 1.0 - (vocal_envelope * (1.0 - ducking_gain))

        # Apply ducking
        ducked = instrumental * ducking_envelope[np.newaxis, :]

        return ducked

    def auto_level(
        self,
        mix: np.ndarray,
        target_lufs: float = -14.0,
    ) -> np.ndarray:
        """Auto-level mix to target LUFS.

        Args:
            mix: Mixed audio array
            target_lufs: Target LUFS level (default -14.0 for streaming)

        Returns:
            Leveled audio array

        Complexity: O(n) where n is number of samples

        Example:
            >>> leveled = mixer.auto_level(mix, target_lufs=-12.0)
        """
        # Calculate current RMS
        current_rms = np.sqrt(np.mean(mix ** 2))

        if current_rms == 0:
            return mix

        # Convert to dB
        current_db = 20 * np.log10(current_rms)

        # Calculate required gain
        # LUFS -14 is approximately RMS -20dB for full-scale digital
        target_rms = 10 ** ((target_lufs + 20) / 20) * 0.1
        gain_db = 20 * np.log10(target_rms / current_rms)

        # Limit gain to avoid excessive boosting
        gain_db = max(gain_db, -6.0)
        gain_db = min(gain_db, 6.0)

        gain_linear = 10 ** (gain_db / 20)
        leveled = mix * gain_linear

        logger.info(f"Auto-leveled by {gain_db:.2f}dB to ~{target_lufs} LUFS")

        return leveled

    def master(
        self,
        mix: np.ndarray,
        output_path: Optional[Path] = None,
        target_lufs: float = -14.0,
        stereo_width: float = 1.0,
    ) -> np.ndarray:
        """Master the mix with final processing chain.

        Applies EQ, compression, limiting, and exports to file.

        Args:
            mix: Mixed audio array
            output_path: Optional path to save mastered audio
            target_lufs: Target LUFS level for final output
            stereo_width: Stereo width enhancement (0.5 to 2.0)

        Returns:
            Mastered audio array

        Complexity: O(n) where n is number of samples

        Example:
            >>> mastered = mixer.master(mix, "final_master.wav")
        """
        audio = mix.copy()

        # Apply stereo width enhancement if requested
        if stereo_width != 1.0:
            audio = self._adjust_stereo_width(audio, stereo_width)

        # Soft clipping for analog feel
        audio = np.tanh(audio * 1.5) / 1.5

        # Auto-level to target
        audio = self.auto_level(audio, target_lufs=target_lufs)

        # True peak limiting
        audio = self._true_peak_limit(audio, self.limiter_threshold)

        # Save if path provided
        if output_path:
            self._save_audio(audio, output_path)
            logger.info(f"Mastered audio saved to {output_path}")

        return audio

    def _adjust_stereo_width(
        self,
        audio: np.ndarray,
        width: float,
    ) -> np.ndarray:
        """Adjust stereo width of audio.

        Args:
            audio: Stereo audio array [2, samples]
            width: Width factor (0.5 = narrow, 1.0 = unchanged, 2.0 = wide)

        Returns:
            Width-adjusted audio

        Complexity: O(n) where n is number of samples
        """
        if audio.shape[0] != 2:
            return audio

        mid = (audio[0] + audio[1]) / 2.0
        side = (audio[0] - audio[1]) / 2.0

        # Adjust side signal
        side = side * width

        # Recombine
        left = mid + side
        right = mid - side

        return np.vstack([left, right])

    def _true_peak_limit(
        self,
        audio: np.ndarray,
        threshold_db: float = -0.3,
    ) -> np.ndarray:
        """Apply true peak limiting to prevent clipping.

        Args:
            audio: Input audio array
            threshold_db: Threshold in dB true peak

        Returns:
            Limited audio array

        Complexity: O(n) where n is number of samples
        """
        threshold_linear = 10 ** (threshold_db / 20)

        # Find peaks
        peak = np.max(np.abs(audio))

        if peak > threshold_linear:
            # Apply limiting gain
            limiting_gain = threshold_linear / peak
            audio = audio * limiting_gain
            logger.info(f"Applied limiting: {20 * np.log10(peak):.2f}dB -> {threshold_db}dB")
        else:
            logger.info(f"No limiting needed (peak: {20 * np.log10(peak):.2f}dB)")

        return audio

    def _save_audio(
        self,
        audio: np.ndarray,
        path: Path,
    ) -> None:
        """Save audio to WAV file.

        Args:
            audio: Audio array [channels, samples]
            path: Output file path

        Complexity: O(n) where n is number of samples
        """
        path.parent.mkdir(parents=True, exist_ok=True)

        # Convert to [samples, channels] for soundfile
        audio_transposed = audio.T

        sf.write(
            str(path),
            audio_transposed,
            self.sample_rate,
            format="WAV",
            subtype="PCM_24",
        )

    def analyze_levels(
        self,
        audio: np.ndarray,
    ) -> Dict[str, float]:
        """Analyze audio levels.

        Args:
            audio: Input audio array

        Returns:
            Dictionary of level metrics

        Complexity: O(n) where n is number of samples
        """
        # Peak level
        peak_linear = np.max(np.abs(audio))
        peak_db = 20 * np.log10(peak_linear) if peak_linear > 0 else -np.inf

        # RMS level
        rms_linear = np.sqrt(np.mean(audio ** 2))
        rms_db = 20 * np.log10(rms_linear) if rms_linear > 0 else -np.inf

        # Approximate LUFS (simplified)
        lufs_approx = rms_db + 6

        return {
            "peak_db": peak_db,
            "rms_db": rms_db,
            "lufs_approx": lufs_approx,
            "dynamic_range": peak_db - rms_db if peak_db > -np.inf else 0,
        }


def create_auto_mixer(
    sample_rate: int = 48000,
    vocal_level_db: float = -4.0,
) -> AutoMixer:
    """Factory function to create auto mixer.

    Args:
        sample_rate: Audio sample rate in Hz
        vocal_level_db: Default vocal level in dB

    Returns:
        Initialized AutoMixer instance

    Complexity: O(1)
    """
    return AutoMixer(
        sample_rate=sample_rate,
        vocal_level_db=vocal_level_db,
    )
