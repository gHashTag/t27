#!/bin/bash
# Rotate API keys across numbered pools (round-robin, random, or health-check)
#
# Reads KEY_FAMILY_1, KEY_FAMILY_2, … from ~/.claude/.env and advances
# the active index tracked in ~/.claude/.rotation-state.json.
# After rotation, calls sync-settings-from-env.sh to update settings.json.
#
# Supported families:
#   zai       — ZAI_KEY_1 … ZAI_KEY_N   (Anthropic / Z.AI tokens)
#   railway   — RAILWAY_TOKEN_1 … N      (Railway API tokens)
#   openai    — OPENAI_KEY_1 … N         (OpenAI keys)
#   gh        — GH_TOKEN_1 … N           (GitHub tokens)
#
# Usage:
#   rotate-keys.sh [OPTIONS]
#
# Options:
#   --round-robin   (default) advance to next key
#   --random        pick a random key
#   --health-check  test each key, use first working one
#   --family NAME   rotate only the given family (zai|railway|openai|gh)
#   --status        show current rotation state and exit
#   --env FILE      path to .env file  (default: ~/.claude/.env)
#   --settings FILE path to settings.json (default: ~/.claude/settings.json)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="${HOME}/.claude/.env"
SETTINGS="${HOME}/.claude/settings.json"
STATE_FILE="${HOME}/.claude/.rotation-state.json"
STRATEGY="round-robin"
FAMILY_FILTER=""
SHOW_STATUS=false

# ── Parse arguments ──────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --round-robin)  STRATEGY="round-robin"; shift ;;
        --random)       STRATEGY="random"; shift ;;
        --health-check) STRATEGY="health-check"; shift ;;
        --family)       FAMILY_FILTER="$2"; shift 2 ;;
        --status)       SHOW_STATUS=true; shift ;;
        --env)          ENV_FILE="$2"; shift 2 ;;
        --settings)     SETTINGS="$2"; shift 2 ;;
        *)              echo "Unknown option: $1"; exit 1 ;;
    esac
done

# ── Helpers ──────────────────────────────────────────────────────────

now_iso() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

ensure_state_file() {
    if [ ! -f "$STATE_FILE" ]; then
        echo '{}' > "$STATE_FILE"
    fi
}

read_state() {
    jq -r ".[\"$1\"] // empty" "$STATE_FILE"
}

# Count keys for a family (e.g. ZAI_KEY_1 … ZAI_KEY_N)
count_keys() {
    local prefix="$1"
    local count=0
    local i=1
    while true; do
        local vn="${prefix}_${i}"
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

# Map family name → env-var prefix
family_prefix() {
    case "$1" in
        zai)     echo "ZAI_KEY" ;;
        railway) echo "RAILWAY_TOKEN" ;;
        openai)  echo "OPENAI_KEY" ;;
        gh)      echo "GH_TOKEN" ;;
        *)       echo ""; return 1 ;;
    esac
}

# Map family → the env var sync-settings uses
family_active_var() {
    case "$1" in
        zai)     echo "ZAI_USE" ;;
        railway) echo "RAILWAY_ACTIVE_TOKEN" ;;
        openai)  echo "OPENAI_API_KEY" ;;
        gh)      echo "GH_TOKEN" ;;
        *)       echo "" ;;
    esac
}

mask_key() {
    local key="$1"
    local len=${#key}
    if [ "$len" -le 8 ]; then
        echo "****"
    else
        echo "${key:0:4}…${key: -4}"
    fi
}

# ── Health checks ────────────────────────────────────────────────────

check_railway_key() {
    local token="$1"
    local resp
    resp=$(curl -s --max-time 10 \
        --request POST \
        --url "https://backboard.railway.com/graphql/v2" \
        --header "Authorization: Bearer $token" \
        --header "Content-Type: application/json" \
        --data '{"query":"{ projects { edges { node { id name } } } }"}' 2>/dev/null || echo "")
    if echo "$resp" | jq -e '.data.projects' >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

check_gh_key() {
    local token="$1"
    local resp
    resp=$(curl -s --max-time 10 \
        -H "Authorization: token $token" \
        "https://api.github.com/user" 2>/dev/null || echo "")
    if echo "$resp" | jq -e '.login' >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

check_key() {
    local family="$1"
    local token="$2"
    case "$family" in
        railway) check_railway_key "$token" ;;
        gh)      check_gh_key "$token" ;;
        *)       return 0 ;;  # no health check available — assume valid
    esac
}

# ── Source .env ──────────────────────────────────────────────────────
if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Missing file: $ENV_FILE"
    exit 1
fi

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

ensure_state_file

