#!/bin/bash
# Legacy entrypoint — delegates to sync-settings-from-env.sh
set -euo pipefail
exec "$(dirname "$0")/sync-settings-from-env.sh" "$@"
