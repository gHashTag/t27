# contrib/backend/music-generator/utils/download.py
# Model download utilities
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Model download and verification utilities."""

import hashlib
from pathlib import Path
from typing import Optional, Dict, Any
import logging
import urllib.request
import os
from tqdm import tqdm

logger = logging.getLogger(__name__)


# Model URLs and checksums
MODEL_REGISTRY: Dict[str, Dict[str, str]] = {
    "musicgen-small": {
        "url": "https://huggingface.co/facebook/musicgen-small/resolve/main/pytorch_model.bin",
        "sha256": "",  # To be filled with actual checksum
    },
    "musicgen-medium": {
        "url": "https://huggingface.co/facebook/musicgen-medium/resolve/main/pytorch_model.bin",
        "sha256": "",
    },
    "musicgen-large": {
        "url": "https://huggingface.co/facebook/musicgen-large/resolve/main/pytorch_model.bin",
        "sha256": "",
    },
}


def download_model(
    model_name: str,
    output_dir: Path,
    show_progress: bool = True,
) -> Path:
    """Download model from remote URL.

    Args:
        model_name: Name of model to download
        output_dir: Directory to save model
        show_progress: Show progress bar

    Returns:
        Path to downloaded model

    Raises:
        ValueError: If model name is not recognized
        RuntimeError: If download fails

    Complexity: O(file_size)

    Example:
        >>> model_path = download_model(
        ...     "musicgen-small",
        ...     Path("models")
        ... )
    """
    if model_name not in MODEL_REGISTRY:
        raise ValueError(f"Unknown model: {model_name}")

    model_info = MODEL_REGISTRY[model_name]
    url = model_info["url"]

    output_dir.mkdir(parents=True, exist_ok=True)

    # Extract filename from URL
    filename = url.split("/")[-1]
    output_path = output_dir / filename

    if output_path.exists():
        logger.info(f"Model already exists at {output_path}")
        return output_path

    logger.info(f"Downloading {model_name} from {url}...")

    try:
        if show_progress:
            # Download with progress bar
            with tqdm(
                unit="B",
                unit_scale=True,
                unit_divisor=1024,
                miniters=1,
                desc=f"Downloading {model_name}",
            ) as t:
                urllib.request.urlretrieve(
                    url,
                    output_path,
                    reporthook=lambda block_num, block_size, total_size: t.update(
                        min(block_num * block_size - t.n, total_size - t.n) if total_size else block_num * block_size
                    ),
                )
        else:
            urllib.request.urlretrieve(url, output_path)

        logger.info(f"Model downloaded to {output_path}")
        return output_path

    except Exception as e:
        # Clean up partial download
        if output_path.exists():
            output_path.unlink()
        raise RuntimeError(f"Failed to download model: {e}") from e


def verify_model(
    model_path: Path,
    expected_sha256: Optional[str] = None,
) -> bool:
    """Verify model file integrity using SHA-256 checksum.

    Args:
        model_path: Path to model file
        expected_sha256: Expected SHA-256 hash (None to skip)

    Returns:
        True if verification passes

    Raises:
        FileNotFoundError: If model file does not exist
        ValueError: If checksum does not match

    Complexity: O(file_size)

    Example:
        >>> is_valid = verify_model(Path("model.pth"), expected_sha256="abc123...")
    """
    if not model_path.exists():
        raise FileNotFoundError(f"Model file not found: {model_path}")

    if expected_sha256 is None:
        logger.warning("No checksum provided, skipping verification")
        return True

    logger.info(f"Verifying {model_path}...")

    sha256_hash = hashlib.sha256()

    with open(model_path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)

    actual_hash = sha256_hash.hexdigest()

    if actual_hash != expected_sha256:
        raise ValueError(
            f"Checksum mismatch for {model_path}\n"
            f"Expected: {expected_sha256}\n"
            f"Actual:   {actual_hash}"
        )

    logger.info("Model verification passed")
    return True


def get_model_path(
    model_name: str,
    models_dir: Path,
    auto_download: bool = True,
) -> Path:
    """Get path to model, downloading if necessary.

    Args:
        model_name: Name of model
        models_dir: Directory containing models
        auto_download: Download if not found

    Returns:
        Path to model file

    Raises:
        ValueError: If model name is not recognized
        FileNotFoundError: If model not found and auto_download is False

    Complexity: O(1) or O(file_size) if downloading

    Example:
        >>> model_path = get_model_path(
        ...     "musicgen-small",
        ...     Path("models"),
        ...     auto_download=True
        ... )
    """
    if model_name not in MODEL_REGISTRY:
        raise ValueError(f"Unknown model: {model_name}")

    # Extract filename from URL
    url = MODEL_REGISTRY[model_name]["url"]
    filename = url.split("/")[-1]
    model_path = models_dir / filename

    if not model_path.exists():
        if auto_download:
            logger.info(f"Model not found, downloading...")
            return download_model(model_name, models_dir)
        else:
            raise FileNotFoundError(
                f"Model not found at {model_path} and auto_download is False"
            )

    return model_path


def list_downloaded_models(models_dir: Path) -> Dict[str, Path]:
    """List all downloaded models in directory.

    Args:
        models_dir: Directory to scan

    Returns:
        Dictionary mapping model names to file paths

    Complexity: O(n) where n is number of files

    Example:
        >>> models = list_downloaded_models(Path("models"))
        >>> for name, path in models.items():
        ...     print(f"{name}: {path}")
    """
    if not models_dir.exists():
        return {}

    models = {}

    for model_name in MODEL_REGISTRY.keys():
        url = MODEL_REGISTRY[model_name]["url"]
        filename = url.split("/")[-1]
        model_path = models_dir / filename

        if model_path.exists():
            models[model_name] = model_path

    return models


def get_model_size(model_path: Path) -> int:
    """Get model file size in bytes.

    Args:
        model_path: Path to model file

    Returns:
        File size in bytes

    Complexity: O(1)

    Example:
        >>> size_bytes = get_model_size(Path("model.pth"))
        >>> size_mb = size_bytes / (1024 * 1024)
    """
    return model_path.stat().st_size


def format_size(size_bytes: int) -> str:
    """Format byte size to human-readable string.

    Args:
        size_bytes: Size in bytes

    Returns:
        Formatted size string

    Complexity: O(1)

    Example:
        >>> formatted = format_size(1073741824)
        >>> print(formatted)  # "1.00 GB"
    """
    for unit in ["B", "KB", "MB", "GB", "TB"]:
        if size_bytes < 1024.0:
            return f"{size_bytes:.2f} {unit}"
        size_bytes /= 1024.0
    return f"{size_bytes:.2f} PB"
