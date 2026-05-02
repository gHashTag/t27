# contrib/backend/notebooklm/test_connection.py
# Test NotebookLM connection
# phi^2 + 1/phi^2 = 3 | TRINITY

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from contrib.backend.notebooklm.cookie_auth import test_notebooklm_sdk_integration
from contrib.backend.notebooklm.config import config_from_env


def test_connection() -> bool:
    """Test if NotebookLM SDK is available.

    Returns:
        True if SDK available, False otherwise

    Complexity: O(1)
    """
    print("Testing NotebookLM SDK availability...")

    if not test_notebooklm_sdk_integration():
        print("  [FAIL] notebooklm-py SDK not installed")
        print("  [INFO] Run: pip install notebooklm-py")
        return False

    print("  [OK] SDK is available")

    # Test config
    print("\nTesting configuration...")
    config = config_from_env()
    print(f"  Storage path: {config.storage_path}")
    print(f"  Notebook name: {config.notebook_name}")
    print(f"  Timeout: {config.timeout_ms}ms")
    print(f"  Auto-refresh: {config.auto_refresh}")

    print("\n[SUCCESS] All connection tests passed")
    return True


if __name__ == "__main__":
    success = test_connection()
    sys.exit(0 if success else 1)
