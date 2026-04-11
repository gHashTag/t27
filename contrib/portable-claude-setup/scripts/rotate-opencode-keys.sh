#!/bin/bash
# Rotate ZAI API keys in opencode auth.json
#
# Reads ZAI_KEY_1..N from ~/.claude/.env (or --env FILE) and rotates
# the "anthropic" credential in ~/.local/share/opencode/auth.json.
# Also writes ZAI_KEY_N as separate providers (zai-plan-1..N) for
# model-level failover.
#
# Usage:
#   rotate-opencode-keys.sh [OPTIONS]
#
# Options:
#   --round-robin   (default) advance to next key
#   --random        pick a random key
#   --env FILE      path to .env file  (default: ~/.claude/.env)
#   --auth FILE     path to auth.json  (default: ~/.local/share/opencode/auth.json)
#   --status        show current state and exit

set -euo pipefail

ENV_FILE="${HOME}/.claude/.env"
AUTH_FILE="${HOME}/.local/share/opencode/auth.json"
STATE_FILE="${HOME}/.claude/.opencode-rotation-state.json"
STRATEGY="round-robin"
SHOW_STATUS=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --round-robin) STRATEGY="round-robin"; shift ;;
        --random)      STRATEGY="random"; shift ;;
        --env)         ENV_FILE="$2"; shift 2 ;;
        --auth)        AUTH_FILE="$2"; shift 2 ;;
        --status)      SHOW_STATUS=true; shift ;;
        *)             echo "Unknown option: $1"; exit 1 ;;
    esac
done

now_iso() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

ensure_state_file() {
    if [ ! -f "$STATE_FILE" ]; then
        echo '{}' > "$STATE_FILE"
    fi
}

read_state() {
    jq -r ".[\"$1\"] // empty" "$STATE_FILE" 2>/dev/null || echo ""
}

mask_key() {
    local key="$1"
    local len=${#key}
    if [ "$len" -le 8 ]; then
        echo "****"
    else
        echo "${key:0:4}...${key: -4}"
    fi
}

count_keys() {
    local count=0
    local i=1
    while true; do
        local vn="ZAI_KEY_${i}"
        local vv
        vv=$(printenv "$vn" 2>/dev/null || true)
        if [ -z "$vv" ]; then
            break
        fi
        count=$((count + 1))
        i=$((i + 1))
    done
    echo "$count"
}

if [ ! -f "$ENV_FILE" ]; then
    echo "Missing: $ENV_FILE"
    echo "Create it: cp contrib/portable-claude-setup/env.example ~/.claude/.env"
    exit 1
fi

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

ensure_state_file

# Collect all ZAI keys
KEYS=()
i=1
while true; do
    vn="ZAI_KEY_${i}"
    vv=$(printenv "$vn" 2>/dev/null || true)
    if [ -z "$vv" ]; then break; fi
    KEYS+=("$vv")
    i=$((i + 1))
done
total=${#KEYS[@]}

if [ "$total" -eq 0 ]; then
    echo "No ZAI_KEY_N found in $ENV_FILE"
    exit 1
fi

# ── Status ────────────────────────────────────────────────────────────
if [ "$SHOW_STATUS" = true ]; then
    echo "=== OpenCode Key Rotation State ==="
    echo "auth.json: $AUTH_FILE"
    echo "env file:  $ENV_FILE"
    echo ""
    echo "ZAI keys found: $total"
    state_json=$(read_state "zai")
    if [ -n "$state_json" ]; then
        cur_idx=$(echo "$state_json" | jq -r '.current_index')
        last_rot=$(echo "$state_json" | jq -r '.last_rotated')
        echo "Current index: $cur_idx"
        echo "Last rotated:  $last_rot"
    else
        echo "No rotation state yet (will start at 1)"
    fi
    echo ""
    echo "Keys:"
    for j in $(seq 1 "$total"); do
        masked=$(mask_key "${KEYS[$((j-1))]}")
        echo "  ZAI_KEY_$j = $masked"
    done
    exit 0
fi

# ── Determine new index ───────────────────────────────────────────────
state_json=$(read_state "zai")
if [ -n "$state_json" ]; then
    cur_idx=$(echo "$state_json" | jq -r '.current_index')
else
    cur_idx=1
fi

case "$STRATEGY" in
    round-robin)
        new_idx=$(( (cur_idx % total) + 1 ))
        ;;
    random)
        new_idx=$(( (RANDOM % total) + 1 ))
        if [ "$total" -gt 1 ] && [ "$new_idx" -eq "$cur_idx" ]; then
            new_idx=$(( (new_idx % total) + 1 ))
        fi
        ;;
esac

# ── Update auth.json ──────────────────────────────────────────────────
if [ ! -f "$AUTH_FILE" ]; then
    echo "{}" > "$AUTH_FILE"
    chmod 600 "$AUTH_FILE"
fi

active_key="${KEYS[$((new_idx-1))]}"

# Build auth.json with all providers
# Primary "anthropic" gets the active key
# Additional "zai-1".."zai-N" get each key for fallback
TMP="${AUTH_FILE}.tmp.$$"

python3 -c "
import json, sys

auth_file = sys.argv[1]
active_key = sys.argv[2]
all_keys = sys.argv[3].split('\t')
new_idx = int(sys.argv[4])

try:
    with open(auth_file) as f:
        auth = json.load(f)
except:
    auth = {}

# Set active key as anthropic provider (Z.AI uses Anthropic API compat)
auth['anthropic'] = {'type': 'api', 'key': active_key}

# Also set zai-coding-plan with active key
auth['zai-coding-plan'] = {'type': 'api', 'key': active_key}

# Store all keys as numbered zai providers for manual fallback
for i, k in enumerate(all_keys, 1):
    auth[f'zai-{i}'] = {'type': 'api', 'key': k}

with open(auth_file, 'w') as f:
    json.dump(auth, f, indent=2)
" "$AUTH_FILE" "$active_key" "$(printf '%s\t' "${KEYS[@]}")" "$new_idx"

chmod 600 "$AUTH_FILE" 2>/dev/null || true

# ── Update rotation state ─────────────────────────────────────────────
jq --arg now "$(now_iso)" --argjson idx "$new_idx" --argjson tot "$total" \
    '.["zai"] = {"current_index": $idx, "last_rotated": $now, "total_keys": $tot}' \
    "$STATE_FILE" > "${STATE_FILE}.tmp" && mv "${STATE_FILE}.tmp" "$STATE_FILE"

masked=$(mask_key "$active_key")
echo "Rotated ZAI keys: index $cur_idx -> $new_idx (of $total) [$STRATEGY]"
echo "Active key: $masked"
echo "auth.json updated: $AUTH_FILE"
