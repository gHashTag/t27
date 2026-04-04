#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_DIR="/home/sandbox/workspace"
log() { echo "[sandbox-init] $*"; }

# ──────────────────────────────────────────────
# 1. GitHub auth
# ──────────────────────────────────────────────
if [[ -n "${GH_TOKEN:-}" ]]; then
  log "GitHub CLI auth..."
  echo "${GH_TOKEN}" | gh auth login --with-token 2>/dev/null
  git config --global url."https://x-access-token:${GH_TOKEN}@github.com/".insteadOf "https://github.com/"
fi

# ──────────────────────────────────────────────
# 2. Clone repo
# ──────────────────────────────────────────────
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
log "Working directory: ${WORK_DIR}"

# ──────────────────────────────────────────────
# 3. Minimal opencode.json
# ──────────────────────────────────────────────
DEFAULT_MODEL="anthropic/claude-sonnet-4-5"
SMALL_MODEL="anthropic/claude-haiku-3-5"
[[ -z "${ANTHROPIC_API_KEY:-}" ]] && [[ -n "${OPENAI_API_KEY:-}" ]] && DEFAULT_MODEL="openai/gpt-4o" && SMALL_MODEL="openai/gpt-4o-mini"

MCP_BLOCK=""
[[ -n "${RAILWAY_API_TOKEN:-}" ]] && MCP_BLOCK='"mcp":{"railway":{"type":"local","command":["npx","-y","@railway/mcp-server"],"environment":{"RAILWAY_API_TOKEN":"'"${RAILWAY_API_TOKEN}"'"}}},'

cat > opencode.json <<ENDJSON
{
  "\$schema": "https://opencode.ai/config.json",
  "model": "${DEFAULT_MODEL}",
  "small_model": "${SMALL_MODEL}",
  ${MCP_BLOCK}
  "server": {"port": ${PORT:-8080}, "hostname": "0.0.0.0"}
}
ENDJSON
mkdir -p "${HOME}/.config/opencode"
cp opencode.json "${HOME}/.config/opencode/opencode.json"

# ──────────────────────────────────────────────
# 4. Start OpenCode web (background for UI)
# ──────────────────────────────────────────────
log "Starting OpenCode web UI..."
if command -v opencode &>/dev/null; then
  opencode web --hostname 0.0.0.0 --port "${PORT:-8080}" &
  WEB_PID=$!
else
  code-server --bind-addr "0.0.0.0:${PORT:-8080}" --auth none . &
  WEB_PID=$!
fi

# Wait for server ready
for i in $(seq 1 30); do
  curl -sf "http://localhost:${PORT:-8080}/global/health" >/dev/null 2>&1 && break
  sleep 2
done
log "Web UI ready (PID ${WEB_PID})"

# ──────────────────────────────────────────────
# 5. Autonomous task via CLI (if TASK_PROMPT set)
# ──────────────────────────────────────────────
if [[ -n "${TASK_PROMPT:-}" ]] && command -v opencode &>/dev/null; then
  log "═══════════════════════════════════════"
  log "  AUTONOMOUS TASK MODE"
  log "═══════════════════════════════════════"
  log "Task: ${TASK_PROMPT:0:200}..."

  # Build full prompt with project context
  FULL_PROMPT="${TASK_PROMPT}"
  [[ -f "SOUL.md" ]] && FULL_PROMPT="You are working in the T27 project. Read SOUL.md first — it is the constitutional law. Then: ${TASK_PROMPT}"
  [[ -f "CLAUDE.md" ]] && FULL_PROMPT="${FULL_PROMPT}. Also follow CLAUDE.md conventions."
  [[ -f "docs/AGENTS.md" ]] && FULL_PROMPT="${FULL_PROMPT}. Agent specs are in docs/AGENTS.md."

  # Run autonomous task via opencode CLI (uses same server)
  # --attach connects to the running web server
  # --format json gives structured output
  log "Launching task via CLI (attached to web server)..."
  opencode run \
    --attach "http://localhost:${PORT:-8080}" \
    --model "${DEFAULT_MODEL}" \
    --title "${TASK_TITLE:-autonomous-phi-loop}" \
    --format json \
    "${FULL_PROMPT}" \
    > /tmp/task-output.json 2>&1 &
  TASK_PID=$!
  log "Task PID: ${TASK_PID}"
  log "Watch progress: https://${RAILWAY_PUBLIC_DOMAIN:-localhost:${PORT:-8080}}"

  # Log task output in background
  (
    wait "${TASK_PID}" 2>/dev/null
    EXIT_CODE=$?
    log "═══════════════════════════════════════"
    log "  TASK COMPLETED (exit: ${EXIT_CODE})"
    log "═══════════════════════════════════════"
    if [[ -f /tmp/task-output.json ]]; then
      log "Output saved to /tmp/task-output.json"
      python3 -c "
import json
try:
    with open('/tmp/task-output.json') as f:
        for line in f:
            line = line.strip()
            if not line: continue
            try:
                d = json.loads(line)
                if d.get('type') == 'text':
                    print(f'[RESULT] {d.get(\"text\",\"\")[:500]}')
            except: pass
except: pass
" 2>/dev/null
    fi
  ) &
fi

# ──────────────────────────────────────────────
# 6. Keep alive
# ──────────────────────────────────────────────
wait "${WEB_PID}"
