#!/usr/bin/env bash
# Install Git hooks for t27 Trinity S³AI
# Enforces L1 TRACEABILITY and other constitutional requirements

set -euo pipefail

# ANSI colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GIT_DIR="$PROJECT_ROOT/.git"
HOOKS_DIR="$GIT_DIR/hooks"

echo -e "${BLUE}Installing Git hooks for t27 Trinity S³AI...${NC}"
echo ""

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Install commit-msg hook for L1 TRACEABILITY
echo "Installing commit-msg hook (L1 TRACEABILITY enforcement)..."
cp "$SCRIPT_DIR/githooks/commit-msg-traceability" "$HOOKS_DIR/commit-msg"
chmod +x "$HOOKS_DIR/commit-msg"
echo -e "${GREEN}✓ commit-msg hook installed${NC}"

# Install pre-commit hook for L7 compliance (4-gate system)
echo ""
echo "Installing pre-commit hook (L7 compliance - 4-gate system)..."
cp "$SCRIPT_DIR/pre-commit" "$HOOKS_DIR/pre-commit"

chmod +x "$HOOKS_DIR/pre-commit"
echo -e "${GREEN}✓ pre-commit hook installed${NC}"

# Install pre-push hook for L4 TESTABILITY
echo ""
echo "Installing pre-push hook (L4 TESTABILITY check)..."
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/usr/bin/env bash
# L4 TESTABILITY Pre-Push Hook
# Warns if .t27 files are being pushed without test/invariant/bench

set -euo pipefail

# ANSI colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check for .t27 files in push
# Compare the local tip of the push range against its remote counterpart.
# pre-push hooks receive on stdin: <local_ref> <local_sha> <remote_ref> <remote_sha>
# We fall back to HEAD vs origin/HEAD when invoked manually.
while read -r local_ref local_sha remote_ref remote_sha; do
    if [ "$local_sha" = "0000000000000000000000000000000000000000" ]; then
        continue  # branch deletion
    fi
    if [ "$remote_sha" = "0000000000000000000000000000000000000000" ]; then
        RANGE="$local_sha"  # new branch — diff everything
    else
        RANGE="${remote_sha}..${local_sha}"
    fi
    T27_FILES=$(git diff --name-only "$RANGE" | grep '\.t27$' || true)
    if [ -n "$T27_FILES" ]; then break; fi
done
T27_FILES="${T27_FILES:-}"

if [ -n "$T27_FILES" ]; then
    echo -e "${YELLOW}⚠️  Pushing .t27 files. Please ensure they contain test/invariant/bench blocks (L4 TESTABILITY)${NC}"
    echo "Files being pushed:"
    echo "$T27_FILES"
fi

# Advisory NMSE seal-staleness check (variant J). Non-blocking: if the
# committed NMSE manifest was certified against an older compiler.rs, warn the
# author at push time. Mirrors scripts/reseal-check.sh and seal-staleness-warn.yml.
# It NEVER reseals and NEVER blocks the push (always falls through to exit 0).
ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
if [ -x "$ROOT_DIR/scripts/reseal-check.sh" ]; then
    if ! "$ROOT_DIR/scripts/reseal-check.sh" --quiet >/dev/null 2>&1; then
        echo -e "${YELLOW}WARNING: NMSE seal is stale or unsealed vs compiler.rs (advisory).${NC}"
        echo "Run 'scripts/reseal-check.sh' for the reseal command. Push not blocked."
    fi
fi

exit 0
EOF

chmod +x "$HOOKS_DIR/pre-push"
echo -e "${GREEN}✓ pre-push hook installed${NC}"

echo ""
echo -e "${GREEN}All Git hooks installed successfully!${NC}"
echo ""
echo "Installed hooks:"
echo "  - commit-msg: Enforces L1 TRACEABILITY (Closes #N required)"
echo "  - pre-commit: L7 compliance (4-gate system - seal coverage, freshness, etc.)"
echo "  - pre-push: Warns about L4 TESTABILITY (test/invariant/bench)"
echo ""
echo "To skip hooks (not recommended):"
echo "  git commit --no-verify -m 'message'"
echo "  git push --no-verify"
echo ""
echo -e "${BLUE}φ² + φ⁻² = 3 | TRINITY${NC}"
