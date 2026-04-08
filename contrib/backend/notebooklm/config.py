# contrib/backend/notebooklm/config.py
# Configuration for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

from dataclasses import dataclass
from typing import Optional
from pathlib import Path
import os


@dataclass
class NotebookLMConfig:
    """Configuration for NotebookLM integration.
    
    Attributes:
        storage_path: Path to storage state file
        notebook_name: Default notebook name
        timeout_ms: Request timeout in milliseconds
        auto_refresh: Auto-refresh token before expiry
    """
    
    storage_path: Path
    notebook_name: str
    timeout_ms: int
    auto_refresh: bool
    
    @classmethod
    def from_env(cls) -> "NotebookLMConfig":
        """Create configuration from environment variables.
        
        Environment Variables:
            NOTEBOOKLM_STORAGE_PATH: Path to storage state (default: ~/.notebooklm/storage_state.json)
            NOTEBOOKLM_NOTEBOOK_NAME: Default notebook name (default: "t27-QUEEN-BRAIN")
            NOTEBOOKLM_TIMEOUT_MS: Request timeout (default: 30000)
            NOTEBOOKLM_AUTO_REFRESH: Auto-refresh token (default: true)
        
        Returns:
            NotebookLMConfig with defaults for unset values
        
        Complexity: O(1)
        """
        storage_path_env = os.getenv("NOTEBOOKLM_STORAGE_PATH")
        notebook_name_env = os.getenv("NOTEBOOKLM_NOTEBOOK_NAME")
        timeout_ms_env = os.getenv("NOTEBOOKLM_TIMEOUT_MS")
        auto_refresh_env = os.getenv("NOTEBOOKLM_AUTO_REFRESH")
        
        storage_path = Path(storage_path_env) if storage_path_env else Path.home() / ".notebooklm" / "storage_state.json"
        notebook_name = notebook_name_env if notebook_name_env else "t27-QUEEN-BRAIN"
        timeout_ms = int(timeout_ms_env) if timeout_ms_env else 30000
        auto_refresh = auto_refresh_env.lower() == "true" if auto_refresh_env else True
        
        return cls(
            storage_path=storage_path,
            notebook_name=notebook_name,
            timeout_ms=timeout_ms,
            auto_refresh=auto_refresh,
        )


def config_from_env() -> NotebookLMConfig:
    """Create configuration from environment variables.
    
    Returns:
        NotebookLMConfig with defaults for unset values
    
    Complexity: O(1)
    """
    return NotebookLMConfig.from_env()


# Default configuration
DEFAULT_CONFIG = config_from_env()
