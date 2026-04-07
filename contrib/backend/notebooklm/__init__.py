# contrib/backend/notebooklm/__init__.py
# NotebookLM Integration Backend for t27
# Ring-071 - RAG-Backed Semantic Memory
# phi^2 + 1/phi^2 = 3 | TRINITY

"""NotebookLM integration for t27.

This module provides synchronous wrappers around the async notebooklm-py SDK
for integration with t27's synchronous workflow.
"""

from .config import config_from_env, NotebookLMConfig, DEFAULT_CONFIG
from .token import token_load, token_save, token_is_valid, token_clear, AuthTokens
from .cookie_auth import authenticate_with_cookies, notebooklm_client_init, test_notebooklm_sdk_integration
from .client import client_new, client_authenticate, client_close, client_is_authenticated
from .notebooks import notebook_create, notebook_list, notebook_get, notebook_find_by_name, notebook_delete
from .sources import source_upload_text, source_upload_file, source_list, source_delete
from .queries import notebook_query
from .session import session_extract_from_trinity
from .wrapup import wrapup_format_summary, wrapup_upload

__version__ = "0.1.0"
__all__ = [
    # Config
    "NotebookLMConfig",
    "config_from_env",
    # Token
    "token_load",
    "token_save",
    "token_is_valid",
    # Auth
    "authenticate_with_cookies",
    "notebooklm_client_init",
    # Client
    "client_new",
    "client_authenticate",
    "client_close",
    "client_is_authenticated",
    # Notebooks
    "notebook_create",
    "notebook_list",
    "notebook_get",
    "notebook_find_by_name",
    "notebook_delete",
    # Sources
    "source_upload_text",
    "source_upload_file",
    "source_list",
    "source_delete",
    # Queries
    "notebook_query",
    # Session
    "session_extract_from_trinity",
    # Wrapup
    "wrapup_format_summary",
    "wrapup_upload",
]
