# contrib/backend/music-generator/lightweight/effects.py
# Simple audio effects (no external dependencies)
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Simple audio effects using only numpy.

Provides basic audio processing without requiring pedalboard
or other external audio effect libraries.
"""

import numpy as np
from typing import Tuple
import logging

logger = logging.getLogger(__name__)


class SimpleEffects:
    """Simple audio effects processor using only numpy.

    Provides compression, reverb, delay, distortion, and EQ
    implemented purely with numpy arrays.

    Attributes:
        sample_rate: Audio sample rate in Hz

    Example:
        >>> fx = SimpleEffects(sample_rate=48000)
        >>> processed = fx.compress(audio, threshold=-20, ratio=4)
    """

    def __init__(self, sample_rate: int = 48000):
        """Initialize effects processor.

        Args:
            sample_rate: Audio sample rate in Hz

        Complexity: O(1)
        """
        self.sample_rate = sample_rate

    def compress(
        self,
        audio: np.ndarray,
        threshold_db: float = -20.0,
        ratio: float = 4.0,
        attack_ms: float = 5.0,
        release_ms: float = 50.0,
    ) -> np.ndarray:
        """Apply dynamic range compression.

        Args:
            audio: Input audio [2, samples] or [samples]
            threshold_db: Threshold in dB
            ratio: Compression ratio
            attack_ms: Attack time in milliseconds
            release_ms: Release time in milliseconds

        Returns:
            Compressed audio

        Complexity: O(n)

        Example:
            >>> compressed = fx.compress(audio, threshold=-20, ratio=4)
        """
        # Ensure 2D
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)

        channels, samples = audio.shape

        # Convert threshold to linear
        threshold_linear = 10 ** (threshold_db / 20)

        # Attack and release coefficients
        attack_coeff = np.exp(-1 / (attack_ms * self.sample_rate / 1000))
        release_coeff = np.exp(-1 / (release_ms * self.sample_rate / 1000))

        # Envelope follower
        envelope = np.zeros(samples)
        level = 0.0

        for i in range(samples):
            # Compute input level
            input_level = np.max(np.abs(audio[:, i]))

            # Envelope follower with attack/release
            if input_level > level:
                level = attack_coeff * level + (1 - attack_coeff) * input_level
            else:
                level = release_coeff * level + (1 - release_coeff) * input_level

            envelope[i] = level

        # Apply compression
        gain = np.ones(samples)

        for i in range(samples):
            if envelope[i] > threshold_linear:
                # Compress
                excess = envelope[i] / threshold_linear
                compressed_excess = excess ** (1 / ratio)
                gain[i] = 1 / compressed_excess

        # Apply gain
        processed = audio * gain[np.newaxis, :]

        return processed[0] if processed.shape[0] == 1 else processed

    def reverb(
        self,
        audio: np.ndarray,
        room_size: float = 0.5,
        decay: float = 0.5,
        wet_level: float = 0.3,
        dry_level: float = 0.7,
    ) -> np.ndarray:
        """Add reverb effect.

        Args:
            audio: Input audio [2, samples] or [samples]
            room_size: Simulated room size (0.0 to 1.0)
            decay: Reverb decay time in seconds
            wet_level: Wet signal level (0.0 to 1.0)
            dry_level: Dry signal level (0.0 to 1.0)

        Returns:
            Audio with reverb

        Complexity: O(n)

        Example:
            >>> with_reverb = fx.reverb(audio, decay=0.5)
        """
        # Ensure 2D
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)

        channels, samples = audio.shape

        # Multiple delay taps for reverb
        delays = [
            int(self.sample_rate * 0.03),
            int(self.sample_rate * 0.05),
            int(self.sample_rate * 0.07),
            int(self.sample_rate * 0.11),
        ]

        gains = [
            0.5 * decay,
            0.3 * decay,
            0.2 * decay,
            0.1 * decay,
        ]

        reverb = np.zeros_like(audio)

        for delay, gain in zip(delays, gains):
            reverb[:, delay:] += audio[:, :-delay] * gain

        # Mix wet and dry
        wet = reverb * wet_level
        dry = audio * dry_level

        return wet + dry

    def delay(
        self,
        audio: np.ndarray,
        delay_seconds: float = 0.25,
        feedback: float = 0.3,
        mix: float = 0.3,
    ) -> np.ndarray:
        """Add delay effect.

        Args:
            audio: Input audio [2, samples] or [samples]
            delay_seconds: Delay time in seconds
            feedback: Feedback amount (0.0 to 1.0)
            mix: Wet/dry mix (0.0 to 1.0)

        Returns:
            Audio with delay

        Complexity: O(n)

        Example:
            >>> delayed = fx.delay(audio, delay=0.25)
        """
        # Ensure 2D
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)

        channels, samples = audio.shape
        delay_samples = int(delay_seconds * self.sample_rate)

        wet = np.zeros_like(audio)
        delayed = np.zeros_like(audio)

        for i in range(samples):
            # Input + feedback
            input_sample = audio[:, i]
            if i >= delay_samples:
                feedback_sample = delayed[:, i - delay_samples] * feedback
            else:
                feedback_sample = 0

            delayed[:, i] = input_sample + feedback_sample
            wet[:, i] = delayed[:, i] if i >= delay_samples else 0

        # Mix
        return audio * (1 - mix) + wet * mix

    def distort(
        self,
        audio: np.ndarray,
        drive_db: float = 15.0,
        tone: float = 0.5,
    ) -> np.ndarray:
        """Add distortion effect.

        Args:
            audio: Input audio [2, samples] or [samples]
            drive_db: Drive amount in dB
            tone: Tone control (0.0 = dark, 1.0 = bright)

        Returns:
            Distorted audio

        Complexity: O(n)

        Example:
            >>> distorted = fx.distort(audio, drive_db=15)
        """
        # Ensure 2D
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)

        # Convert drive to linear gain
        drive_linear = 10 ** (drive_db / 20)

        # Apply drive (gain + soft clipping)
        driven = audio * drive_linear

        # Soft clipping (tanh)
        clipped = np.tanh(driven)

        # Tone control (simple low-pass)
        if tone < 1.0:
            cutoff = 2000 + tone * 8000
            clipped = self._lowpass_filter(clipped, cutoff)

        # Normalize
        peak = np.max(np.abs(clipped))
        if peak > 0.99:
            clipped = clipped / peak * 0.95

        return clipped[0] if clipped.shape[0] == 1 else clipped

    def eq(
        self,
        audio: np.ndarray,
        low_db: float = 0.0,
        mid_db: float = 0.0,
        high_db: float = 0.0,
    ) -> np.ndarray:
        """Apply 3-band EQ.

        Args:
            audio: Input audio [2, samples] or [samples]
            low_db: Low band gain in dB
            mid_db: Mid band gain in dB
            high_db: High band gain in dB

        Returns:
            Equalized audio

        Complexity: O(n)

        Example:
            >>> equalized = fx.eq(audio, low_db=3, high_db=-2)
        """
        # Ensure 2D
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)

        # Convert to frequency domain
        fft_result = np.fft.fft(audio, axis=1)
        freqs = np.fft.fftfreq(audio.shape[1], 1 / self.sample_rate)

        # Band gains
        low_gain = 10 ** (low_db / 20)
        mid_gain = 10 ** (mid_db / 20)
        high_gain = 10 ** (high_db / 20)

        # Apply gains based on frequency
        for i, freq in enumerate(freqs):
            if freq < 250:
                gain = low_gain
            elif freq < 4000:
                gain = mid_gain
            else:
                gain = high_gain

            fft_result[:, i] *= gain

        # Convert back to time domain
        result = np.fft.ifft(fft_result, axis=1).real

        return result[0] if result.shape[0] == 1 else result

    def _lowpass_filter(self, audio: np.ndarray, cutoff: float) -> np.ndarray:
        """Simple low-pass filter."""
        kernel_size = int(self.sample_rate / cutoff)
        if kernel_size < 1:
            kernel_size = 1

        kernel = np.ones(kernel_size) / kernel_size

        filtered = np.zeros_like(audio)
        for channel in range(audio.shape[0]):
            padded = np.pad(audio[channel], kernel_size // 2, mode='edge')
            conv_result = np.convolve(padded, kernel, mode='same')
            # Slice to match original audio length
            filtered[channel] = conv_result[:audio.shape[1]]

        return filtered

    def limit(self, audio: np.ndarray, threshold_db: float = -0.3) -> np.ndarray:
        """Apply true peak limiting.

        Args:
            audio: Input audio
            threshold_db: Threshold in dB TP

        Returns:
            Limited audio

        Complexity: O(n)

        Example:
            >>> limited = fx.limit(audio, threshold_db=-0.3)
        """
        threshold_linear = 10 ** (threshold_db / 20)
        peak = np.max(np.abs(audio))

        if peak > threshold_linear:
            gain = threshold_linear / peak
            audio = audio * gain

        return audio


def create_simple_effects(sample_rate: int = 48000) -> SimpleEffects:
    """Factory function to create effects processor.

    Args:
        sample_rate: Audio sample rate in Hz

    Returns:
        SimpleEffects instance

    Complexity: O(1)
    """
    return SimpleEffects(sample_rate=sample_rate)
