# contrib/backend/music-generator/lightweight/generative.py
# Procedural audio generation (no ML)
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Procedural audio generation using only numpy.

Generates music through algorithmic methods - no ML models required.
"""

import numpy as np
from typing import Optional, Tuple, List
from pathlib import Path
import logging

logger = logging.getLogger(__name__)

# Golden ratio for generative patterns
PHI = (1 + 5 ** 0.5) / 2


class ProceduralGenerator:
    """Procedural music generator using algorithmic methods.

    Generates beats, basslines, and melodies using mathematical patterns
    based on the golden ratio and other constants.

    Attributes:
        sample_rate: Audio sample rate in Hz
        seed: Random seed for reproducibility

    Example:
        >>> gen = ProceduralGenerator()
        >>> beat = gen.generate_beat(duration=30, bpm=140, style="phonk")
    """

    def __init__(self, sample_rate: int = 48000, seed: Optional[int] = None):
        """Initialize procedural generator.

        Args:
            sample_rate: Audio sample rate in Hz
            seed: Random seed for reproducibility

        Complexity: O(1)
        """
        self.sample_rate = sample_rate
        self.seed = seed
        self._rng = np.random.RandomState(seed)

    def generate_beat(
        self,
        duration: float,
        bpm: float,
        style: str = "phonk",
        kick_pattern: Optional[List[int]] = None,
        snare_pattern: Optional[List[int]] = None,
        hihat_pattern: Optional[List[int]] = None,
    ) -> np.ndarray:
        """Generate a drum beat.

        Args:
            duration: Duration in seconds
            bpm: Beats per minute
            style: Beat style (phonk, trap, hip_hop, drill, lofi)
            kick_pattern: Custom kick pattern (16 steps)
            snare_pattern: Custom snare pattern (16 steps)
            hihat_pattern: Custom hi-hat pattern (16 steps)

        Returns:
            Stereo audio array [2, samples]

        Complexity: O(n) where n is number of samples

        Example:
            >>> beat = gen.generate_beat(duration=30, bpm=140, style="phonk")
        """
        samples = int(duration * self.sample_rate)
        beat_len = int(60 / bpm * self.sample_rate)

        # Generate patterns based on style
        kick_pat = kick_pattern or self._get_kick_pattern(style)
        snare_pat = snare_pattern or self._get_snare_pattern(style)
        hihat_pat = hihat_pattern or self._get_hihat_pattern(style)

        # Generate individual drum sounds
        kick = self._generate_kick_sound(style)
        snare = self._generate_snare_sound(style)
        hihat = self._generate_hihat_sound(style)

        # Pattern audio buffers
        audio = np.zeros((2, samples))

        # Apply patterns
        steps = 16
        step_samples = beat_len // 4  # 16th notes

        for step in range(int(samples / step_samples)):
            step_idx = step % steps

            # Kick
            if kick_pat[step_idx]:
                start = step * step_samples
                end = min(start + len(kick), samples)
                if start < samples:
                    kick_len = min(len(kick), end - start)
                    audio[:, start:start + kick_len] += kick[:, :kick_len] * 0.8

            # Snare
            if snare_pat[step_idx]:
                start = step * step_samples
                end = min(start + len(snare), samples)
                if start < samples:
                    snare_len = min(len(snare), end - start)
                    audio[:, start:start + snare_len] += snare[:, :snare_len] * 0.7

            # Hi-hat
            if hihat_pat[step_idx]:
                start = step * step_samples
                end = min(start + len(hihat), samples)
                if start < samples:
                    hihat_len = min(len(hihat), end - start)
                    audio[:, start:start + hihat_len] += hihat[:, :hihat_len] * 0.3

        # Normalize
        audio = self._normalize(audio)

        logger.debug(f"Generated {style} beat: {duration}s @ {bpm} BPM")
        return audio

    def generate_bassline(
        self,
        duration: float,
        bpm: float,
        root_note: float = 55.0,
        style: str = "phonk",
    ) -> np.ndarray:
        """Generate a bassline using phi-based patterns.

        Args:
            duration: Duration in seconds
            bpm: Beats per minute
            root_note: Root note frequency in Hz
            style: Bassline style

        Returns:
            Stereo audio array [2, samples]

        Complexity: O(n)

        Example:
            >>> bass = gen.generate_bassline(duration=30, bpm=140, style="phonk")
        """
        samples = int(duration * self.sample_rate)
        beat_len = int(60 / bpm * self.sample_rate)

        # Phi-based sequence generator
        notes = self._generate_phi_sequence(root_note, style)

        audio = np.zeros((2, samples))
        t = np.linspace(0, duration, samples)

        for i, note_freq in enumerate(notes):
            start = i * beat_len
            if start >= samples:
                break

            note_duration = min(beat_len, samples - start)
            note_t = t[start:start + int(note_duration)]

            # Generate 808-style bass with glide
            glide_amount = note_freq * 0.2 if style in ["phonk", "trap"] else 0
            freq = note_freq + glide_amount * (1 - note_t / note_t[-1])

            # Sawtooth-like waveform for 808
            waveform = 2 * (freq * note_t % 1) - 1

            # Envelope
            envelope = np.exp(-3 * (note_t / note_t[-1]))  # Decay
            envelope = np.maximum(0, envelope - 0.5 * (note_t / note_t[-1]))

            bass_note = waveform * envelope * 0.6

            end = min(start + len(bass_note), samples)
            audio[:, start:end] += bass_note[np.newaxis, :end - start]

        # Apply low-pass filter for 808 sound
        audio = self._lowpass_filter(audio, 200)

        audio = self._normalize(audio)
        return audio

    def generate_melody(
        self,
        duration: float,
        bpm: float,
        scale: str = "minor",
        style: str = "eerie",
    ) -> np.ndarray:
        """Generate a procedural melody.

        Args:
            duration: Duration in seconds
            bpm: Beats per minute
            scale: Musical scale (minor, major, pentatonic)
            style: Melody style

        Returns:
            Stereo audio array [2, samples]

        Complexity: O(n)

        Example:
            >>> melody = gen.generate_melody(duration=30, bpm=140, scale="minor")
        """
        samples = int(duration * self.sample_rate)
        t = np.linspace(0, duration, samples)

        # Generate scale notes
        scale_freqs = self._get_scale_freqs(220.0, scale)

        # Phi-based melody generation
        melody = np.zeros(samples)

        for i in range(int(duration * bpm / 60 * 8)):  # 8th notes
            note_idx = int((i * PHI) % len(scale_freqs))
            freq = scale_freqs[note_idx]

            start = int(i * (samples / (duration * bpm / 60 * 8)))
            end = min(start + int(samples / (duration * bpm / 60 * 16)), samples)

            note_t = t[start:end]

            # Sine wave with vibrato
            vibrato = 1 + 0.02 * np.sin(2 * np.pi * 6 * note_t)
            waveform = np.sin(2 * np.pi * freq * note_t * vibrato)

            # Envelope
            note_length = end - start
            envelope = np.sin(np.pi * np.linspace(0, 1, note_length))

            melody[start:end] = waveform * envelope * 0.2

        # Stereo spread
        audio = np.vstack([melody, melody])

        # Apply effects based on style
        if style == "ethereal":
            audio = self._add_reverb(audio, decay=0.5)
        elif style == "eerie":
            audio = self._add_detune(audio, amount=10)

        return audio

    def _generate_kick_sound(self, style: str) -> np.ndarray:
        """Generate a kick drum sound."""
        length = int(0.2 * self.sample_rate)
        t = np.linspace(0, 0.2, length)

        # Frequency sweep from ~150Hz to ~50Hz
        freq = 150 * np.exp(-10 * t) + 50

        # Sine wave with frequency modulation
        waveform = np.sin(2 * np.pi * freq * t)

        # Amplitude envelope
        envelope = np.exp(-15 * t)
        envelope = np.maximum(0, envelope - 0.5 * (t / 0.2))

        kick = waveform * envelope
        return np.vstack([kick, kick])

    def _generate_snare_sound(self, style: str) -> np.ndarray:
        """Generate a snare drum sound."""
        length = int(0.15 * self.sample_rate)
        t = np.linspace(0, 0.15, length)

        # Tone component
        tone_freq = 200 if style == "phonk" else 250
        tone = np.sin(2 * np.pi * tone_freq * t) * np.exp(-20 * t)

        # Noise component
        noise = self._rng.randn(length)
        noise = self._bandpass_filter(noise, 1000, 3000)
        noise = noise * np.exp(-15 * t)

        snare = (tone + noise) * 0.5
        return np.vstack([snare, snare])

    def _generate_hihat_sound(self, style: str) -> np.ndarray:
        """Generate a hi-hat sound."""
        length = int(0.05 * self.sample_rate)
        noise = self._rng.randn(length)

        # High-pass filter for metallic sound
        hihat = self._highpass_filter(noise, 7000)

        # Short decay
        envelope = np.exp(-40 * np.linspace(0, 0.05, length))
        hihat = hihat * envelope * 0.3

        return np.vstack([hihat, hihat])

    def _get_kick_pattern(self, style: str) -> List[int]:
        """Get kick pattern for style (16 steps)."""
        patterns = {
            "phonk": [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            "trap": [1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0],
            "hip_hop": [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1],
            "drill": [1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0],
            "lofi": [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0],
        }
        return patterns.get(style, patterns["phonk"])

    def _get_snare_pattern(self, style: str) -> List[int]:
        """Get snare pattern for style (16 steps)."""
        patterns = {
            "phonk": [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
            "trap": [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
            "hip_hop": [0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0],
            "drill": [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1],
            "lofi": [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        }
        return patterns.get(style, patterns["phonk"])

    def _get_hihat_pattern(self, style: str) -> List[int]:
        """Get hi-hat pattern for style (16 steps)."""
        if style == "trap":
            return [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
        elif style == "drill":
            return [1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0]
        else:
            return [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0]

    def _generate_phi_sequence(self, root: float, style: str) -> List[float]:
        """Generate a note sequence based on phi."""
        # Phi-based interval ratios
        intervals = [1, PHI, PHI ** 2, 2, 2 / PHI, PHI / 2, 1.5]

        sequence = []
        for i in range(16):  # 16 notes
            idx = int((i * PHI) % len(intervals))
            freq = root * intervals[idx]
            sequence.append(freq)

        return sequence

    def _get_scale_freqs(self, root: float, scale: str) -> List[float]:
        """Get frequencies for a scale."""
        # Ratios for different scales
        scales = {
            "minor": [1, 9/8, 6/5, 4/3, 3/2, 8/5, 9/5, 2],  # Natural minor
            "major": [1, 9/8, 5/4, 4/3, 3/2, 5/3, 15/8, 2],  # Major
            "pentatonic": [1, 9/8, 5/4, 3/2, 5/3],  # Major pentatonic
        }

        ratios = scales.get(scale, scales["minor"])
        return [root * r for r in ratios]

    def _normalize(self, audio: np.ndarray) -> np.ndarray:
        """Normalize audio to prevent clipping."""
        peak = np.max(np.abs(audio))
        if peak > 0.99:
            audio = audio / peak * 0.95
        return audio

    def _lowpass_filter(self, audio: np.ndarray, cutoff: float) -> np.ndarray:
        """Simple low-pass filter using moving average."""
        # Handle both 1D and 2D audio
        if audio.ndim == 1:
            audio_2d = audio.reshape(1, -1)
            return self._lowpass_filter_2d(audio_2d, cutoff)[0]
        else:
            return self._lowpass_filter_2d(audio, cutoff)

    def _lowpass_filter_2d(self, audio: np.ndarray, cutoff: float) -> np.ndarray:
        """Low-pass filter for 2D audio."""
        # Simple FIR low-pass
        kernel_size = int(self.sample_rate / cutoff)
        kernel = np.ones(kernel_size) / kernel_size

        filtered = np.zeros_like(audio)
        for channel in range(audio.shape[0]):
            padded = np.pad(audio[channel], kernel_size // 2, mode='edge')
            conv_result = np.convolve(padded, kernel, mode='same')
            # Ensure same length
            filtered[channel] = conv_result[:audio.shape[1]]

        return filtered

    def _highpass_filter(self, audio: np.ndarray, cutoff: float) -> np.ndarray:
        """Simple high-pass filter."""
        # High-pass = original - low-pass
        lowpass = self._lowpass_filter(audio, cutoff)
        return audio - lowpass

    def _bandpass_filter(self, audio: np.ndarray, low: float, high: float) -> np.ndarray:
        """Band-pass filter."""
        # Band-pass = low-pass(high) + high-pass(low)
        lowpass = self._lowpass_filter(audio, high)
        highpass = self._highpass_filter(audio, low)
        return lowpass + highpass

    def _add_reverb(self, audio: np.ndarray, decay: float = 0.5) -> np.ndarray:
        """Add simple reverb effect."""
        delay_samples = int(self.sample_rate * 0.05)  # 50ms delay
        reverb = np.zeros_like(audio)

        for channel in range(2):
            reverb[channel, delay_samples:] += audio[channel, :-delay_samples] * decay

        return audio + reverb * 0.3

    def _add_detune(self, audio: np.ndarray, amount: float = 10) -> np.ndarray:
        """Add detune effect (chorus)."""
        delay = max(1, int(amount / self.sample_rate * 1000))  # samples

        detuned = np.zeros_like(audio)
        if delay < audio.shape[1]:
            detuned[:, delay:] = audio[:, :-delay] * 0.5

        return audio + detuned

    def save(self, audio: np.ndarray, path: Path) -> None:
        """Save audio to file.

        Args:
            audio: Audio array [2, samples]
            path: Output file path

        Complexity: O(n)
        """
        try:
            import soundfile as sf
            path.parent.mkdir(parents=True, exist_ok=True)
            sf.write(str(path), audio.T, self.sample_rate)
            logger.debug(f"Saved audio to {path}")
        except ImportError:
            logger.warning("soundfile not available, save skipped")


def create_procedural_generator(sample_rate: int = 48000) -> ProceduralGenerator:
    """Factory function to create procedural generator.

    Args:
        sample_rate: Audio sample rate in Hz

    Returns:
        ProceduralGenerator instance

    Complexity: O(1)
    """
    return ProceduralGenerator(sample_rate=sample_rate)
