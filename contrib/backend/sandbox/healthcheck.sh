#!/usr/bin/env bash
# Health check: verify OpenCode web server is responding.
# OpenCode exposes /global/health; we proxy it from the Docker HEALTHCHECK.
# Falls back to a plain TCP check on port 8080 if curl is not available.

set -euo pipefail

HOST="localhost"
PORT="${HEALTH_PORT:-8080}"
ENDPOINT="/global/health"

if command -v curl &>/dev/null; then
  curl --silent --fail --max-time 2 "http://${HOST}:${PORT}${ENDPOINT}" > /dev/null
else
  # Minimal fallback: just check that the port accepts connections
  exec 3<>/dev/tcp/${HOST}/${PORT} 2>/dev/null && exec 3>&-
fi
