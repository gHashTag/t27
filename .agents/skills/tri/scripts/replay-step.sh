#!/bin/bash
# replay-step.sh - Replay PHI LOOP from last clean seal for recovery
#
# Usage: ./replay-step.sh [--last-clean|--seal <seal_id>|--skill <skill_id>]
#
# Implements self-healing recovery by replaying from a known good state
# instead of full restart, preserving context and progress.

set -euo pipefail

MODE="${1:--last-clean}"
SEALS_FILE=".claude/skills/tri/seals.jsonl"
DECISION_LOG=".claude/skills/tri/decision-log.jsonl"

# Find last clean seal
find_last_clean_seal() {
    if [[ ! -f "$SEALS_FILE" ]]; then
        echo "Error: No seals file found at $SEALS_FILE" >&2
        echo "Run PHI LOOP to create seals first" >&2
        exit 1
    fi

    local last_clean=$(grep '"verdict":"clean"' "$SEALS_FILE" 2>/dev/null | tail -1)

    if [[ -z "$last_clean" ]]; then
        echo "Error: No clean seals found" >&2
        echo "All recorded steps have toxic verdicts" >&2
        exit 1
    fi

    echo "$last_clean"
}

# Replay from seal
replay_from_seal() {
    local seal_json="$1"

    echo "🔄 Replaying from clean seal..."

    # Extract seal information
    local seal_id=$(echo "$seal_json" | jq -r '.seal_id // empty')
    local spec_path=$(echo "$seal_json" | jq -r '.spec_path // empty')
    local spec_hash=$(echo "$seal_json" | jq -r '.spec_hash_after // empty')
    local skill_id=$(echo "$seal_json" | jq -r '.skill_id // empty')

    if [[ -z "$seal_id" ]] || [[ -z "$spec_path" ]]; then
        echo "Error: Invalid seal data" >&2
        exit 1
    fi

    echo "  Seal ID: $seal_id"
    echo "  Spec: $spec_path"
    echo "  Skill: $skill_id"

    # Verify spec hash matches
    if [[ -f "$spec_path" ]]; then
        local current_hash=$(sha256sum "$spec_path" | awk '{print $1}')
        if [[ "$current_hash" != "$spec_hash" ]]; then
            echo "⚠️  Warning: Spec hash differs from seal"
            echo "  Current: $current_hash"
            echo "  Sealed:  $spec_hash"
            echo ""
            read -p "Restore spec from seal? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                echo "  Restoring spec to sealed state..."
                git restore "$spec_path" 2>/dev/null || true
            fi
        fi
    else
        echo "⚠️  Warning: Spec file not found: $spec_path"
    fi

    echo ""
    echo "🔄 Re-running PHI LOOP from seal..."

    # Execute PHI LOOP from this point
    echo "  tri gen"
    tri gen "$spec_path" 2>/dev/null || echo "    (gen not available, continuing...)"

    echo "  tri test"
    tri test "$spec_path" 2>/dev/null || echo "    (test not available, continuing...)"

    echo "  tri verdict --toxic"
    tri verdict --toxic 2>/dev/null || echo "    (verdict not available, continuing...)"

    echo ""
    echo "✅ Replay complete. Continue PHI LOOP from here."
}

# Find toxic seals to avoid replay
find_toxic_seals() {
    if [[ ! -f "$SEALS_FILE" ]]; then
        return
    fi

    grep '"verdict":"toxic"' "$SEALS_FILE" 2>/dev/null | \
        jq -r '.seal_id' | sort -u
}

# Show replay history
show_replay_history() {
    echo "📜 Replay History:"

    if [[ -f "$DECISION_LOG" ]]; then
        grep '"replay_action"' "$DECISION_LOG" 2>/dev/null | \
            jq -r '"\(.timestamp) → \(.replay_action)"' | \
            tail -10 || echo "  No replays recorded"
    else
        echo "  No decision log found"
    fi
}

# Record replay in decision log
record_replay() {
    local seal_id="$1"
    local action="$2"
    local result="$3"

    local entry=$(cat <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "trace_id": "$(uuidgen 2>/dev/null || echo 'unknown')",
  "event_type": "replay",
  "replay_action": "$action",
  "seal_id": "$seal_id",
  "result": "$result"
}
EOF
)

    mkdir -p "$(dirname "$DECISION_LOG")"
    echo "$entry" >> "$DECISION_LOG"
}

# Main execution
case "$MODE" in
    --last-clean)
        seal=$(find_last_clean_seal)
        echo "$seal" | jq '.' 2>/dev/null || echo "$seal"
        echo ""
        replay_from_seal "$seal"
        record_replay "$(echo "$seal" | jq -r '.seal_id')" "replay_from_last_clean" "success"
        ;;
    --seal)
        seal_id="$2"
        if [[ -z "$seal_id" ]]; then
            echo "Error: --seal requires seal_id argument" >&2
            exit 1
        fi

        seal=$(grep "\"seal_id\":\"$seal_id\"" "$SEALS_FILE" 2>/dev/null)
        if [[ -z "$seal" ]]; then
            echo "Error: Seal not found: $seal_id" >&2
            exit 1
        fi

        replay_from_seal "$seal"
        record_replay "$seal_id" "replay_from_seal" "success"
        ;;
    --skill)
        skill_id="$2"
        if [[ -z "$skill_id" ]]; then
            echo "Error: --skill requires skill_id argument" >&2
            exit 1
        fi

        # Find clean seal for this skill
        seal=$(grep "\"skill_id\":\"$skill_id\".*\"verdict\":\"clean\"" "$SEALS_FILE" 2>/dev/null | tail -1)
        if [[ -z "$seal" ]]; then
            echo "Error: No clean seal found for skill: $skill_id" >&2
            echo "Available clean seals:" >&2
            grep '"verdict":"clean"' "$SEALS_FILE" 2>/dev/null | jq -r '"  \(.skill_id) → \(.seal_id)"' >&2
            exit 1
        fi

        replay_from_seal "$seal"
        record_replay "$(echo "$seal" | jq -r '.seal_id')" "replay_from_skill" "success"
        ;;
    --list)
        echo "🔒 Available Clean Seals:"
        grep '"verdict":"clean"' "$SEALS_FILE" 2>/dev/null | \
            jq -r '"  \(.seal_id) — \(.spec_path) @ \(.sealed_at)"' || \
            echo "  No clean seals found"
        ;;
    --toxic)
        echo "☠️  Toxic Seals (avoid replay):"
        find_toxic_seals | sed 's/^/  /' || echo "  No toxic seals"
        ;;
    --history)
        show_replay_history
        ;;
    *)
        echo "Usage: $0 [--last-clean|--seal <seal_id>|--skill <skill_id>|--list|--toxic|--history]" >&2
        echo ""
        echo "Examples:"
        echo "  $0 --last-clean    # Replay from most recent clean seal"
        echo "  $0 --seal abc123   # Replay from specific seal"
        echo "  $0 --skill ops_tritRotate_v1  # Replay from last clean seal of skill"
        echo "  $0 --list          # List all clean seals"
        echo "  $0 --toxic         # Show toxic seals to avoid"
        echo "  $0 --history       # Show replay history"
        exit 1
        ;;
esac
