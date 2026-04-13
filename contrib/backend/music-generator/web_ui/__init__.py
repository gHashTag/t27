# contrib/backend/music-generator/web_ui/__init__.py
# Gradio Web Interface Module
# phi^2 + 1/phi^2 = 3 | TRINITY

from .app import create_app, run_app

__all__ = [
    "create_app",
    "run_app",
]
