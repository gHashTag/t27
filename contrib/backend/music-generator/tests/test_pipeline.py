# contrib/backend/music-generator/tests/test_pipeline.py
# Tests for pipeline orchestration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit and integration tests for music generation pipeline."""

import pytest
import numpy as np
from pathlib import Path
import tempfile
import sys
sys.path.insert(0, '.')

import config
import pipeline
from config import MusicGenConfig
from pipeline import MusicPipeline, create_pipeline


class TestMusicPipeline:
    """Test cases for MusicPipeline class."""

    def test_init_default(self) -> None:
        """Test pipeline initialization with defaults."""
        pipeline = MusicPipeline()
        assert pipeline is not None
        assert pipeline.device == "cpu"
        assert pipeline.config is not None

    def test_init_with_config(self) -> None:
        """Test pipeline initialization with custom config."""
        config = MusicGenConfig(
            model_path=Path("models"),
            output_path=Path("outputs"),
            temp_path=Path("temp"),
            model_size="small",
            sample_rate=48000,
            chunk_duration=30,
            device="cpu",
            vocal_level_db=-4.0,
            limiter_threshold=-0.3,
            cache_models=True,
            batch_size=1,
        )

        pipeline = MusicPipeline(config=config)
        assert pipeline.config == config

    def test_get_components_lazy_loading(self) -> None:
        """Test that components are lazily loaded."""
        pipeline = MusicPipeline()

        assert pipeline.musicgen is None
        assert pipeline.synthesizer is None
        assert pipeline.vocal_processor is None
        assert pipeline.instrumental_processor is None
        assert pipeline.mixer is None

        # Getting components should create them
        musicgen = pipeline._get_musicgen()
        assert musicgen is not None
        assert pipeline.musicgen is musicgen

    def test_skip_vocals_generation(self) -> None:
        """Test generation with vocals skipped."""
        pytest.skip("Integration test - requires model")

        pipeline = MusicPipeline()

        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = Path(tmpdir) / "output.wav"

            result = pipeline.generate(
                lyrics="test lyrics",
                style="phonk",
                output_path=output_path,
                skip_vocals=True,
                duration=10,
            )

            assert "instrumental" in result
            assert "vocals" in result
            assert result["vocals"] is None
            assert result["mix"] is not None
            assert output_path.exists()

    def test_skip_effects(self) -> None:
        """Test generation without effects."""
        pytest.skip("Integration test - requires model")

        pipeline = MusicPipeline()

        result = pipeline.generate(
            lyrics="test",
            style="phonk",
            skip_effects=True,
            skip_vocals=True,
            duration=10,
        )

        assert "mix" in result
        assert result["mix"] is not None

    def test_skip_mastering(self) -> None:
        """Test generation without mastering."""
        pytest.skip("Integration test - requires model")

        pipeline = MusicPipeline()

        result = pipeline.generate(
            lyrics="test",
            style="phonk",
            skip_mastering=True,
            skip_vocals=True,
            duration=10,
        )

        assert "mastered" in result
        assert result["mastered"] is not None
        # Without mastering, mastered should equal mix
        assert np.array_equal(result["mastered"], result["mix"])


class TestCreatePipeline:
    """Test cases for pipeline factory function."""

    def test_create_default(self) -> None:
        """Test factory with defaults."""
        pipeline = create_pipeline()
        assert isinstance(pipeline, MusicPipeline)
        assert pipeline.device == "cpu"

    def test_create_with_device(self) -> None:
        """Test factory with custom device."""
        pipeline = create_pipeline(device="cpu")
        assert isinstance(pipeline, MusicPipeline)
        assert pipeline.device == "cpu"


