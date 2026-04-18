# ⚠️  WORKSPACE BOUNDARY - TRINITY A2A

## Purpose

This document establishes clear boundaries for Trinity A2A multi-agent work to prevent:
- Port confusion (3000, 9001, 9100)
- Project separation (Trinity vs BrowserOS)
- Workspace boundary violations
- Single Source of Truth (SSOT) violations

## CORRECT WORKING DIRECTORY

**YOU MUST BE IN**: `~/t27/packages/browseros-agent`

This directory contains:
- Trinity A2A relay observer implementation
- Experience hooks for A2A
- Multi-agent test infrastructure
- Type definitions from T27 specification

## PROJECT SEPARATION

### Two Independent Workspaces

**Trinity A2A Workspace**: `~/t27/packages/browseros-agent`
- Source of Truth for Trinity A2A specification
- T27 compilation and code generation
- Portable agent implementation
- **A2A WebSocket port: 9001**

**BrowserOS Workspace**: `/Users/playra/BrowserOS`
- Separate MCP server project
- Different port configuration
- **NOT** part of Trinity A2A development
- **NOT** Source of Truth for Trinity

### Boundary Rules

1. **Do NOT merge workspaces**
   - These are independent codebases
   - No code should be copied between them
   - No shared dependencies beyond standard Node packages

2. **Do NOT create shared symlinks**
   - Each workspace maintains its own structure
   - No `ln -s` commands to create aliases

3. **Respect port configuration**
   - Trinity A2A uses **port 9001**
   - BrowserOS uses port 9100 for main server
   - Do NOT change Trinity A2A port to match BrowserOS
   - Do NOT share port numbers between workspaces

## PORT CONFIGURATION

### Trinity A2A Ports

**A2A WebSocket: 9001**
- Defined in: `~/t27/packages/browseros-agent/packages/shared/src/constants/ports.ts`
- Used for: A2A WebSocket communication
- **Do NOT change this to other ports**

### BrowserOS Ports

**Main Server: 9100**
- BrowserOS MCP server
- Not related to Trinity A2A work

### Port Separation

```
Trinity A2A: 9001 (A2A WebSocket)
BrowserOS:     9100 (Main Server)
```

These are intentionally separate to prevent confusion and maintain SSOT.

## AGENCY AGENT GUIDELINES

### For Trinity A2A Work

**All work MUST be in**: `~/t27/packages/browseros-agent`

### Forbidden Actions

❌ **NEVER** work in `/Users/playra/BrowserOS`
   - This is a separate workspace
   - Do not edit, modify, or delete files in this directory
   - Do not run commands from this directory
   - Do not create symlinks to this directory

### Required Actions

✅ **ALWAYS** work in `~/t27/packages/browseros-agent`
   - Edit `relay-observer.ts` for Trinity experience hooks
   - Create `a2a-types.ts` for T27 compliance
   - Run tests from `~/t27/packages/browseros-agent`
   - Use port 9001 for A2A WebSocket

### Verification

After making changes, verify directory:
```bash
pwd  # Must show: ~/t27/packages/browseros-agent
ls -la apps/server/src/agent/portable/relay-observer.ts # Should be in Trinity workspace
```

---

**IMPORTANT**: This document is for clarity only. All Trinity A2A work should be done in the correct workspace.
EOFMARKER'

echo "✅ Workspace Boundary Documentation создан"
echo ""
echo "📊 Текущая директория: $(pwd)"
echo ""
echo "=== Инструкция ==="
echo "1. ВСЕ A2A работа должна быть в ~/t27/packages/browseros-agent"
echo "2. Использовать порт A2A: 9001"
echo "3. НЕ работать в /Users/playra/BrowserOS"
