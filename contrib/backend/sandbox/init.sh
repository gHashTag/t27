#!/usr/bin/env bash
set -euo pipefail
log() { echo "[sandbox-init] $*"; }

WORKSPACE_DIR="/home/sandbox/workspace"

# ── 1. GitHub auth ──
if [[ -n "${GH_TOKEN:-}" ]]; then
  echo "${GH_TOKEN}" | gh auth login --with-token 2>/dev/null
  git config --global url."https://x-access-token:${GH_TOKEN}@github.com/".insteadOf "https://github.com/"
  log "GitHub authenticated."
fi

# ── 2. Clone repo ──
WORK_DIR="${WORKSPACE_DIR}"
if [[ -n "${SANDBOX_REPO_URL:-}" ]]; then
  REPO_NAME="$(basename "${SANDBOX_REPO_URL}" .git)"
  TARGET_DIR="${WORKSPACE_DIR}/${REPO_NAME}"
  if [[ -d "${TARGET_DIR}/.git" ]]; then
    git -C "${TARGET_DIR}" pull --ff-only 2>/dev/null || true
  else
    git clone "${SANDBOX_REPO_URL}" "${TARGET_DIR}" 2>/dev/null || log "Clone failed"
  fi
  [[ -d "${TARGET_DIR}" ]] && WORK_DIR="${TARGET_DIR}"
fi
cd "${WORK_DIR}"
log "CWD: ${WORK_DIR}"

# ── 3. Write opencode config ──
DEFAULT_MODEL="anthropic/claude-sonnet-4-5"
SMALL_MODEL="anthropic/claude-haiku-3-5"
[[ -z "${ANTHROPIC_API_KEY:-}" ]] && [[ -n "${OPENAI_API_KEY:-}" ]] && DEFAULT_MODEL="openai/gpt-4o" && SMALL_MODEL="openai/gpt-4o-mini"
MCP_BLOCK=""
[[ -n "${RAILWAY_API_TOKEN:-}" ]] && MCP_BLOCK='"mcp":{"railway":{"type":"local","command":["npx","-y","@railway/mcp-server"],"environment":{"RAILWAY_API_TOKEN":"'"${RAILWAY_API_TOKEN}"'"}}},'
cat > opencode.json <<ENDJSON
{"\$schema":"https://opencode.ai/config.json","model":"${DEFAULT_MODEL}","small_model":"${SMALL_MODEL}",${MCP_BLOCK}"server":{"port":${PORT:-8080},"hostname":"0.0.0.0"}}
ENDJSON
mkdir -p "${HOME}/.config/opencode"
cp opencode.json "${HOME}/.config/opencode/opencode.json"

# ── 4. Start OpenCode web (background, for browser observation) ──
if command -v opencode &>/dev/null; then
  log "Starting OpenCode web UI on :${PORT:-8080}..."
  opencode web --hostname 0.0.0.0 --port "${PORT:-8080}" &
  WEB_PID=$!
  for i in $(seq 1 30); do
    curl -sf "http://localhost:${PORT:-8080}/global/health" >/dev/null 2>&1 && break
    sleep 2
  done
  log "Web UI ready (PID ${WEB_PID})"
else
  log "OpenCode not found, starting code-server..."
  code-server --bind-addr "0.0.0.0:${PORT:-8080}" --auth none . &
  WEB_PID=$!
fi

# ── 5. Run agent ──
if [[ -n "${TASK_PROMPT:-}" ]]; then
  log "═══════════════════════════════════════"
  log "  AUTONOMOUS AGENT MODE"
  log "═══════════════════════════════════════"

  if command -v t27-agent-runner &>/dev/null; then
    # Rust agent runner (preferred)
    log "Using Rust agent runner..."
    t27-agent-runner &
    AGENT_PID=$!
    log "Agent PID: ${AGENT_PID}"
  else
    # Python fallback
    log "Rust runner not found, using Python fallback..."
    python3 /usr/local/bin/agent-runner.py &
    AGENT_PID=$!
    log "Python agent PID: ${AGENT_PID}"
  fi
fi

# ── 6. Keep alive ──
wait "${WEB_PID}"
