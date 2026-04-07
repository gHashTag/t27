# contrib/backend/notebooklm/token.py
# Authentication token management for NotebookLM
# phi^2 + 1/phi^2 = 3 | TRINITY

from dataclasses import dataclass, asdict
from typing import Optional
from pathlib import Path
import json
from datetime import datetime, timedelta


@dataclass
class AuthTokens:
    """Authentication tokens for NotebookLM.
    
    Attributes:
        access_token: OAuth access token
        refresh_token: Refresh token for getting new access token
        expires_at: Token expiry timestamp
        token_type: Type of token (bearer, etc.)
    """
    
    access_token: str
    refresh_token: str
    expires_at: datetime
    token_type: str
    
    def to_dict(self) -> dict:
        """Convert to dictionary.
        
        Returns:
            Dict representation of tokens
            
        Complexity: O(1)
        """
        return asdict(self)
    
    def is_expired(self, buffer_seconds: int = 300) -> bool:
        """Check if token is expired.
        
        Args:
            buffer_seconds: Buffer time before expiry (default: 300 seconds)
        
        Returns:
            True if token is expired, False otherwise
            
        Complexity: O(1)
        """
        return datetime.now() >= (self.expires_at - timedelta(seconds=buffer_seconds))


TOKEN_PATH = Path.home() / ".t27" / "notebooklm_tokens.json"


def token_load() -> Optional[AuthTokens]:
    """Load tokens from storage.
    
    Reads from ~/.t27/notebooklm_tokens.json
    
    Returns:
        AuthTokens if file exists and valid, None otherwise
        
    Complexity: O(1)
    """
    if not TOKEN_PATH.exists():
        return None
    
    try:
        with open(TOKEN_PATH, "r") as f:
            data = json.load(f)
            tokens = AuthTokens(
                access_token=data.get("access_token", ""),
                refresh_token=data.get("refresh_token", ""),
                expires_at=datetime.fromisoformat(data["expires_at"]) if "expires_at" in data else datetime.now(),
                token_type=data.get("token_type", "bearer"),
            )
            # Return None if expired
            return tokens if not tokens.is_expired() else None
    except (json.JSONDecodeError, KeyError, ValueError):
        return None


def token_save(tokens: AuthTokens) -> bool:
    """Save tokens to storage.
    
    Writes to ~/.t27/notebooklm_tokens.json
    
    Args:
        tokens: AuthTokens to save
        
    Returns:
        True if successful, False otherwise
        
    Complexity: O(1)
    """
    TOKEN_PATH.parent.mkdir(parents=True, exist_ok=True)
    
    try:
        with open(TOKEN_PATH, "w") as f:
            json.dump(tokens.to_dict(), f, indent=2)
        return True
    except (IOError, TypeError):
        return False


def token_is_valid(tokens: Optional[AuthTokens]) -> bool:
    """Check if tokens are valid and not expired.
    
    Args:
        tokens: AuthTokens to check (can be None)
        
    Returns:
        True if tokens exist and are not expired, False otherwise
        
    Complexity: O(1)
    """
    if tokens is None:
        return False
    
    return not tokens.is_expired()


def token_clear() -> bool:
    """Clear stored tokens.
    
    Deletes ~/.t27/notebooklm_tokens.json
    
    Returns:
        True if file was deleted or didn't exist, False on error
        
    Complexity: O(1)
    """
    if TOKEN_PATH.exists():
        try:
            TOKEN_PATH.unlink()
            return True
        except OSError:
            return False
    return True
