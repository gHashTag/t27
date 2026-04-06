#!/bin/bash
# Merge secrets and optional env vars from ~/.claude/.env into ~/.claude/settings.json
#
# Anthropic token resolution (same as legacy apply-anthropic):
#   1) ANTHROPIC_AUTH_TOKEN
#   2) ZAI_KEY_${ZAI_USE} when ZAI_USE is set
#   3) first non-empty ZAI_KEY_<N> in numeric order
#   4) migrate from settings.json → .env if only settings had a token
#
# Usage:
#   sync-settings-from-env.sh [ENV_FILE] [SETTINGS_JSON]
#
# Optional .env variables (applied only when non-empty after source):
#   GH_TOKEN, KAGGLE_API_TOKEN, TRINITY_PROJECT_ROOT, ZIG_VERSION, TRINITY_MCP_PORT,
#   ANTHROPIC_BASE_URL, API_TIMEOUT_MS, CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC,
#   RAILWAY_API_TOKEN (or active RAILWAY_TOKEN_N from rotation state)
#   VIBEE_MCP_RUN_SCRIPT → mcpServers.vibee.command
#   TRINITY_MCP_BIN      → mcpServers.trinity.command

set -euo pipefail

ENV_FILE="${1:-$HOME/.claude/.env}"
SETTINGS="${2:-$HOME/.claude/settings.json}"
STATE_FILE="${HOME}/.claude/.rotation-state.json"

if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Missing file: $ENV_FILE"
    echo "   cp env.example ~/.claude/.env && chmod 600 ~/.claude/.env  (see env.example in this repo)"
    exit 1
fi

if [ ! -f "$SETTINGS" ]; then
    echo "❌ Missing: $SETTINGS"
    exit 1
fi

list_sorted_zai_key_names() {
    local f="$1"
    [ -f "$f" ] || return 0
    grep -E '^ZAI_KEY_[1-9][0-9]*=' "$f" 2>/dev/null | cut -d= -f1 | awk -F_ '{printf "%012d %s\n", $3, $0}' | sort -n | awk '{print $2}'
}

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

# ── Apply rotation state (if present) ───────────────────────────────
# When rotate-keys.sh has run, it writes .rotation-state.json with the
# active index for each key family.  Honour those indices unless the
# caller already exported an override (e.g. ZAI_USE set explicitly).
if [ -f "$STATE_FILE" ]; then
    _rot_idx() { jq -r ".[\"$1\"].current_index // empty" "$STATE_FILE" 2>/dev/null; }

    # ZAI — only override if ZAI_USE was not set by the user
    _zai_idx=$(_rot_idx zai)
    if [ -n "$_zai_idx" ] && [ -z "${ZAI_USE:-}" ]; then
        export ZAI_USE="$_zai_idx"
    fi

    # Railway — resolve RAILWAY_TOKEN_N → RAILWAY_API_TOKEN
    _rail_idx=$(_rot_idx railway)
    if [ -n "$_rail_idx" ] && [ -z "${RAILWAY_API_TOKEN:-}" ]; then
        _rail_vn="RAILWAY_TOKEN_${_rail_idx}"
        _rail_vv=$(printenv "$_rail_vn" 2>/dev/null || true)
        if [ -n "$_rail_vv" ]; then
            export RAILWAY_API_TOKEN="$_rail_vv"
        fi
    fi

    # OpenAI — resolve OPENAI_KEY_N → OPENAI_API_KEY
    _oai_idx=$(_rot_idx openai)
    if [ -n "$_oai_idx" ] && [ -z "${OPENAI_API_KEY:-}" ]; then
        _oai_vn="OPENAI_KEY_${_oai_idx}"
        _oai_vv=$(printenv "$_oai_vn" 2>/dev/null || true)
        if [ -n "$_oai_vv" ]; then
            export OPENAI_API_KEY="$_oai_vv"
        fi
    fi

    # GitHub — resolve GH_TOKEN_N → GH_TOKEN
    _gh_idx=$(_rot_idx gh)
    if [ -n "$_gh_idx" ] && [ -z "${GH_TOKEN:-}" ]; then
        _gh_vn="GH_TOKEN_${_gh_idx}"
        _gh_vv=$(printenv "$_gh_vn" 2>/dev/null || true)
        if [ -n "$_gh_vv" ]; then
            export GH_TOKEN="$_gh_vv"
        fi
    fi
fi

SOURCE_LABEL=""
ANTHROPIC_RESOLVED=""

if [ -n "${ANTHROPIC_AUTH_TOKEN:-}" ]; then
    ANTHROPIC_RESOLVED="$ANTHROPIC_AUTH_TOKEN"
    SOURCE_LABEL="ANTHROPIC_AUTH_TOKEN"
