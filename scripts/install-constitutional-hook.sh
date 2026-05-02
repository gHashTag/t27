#!/usr/bin/env sh
# Installs pre-commit hook: runs `cargo build` in bootstrap/ (Rust-only gates: FROZEN seal, LANG-EN, required files).
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOOK_DST="$ROOT/.git/hooks/pre-commit"
printf '%s\n' '#!/bin/sh' 'set -e' "cd \"\$(git rev-parse --show-toplevel)/bootstrap\" && cargo build -q" >"$HOOK_DST"
chmod +x "$HOOK_DST"
echo "Installed: $HOOK_DST (runs: cd bootstrap && cargo build)"
