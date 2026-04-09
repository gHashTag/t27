#!/usr/bin/env bash
# UserPromptSubmit Hook — Inject NotebookLM Context
#
# Reads the notebook context and injects it into each user prompt.
# Ensures Claude always knows the current task state.
#
# phi^2 + 1/phi^2 = 3 | TRINITY

set -euo pipefail

# Paths
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NOTEBOOK_DIR="$PROJECT_ROOT/.trinity/current_task"
NOTEBOOK_ID_FILE="$NOTEBOOK_DIR/.notebook_id"
NOTEBOOK_META_FILE="$NOTEBOOK_DIR/notebook_meta.json"
TEMPLATE_FILE="$NOTEBOOK_DIR/notebook_context.template"
ACTIVITY_FILE="$NOTEBOOK_DIR/activity.md"

# Load notebook info
load_notebook_info() {
    # Read notebook ID
    local nb_id=""
    if [[ -f "$NOTEBOOK_ID_FILE" ]]; then
        nb_id=$(cat "$NOTEBOOK_ID_FILE")
    fi

    # Read notebook metadata
    if [[ -f "$NOTEBOOK_META_FILE" ]]; then
        TASK_TITLE=$(jq -r '.task_title // empty' "$NOTEBOOK_META_FILE")
        else
        TASK_TITLE="Unknown Task"
    fi
}

# Generate context from notebook
generate_context() {
    local nb_id="$1"
    local context=""

    # Get recent activity (last 10 lines, trimmed)
    if [[ -f "$ACTIVITY_FILE" ]]; then
        context=$(tail -n 10 "$ACTIVITY_FILE" | sed 's/^[[:space:]]*//' | sed 's/^/  //' | sed 's/^  */ //')
    fi

    # Get task title
    load_notebook_info

    # Build JSON output
    cat << JSON
{
  "notebook_id": "$nb_id",
  "task_title": "$TASK_TITLE",
  "activity_snapshot": $(echo "$context" | jq -Rrs '.' | jq -Rs 'join("\\n")' | sed 's/^"|$/""'),
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "sources_count": "0"
  "last_sync": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
JSON
}

# Main
main() {
    # Read notebook ID
    local nb_id=""
    if [[ -f "$NOTEBOOK_ID_FILE" ]]; then
        nb_id=$(cat "$NOTEBOOK_ID_FILE")
        if [[ "$nb_id" == "disabled" ]]; then
            echo '{"notebook_id": "disabled", "blocked": true}'
            exit 0
        fi
    fi

    # Generate context
    generate_context
}

main "$@"
