#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_DIR="/home/sandbox/workspace"
OPENCODE_CONFIG_DIR="${WORKSPACE_DIR}/.opencode"

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

  # Derive a clean directory name from the repo URL
  REPO_NAME="$(basename "${SANDBOX_REPO_URL}" .git)"
  TARGET_DIR="${WORKSPACE_DIR}/${REPO_NAME}"

  if [[ -d "${TARGET_DIR}/.git" ]]; then
    log "Repository already cloned at ${TARGET_DIR}, pulling latest..."
    git -C "${TARGET_DIR}" pull --ff-only || log "WARNING: git pull failed, continuing with existing state."
  else
    git clone "${SANDBOX_REPO_URL}" "${TARGET_DIR}"
    log "Repository cloned to ${TARGET_DIR}."
  fi

  # Switch to the cloned repo as working directory for opencode
  cd "${TARGET_DIR}"
else
  log "SANDBOX_REPO_URL or GH_TOKEN not set — starting opencode in empty workspace."
  cd "${WORKSPACE_DIR}"
fi

# ──────────────────────────────────────────────
# 3. Write opencode.json with provider settings
# ──────────────────────────────────────────────
log "Writing opencode configuration..."
mkdir -p "${OPENCODE_CONFIG_DIR}"

# Build the providers block dynamically based on available API keys
PROVIDERS_JSON=""

if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
  PROVIDERS_JSON="${PROVIDERS_JSON}
    \"anthropic\": {
      \"apiKey\": \"${ANTHROPIC_API_KEY}\"
    },"
fi

if [[ -n "${OPENAI_API_KEY:-}" ]]; then
  PROVIDERS_JSON="${PROVIDERS_JSON}
    \"openai\": {
      \"apiKey\": \"${OPENAI_API_KEY}\"
    },"
fi

if [[ -n "${GEMINI_API_KEY:-}" ]]; then
  PROVIDERS_JSON="${PROVIDERS_JSON}
    \"google\": {
      \"apiKey\": \"${GEMINI_API_KEY}\"
    },"
fi

if [[ -n "${GROQ_API_KEY:-}" ]]; then
  PROVIDERS_JSON="${PROVIDERS_JSON}
    \"groq\": {
      \"apiKey\": \"${GROQ_API_KEY}\"
    },"
fi

# Trim trailing comma from last provider entry
PROVIDERS_JSON="${PROVIDERS_JSON%,}"

# Determine default model based on available keys (prefer Anthropic → OpenAI → Google)
DEFAULT_MODEL="anthropic/claude-sonnet-4-5"
if [[ -z "${ANTHROPIC_API_KEY:-}" ]] && [[ -n "${OPENAI_API_KEY:-}" ]]; then
  DEFAULT_MODEL="openai/gpt-4o"
elif [[ -z "${ANTHROPIC_API_KEY:-}" ]] && [[ -z "${OPENAI_API_KEY:-}" ]] && [[ -n "${GEMINI_API_KEY:-}" ]]; then
  DEFAULT_MODEL="google/gemini-2.0-flash"
fi

# Write the config file. The copy in the repo root takes precedence at runtime;
# we also write to the default XDG config location as a fallback.
cat > opencode.json <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "model": "${DEFAULT_MODEL}",
  "providers": {${PROVIDERS_JSON}
  },
  "keybindings": {
    "leader": "ctrl+k"
  },
  "mcp": {
    "railway": {
      "type": "local",
      "command": "npx",
      "args": ["-y", "@railway/mcp-server"],
      "env": {
        "RAILWAY_API_TOKEN": "${RAILWAY_API_TOKEN:-}"
      }
    }
  },
  "agents": {
    "t27-swe": {
      "name": "T27 SWE Agent",
      "description": "Software engineering agent for T27 Railway-based sandbox.",
      "model": "${DEFAULT_MODEL}",
      "tools": [
        "bash",
        "read",
        "write",
        "edit",
        "glob",
        "grep",
        "fetch",
        "mcp:railway"
      ],
      "system": "You are an expert software engineer operating inside a Railway sandbox. You have full access to the cloned repository. You can run arbitrary shell commands, read and write files, and interact with the Railway platform via MCP. Be concise and precise. Prefer small, targeted edits over large rewrites. Always verify your changes compile or pass lint before finishing."
    }
  },
  "tools": {
    "bash": { "enabled": true },
    "read": { "enabled": true },
    "write": { "enabled": true },
    "edit": { "enabled": true },
    "glob": { "enabled": true },
    "grep": { "enabled": true },
    "fetch": { "enabled": true }
  }
}
EOF

# Also copy to a global config location so opencode can find it regardless of CWD
mkdir -p "${HOME}/.config/opencode"
cp opencode.json "${HOME}/.config/opencode/opencode.json"

log "opencode.json written."

# ──────────────────────────────────────────────
# 4. Start OpenCode web server
# ──────────────────────────────────────────────
log "Starting OpenCode web server on 0.0.0.0:8080..."

# Give opencode a chance to start; if it exits immediately fall back to code-server
if command -v opencode &>/dev/null; then
  exec opencode web --hostname 0.0.0.0 --port 8080
else
  log "ERROR: opencode binary not found on PATH (${PATH}). Falling back to code-server..."
  exec code-server \
    --bind-addr 0.0.0.0:8080 \
    --auth none \
    --disable-telemetry \
    .
fi
