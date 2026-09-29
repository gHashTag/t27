#!/usr/bin/env bash
# L1 TRACEABILITY gate — thin forwarder to the Rust implementation.
#
# Real logic lives in `cli/tri` (`tri hooks l1-check`). This file exists
# only so that pre-existing harness wiring that exec's the .sh path keeps
# working. Do not add logic here — edit `cli/tri/src/hooks.rs` instead.
#
# This is a commit/CI-time gate. It is deliberately NOT wired into Claude
# Code PreToolUse (see MIGRATION_AUDIT.md): re-checking an already-existing
# HEAD commit message on every tool call cannot prevent anything — the
# commit already exists — and only produced non-blocking hook-error spam
# whenever a wave branch legitimately carries its `Closes #N` in the PR
# body instead of per-commit. L1 for merges is enforced by the PR issue
# gate; commit-time gates live in `.githooks/`.
#
# The old bash fallback is gone on purpose: it had drifted from the Rust
# canon (accepted `Refs #N` / `Updates #N`, which `tri hooks l1-check`
# rejects). One rule, one home — `cli/tri/src/hooks.rs`.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

for p in \
  "$REPO_ROOT/target/release/tri" \
  "$REPO_ROOT/target/debug/tri" \
  ; do
  if [[ -x "$p" ]]; then
    exec "$p" hooks l1-check
  fi
done

echo "L1 gate: tri binary not built — run: cargo build --release -p tri" >&2
exit 1
