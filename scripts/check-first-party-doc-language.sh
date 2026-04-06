#!/usr/bin/env bash
# Wrapper: use Python for reliable Unicode (macOS grep can false-positive on φ, etc.)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec python3 "$ROOT/scripts/check_first_party_doc_language.py"
