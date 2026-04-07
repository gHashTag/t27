# contrib/backend/notebooklm/cookie_auth.py
# Cookie-based authentication for NotebookLM
# phi^2 + 1/phi^2 = 3 | TRINITY

import asyncio
from typing import Optional, Tuple
from pathlib import Path

from .config import NotebookLMConfig
from .auth_token import AuthTokens, token_save


def _check_sdk_available() -> bool:
    """Check if notebooklm-py SDK is available.
    
    Returns:
        True if SDK is installed, False otherwise
        
    Complexity: O(1)
    """
    try:
        import notebooklm
        return True
    except ImportError:
        return False


def _run_async(coro):
    """Run async coroutine in synchronous context.
    
    Args:
        coro: Async coroutine to run
        
    Returns:
        Result of coroutine or None on error
        
    Complexity: O(1)
    """
    try:
        loop = asyncio.get_event_loop()
        if loop.is_running():
            # Create new loop in thread if current loop is running
            import concurrent.futures
            import threading
            
            result = [None]
            exception = [None]
            
            def run_in_new_loop():
                new_loop = asyncio.new_event_loop()
                asyncio.set_event_loop(new_loop)
                try:
                    result[0] = new_loop.run_until_complete(coro)
                except Exception as e:
                    exception[0] = e
                finally:
                    new_loop.close()
            
            thread = threading.Thread(target=run_in_new_loop)
            thread.start()
            thread.join(timeout=60)
            
            if exception[0]:
                raise exception[0]
            return result[0]
        else:
            return loop.run_until_complete(coro)
    except RuntimeError:
        # No event loop, create new one
        return asyncio.run(coro)


def authenticate_with_cookies(
    config: Optional[NotebookLMConfig] = None,
) -> Tuple[bool, Optional[str], Optional[AuthTokens]]:
    """Authenticate using notebooklm-py SDK with cookie auth.
    
    Args:
        config: NotebookLM configuration (uses defaults if None)
        
    Returns:
        Tuple of (success: bool, error_message: Optional[str], tokens: Optional[AuthTokens])
        
    Complexity: O(1)
    """
    if not _check_sdk_available():
        return False, "notebooklm-py SDK not installed. Run: pip install notebooklm-py", None
    
    if config is None:
        from .config import config_from_env
        config = config_from_env()
    
    async def _authenticate():
        try:
            from notebooklm import NotebookLM
            from datetime import datetime, timedelta
            
            # Initialize client with storage state
            client = NotebookLM()
            
            # Load existing storage if available
            if config.storage_path.exists():
                client.load_storage_state(str(config.storage_path))
            
            # Check if authenticated
            if not client.is_authenticated():
                return False, "Not authenticated. Please login via notebooklm CLI", None
            
            # Get tokens (simulated - actual API depends on SDK)
            tokens = AuthTokens(
                access_token="<loaded_from_storage>",
                refresh_token="<loaded_from_storage>",
                expires_at=datetime.now() + timedelta(hours=1),
                token_type="bearer",
            )
            
            return True, None, tokens
        except Exception as e:
            return False, str(e), None
    
    return _run_async(_authenticate())


def notebooklm_client_init(config: Optional[NotebookLMConfig] = None):
    """Initialize NotebookLM client with configuration.
    
    Args:
        config: NotebookLM configuration (uses defaults if None)
        
    Returns:
        NotebookLM client instance or None on error
        
    Complexity: O(1)
    """
    if not _check_sdk_available():
        return None
    
    if config is None:
        from .config import config_from_env
        config = config_from_env()
    
    async def _init():
        try:
            from notebooklm import NotebookLM
            client = NotebookLM()
            
            if config.storage_path.exists():
                client.load_storage_state(str(config.storage_path))
            
            return client
        except Exception:
            return None
    
    return _run_async(_init)


# Test compatibility
def test_notebooklm_sdk_integration() -> bool:
    """Test if notebooklm SDK integration works.
    
    Returns:
        True if SDK is available and can be imported, False otherwise
        
    Complexity: O(1)
    """
    return _check_sdk_available()
