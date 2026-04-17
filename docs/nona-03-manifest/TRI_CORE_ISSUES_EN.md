# /tri Core Issues and Improvement Plan

## Problem Statement

The `/tri` CLI currently operates in "detection mode" instead of following constitutional protocol. It detects file changes and spec modifications but skips critical coordination layer, creating a disconnect between agent actions and swarm state.

## Constitutional Violation

**SOUL.md Law #6 (Akashic Coordination First):**
> "Before any task, every agent must read `.trinity` Akashic Chronicle, inspect active claims, queue, and swarm state, then acquire an exclusive claim on its target spec_path or graph_node. No mutation without prior read + claim."

**What `/tri` currently does:**
```
1. Detects SOUL.md changes ✅
2. Detects gf4.t27 changes ✅
3. Says "No active claims" ❌ (doesn't read .trinity/claims!)
4. Asks what to do instead of following protocol
```

**Missing steps:**
1. ❌ Read `.trinity/events/akashic-log.jsonl`
2. ❌ Read `.trinity/claims/active/`
3. ❌ Check `.trinity/state/queen-health.json`
4. ❌ Check if target resource is claimed
5. ❌ Acquire claim (if available)
6. ❌ Record `task.intent` event

## Current Architecture Issues

### Single Responsibility Problem

`/tri` mixes detection logic with action prompts. This violates:
- **Single Responsibility Principle**: One component should do one thing well
- **Separation of Concerns**: Coordination (read .trinity, check claims) vs Action (run tri gen, commit)
- **Unclear Control Flow**: User can't see what happened and what's happening

### Coordination Disconnect

The coordination layer (`.trinity/`) is created but not integrated:
- Doctor agent exists and runs independently
- Event schemas are defined
- Scripts (`swarm-health.sh`, `replay-step.sh`) are ready
- BUT `/tri` doesn't use them before showing options

## Improvement Plan: Native Coordination for /tri

### Phase 1: Coordination Foundation

**Goal:** Embed `.trinity` read directly into `/tri` before any action.

### Phase 2: Coordination Foundation

**Goal:** Embed `.trinity` read directly into `/tri` before any action.
