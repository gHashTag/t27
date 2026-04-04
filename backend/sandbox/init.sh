#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_DIR="/home/sandbox/workspace"

log() {
  echo "[sandbox-init] $*"
}

# ──────────────────────────────────────────────
# 1. Authenticate gh CLI with GH_TOKEN
# ──────────────────────────────────────────────
if [[ -n "${GH_TOKEN:-}" ]]; then
  log "Configuring GitHub CLI authentication..."
  echo "${GH_TOKEN}" | gh auth login --with-token
  git config --global url."https://x-access-token:${GH_TOKEN}@github.com/".insteadOf "https://github.com/"
  log "GitHub CLI authenticated."
else
  log "WARNING: GH_TOKEN not set — skipping GitHub authentication."
fi

# ──────────────────────────────────────────────
# 2. Clone target repository
# ──────────────────────────────────────────────
if [[ -n "${SANDBOX_REPO_URL:-}" ]] && [[ -n "${GH_TOKEN:-}" ]]; then
  log "Cloning repository: ${SANDBOX_REPO_URL}"

  REPO_NAME="$(basename "${SANDBOX_REPO_URL}" .git)"
  TARGET_DIR="${WORKSPACE_DIR}/${REPO_NAME}"

  if [[ -d "${TARGET_DIR}/.git" ]]; then
    log "Repository already cloned at ${TARGET_DIR}, pulling latest..."
    git -C "${TARGET_DIR}" pull --ff-only || log "WARNING: git pull failed, continuing with existing state."
  else
    git clone "${SANDBOX_REPO_URL}" "${TARGET_DIR}"
    log "Repository cloned to ${TARGET_DIR}."
  fi

  cd "${TARGET_DIR}"
else
  log "SANDBOX_REPO_URL or GH_TOKEN not set — starting opencode in empty workspace."
  cd "${WORKSPACE_DIR}"
fi

# ──────────────────────────────────────────────
# 3. Write opencode.json (correct schema)
# ──────────────────────────────────────────────
log "Writing opencode configuration..."

# Determine default model based on available keys
DEFAULT_MODEL="anthropic/claude-sonnet-4-5"
SMALL_MODEL="anthropic/claude-haiku-3-5"
if [[ -z "${ANTHROPIC_API_KEY:-}" ]] && [[ -n "${OPENAI_API_KEY:-}" ]]; then
  DEFAULT_MODEL="openai/gpt-4o"
  SMALL_MODEL="openai/gpt-4o-mini"
elif [[ -z "${ANTHROPIC_API_KEY:-}" ]] && [[ -z "${OPENAI_API_KEY:-}" ]] && [[ -n "${GEMINI_API_KEY:-}" ]]; then
  DEFAULT_MODEL="google/gemini-2.5-flash"
  SMALL_MODEL="google/gemini-2.5-flash"
fi

# Build providers JSON
PROVIDERS_JSON=""
if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
  PROVIDERS_JSON="${PROVIDERS_JSON}\"anthropic\":{\"apiKey\":\"${ANTHROPIC_API_KEY}\"},"
fi
if [[ -n "${OPENAI_API_KEY:-}" ]]; then
  PROVIDERS_JSON="${PROVIDERS_JSON}\"openai\":{\"apiKey\":\"${OPENAI_API_KEY}\"},"
fi
if [[ -n "${GEMINI_API_KEY:-}" ]]; then
  PROVIDERS_JSON="${PROVIDERS_JSON}\"gemini\":{\"apiKey\":\"${GEMINI_API_KEY}\"},"
fi
if [[ -n "${GROQ_API_KEY:-}" ]]; then
  PROVIDERS_JSON="${PROVIDERS_JSON}\"groq\":{\"apiKey\":\"${GROQ_API_KEY}\"},"
fi
PROVIDERS_JSON="${PROVIDERS_JSON%,}"

# Build mcpServers JSON (only if RAILWAY_API_TOKEN is set)
MCP_JSON=""
if [[ -n "${RAILWAY_API_TOKEN:-}" ]]; then
  MCP_JSON="\"mcpServers\":{\"railway\":{\"type\":\"local\",\"command\":[\"npx\",\"-y\",\"@railway/mcp-server\"],\"environment\":{\"RAILWAY_API_TOKEN\":\"${RAILWAY_API_TOKEN}\"}}},"
fi

# Write config following the official schema:
# https://opencode.ai/docs/config/
# https://mintlify.com/opencode-ai/opencode/reference/config-schema
cat > opencode.json <<ENDJSON
{
  "\$schema": "https://opencode.ai/config.json",
  "model": "${DEFAULT_MODEL}",
  "small_model": "${SMALL_MODEL}",
  "providers": {${PROVIDERS_JSON}},
  ${MCP_JSON}
  "agents": {
    "coder": {
      "model": "${DEFAULT_MODEL}",
      "maxTokens": 8000
    },
    "task": {
      "model": "${SMALL_MODEL}",
      "maxTokens": 4000
    },
    "title": {
      "model": "${SMALL_MODEL}"
    }
  }
}
ENDJSON

# Copy to global config location
mkdir -p "${HOME}/.config/opencode"
cp opencode.json "${HOME}/.config/opencode/opencode.json"

log "opencode.json written."

# ──────────────────────────────────────────────
# 4. Start OpenCode web server
# ──────────────────────────────────────────────
log "Starting OpenCode web server on 0.0.0.0:${PORT:-8080}..."

if command -v opencode &>/dev/null; then
  exec opencode web --hostname 0.0.0.0 --port "${PORT:-8080}"
else
  log "ERROR: opencode binary not found. Falling back to code-server..."
  exec code-server \
    --bind-addr "0.0.0.0:${PORT:-8080}" \
    --auth none \
    --disable-telemetry \
    .
fi
