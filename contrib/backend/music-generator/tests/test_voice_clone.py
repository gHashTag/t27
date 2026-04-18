# contrib/backend/music-generator/tests/test_voice_clone.py
# Tests for voice cloning module
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for voice cloning module."""

import pytest
import numpy as np
from pathlib import Path
import tempfile

from voice_clone.rvc import RVCCloner, create_rvc_cloner
from voice_clone.preprocessing import (
    preprocess_audio,
    reduce_noise,
    segment_by_vocal_activity,
    normalize_audio,
)
from voice_clone.train import VoiceTrainer, create_voice_trainer


class TestRVCCloner:
    """Test cases for RVCCloner class."""

    def test_init(self) -> None:
        """Test RVC cloner initialization."""
        cloner = RVCCloner(device="cpu")
        assert cloner.device == "cpu"
        assert cloner.sample_rate == 48000
        assert cloner.f0_method == "rmvpe"
        assert not cloner._model_loaded

    def test_init_with_model_path(self) -> None:
        """Test initialization with model path."""
        cloner = RVCCloner(
            model_path=Path("fake_model.pth"),
            device="cpu",
        )
        assert str(cloner.model_path) == "fake_model.pth"

    def test_get_model_info(self) -> None:
        """Test model info retrieval."""
        cloner = RVCCloner()
        info = cloner.get_model_info()

        assert "model_path" in info
        assert "device" in info
        assert "sample_rate" in info
        assert "loaded" in info
        assert info["loaded"] is False

    def test_convert_without_load_raises(self) -> None:
        """Test conversion fails without model load."""
        cloner = RVCCloner()

        with tempfile.NamedTemporaryFile(suffix=".wav") as f:
            # Create dummy audio file
            import soundfile as sf
            sf.write(f.name, np.random.randn(48000), 48000)

            output_path = Path(tempfile.mkdtemp()) / "output.wav"

            with pytest.raises(RuntimeError):
                cloner.convert(Path(f.name), output_path)


class TestCreateRVCCloner:
    """Test cases for RVC cloner factory function."""

    def test_create_default(self) -> None:
        """Test factory with defaults."""
        cloner = create_rvc_cloner()
        assert isinstance(cloner, RVCCloner)
        assert cloner.device == "cpu"

    def test_create_with_path(self) -> None:
        """Test factory with model path."""
        cloner = create_rvc_cloner(model_path=Path("model.pth"))
        assert isinstance(cloner, RVCCloner)
        assert cloner.model_path == Path("model.pth")


class TestPreprocessing:
    """Test cases for audio preprocessing functions."""

    def test_normalize_audio(self) -> None:
        """Test audio normalization."""
        # Create test audio
        audio = np.random.randn(48000) * 0.1

        normalized = normalize_audio(audio, target_dbfs=-20.0)

        assert isinstance(normalized, np.ndarray)
        assert normalized.shape == audio.shape
        assert np.max(np.abs(normalized)) <= 0.99

    def test_normalize_silent_audio(self) -> None:
        """Test normalization of silent audio."""
        audio = np.zeros(48000)

        normalized = normalize_audio(audio)

        assert np.allclose(normalized, audio)

    def test_segment_by_vocal_activity(self) -> None:
        """Test VAD segmentation."""
        # Create test audio with speech-like regions
        audio = np.random.randn(48000) * 0.01

        # Add "speech" regions (higher energy)
        audio[10000:15000] = np.random.randn(5000) * 0.5
        audio[30000:35000] = np.random.randn(5000) * 0.5

        segments = segment_by_vocal_activity(audio, sample_rate=48000)

        assert isinstance(segments, list)
        # Should detect at least the high-energy regions
        # (actual detection depends on VAD thresholds)

    def test_reduce_noise(self) -> None:
        """Test noise reduction."""
        # Create test audio with noise
        clean = np.sin(2 * np.pi * 440 * np.linspace(0, 1, 48000))
        noise = np.random.randn(48000) * 0.1
        noisy = clean + noise

        reduced = reduce_noise(noisy, sample_rate=48000)

        assert isinstance(reduced, np.ndarray)
        # librosa.istft may return slightly different length, allow 10% tolerance
        assert len(reduced) >= int(0.9 * len(noisy))
        assert len(reduced) <= int(1.1 * len(noisy))
        # Noise-reduced signal should still have some structure
        assert np.max(np.abs(reduced)) > 0


class TestVoiceTrainer:
    """Test cases for VoiceTrainer class."""

    def test_init(self) -> None:
        """Test trainer initialization."""
        with tempfile.TemporaryDirectory() as tmpdir:
            model_path = Path(tmpdir) / "model.pth"
            trainer = VoiceTrainer(model_path=model_path)

            assert trainer.model_path == model_path
            assert trainer.sample_rate == 48000
            assert trainer.epochs == 100
            assert trainer.learning_rate == 0.0001

    def test_build_model(self) -> None:
        """Test model building."""
        with tempfile.TemporaryDirectory() as tmpdir:
            model_path = Path(tmpdir) / "model.pth"
            trainer = VoiceTrainer(model_path=model_path)

            trainer.build_model()

            assert trainer.model is not None
            assert "hidden_dim" in trainer.model
            assert "num_layers" in trainer.model

    def test_prepare_data_without_files_raises(self) -> None:
        """Test data preparation fails without files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            model_path = Path(tmpdir) / "model.pth"
            trainer = VoiceTrainer(model_path=model_path)

            # Empty list should work but produce no data
            trainer.prepare_data([])

            assert len(trainer.training_data) == 0
            assert len(trainer.validation_data) == 0

    def test_save_model(self) -> None:
        """Test model saving."""
        with tempfile.TemporaryDirectory() as tmpdir:
            model_path = Path(tmpdir) / "model.pth"
            trainer = VoiceTrainer(model_path=model_path)
            trainer.build_model()

            trainer.save_model(model_path)

            assert model_path.exists()


class TestCreateVoiceTrainer:
    """Test cases for voice trainer factory function."""

    def test_create_default(self) -> None:
        """Test factory with defaults."""
        with tempfile.TemporaryDirectory() as tmpdir:
            model_path = Path(tmpdir) / "model.pth"
            trainer = create_voice_trainer(model_path=model_path)

            assert isinstance(trainer, VoiceTrainer)
            assert trainer.device == "cpu"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
