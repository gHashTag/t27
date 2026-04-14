# contrib/backend/music-generator/effects/processor.py
# Audio effects processing using pedalboard
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Audio effects processing using Spotify Pedalboard.

Provides vocal and instrumental effect chains for different
musical genres and styles.
"""

import numpy as np
from pathlib import Path
from typing import List, Optional, Dict, Any, Union
import logging

logger = logging.getLogger(__name__)


try:
    from pedalboard import (
        Pedalboard,
        Compressor,
        Reverb,
        Delay,
        Chorus,
        Distortion,
        HighpassFilter,
        LowpassFilter,
        PeakFilter,
        Gain,
        Limiter,
        Phaser,
        Tremolo,
        NoiseGate,
    )
    PEDALBOARD_AVAILABLE = True
except ImportError:
    PEDALBOARD_AVAILABLE = False
    logger.warning("pedalboard not available, effects will be limited")

    # Stub classes for when pedalboard is not available
    class Pedalboard:
        def __init__(self, plugins, sample_rate):
            self.plugins = plugins
            self.sample_rate = sample_rate

        def process(self, audio):
            return audio

    class Compressor:
        def __init__(self, **kwargs):
            pass

    class Reverb:
        def __init__(self, **kwargs):
            pass

    class Delay:
        def __init__(self, **kwargs):
            pass

    class Chorus:
        def __init__(self, **kwargs):
            pass

    class Distortion:
        def __init__(self, **kwargs):
            pass

    class HighpassFilter:
        def __init__(self, **kwargs):
            pass

    class LowpassFilter:
        def __init__(self, **kwargs):
            pass

    class PeakFilter:
        def __init__(self, **kwargs):
            pass

    class Gain:
        def __init__(self, **kwargs):
            pass

    class Limiter:
        def __init__(self, **kwargs):
            pass

    class Phaser:
        def __init__(self, **kwargs):
            pass

    class Tremolo:
        def __init__(self, **kwargs):
            pass

    class NoiseGate:
        def __init__(self, **kwargs):
            pass


class EffectChain:
    """Reusable effect chain configuration.

    Attributes:
        name: Chain name
        plugins: List of effect plugins
        sample_rate: Audio sample rate

    Example:
        >>> chain = EffectChain("phonk_vocal", sample_rate=48000)
        >>> chain.add_reverb(room_size=0.5)
        >>> chain.add_delay(delay_seconds=0.25)
        >>> processed = chain.process(audio)
    """

    def __init__(self, name: str, sample_rate: int = 48000):
        """Initialize effect chain.

        Args:
            name: Chain name for identification
            sample_rate: Audio sample rate in Hz

        Complexity: O(1)
        """
        self.name = name
        self.sample_rate = sample_rate
        self.plugins: List[Any] = []
        self._board: Optional[Pedalboard] = None

    def add_compressor(
        self,
        threshold_db: float = -20.0,
        ratio: float = 4.0,
        attack_ms: float = 5.0,
        release_ms: float = 50.0,
    ) -> "EffectChain":
        """Add compressor to chain.

        Args:
            threshold_db: Threshold in dB
            ratio: Compression ratio
            attack_ms: Attack time in milliseconds
            release_ms: Release time in milliseconds

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(Compressor(
            threshold_db=threshold_db,
            ratio=ratio,
            attack_ms=attack_ms,
            release_ms=release_ms,
        ))
        return self

    def add_reverb(
        self,
        room_size: float = 0.5,
        damping: float = 0.5,
        wet_level: float = 0.3,
        dry_level: float = 0.7,
        width: float = 1.0,
    ) -> "EffectChain":
        """Add reverb to chain.

        Args:
            room_size: Room size (0.0 to 1.0)
            damping: Damping factor (0.0 to 1.0)
            wet_level: Wet signal level (0.0 to 1.0)
            dry_level: Dry signal level (0.0 to 1.0)
            width: Stereo width (0.0 to 1.0)

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(Reverb(
            room_size=room_size,
            damping=damping,
            wet_level=wet_level,
            dry_level=dry_level,
            width=width,
        ))
        return self

    def add_delay(
        self,
        delay_seconds: float = 0.5,
        feedback: float = 0.3,
        mix: float = 0.3,
    ) -> "EffectChain":
        """Add delay to chain.

        Args:
            delay_seconds: Delay time in seconds
            feedback: Feedback amount (0.0 to 1.0)
            mix: Wet/dry mix (0.0 to 1.0)

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(Delay(
            delay_seconds=delay_seconds,
            feedback=feedback,
            mix=mix,
        ))
        return self

    def add_distortion(
        self,
        drive_db: float = 25.0,
    ) -> "EffectChain":
        """Add distortion to chain.

        Args:
            drive_db: Drive amount in dB

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(Distortion(drive_db=drive_db))
        return self

    def add_highpass(
        self,
        cutoff_frequency_hz: float = 100.0,
    ) -> "EffectChain":
        """Add high-pass filter to chain.

        Args:
            cutoff_frequency_hz: Cutoff frequency in Hz

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(HighpassFilter(cutoff_frequency_hz=cutoff_frequency_hz))
        return self

    def add_lowpass(
        self,
        cutoff_frequency_hz: float = 8000.0,
    ) -> "EffectChain":
        """Add low-pass filter to chain.

        Args:
            cutoff_frequency_hz: Cutoff frequency in Hz

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(LowpassFilter(cutoff_frequency_hz=cutoff_frequency_hz))
        return self

    def add_gain(
        self,
        gain_db: float = 0.0,
    ) -> "EffectChain":
        """Add gain to chain.

        Args:
            gain_db: Gain in dB

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(Gain(gain_db=gain_db))
        return self

    def add_limiter(
        self,
        threshold_db: float = -0.3,
        release_ms: float = 100.0,
    ) -> "EffectChain":
        """Add limiter to chain.

        Args:
            threshold_db: Threshold in dB
            release_ms: Release time in milliseconds

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(Limiter(
            threshold_db=threshold_db,
            release_ms=release_ms,
        ))
        return self

    def add_chorus(
        self,
        rate_hz: float = 1.5,
        depth: float = 0.5,
        centre_delay_ms: float = 7.0,
        feedback: float = 0.25,
        mix: float = 0.5,
    ) -> "EffectChain":
        """Add chorus to chain.

        Args:
            rate_hz: Modulation rate in Hz
            depth: Modulation depth (0.0 to 1.0)
            centre_delay_ms: Center delay in milliseconds
            feedback: Feedback amount (0.0 to 1.0)
            mix: Wet/dry mix (0.0 to 1.0)

        Returns:
            Self for chaining

        Complexity: O(1)
        """
        self.plugins.append(Chorus(
            rate_hz=rate_hz,
            depth=depth,
            centre_delay_ms=centre_delay_ms,
            feedback=feedback,
            mix=mix,
        ))
        return self

    def build(self) -> Pedalboard:
        """Build the pedalboard from configured plugins.

        Returns:
            Configured Pedalboard instance

        Complexity: O(n) where n is number of plugins
        """
        self._board = Pedalboard(self.plugins, sample_rate=self.sample_rate)
        return self._board

    def process(self, audio: np.ndarray) -> np.ndarray:
        """Process audio through effect chain.

        Args:
            audio: Input audio array [channels, samples] or [samples]

        Returns:
            Processed audio array

        Complexity: O(n) where n is number of samples
        """
        if self._board is None:
            self.build()

        # Ensure 2D array for stereo processing
        if audio.ndim == 1:
            audio = audio.reshape(1, -1)

        return self._board.process(audio)


class VocalProcessor:
    """Processor for vocal audio with genre-specific presets.

    Provides presets for different vocal delivery styles:
    - Aggressive verse processing (distortion, compression)
    - Ethereal chorus processing (reverb, chorus, delay)
    - Smooth processing (compression, EQ)

    Attributes:
        sample_rate: Audio sample rate in Hz
        default_chain: Default effect chain

    Example:
        >>> processor = VocalProcessor(sample_rate=48000)
        >>> processed = processor.process_verses(vocal_audio)
    """

    def __init__(self, sample_rate: int = 48000):
        """Initialize vocal processor.

        Args:
            sample_rate: Audio sample rate in Hz

        Complexity: O(1)
        """
        self.sample_rate = sample_rate

    def process_verses(
        self,
        audio: np.ndarray,
        intensity: float = 1.0,
    ) -> np.ndarray:
        """Process verse vocals with aggressive style.

        Applies compression, distortion, and EQ for punchy,
        in-your-face vocal delivery.

        Args:
            audio: Input vocal audio
            intensity: Effect intensity (0.0 to 2.0)

        Returns:
            Processed vocal audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = VocalProcessor()
            >>> processed = processor.process_verses(vocal, intensity=1.2)
        """
        chain = EffectChain("aggressive_verses", self.sample_rate)
        chain.add_highpass(cutoff_frequency_hz=120)
        chain.add_compressor(
            threshold_db=-24.0 * intensity,
            ratio=4.0,
            attack_ms=3.0,
            release_ms=50.0,
        )
        chain.add_distortion(drive_db=15.0 * intensity)
        chain.add_gain(gain_db=2.0 * intensity)

        return chain.process(audio)

    def process_chorus(
        self,
        audio: np.ndarray,
        ethereal: float = 1.0,
    ) -> np.ndarray:
        """Process chorus vocals with ethereal style.

        Applies reverb, chorus, and delay for atmospheric,
        dreamy vocal delivery.

        Args:
            audio: Input vocal audio
            ethereal: Ethereal effect amount (0.0 to 2.0)

        Returns:
            Processed vocal audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = VocalProcessor()
            >>> processed = processor.process_chorus(vocal, ethereal=1.5)
        """
        chain = EffectChain("ethereal_chorus", self.sample_rate)
        chain.add_compressor(
            threshold_db=-18.0,
            ratio=3.0,
            attack_ms=10.0,
            release_ms=100.0,
        )
        chain.add_chorus(
            rate_hz=1.2 * ethereal,
            depth=0.6 * ethereal,
            centre_delay_ms=8.0,
            mix=0.4 * ethereal,
        )
        chain.add_reverb(
            room_size=0.7 * ethereal,
            wet_level=0.5 * ethereal,
            dry_level=0.5,
        )
        chain.add_delay(
            delay_seconds=0.3,
            feedback=0.3,
            mix=0.3 * ethereal,
        )

        return chain.process(audio)

    def process_bridge(
        self,
        audio: np.ndarray,
        build: float = 1.0,
    ) -> np.ndarray:
        """Process bridge vocals with building intensity.

        Applies increasing compression and saturation for
        tension-building sections.

        Args:
            audio: Input vocal audio
            build: Build intensity (0.0 to 2.0)

        Returns:
            Processed vocal audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = VocalProcessor()
            >>> processed = processor.process_bridge(vocal, build=1.3)
        """
        chain = EffectChain("building_bridge", self.sample_rate)
        chain.add_compressor(
            threshold_db=-20.0 * build,
            ratio=5.0 * build,
            attack_ms=2.0,
            release_ms=30.0,
        )
        chain.add_distortion(drive_db=10.0 * build)
        chain.add_reverb(
            room_size=0.5,
            wet_level=0.3,
            dry_level=0.7,
        )

        return chain.process(audio)

    def clean_vocal(
        self,
        audio: np.ndarray,
    ) -> np.ndarray:
        """Clean vocal audio with minimal processing.

        Applies subtle compression and EQ for natural sound.

        Args:
            audio: Input vocal audio

        Returns:
            Cleaned vocal audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = VocalProcessor()
            >>> clean = processor.clean_vocal(raw_vocal)
        """
        chain = EffectChain("clean_vocal", self.sample_rate)
        chain.add_highpass(cutoff_frequency_hz=80)
        chain.add_compressor(
            threshold_db=-16.0,
            ratio=2.5,
            attack_ms=15.0,
            release_ms=100.0,
        )

        return chain.process(audio)


class InstrumentalProcessor:
    """Processor for instrumental audio with genre-specific presets.

    Provides presets for different musical genres:
    - Phonk processing (sidechain, distortion, EQ)
    - Trap processing (808 enhancement, saturation)
    - Lofi processing (vinyl, filtering, warmth)

    Attributes:
        sample_rate: Audio sample rate in Hz

    Example:
        >>> processor = InstrumentalProcessor(sample_rate=48000)
        >>> processed = processor.process_phonk(instrumental)
    """

    def __init__(self, sample_rate: int = 48000):
        """Initialize instrumental processor.

        Args:
            sample_rate: Audio sample rate in Hz

        Complexity: O(1)
        """
        self.sample_rate = sample_rate

    def process_phonk(
        self,
        audio: np.ndarray,
        intensity: float = 1.0,
    ) -> np.ndarray:
        """Process audio for phonk style.

        Applies heavy compression, saturation, and pumping effects
        characteristic of drift phonk.

        Args:
            audio: Input instrumental audio
            intensity: Effect intensity (0.0 to 2.0)

        Returns:
            Processed phonk audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = InstrumentalProcessor()
            >>> processed = processor.process_phonk(instrumental, intensity=1.2)
        """
        chain = EffectChain("phonk", self.sample_rate)
        chain.add_lowpass(cutoff_frequency_hz=12000)
        chain.add_compressor(
            threshold_db=-22.0 * intensity,
            ratio=6.0,
            attack_ms=5.0,
            release_ms=50.0,
        )
        chain.add_distortion(drive_db=12.0 * intensity)
        chain.add_gain(gain_db=3.0 * intensity)
        chain.add_limiter(threshold_db=-0.5)

        return chain.process(audio)

    def process_trap(
        self,
        audio: np.ndarray,
        sub_bass_boost: float = 1.0,
    ) -> np.ndarray:
        """Process audio for trap style.

        Enhances 808 bass and applies punchy compression
        characteristic of trap beats.

        Args:
            audio: Input instrumental audio
            sub_bass_boost: Sub-bass boost amount (0.0 to 2.0)

        Returns:
            Processed trap audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = InstrumentalProcessor()
            >>> processed = processor.process_trap(instrumental, sub_bass_boost=1.3)
        """
        chain = EffectChain("trap", self.sample_rate)
        chain.add_compressor(
            threshold_db=-20.0,
            ratio=4.0,
            attack_ms=2.0,
            release_ms=40.0,
        )
        chain.add_distortion(drive_db=6.0 * sub_bass_boost)
        chain.add_highpass(cutoff_frequency_hz=30)
        chain.add_limiter(threshold_db=-0.5)

        return chain.process(audio)

    def process_lofi(
        self,
        audio: np.ndarray,
        warmth: float = 1.0,
    ) -> np.ndarray:
        """Process audio for lofi style.

        Applies filtering, saturation, and effects for
        nostalgic, warm lofi sound.

        Args:
            audio: Input instrumental audio
            warmth: Warmth effect amount (0.0 to 2.0)

        Returns:
            Processed lofi audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = InstrumentalProcessor()
            >>> processed = processor.process_lofi(instrumental, warmth=1.5)
        """
        chain = EffectChain("lofi", self.sample_rate)
        chain.add_lowpass(cutoff_frequency_hz=8000 * (2.0 - warmth * 0.5))
        chain.add_highpass(cutoff_frequency_hz=60)
        chain.add_distortion(drive_db=8.0 * warmth)
        chain.add_compressor(
            threshold_db=-18.0,
            ratio=3.0,
            attack_ms=20.0,
            release_ms=150.0,
        )
        chain.add_reverb(
            room_size=0.3,
            wet_level=0.2,
            dry_level=0.8,
        )

        return chain.process(audio)

    def process_drill(
        self,
        audio: np.ndarray,
        darkness: float = 1.0,
    ) -> np.ndarray:
        """Process audio for drill style.

        Applies dark, heavy processing characteristic
        of UK drill and Brooklyn drill.

        Args:
            audio: Input instrumental audio
            darkness: Darkness/dark atmosphere amount (0.0 to 2.0)

        Returns:
            Processed drill audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = InstrumentalProcessor()
            >>> processed = processor.process_drill(instrumental, darkness=1.3)
        """
        chain = EffectChain("drill", self.sample_rate)
        chain.add_lowpass(cutoff_frequency_hz=10000)
        chain.add_compressor(
            threshold_db=-24.0 * darkness,
            ratio=8.0,
            attack_ms=1.0,
            release_ms=30.0,
        )
        chain.add_distortion(drive_db=15.0 * darkness)
        chain.add_reverb(
            room_size=0.4 * darkness,
            wet_level=0.3,
            dry_level=0.7,
        )
        chain.add_limiter(threshold_db=-1.0)

        return chain.process(audio)

    def master_track(
        self,
        audio: np.ndarray,
        target_lufs: float = -14.0,
    ) -> np.ndarray:
        """Master track for final output.

        Applies EQ, compression, and limiting for
        competitive loudness and clarity.

        Args:
            audio: Input audio to master
            target_lufs: Target LUFS level (default -14.0 for streaming)

        Returns:
            Mastered audio

        Complexity: O(n) where n is number of samples

        Example:
            >>> processor = InstrumentalProcessor()
            >>> mastered = processor.master_track(mix)
        """
        chain = EffectChain("master", self.sample_rate)
        chain.add_compressor(
            threshold_db=-16.0,
            ratio=3.0,
            attack_ms=10.0,
            release_ms=100.0,
        )
        chain.add_limiter(
            threshold_db=-0.3,
            release_ms=100.0,
        )

        processed = chain.process(audio)

        # Normalize to target level
        current_rms = np.sqrt(np.mean(processed ** 2))
        if current_rms > 0:
            target_rms = 10 ** ((target_lufs + 20) / 20) * 0.01
            gain = target_rms / current_rms
            processed = processed * gain

        # Final limiting
        final_chain = EffectChain("final_limiter", self.sample_rate)
        final_chain.add_limiter(threshold_db=-0.1)
        processed = final_chain.process(processed)

        return processed


def create_vocal_processor(sample_rate: int = 48000) -> VocalProcessor:
    """Factory function to create vocal processor.

    Args:
        sample_rate: Audio sample rate in Hz

    Returns:
        Initialized VocalProcessor instance

    Complexity: O(1)
    """
    return VocalProcessor(sample_rate=sample_rate)


def create_instrumental_processor(sample_rate: int = 48000) -> InstrumentalProcessor:
    """Factory function to create instrumental processor.

    Args:
        sample_rate: Audio sample rate in Hz

    Returns:
        Initialized InstrumentalProcessor instance

    Complexity: O(1)
    """
    return InstrumentalProcessor(sample_rate=sample_rate)
