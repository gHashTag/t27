#!/bin/bash
# Trinity MCP server wrapper - routes to traceability server
cd /Users/playra/t27
exec node scripts/mcp-traceability-server.js "$@"
