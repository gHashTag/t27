#!/usr/bin/env bash
# scripts/verify-ssot-integration.sh
# Verification script for GitHub ↔ NotebookLM SSOT integration
# phi^2 + 1/phi^2 = 3 | TRINITY

set -euo pipefail

echo "=== GitHub ↔ NotebookLM SSOT Integration Verification ==="
echo

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

PASSED=0
FAILED=0

check_pass() {
    echo -e "${GREEN}✓${NC} $1"
    PASSED=$((PASSED + 1))
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    FAILED=$((FAILED + 1))
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. Check module structure
echo "1. Checking module structure..."

for module in "contrib/backend/github" "contrib/backend/notebooklm"; do
    if [[ -d "$module" ]]; then
        check_pass "$module/ exists"
    else
        check_fail "$module/ missing"
    fi
done

# 2. Check Python imports
echo
echo "2. Checking Python imports..."

if python3 -c "import sys; sys.path.insert(0, 'contrib/backend'); from github import GitHubClient" 2>/dev/null; then
    check_pass "github.GitHubClient imports"
else
    check_fail "github.GitHubClient import failed"
fi

if python3 -c "import sys; sys.path.insert(0, 'contrib/backend'); from notebooklm import UnifiedSyncOrchestrator" 2>/dev/null; then
    check_pass "notebooklm.UnifiedSyncOrchestrator imports"
else
    check_fail "notebooklm.UnifiedSyncOrchestrator import failed"
fi

# 3. Check wrapper scripts
echo
echo "3. Checking wrapper scripts..."

for script in "tri-issue-create.py" "tri-sync.py" "tri-search.py" "tri-doc-sync.py" "tri-pr-create.py"; do
    if [[ -f "scripts/$script" ]]; then
        check_pass "scripts/$script exists"
        if [[ -x "scripts/$script" ]]; then
            check_pass "scripts/$script is executable"
        else
            check_warn "scripts/$script not executable (run: chmod +x scripts/$script)"
        fi
    else
        check_fail "scripts/$script missing"
    fi
done

# 4. Check state files
echo
echo "4. Checking Trinity state files..."

if [[ -f ".trinity/state/github-bridge.json" ]]; then
    check_pass ".trinity/state/github-bridge.json exists"
else
    check_fail ".trinity/state/github-bridge.json missing"
fi

# 5. Check skill configuration
echo
echo "5. Checking /tri skill configuration..."

if grep -q "GitHub + NotebookLM Integration" .claude/skills/tri/skill.md 2>/dev/null; then
    check_pass "/tri skill has GitHub commands documented"
else
    check_fail "/tri skill missing GitHub commands"
fi

# 6. Check MCP server
echo
echo "6. Checking MCP server configuration..."

if [[ -f ".claude/mcp/tri-ssot/manifest.json" ]]; then
    check_pass "MCP manifest exists"
else
    check_fail "MCP manifest missing"
fi

# Summary
echo
echo "=== Summary ==="
echo -e "${GREEN}Passed:${NC} $PASSED"
echo -e "${RED}Failed:${NC} $FAILED"

if [[ $FAILED -eq 0 ]]; then
    echo -e "\n${GREEN}All checks passed!${NC}"
    exit 0
else
    echo -e "\n${RED}Some checks failed. Please fix the issues above.${NC}"
    exit 1
fi
