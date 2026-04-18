---
name: wrap-up
description: Format and upload session wrap-up to NotebookLM for persistent semantic memory
version: 1.1.0
author: Trinity S3AI Framework
---

# Wrap-Up Skill

Upload session summaries to NotebookLM for cross-session memory persistence.

## MANDATORY: Notebook ID Required

**L7 UNITY Requirement:** Wrap-up without `notebook_id` is rejected.

Before using this skill, you MUST have:
1. Run `t27c bridge task start --title "your task"`
2. Or run `t27c bridge task attach --notebook_id "..."`

The wrap-up will be uploaded to the notebook specified in `.trinity/current_task/.notebook_id`.

If no notebook is configured, this skill will fail with an error.

<<<<<<< Updated upstream
<<<<<<< Updated upstream
=======
## What It Does

>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
## What It Does

1. Extracts session context from `.trinity/` state files
2. Formats summary as Markdown with metadata
3. Uploads to NotebookLM as searchable source

## MANDATORY: Notebook Required

**⚠️ Wrap-up without a task notebook is REJECTED**

Before running wrap-up, you MUST have:
- A valid `.trinity/current_task/.notebook_id` file
- Run `t27c task start --title "your task"` to create one

## Usage

```
/wrap-up "Session completed Phi Loop iterations for Ring-071"
```

Or with full details:

```
/wrap-up --summary "Implemented NotebookLM backend" \
         --decisions "Used notebooklm-py SDK with cookie auth" \
         --files "contrib/backend/notebooklm/*.py" \
         --steps "Run integration tests"
```

## Prerequisites

```bash
# 1. Initialize task (creates notebook)
t27c task start --title "Your task description"

# 2. Do your work...

# 3. Run wrap-up (requires valid notebook)
/wrap-up --summary "completed task" --decisions "..." --files "..." --steps "..."
```

## Implementation

This skill uses the t27 spec-first approach:

- **Spec**: `specs/automation/wrapup-auto.t27`
- **Backend**: `contrib/backend/notebooklm/wrapup_auto.py`
- **Invocation**: Python script directly (no shell scripts - L7 compliant)

**Direct invocation (for debugging):**
```bash
.trinity/notebooklm-venv/bin/python3 \
    contrib/backend/notebooklm/wrapup_auto.py \
    --summary "Session summary" \
    --session-id "$(git rev-parse --short HEAD)" \
    --decisions "Key decisions" \
    --files "file1.py,file2.py" \
    --steps "Next steps"
```

## Configuration

- **Auth**: Cookie-based via `notebooklm login` (stores in `~/.notebooklm/storage_state.json`)
- **Active Notebook**: Read from `.trinity/current_task/.notebook_id`
- **Default Notebook**: "t27-QUEEN-BRAIN" (creates if not exists)
- **Storage**: `~/.notebooklm/` — browser profile, storage state

**Setup Commands:**
```bash
python -m venv .trinity/notebooklm-venv
.trinity/notebooklm-venv/bin/pip install notebooklm-py
notebooklm login              # Authenticate via browser (one-time)
```

## Output

Returns source ID and notebook ID for verification:
```json
{
  "notebook_id": "...",
  "source_id": "...",
  "uploaded_at": "2026-04-08T..."
}
```
