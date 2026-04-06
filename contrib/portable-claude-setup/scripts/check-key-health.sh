#!/bin/bash
# Test all configured API keys and report their health status
#
# Reads KEY_FAMILY_N variables from ~/.claude/.env and probes each
# provider's API. Never prints actual key values — only indices and
# human-readable status.
#
# Usage:
#   check-key-health.sh [--env FILE]

set -euo pipefail

ENV_FILE="${HOME}/.claude/.env"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --env) ENV_FILE="$2"; shift 2 ;;
        *)     echo "Unknown option: $1"; exit 1 ;;
    esac
done

if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Missing file: $ENV_FILE"
    exit 1
fi

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

PASS=0
FAIL=0
SKIP=0

result() {
    local label="$1" status="$2" detail="${3:-}"
    if [ "$status" = "ok" ]; then
        PASS=$((PASS + 1))
        if [ -n "$detail" ]; then
            echo "  $label: ✓ valid ($detail)"
        else
            echo "  $label: ✓ valid"
        fi
    elif [ "$status" = "skip" ]; then
        SKIP=$((SKIP + 1))
        echo "  $label: ○ skipped (no health-check endpoint)"
    else
        FAIL=$((FAIL + 1))
        if [ -n "$detail" ]; then
            echo "  $label: ✗ $detail"
        else
            echo "  $label: ✗ failed"
        fi
    fi
}

# ── ZAI / Anthropic keys ────────────────────────────────────────────
echo "=== ZAI (Anthropic) ==="
i=1
found=false
while true; do
    vn="ZAI_KEY_${i}"
    vv=$(printenv "$vn" 2>/dev/null || true)
    [ -z "$vv" ] && break
    found=true
    # No reliable free health-check endpoint for Anthropic
    result "$vn" "skip"
    i=$((i + 1))
done
if [ "$found" = false ]; then
    echo "  (none configured)"
fi

# ── Railway tokens ───────────────────────────────────────────────────
echo ""
echo "=== Railway ==="
i=1
found=false
while true; do
    vn="RAILWAY_TOKEN_${i}"
    vv=$(printenv "$vn" 2>/dev/null || true)
    [ -z "$vv" ] && break
    found=true
    resp=$(curl -s --max-time 10 \
        --request POST \
        --url "https://backboard.railway.com/graphql/v2" \
        --header "Authorization: Bearer $vv" \
        --header "Content-Type: application/json" \
        --data '{"query":"{ projects { edges { node { id name } } } }"}' 2>/dev/null || echo "")
    if echo "$resp" | jq -e '.data.projects' >/dev/null 2>&1; then
        # Extract first project name for display
        proj=$(echo "$resp" | jq -r '.data.projects.edges[0].node.name // "unknown"')
        result "$vn" "ok" "project: $proj"
    else
        err=$(echo "$resp" | jq -r '.errors[0].message // "unknown error"' 2>/dev/null || echo "request failed")
        result "$vn" "fail" "$err"
    fi
    i=$((i + 1))
done
if [ "$found" = false ]; then
    echo "  (none configured)"
fi

# ── OpenAI keys ──────────────────────────────────────────────────────
echo ""
echo "=== OpenAI ==="
i=1
found=false
while true; do
    vn="OPENAI_KEY_${i}"
    vv=$(printenv "$vn" 2>/dev/null || true)
    [ -z "$vv" ] && break
    found=true
    resp=$(curl -s --max-time 10 \
        -H "Authorization: Bearer $vv" \
        "https://api.openai.com/v1/models" 2>/dev/null || echo "")
    if echo "$resp" | jq -e '.data' >/dev/null 2>&1; then
        result "$vn" "ok"
    else
        err=$(echo "$resp" | jq -r '.error.message // "unknown error"' 2>/dev/null || echo "request failed")
        result "$vn" "fail" "$err"
    fi
    i=$((i + 1))
done
if [ "$found" = false ]; then
    echo "  (none configured)"
fi

# ── GitHub tokens ────────────────────────────────────────────────────
echo ""
echo "=== GitHub ==="
i=1
found=false
while true; do
    vn="GH_TOKEN_${i}"
    vv=$(printenv "$vn" 2>/dev/null || true)
    [ -z "$vv" ] && break
    found=true
    resp=$(curl -s --max-time 10 \
        -H "Authorization: token $vv" \
        "https://api.github.com/user" 2>/dev/null || echo "")
    if echo "$resp" | jq -e '.login' >/dev/null 2>&1; then
        user=$(echo "$resp" | jq -r '.login')
        result "$vn" "ok" "user: $user"
    else
        err=$(echo "$resp" | jq -r '.message // "unknown error"' 2>/dev/null || echo "request failed")
        result "$vn" "fail" "$err"
    fi
    i=$((i + 1))
done
if [ "$found" = false ]; then
    echo "  (none configured)"
fi

# ── Summary ──────────────────────────────────────────────────────────
echo ""
echo "--- Summary: $PASS valid, $FAIL failed, $SKIP skipped ---"
if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
exit 0
