#!/usr/bin/env bash
#
# reseal-apply.sh -- the explicit, reviewed reseal action for the NMSE
# certification. This is the deliberate "yes, recertify now" companion to the
# read-only scripts/reseal-check.sh reporter.
#
# Background: the certifying NMSE manifest (repro/numerics/nmse_manifest.json)
# is sealed against sha256(bootstrap/src/compiler.rs) via
# bootstrap/stage0/FROZEN_HASH. When compiler.rs changes, the seal goes stale.
# reseal-check.sh only REPORTS staleness and prints the two-step reseal command;
# it never rewrites a seal. Until now there was no single safe entry point that
# actually performs the reseal -- the author had to copy/paste the two commands.
#
# This script provides that entry point WITHOUT weakening the honest-by-design
# seal model:
#   - It does NOTHING unless the operator explicitly confirms (interactive
#     "yes" prompt, or RESEAL_YES=1 for non-interactive/CI-author use).
#   - It performs exactly the two documented, reviewed steps, then re-verifies.
#   - It refuses to fabricate a seal: regeneration goes through
#     repro/numerics/nmse_gf16.py --seal, which only seals when the live
#     compiler hashes to FROZEN_HASH (compute_seal()). This script updates
#     FROZEN_HASH first precisely so that contract is satisfiable, and the
#     update is the explicit, logged action the operator just confirmed.
#
# Refreezing remains a deliberate human decision: this script makes it
# one-command and auditable, it does not make it automatic. It is NOT wired into
# CI and is never run by any required check.
#
# Usage:
#   scripts/reseal-apply.sh            # report, then prompt to confirm
#   RESEAL_YES=1 scripts/reseal-apply.sh   # confirm non-interactively
#   make seal                          # same, via the Makefile entry point
#
# Exit codes:
#   0 = already fresh (nothing to do) OR reseal applied and verified fresh
#   1 = declined / aborted by the operator
#   2 = reseal attempted but verification still not fresh (manual review needed)
#   3 = preconditions missing (source/manifest absent)
#
# Anchor: phi^2 + phi^-2 = 3

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

MANIFEST="repro/numerics/nmse_manifest.json"
SRC="bootstrap/src/compiler.rs"
FROZEN="bootstrap/stage0/FROZEN_HASH"
GEN="repro/numerics/nmse_gf16.py"

if [ ! -f "$SRC" ]; then
    echo "reseal-apply: seal source missing ($SRC); cannot reseal." >&2
    exit 3
fi
if [ ! -f "$MANIFEST" ]; then
    echo "reseal-apply: manifest missing ($MANIFEST); cannot reseal." >&2
    exit 3
fi

# 1) Report current status via the read-only checker (single source of truth).
echo "Current seal status:"
echo "----------------------------------------------------------------"
"$SCRIPT_DIR/reseal-check.sh"
CHECK_RC=$?
echo "----------------------------------------------------------------"

if [ "$CHECK_RC" -eq 0 ]; then
    echo "reseal-apply: seal already FRESH -- nothing to do."
    exit 0
fi

LIVE="$(sha256sum "$SRC" | awk '{print $1}')"

echo ""
echo "This will RECERTIFY the NMSE seal against the current compiler.rs by:"
echo "  1) refreezing the hash:"
echo "       printf '%s  %s\\n' \"$LIVE\" \"$SRC\" > $FROZEN"
echo "  2) regenerating the sealed manifests (protocol default 2M samples,"
echo "     seed 2718281):"
echo "       python $GEN --seal"
echo ""
echo "Refreezing is a deliberate, reviewed action: the committed NMSE numbers"
echo "will be re-attributed to THIS compiler. Only proceed if that is intended."
echo ""

# 2) Require explicit confirmation. RESEAL_YES=1 confirms non-interactively.
if [ "${RESEAL_YES:-0}" = "1" ]; then
    echo "RESEAL_YES=1 set -- proceeding without interactive prompt."
else
    if [ ! -t 0 ]; then
        echo "reseal-apply: no TTY and RESEAL_YES is not set; aborting (no reseal)."
        echo "  Re-run interactively, or set RESEAL_YES=1 to confirm."
        exit 1
    fi
    printf "Type 'yes' to recertify now: "
    read -r ANSWER
    if [ "$ANSWER" != "yes" ]; then
        echo "reseal-apply: not confirmed (got '$ANSWER') -- aborted, no changes made."
        exit 1
    fi
fi

# 3) Perform the two explicit, reviewed steps.
echo ""
echo "[1/2] refreezing FROZEN_HASH ..."
printf '%s  %s\n' "$LIVE" "$SRC" > "$FROZEN"
echo "      $FROZEN now pins ${LIVE:0:12}..."

echo "[2/2] regenerating sealed manifests via $GEN --seal ..."
if ! python3 "$GEN" --seal; then
    echo "reseal-apply: manifest regeneration FAILED; review the output above." >&2
    exit 2
fi

# 4) Re-verify: a successful reseal must now report FRESH.
echo ""
echo "Re-verifying ..."
echo "----------------------------------------------------------------"
"$SCRIPT_DIR/reseal-check.sh"
VERIFY_RC=$?
echo "----------------------------------------------------------------"
if [ "$VERIFY_RC" -eq 0 ]; then
    echo "reseal-apply: DONE -- seal is now fresh. Review and commit the updated"
    echo "  $FROZEN and $MANIFEST as part of an explicit reseal commit."
    exit 0
fi
echo "reseal-apply: reseal ran but verification is still not fresh (rc=$VERIFY_RC);"
echo "  manual review needed -- do NOT commit a half-applied reseal."
exit 2
