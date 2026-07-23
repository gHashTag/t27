#!/usr/bin/env bash
# SessionStart Hook — Claude Code Hooks
#
# This hook runs as the FIRST LINE of user prompts.
# It blocks Claude Code until a notebook is available.
#
# phi^2 + 1/phi^2 = 3 | TRINITY

set -euo pipefail

# Paths
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NOTEBOOK_DIR="$PROJECT_ROOT/.trinity/current_task"
NOTEBOOK_ID_FILE="$NOTEBOOK_DIR/.notebook_id"
NOTEBOOK_META_FILE="$NOTEBOOK_DIR/notebook_meta.json"

# Logging
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [session-gate] $*"
}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if notebook gating is enabled
if [[ -f "$PROJECT_ROOT/.trinity/enable_notebook_gate" ]]; then
    GATE_ENABLED=true
else
    GATE_ENABLED=false
fi

# Main check
check_notebook() {
    # Check for active notebook ID
    if [[ -f "$NOTEBOOK_ID_FILE" ]]; then
        CURRENT_ID=$(cat "$NOTEBOOK_ID_FILE")
        if [[ -n "$CURRENT_ID" ]] && [[ "$CURRENT_ID" != "disabled" ]]; then
            log "${GREEN}Notebook found: ${NC}$CURRENT_ID${NC}"
            return 0
        fi
    fi

    log "${YELLOW}No active notebook or gating disabled"
    return 1
}

# Create notebook with fallback
create_notebook() {
    local task_title="$1"

    log "${YELLOW}Creating notebook for task: ${NC}$task_title${NC}"
    log "${YELLOW}Calling: ${NC}t27c task start${NC}"

    # Call t27c to create notebook
    cd "$PROJECT_ROOT"
    local output
    output=$(cargo run --release --quiet -- task start --title "$task_title" 2>&1)

    # Parse notebook ID from output (handle format: "Notebook: nb-XXXX-YYYYMMDD-HHMMSS")
    local nb_id
    if [[ "$output" =~ Notebook:\ (nb-[0-9a-f]{8}-[0-9]{4}-[0-9]{2}-[0-9]{2} ]]; then
        nb_id="${BASH_REMATCH[0]}"
        log "${GREEN}Notebook ID: ${NC}$nb_id${NC}"
    else
        log "${RED}Failed to parse notebook ID"
        nb_id="nb-fallback-$(date +%s)"
    fi

    # Save notebook ID
    echo "$nb_id" > "$NOTEBOOK_ID_FILE"

    # Create metadata file
    cat > "$NOTEBOOK_META_FILE" << EOF
{
  "notebook_id": "$nb_id",
  "task_title": "$task_title",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "last_sync": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

    log "${GREEN}Notebook created: ${NC}$nb_id${NC}"
    return 0
}

# Main
main() {
    log "SessionStart gate - checking for active notebook"

    # Check if gating is enabled
    if [[ "$GATE_ENABLED" != "true" ]]; then
        log "Notebook gating is DISABLED (via .trinity/enable_notebook_gate)"
        log "Proceeding without gate"
        # Continue without notebook (fallback behavior)
        exit 0
    fi

    # Check for existing notebook
    if check_notebook; then
        log "Active notebook found, proceeding"
        # Write the notebook ID to stdout for Claude to see
        echo "$nb_id"
        exit 0
    fi

    # Create new notebook
    if create_notebook; then
        log "Notebook created successfully"
        # Write the notebook ID to stdout for Claude to see
        echo "$nb_id"
        exit 0
    fi

    # No notebook found - block
    log "${RED}No notebook available - BLOCKING"
    echo "BLOCKED: No NotebookLM notebook found"
    echo ""
    log "To unblock:"
    log "  1. Run: t27c task start --title 'YOUR_TASK' --sources relevant.md"
    log "  2. Restart Claude Code"
    exit 1
}

main "$@"