# ── Status display ───────────────────────────────────────────────────
if [ "$SHOW_STATUS" = true ]; then
    echo "=== Key Rotation State ==="
    echo "State file: $STATE_FILE"
    echo ""
    for fam in zai railway openai gh; do
        prefix=$(family_prefix "$fam")
        total=$(count_keys "$prefix")
        if [ "$total" -eq 0 ]; then
            echo "  $fam: no keys configured"
            continue
        fi
        state_json=$(read_state "$fam")
        if [ -n "$state_json" ]; then
            cur_idx=$(echo "$state_json" | jq -r '.current_index')
            last_rot=$(echo "$state_json" | jq -r '.last_rotated')
            echo "  $fam: index=$cur_idx/$total  last_rotated=$last_rot"
        else
            echo "  $fam: $total keys, no rotation state yet (will start at 1)"
        fi
    done
    exit 0
fi

# ── Build family list ────────────────────────────────────────────────
FAMILIES="zai railway openai gh"
if [ -n "$FAMILY_FILTER" ]; then
    if ! family_prefix "$FAMILY_FILTER" >/dev/null 2>&1; then
        echo "❌ Unknown family: $FAMILY_FILTER (valid: zai, railway, openai, gh)"
        exit 1
    fi
    FAMILIES="$FAMILY_FILTER"
fi

ROTATED_ANY=false

# ── Rotate each family ──────────────────────────────────────────────
for fam in $FAMILIES; do
    prefix=$(family_prefix "$fam")
    total=$(count_keys "$prefix")

    if [ "$total" -eq 0 ]; then
        continue
    fi
    if [ "$total" -eq 1 ]; then
        # Single key — ensure state is set but nothing to rotate
        jq --arg f "$fam" --arg now "$(now_iso)" \
            '.[$f] = {"current_index": 1, "last_rotated": $now, "total_keys": 1}' \
            "$STATE_FILE" > "${STATE_FILE}.tmp" && mv "${STATE_FILE}.tmp" "$STATE_FILE"
        continue
    fi

    # Current index (default: 1)
    state_json=$(read_state "$fam")
    if [ -n "$state_json" ]; then
        cur_idx=$(echo "$state_json" | jq -r '.current_index')
    else
        cur_idx=1
    fi
    old_idx=$cur_idx

    case "$STRATEGY" in
        round-robin)
            new_idx=$(( (cur_idx % total) + 1 ))
            ;;
        random)
            new_idx=$(( (RANDOM % total) + 1 ))
            # Avoid picking the same key if possible
            if [ "$total" -gt 1 ] && [ "$new_idx" -eq "$cur_idx" ]; then
                new_idx=$(( (new_idx % total) + 1 ))
            fi
            ;;
        health-check)
            new_idx=""
            # Start from the next key after current
            for offset in $(seq 1 "$total"); do
                candidate=$(( ((cur_idx - 1 + offset) % total) + 1 ))
                vn="${prefix}_${candidate}"
                vv=$(printenv "$vn" 2>/dev/null || true)
                if [ -n "$vv" ] && check_key "$fam" "$vv"; then
                    new_idx=$candidate
                    break
                else
                    echo "  ⚠  $fam: key #$candidate failed health check"
                fi
            done
            if [ -z "$new_idx" ]; then
                echo "  ❌ $fam: all $total keys failed health check — keeping index $cur_idx"
                continue
            fi
            ;;
    esac

    # Update state
    jq --arg f "$fam" --argjson idx "$new_idx" --arg now "$(now_iso)" --argjson tot "$total" \
        '.[$f] = {"current_index": $idx, "last_rotated": $now, "total_keys": $tot}' \
        "$STATE_FILE" > "${STATE_FILE}.tmp" && mv "${STATE_FILE}.tmp" "$STATE_FILE"

    echo "🔄 $fam: rotated index $old_idx → $new_idx (of $total keys) [$STRATEGY]"

    # Export the active key so sync can pick it up
    case "$fam" in
        zai)
            # Set ZAI_USE so sync-settings uses the right key
            export ZAI_USE="$new_idx"
            ;;
        railway)
            vn="${prefix}_${new_idx}"
            export RAILWAY_API_TOKEN
            RAILWAY_API_TOKEN=$(printenv "$vn" 2>/dev/null || true)
            ;;
        openai)
            vn="${prefix}_${new_idx}"
            export OPENAI_API_KEY
            OPENAI_API_KEY=$(printenv "$vn" 2>/dev/null || true)
            ;;
        gh)
            vn="${prefix}_${new_idx}"
            export GH_TOKEN
            GH_TOKEN=$(printenv "$vn" 2>/dev/null || true)
            ;;
    esac

    ROTATED_ANY=true
done

# ── Re-sync settings.json ───────────────────────────────────────────
if [ "$ROTATED_ANY" = true ]; then
    echo ""
    echo "Syncing settings.json …"
    bash "${SCRIPT_DIR}/sync-settings-from-env.sh" "$ENV_FILE" "$SETTINGS"
fi

echo ""
echo "Done. Use --status to view current state."
