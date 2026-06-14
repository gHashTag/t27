#!/usr/bin/env bash
#
# reseal-check.sh -- local seal-staleness reporter for the NMSE certification.
#
# The certifying NMSE manifest (repro/numerics/nmse_manifest.json) is sealed
# against sha256(bootstrap/src/compiler.rs) via bootstrap/stage0/FROZEN_HASH
# (see compute_seal() in repro/numerics/nmse_gf16.py: a run is sealed only when
# --seal is passed AND the live compiler hashes to exactly the FROZEN_HASH
# digest). When compiler.rs changes, the manifest seal goes stale: the committed
# NMSE numbers were certified against an older compiler.
#
# This script tells you, locally and before you submit a PR, whether the seal is
# fresh and -- if stale -- prints the exact two-step reseal command. It NEVER
# rewrites any seal itself; refreezing is always an explicit, reviewed action.
# It mirrors the read-only logic of the .github/workflows/seal-staleness-warn.yml
# advisory CI job, so "green locally" matches "no warning in CI".
#
# Usage:
#   scripts/reseal-check.sh            # report; exit 0 fresh, 2 stale, 3 unsealed
#   scripts/reseal-check.sh --quiet    # only print the one-line verdict
#
# Exit codes: 0 = fresh, 2 = stale (compiler.rs != seal), 3 = unsealed/missing.

set -uo pipefail

# Resolve repo root from this script's location so it works from any CWD.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

MANIFEST="repro/numerics/nmse_manifest.json"
SRC="bootstrap/src/compiler.rs"
FROZEN="bootstrap/stage0/FROZEN_HASH"

QUIET=0
if [ "${1:-}" = "--quiet" ]; then QUIET=1; fi

log() { if [ "$QUIET" -eq 0 ]; then echo "$@"; fi; }

# Live digest of the stage0 compiler on this working tree.
if [ ! -f "$SRC" ]; then
  echo "reseal-check: seal source missing ($SRC); cannot verify." >&2
  exit 3
fi
LIVE="$(sha256sum "$SRC" | awk '{print $1}')"

# Digest the certifying manifest was sealed with.
SEAL="$(python3 -c "import json,sys; print(json.load(open('$MANIFEST')).get('seal',''))" 2>/dev/null || echo '')"

# Digest pinned in FROZEN_HASH (the value compute_seal certifies to).
FROZEN_DIGEST="$(awk '{print $1}' "$FROZEN" 2>/dev/null | head -n1)"

log "live  sha256(compiler.rs) = $LIVE"
log "manifest seal            = $SEAL"
log "FROZEN_HASH digest       = $FROZEN_DIGEST"
log ""

if [ -z "$SEAL" ] || [ "$SEAL" = "unsealed" ]; then
  echo "SEAL: UNSEALED -- manifest carries no seal; nothing to compare."
  echo "  To seal: $ python repro/numerics/nmse_gf16.py --seal"
  exit 3
fi

if [ "$LIVE" = "$SEAL" ]; then
  echo "SEAL: FRESH -- sha256(compiler.rs) matches the manifest seal. Certification is current."
  exit 0
fi

# Stale: the manifest was certified against a different compiler.
echo "SEAL: STALE -- sha256(compiler.rs)=${LIVE:0:12} != manifest seal=${SEAL:0:12}."
echo "  The committed NMSE numbers were certified against an older compiler.rs."
echo ""
echo "  To recertify against the current compiler (two explicit, reviewed steps):"
echo "    1) refreeze the hash:"
echo "       printf '%s  %s\\n' \"$LIVE\" \"$SRC\" > $FROZEN"
echo "    2) regenerate the sealed manifests (protocol default 2M samples, seed 2718281):"
echo "       python repro/numerics/nmse_gf16.py --seal"
echo ""
if [ -n "$FROZEN_DIGEST" ] && [ "$LIVE" != "$FROZEN_DIGEST" ]; then
  echo "  NOTE: sha256(compiler.rs) also differs from FROZEN_HASH=${FROZEN_DIGEST:0:12};"
  echo "        until FROZEN_HASH is updated, '--seal' will report 'unsealed'."
fi
exit 2
