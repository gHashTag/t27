#!/usr/bin/env python3
# scripts/wrapup/extract-context.py
# Extract session context from .trinity state files
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Extract session context from .trinity state and output as JSON."""

import sys
import json
from pathlib import Path

# Add project root to path
repo_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(repo_root))

from contrib.backend.notebooklm.session import session_extract_from_current_dir


def main():
    """Extract and print session context as JSON."""
    # Extract context from current directory
    context = session_extract_from_current_dir()

    if context is None:
        print(json.dumps({
            "error": "Not in a t27 repository",
            "message": ".trinity directory not found"
        }, indent=2))
        sys.exit(1)

    # Output as JSON
    print(json.dumps(context, indent=2))


if __name__ == "__main__":
    main()
