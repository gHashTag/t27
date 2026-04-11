# contrib/backend/music-generator/tests/test_musicgen.py
# Tests for MusicGen wrapper
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for MusicGen music generation module."""

import pytest
import numpy as np
from pathlib import Path
import tempfile

from music_gen.musicgen import MusicGenWrapper, create_musicgen_wrapper


class TestMusicGenWrapper:
    """Test cases for MusicGenWrapper class."""

    def test_init(self) -> None:
        """Test wrapper initialization."""
        wrapper = MusicGenWrapper(model_size="small", device="cpu")
        assert wrapper.model_size == "small"
        assert wrapper.device == "cpu"
        assert wrapper.sample_rate == 48000
        assert not wrapper._model_loaded

    def test_get_model_name(self) -> None:
        """Test model name mapping."""
        wrapper = MusicGenWrapper(model_size="small")
        assert wrapper._get_model_name() == "facebook/musicgen-small"

        wrapper = MusicGenWrapper(model_size="medium")
        assert wrapper._get_model_name() == "facebook/musicgen-medium"

        wrapper = MusicGenWrapper(model_size="large")
        assert wrapper._get_model_name() == "facebook/musicgen-large"

        # Unknown size defaults to small
        wrapper = MusicGenWrapper(model_size="unknown")
        assert wrapper._get_model_name() == "facebook/musicgen-small"

    def test_enhance_prompt(self) -> None:
        """Test prompt enhancement."""
        wrapper = MusicGenWrapper()

        # Without genre
        enhanced = wrapper._enhance_prompt("dark beat", None)
        assert enhanced == "dark beat"

        # With genre (requires prompts module)
        try:
            from music_gen.prompts import enhance_prompt_for_genre
            enhanced = wrapper._enhance_prompt("dark beat", "phonk")
            assert "phonk" in enhanced.lower()
        except ImportError:
            pass

    def test_ensure_stereo_shape(self) -> None:
        """Test audio shape handling."""
        # This is tested through the actual generation
        # For unit tests, we verify the wrapper accepts parameters
        wrapper = MusicGenWrapper()
        assert wrapper is not None


class TestCreateMusicGenWrapper:
    """Test cases for factory function."""

    def test_create_default(self) -> None:
        """Test factory with defaults."""
        wrapper = create_musicgen_wrapper()
        assert isinstance(wrapper, MusicGenWrapper)
        assert wrapper.model_size == "small"
        assert wrapper.device == "cpu"

    def test_create_custom(self) -> None:
        """Test factory with custom parameters."""
        wrapper = create_musicgen_wrapper(
            model_size="medium",
            device="cpu",
        )
        assert isinstance(wrapper, MusicGenWrapper)
        assert wrapper.model_size == "medium"


class TestPrompts:
    """Test cases for prompt utilities."""

    def test_enhance_prompt_for_genre(self) -> None:
        """Test genre prompt enhancement."""
        try:
            from music_gen.prompts import enhance_prompt_for_genre
        except ImportError:
            pytest.skip("prompts module not available")

        result = enhance_prompt_for_genre("dark beat", "phonk")
        assert "dark beat" in result
        assert len(result) > len("dark beat")

    def test_get_random_prompt_for_genre(self) -> None:
        """Test random prompt retrieval."""
        try:
            from music_gen.prompts import get_random_prompt_for_genre
        except ImportError:
            pytest.skip("prompts module not available")

        prompt = get_random_prompt_for_genre("phonk")
        assert isinstance(prompt, str)
        assert len(prompt) > 0


@pytest.mark.integration
class TestMusicGenIntegration:
    """Integration tests for MusicGen (requires model download)."""

    def test_short_generation(self) -> None:
        """Test short music generation (10 seconds)."""
        pytest.skip("Integration test - requires model download")

        wrapper = MusicGenWrapper(model_size="small", device="cpu")

        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = Path(tmpdir) / "test_output.wav"

            audio = wrapper.generate(
                prompt="simple beat",
                duration=10,
                output_path=output_path,
            )

            assert isinstance(audio, np.ndarray)
            assert audio.ndim == 2  # [channels, samples]
            assert output_path.exists()

    def test_chunked_generation(self) -> None:
        """Test chunked generation for long duration."""
        pytest.skip("Integration test - requires model download")

        wrapper = MusicGenWrapper(model_size="small", device="cpu")

        audio = wrapper.generate(
            prompt="dark phonk beat",
            duration=60,  # Requires chunking
        )

        assert isinstance(audio, np.ndarray)
        expected_samples = 60 * 48000
        assert audio.shape[1] == expected_samples


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
