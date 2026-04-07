# contrib/backend/notebooklm/tests/test_config.py
# Unit tests for config.py
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Unit tests for NotebookLM configuration."""

import sys
from pathlib import Path
import os

# Add project root to path
repo_root = Path(__file__).parent.parent.parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.config import config_from_env, DEFAULT_CONFIG, NotebookLMConfig


def test_config_from_env_has_defaults():
    """Test that config_from_env() has correct defaults."""
    config = config_from_env()

    assert config.notebook_name == "t27-QUEEN-BRAIN", f"Wrong notebook name: {config.notebook_name}"
    assert config.timeout_ms == 30000, f"Wrong timeout: {config.timeout_ms}"
    assert config.auto_refresh == True, f"Wrong auto-refresh: {config.auto_refresh}"
    assert config.storage_path.name == "storage_state.json", f"Wrong storage path: {config.storage_path}"
    print("[PASS] test_config_from_env_has_defaults")


def test_config_from_env_reads_env_vars():
    """Test that config_from_env() reads environment variables."""
    os.environ["NOTEBOOKLM_NOTEBOOK_NAME"] = "test-notebook"
    os.environ["NOTEBOOKLM_TIMEOUT_MS"] = "60000"
    os.environ["NOTEBOOKLM_AUTO_REFRESH"] = "false"

    config = config_from_env()

    assert config.notebook_name == "test-notebook", f"Wrong notebook name from env: {config.notebook_name}"
    assert config.timeout_ms == 60000, f"Wrong timeout from env: {config.timeout_ms}"
    assert config.auto_refresh == False, f"Wrong auto-refresh from env: {config.auto_refresh}"

    # Clean up
    if "NOTEBOOKLM_NOTEBOOK_NAME" in os.environ:
        del os.environ["NOTEBOOKLM_NOTEBOOK_NAME"]
    if "NOTEBOOKLM_TIMEOUT_MS" in os.environ:
        del os.environ["NOTEBOOKLM_TIMEOUT_MS"]
    if "NOTEBOOKLM_AUTO_REFRESH" in os.environ:
        del os.environ["NOTEBOOKLM_AUTO_REFRESH"]

    print("[PASS] test_config_from_env_reads_env_vars")


def test_default_config_matches_constants():
    """Test that DEFAULT_CONFIG has correct values."""
    assert DEFAULT_CONFIG.notebook_name == "t27-QUEEN-BRAIN"
    assert DEFAULT_CONFIG.timeout_ms == 30000
    assert DEFAULT_CONFIG.auto_refresh == True
    assert DEFAULT_CONFIG.storage_path.name == "storage_state.json"
    print("[PASS] test_default_config_matches_constants")


def test_notebooklm_config_dataclass():
    """Test that NotebookLMConfig is a proper dataclass."""
    from pathlib import Path
    config = NotebookLMConfig(
        storage_path=Path("/tmp/test.json"),
        notebook_name="test",
        timeout_ms=5000,
        auto_refresh=False,
    )

    assert config.notebook_name == "test"
    assert config.timeout_ms == 5000
    assert config.auto_refresh == False
    assert config.storage_path == Path("/tmp/test.json")
    print("[PASS] test_notebooklm_config_dataclass")


def test_config_to_dict():
    """Test that config can be converted to dict (dataclasses have __dict__)."""
    from pathlib import Path
    config = NotebookLMConfig(
        storage_path=Path("/tmp/test.json"),
        notebook_name="test",
        timeout_ms=5000,
        auto_refresh=False,
    )

    as_dict = config.__dict__

    assert isinstance(as_dict, dict)
    assert as_dict["notebook_name"] == "test"
    assert as_dict["timeout_ms"] == 5000
    print("[PASS] test_config_to_dict")


if __name__ == "__main__":
    test_config_from_env_has_defaults()
    test_config_from_env_reads_env_vars()
    test_default_config_matches_constants()
    test_notebooklm_config_dataclass()
    test_config_to_dict()
    print("\nAll config tests passed!")
