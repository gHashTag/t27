#!/usr/bin/env bash
# Rebuild an evidential binary from a NAMED commit and seal it in the same step,
# so that provenance is captured at build time rather than guessed afterwards.
#
#   scripts/ci/rebuild_evidence.sh <commit-ish> <label> [out-dir]
#
# The binary is written under out-dir (default docs/evidence/bin, gitignored --
# a 14 MB binary does not belong in the repository; the seal beside it does).
set -euo pipefail
COMMIT="${1:?usage: rebuild_evidence.sh <commit-ish> <label> [out-dir]}"
LABEL="${2:?label required}"
OUTDIR="${3:-docs/evidence/bin}"
REPO="$(git rev-parse --show-toplevel)"
SHA="$(git rev-parse "$COMMIT")"
# A FIXED path, not mktemp. A debug build embeds the absolute path of its source
# tree in debug info, so a randomly named worktree makes every build produce a
# different SHA-256 and the digest comparison in `artifact-seal verify --rebuild`
# can never succeed. Measured: two builds of the same commit from differently
# named worktrees differed in 39,830,933 bytes, and the sealed binary contained
# the string /tmp/t27_evidence_5UEbqk. The seal must be rebuilt from the same
# path it was built from, so that path is a constant shared with the verifier.
WT="${TRI_SEAL_BUILD_DIR:-/tmp/t27_seal_build}"
rm -rf "$WT"

cleanup() { git -C "$REPO" worktree remove --force "$WT" >/dev/null 2>&1 || true; rm -rf "$WT"; }
trap cleanup EXIT

git -C "$REPO" worktree add --detach "$WT" "$SHA" >/dev/null
# Reuse the already-populated target dir: the native aws-lc-sys dependency does
# not build from a cold cache in this sandbox, and rebuilding it is not what is
# being demonstrated. The SOURCE still comes from the named commit.
( cd "$WT/bootstrap" && CARGO_TARGET_DIR="${TRI_EVIDENCE_TARGET:-$REPO/target/evidence}" cargo build --quiet )

mkdir -p "$REPO/$OUTDIR"
BIN="$REPO/$OUTDIR/t27c.$LABEL"
cp "${TRI_EVIDENCE_TARGET:-$REPO/target/evidence}/debug/t27c" "$BIN"

# --repo points at the worktree, so the seal records the commit that was
# actually built, not whatever HEAD the caller happens to be sitting on.
python3 "$REPO/scripts/ci/artifact_seal.py" create \
  --label "$LABEL" \
  --purpose "evidential binary rebuilt from a named commit" \
  --artifact "$BIN" \
  --produced "${TRI_EVIDENCE_TARGET:-$REPO/target/evidence}/debug/t27c" \
  --profile dev \
  --build-cmd "cd bootstrap && CARGO_TARGET_DIR=${TRI_EVIDENCE_TARGET:-$REPO/target/evidence} cargo build --quiet" \
  --repo "$WT" \
  --out "$REPO/docs/evidence/seal_$LABEL.json"
