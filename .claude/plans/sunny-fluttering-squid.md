# Plan: NotebookLM Continuous Sync via Claude Code Hooks
# phi^2 + 1/phi^2 = 3 | TRINITY

## Context
User wants to implement a comprehensive NotebookLM Gate system using Claude Code Hooks that:
1. Blocks Claude Code until a notebook is available (SessionStart hook)
2. Injects notebook context into each user prompt (inject hook)
3. Logs all actions to activity.md (log-activity hook)
4. Blocks git push without a notebook (pre-push-gate hook)
5. Registers all hooks in settings.json

## Files to Create/Modify

### New Files (4 shell hooks)
- `.claude/hooks/session-gate.sh` - SessionStart hook, checks for notebook, creates fallback
- `.claude/hooks/inject-notebook-context.sh` - UserPromptSubmit hook, injects context from notebook
- `.claude/hooks/log-activity.sh` - PostToolUse hook, logs to activity.md
- `.claude/hooks/pre-push-gate.sh` - PreToolUse hook, blocks git push

### Files to Modify (1)
- `.claude/settings.json` - Register all hooks

## Implementation Plan

### Task 1: Create SessionStart Hook (`session-gate.sh`)
**File:** `.claude/hooks/session-gate.sh`

**Purpose:**
- Runs as the FIRST line of user prompts
- Blocks Claude until an active notebook is available
- If no notebook exists, blocks with clear error message

**Key Logic:**
1. Check if gating is enabled via `.trinity/enable_notebook_gate`
2. Read notebook ID from `.trinity/current_task/.notebook_id`
3. If ID exists → allow execution
4. If ID doesn't exist → create fallback notebook with `t27c task start --title "Task Name"`
5. Write notebook ID to file for persistence

**Exit Codes:**
- `0` - Success (notebook available)
- `2` - No notebook (BLOCKED) with stderr: "BLOCKED: No NotebookLM notebook. Run: t27c task start --title 'Create Notebook' --sources relevant.md"

### Task 2: Create Inject Hook (`inject-notebook-context.sh`)
**File:** `.claude/hooks/inject-notebook-context.sh`

**Purpose:**
- Runs on every UserPromptSubmit event
- Reads notebook info (ID, task title, activity)
- Reads recent activity (last 10 lines from activity.md)
- Injects JSON context into prompt

**Key Logic:**
1. Load notebook ID from `.trinity/current_task/.notebook_id`
2. Load notebook metadata from `.trinity/current_task/notebook_meta.json`
3. Read activity snapshot from `.trinity/current_task/activity.md`
4. Generate JSON context with: notebook_id, task_title, activity_snapshot, timestamp, sources_count, last_sync
5. Write to stdout for Claude to consume

**Exit Codes:**
- `0` - Success, context injected
- `1` - No notebook ID (write minimal context)
- `2` - Error reading files

### Task 3: Create Log Hook (`log-activity.sh`)
**File:** `.claude/hooks/log-activity.sh`

**Purpose:**
- Runs on PostToolUse events (after Bash, Write, Edit)
- Logs actions to `.trinity/current_task/activity.md`
- Maintains action counter, resets every 10 actions

**Key Logic:**
1. Parse tool_name and tool_input from stdin JSON
2. Read action counter from `.trinity/current_task/.action_count`
3. Increment and write back
4. Format log line with timestamp, action count
5. Append to activity.md

**Log Format:**
```
[YYYY-MM-DD HH:MM:SS] {tool_name}: {description}
```

### Task 4: Create Pre-Push Gate (`pre-push-gate.sh`)
**File:** `.claude/hooks/pre-push-gate.sh`

**Purpose:**
- Runs before git push
- Blocks push if no active notebook
- Validates git status (no uncommitted changes)

**Key Logic:**
1. Check notebook ID from `.trinity/current_task/.notebook_id`
2. If "disabled" → skip gate (exit 0)
3. If empty → block push (exit 2 with stderr: "BLOCKED: Cannot push without notebook")
4. Otherwise allow push (exit 0)

**Exit Codes:**
- `0` - Success (push allowed)
- `2` - BLOCKED (no notebook)
- `1` - Error (unexpected)

### Task 5: Update Settings JSON
**File:** `.claude/settings.json`

**Purpose:**
- Register all hooks with Claude Code
- Ensures proper hook ordering (session-gate MUST be first)

**Required Structure:**
```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": {
          "bash": ".claude/hooks/session-gate.sh"
        },
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/session-gate.sh"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "matcher": {
          "bash": ".claude/hooks/inject-notebook-context.sh"
        },
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/inject-notebook-context.sh"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": {
          "bash": ".claude/hooks/log-activity.sh"
        },
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/log-activity.sh"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "bash": ".claude/hooks/pre-push-gate.sh"
        },
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/pre-push-gate.sh"
          }
        ]
      }
    ]
  }
}
```

## File Creation Order
1. `.claude/hooks/session-gate.sh`
2. `.claude/hooks/inject-notebook-context.sh`
3. `.claude/hooks/log-activity.sh`
4. `.claude/hooks/pre-push-gate.sh`
5. Update `.claude/settings.json`

## Implementation Dependencies
- Python 3.10 (for sync.py)
- t27c CLI (for task start/notebook operations)
- jq (for JSON parsing in hooks)

## Verification
```bash
# Test SessionStart gate (simulating no notebook)
rm -f .trinity/current_task/.notebook_id
bash .claude/hooks/session-gate.sh
# Should see BLOCKED message

# Test inject hook (simulating prompt)
echo '{"tool_name":"Write","tool_input":{"file_path":"test.txt"}}' | bash .claude/hooks/inject-notebook-context.sh

# Test pre-push gate (simulating no notebook)
echo '{"tool_name":"Bash","tool_input":{"command":"git status"}}' | bash .claude/hooks/pre-push-gate.sh
# Should see BLOCKED message with exit 2
```

## Testing Strategy
1. Create test tasks to verify each hook independently
2. Run all hooks in sequence to verify full workflow
3. Use `--dry-run` mode where applicable

## Notes
- All hooks must be executable: `chmod +x .claude/hooks/*.sh`
- Settings.json must be valid JSON
- Activity.md will be created/updated automatically by log hook