elif [ -n "${ZAI_USE:-}" ]; then
    idx="$ZAI_USE"
    if ! [[ "$idx" =~ ^[1-9][0-9]*$ ]]; then
        echo "❌ ZAI_USE must be a positive integer. Got: $ZAI_USE"
        exit 1
    fi
    vn="ZAI_KEY_${idx}"
    vv=$(printenv "$vn" 2>/dev/null || true)
    if [ -n "$vv" ]; then
        ANTHROPIC_RESOLVED="$vv"
        SOURCE_LABEL="ZAI_KEY_${idx} (ZAI_USE=${idx})"
    else
        echo "❌ ZAI_USE=$ZAI_USE but $vn is missing or empty in $ENV_FILE"
        exit 1
    fi
else
    while IFS= read -r vn; do
        [ -z "$vn" ] && continue
        vv=$(printenv "$vn" 2>/dev/null || true)
        if [ -n "$vv" ] && [[ "$vv" != *"@"* ]]; then
            ANTHROPIC_RESOLVED="$vv"
            SOURCE_LABEL="$vn (first non-empty)"
            break
        fi
    done < <(list_sorted_zai_key_names "$ENV_FILE")
fi

if [ -z "${ANTHROPIC_RESOLVED:-}" ]; then
    FROM_SETTINGS=$(jq -r '.env.ANTHROPIC_AUTH_TOKEN // empty' "$SETTINGS" 2>/dev/null || true)
    if [ -n "$FROM_SETTINGS" ]; then
        echo "⚠️  No token in $ENV_FILE — migrating ANTHROPIC_AUTH_TOKEN from $SETTINGS → $ENV_FILE"
        umask 077
        printf 'ANTHROPIC_AUTH_TOKEN=%s\n' "$FROM_SETTINGS" >>"$ENV_FILE"
        chmod 600 "$ENV_FILE" 2>/dev/null || true
        ANTHROPIC_RESOLVED="$FROM_SETTINGS"
        SOURCE_LABEL="settings.json (appended to .env)"
    fi
fi

if [ -z "${ANTHROPIC_RESOLVED:-}" ]; then
    echo "❌ Nothing to inject: set ANTHROPIC_AUTH_TOKEN= and/or ZAI_KEY_1= in $ENV_FILE"
    exit 1
fi

TMP="${SETTINGS}.tmp.$$"
jq \
    --arg anth "$ANTHROPIC_RESOLVED" \
    --arg gh "${GH_TOKEN:-}" \
    --arg kag "${KAGGLE_API_TOKEN:-}" \
    --arg triroot "${TRINITY_PROJECT_ROOT:-}" \
    --arg zig "${ZIG_VERSION:-}" \
    --arg triport "${TRINITY_MCP_PORT:-}" \
    --arg abase "${ANTHROPIC_BASE_URL:-}" \
    --arg apito "${API_TIMEOUT_MS:-}" \
    --arg cdn "${CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC:-}" \
    --arg rail "${RAILWAY_API_TOKEN:-}" \
    --arg vibee "${VIBEE_MCP_RUN_SCRIPT:-}" \
    --arg tribin "${TRINITY_MCP_BIN:-}" \
    '
    .env.ANTHROPIC_AUTH_TOKEN = $anth
    | if $gh != "" then .env.GH_TOKEN = $gh else . end
    | if $kag != "" then .env.KAGGLE_API_TOKEN = $kag else . end
    | if $rail != "" then .env.RAILWAY_API_TOKEN = $rail else . end
    | if $triroot != "" then .env.TRINITY_PROJECT_ROOT = $triroot else . end
    | if $zig != "" then .env.ZIG_VERSION = $zig else . end
    | if $triport != "" then .env.TRINITY_MCP_PORT = $triport else . end
    | if $abase != "" then .env.ANTHROPIC_BASE_URL = $abase else . end
    | if $apito != "" then .env.API_TIMEOUT_MS = $apito else . end
    | if $cdn != "" then .env.CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = $cdn else . end
    | if $vibee != "" then .mcpServers.vibee.command = $vibee else . end
    | if $tribin != "" then .mcpServers.trinity.command = $tribin else . end
    ' \
    "$SETTINGS" >"$TMP"
mv "$TMP" "$SETTINGS"

echo "✅ settings.json updated from $ENV_FILE"
echo "   Anthropic: $SOURCE_LABEL"
[ -n "${GH_TOKEN:-}" ] && echo "   GH_TOKEN: set"
[ -n "${RAILWAY_API_TOKEN:-}" ] && echo "   RAILWAY_API_TOKEN: set"
[ -n "${KAGGLE_API_TOKEN:-}" ] && echo "   KAGGLE_API_TOKEN: set"
[ -n "${VIBEE_MCP_RUN_SCRIPT:-}" ] && echo "   mcp vibee command: overridden"
[ -n "${TRINITY_MCP_BIN:-}" ] && echo "   mcp trinity command: overridden"
exit 0