class TestPipelineComponents:
    """Test cases for individual pipeline components."""

    def test_vocal_processor_creation(self) -> None:
        """Test vocal processor creation."""
        pipeline = MusicPipeline()
        processor = pipeline._get_vocal_processor()

        assert processor is not None
        assert processor.sample_rate == 48000

    def test_instrumental_processor_creation(self) -> None:
        """Test instrumental processor creation."""
        pipeline = MusicPipeline()
        processor = pipeline._get_instrumental_processor()

        assert processor is not None
        assert processor.sample_rate == 48000

    def test_mixer_creation(self) -> None:
        """Test mixer creation."""
        pipeline = MusicPipeline()
        mixer = pipeline._get_mixer()

        assert mixer is not None
        assert mixer.sample_rate == 48000
        assert mixer.vocal_level_db == -4.0

    def test_mix_function(self) -> None:
        """Test mixing function."""
        pipeline = MusicPipeline()

        # Create dummy stereo audio
        vocals = np.random.randn(2, 48000) * 0.1
        instrumental = np.random.randn(2, 48000) * 0.3

        mix = pipeline._mix(vocals, instrumental, vocal_level_db=-6.0)

        assert isinstance(mix, np.ndarray)
        assert mix.shape == vocals.shape
        # Mix should be combination of both
        assert np.max(np.abs(mix)) > 0

    def test_match_lengths(self) -> None:
        """Test length matching function."""
        from mixing import AutoMixer
        mixer = AutoMixer()

        audio1 = np.random.randn(2, 24000)
        audio2 = np.random.randn(2, 48000)

        matched1, matched2 = mixer._match_lengths(audio1, audio2)

        assert matched1.shape == matched2.shape
        assert matched1.shape[1] == 48000
        # Check that audio1 was padded
        assert np.allclose(matched1[:, :24000], audio1)
        assert np.allclose(matched1[:, 24000:], 0)

    def test_ensure_stereo(self) -> None:
        """Test stereo conversion function."""
        from mixing import AutoMixer
        mixer = AutoMixer()

        # Mono input
        mono = np.random.randn(48000)
        stereo = mixer._ensure_stereo(mono)

        assert stereo.ndim == 2
        assert stereo.shape[0] == 2
        assert np.allclose(stereo[0], mono)
        assert np.allclose(stereo[1], mono)

        # Already stereo input
        stereo_input = np.random.randn(2, 48000)
        stereo_output = mixer._ensure_stereo(stereo_input)

        assert np.allclose(stereo_output, stereo_input)


@pytest.mark.integration
class TestPipelineIntegration:
    """Integration tests for full pipeline."""

    def test_short_generation_with_vocals(self) -> None:
        """Test complete short generation with vocals."""
        pytest.skip("Integration test - requires model download")

        pipeline = MusicPipeline()

        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = Path(tmpdir) / "full_output.wav"

            result = pipeline.generate(
                lyrics="dark shadows creeping",
                style="phonk",
                output_path=output_path,
                duration=15,
                vocal_style="aggressive",
            )

            assert "instrumental" in result
            assert "vocals" in result
            assert "mix" in result
            assert "mastered" in result
            assert "metadata" in result

            assert result["instrumental"] is not None
            assert result["vocals"] is not None
            assert result["mix"] is not None
            assert result["mastered"] is not None
            assert output_path.exists()

            # Verify output file
            import soundfile as sf
            audio, sr = sf.read(str(output_path))
            assert sr == 48000
            assert len(audio) > 0

    def test_instrumental_only(self) -> None:
        """Test instrumental-only generation."""
        pytest.skip("Integration test - requires model download")

        pipeline = MusicPipeline()

        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = Path(tmpdir) / "instrumental.wav"

            result = pipeline.generate(
                lyrics="",
                style="trap",
                output_path=output_path,
                skip_vocals=True,
                duration=30,
            )

            assert result["vocals"] is None
            assert np.array_equal(result["mix"], result["instrumental"])
            assert np.array_equal(result["mastered"], result["mix"])

    def test_all_styles(self) -> None:
        """Test generation for all supported styles."""
        pytest.skip("Integration test - requires model download")

        pipeline = MusicPipeline()
        styles = ["phonk", "trap", "hip_hop", "drill", "lofi"]

        with tempfile.TemporaryDirectory() as tmpdir:
            for style in styles:
                output_path = Path(tmpdir) / f"{style}.wav"

                result = pipeline.generate(
                    lyrics="test",
                    style=style,
                    output_path=output_path,
                    skip_vocals=True,
                    duration=10,
                )

                assert output_path.exists()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
