# ⚠️  WORKSPACE BOUNDARY - READ FIRST

## CORRECT WORKING DIRECTORY

For ALL Trinity A2A + relay-observer + experience hooks work:

**YOU MUST BE IN**: `~/t27/packages/browseros-agent`

This directory contains:
- Trinity A2A relay observer implementation
- Experience hooks for measurable progress
- A2A type definitions from T27 spec
- Multi-agent test infrastructure

## FORBIDDEN DIRECTORY

**NEVER work in**: `/Users/playra/BrowserOS` (root directory)

This is a DIFFERENT project with:
- Different port configurations (9001, 9100, 3001)
- Different configuration system
- NO Trinity experience hooks
- NOT part of Single-Provider Verified path

## PORT SSOT (Single Source of Truth)

All port configurations are defined in:
`~/t27/packages/browseros-agent/packages/shared/src/constants/ports.ts`

A2A WebSocket port: **9001** (not 3000, not 9100)

## VIOLATION DETECTION

If you find yourself working in `/Users/playra/BrowserOS`:

1. STOP immediately
2. Switch to `~/t27/packages/browseros-agent`
3. Re-read task context

## AGENCY AGENT GUIDELINES

1. **Trinity experience hooks** are ONLY in `~/t27/packages/browseros-agent`
   - Do NOT try to add them to `/Users/playra/BrowserOS`
   - This would duplicate work and break single-source-of-truth

2. **Port confusion**
   - Do NOT change A2A port 9001 to 3000 or 9100
   - Do NOT add new port definitions
   - This breaks verified single-provider setup

3. **Workspace boundary**
   - Respect that Trinity workspace and BrowserOS workspace are SEPARATE
   - Do NOT merge code between them without explicit direction

## VERIFICATION STEPS

Before making ANY changes to A2A code:

1. Check current directory: `pwd`
2. Verify you are in: `~/t27/packages/browseros-agent`
3. Ask user: "Work on Trinity A2A workspace boundary documentation?"

## ACTIONS FOR THIS EPISODE

### FORBIDDEN ACTIONS (Do NOT do):

❌ **NEVER edit files in `/Users/playra/BrowserOS`**:
   - Do NOT add Trinity experience hooks to BrowserOS relay-observer.ts
   - Do NOT modify port constants in BrowserOS
   - Do NOT try to "sync" the two workspaces
   - This violates single-provider verification

❌ **NEVER investigate `/Users/playra/BrowserOS` structure**
   - Do NOT ls, find, grep in root directory
   - Do NOT try to understand BrowserOS architecture
   - This is NOT your workspace for Trinity A2A work

### REQUIRED ACTIONS (Do THIS):

✅ **Create workspace boundary documentation**:
   - File: `~/t27/packages/browseros-agent/CLAUDE.md` (this file)
   - Content: Workspace boundaries, port configurations, project separation
   - Add warnings section with emoji indicators
   - Keep documentation SHORT and UPPERCASE
   - No explanations of why things are wrong

✅ **Verify Trinity A2A setup**:
   - Confirm port 9001 usage
   - Confirm experience hooks are in Trinity workspace
   - Do NOT try to modify anything in BrowserOS

---

**STATUS**: ⚠️  WORKSPACE BOUNDARY - READ FIRST
**CURRENT DIRECTORY**: `~/t27/packages/browseros-agent` ✅
**FORBIDDEN DIRECTORY**: `/Users/playra/BrowserOS` ❌

**NOTE**: Read this boundary document before starting ANY work. All A2A work MUST stay in Trinity workspace.
EOFMARKER
echo "✅ Workspace Boundary Documentation создан"
echo ""
echo "📊 Текущая директория: $(pwd)"
echo ""
echo "=== Инструкция ==="
echo "1. ВСЕ A2A работа должна быть в: ~/t27/packages/browseros-agent"
echo "2. Никаких изменений в /Users/playra/BrowserOS"
echo "3. Использовать существующий порт A2A: 9001"
echo ""
echo "✅ Готово к продолжению работы"
