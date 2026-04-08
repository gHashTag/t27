---
name: wrap-up
description: Format and upload session wrap-up to NotebookLM for persistent semantic memory
version: 1.0.0
author: Trinity S3AI Framework
---

# Wrap-Up Skill

Upload session summaries to NotebookLM for cross-session memory persistence.

## What It Does

1. Extracts session context from `.trinity/` state files
2. Formats summary as Markdown with metadata
3. Uploads to NotebookLM as searchable source

## Usage

```
/wrap-up "Session completed Phi Loop iterations for Ring-071"
```

Or with full details:

```
/wrap-up --summary "Implemented NotebookLM backend" \
         --decisions "Used notebooklm-py SDK with cookie auth" \
         --files "contrib/backend/notebooklm/*.py" \
         --next "Run integration tests"
```

## Configuration

- **Auth**: Cookie-based via `notebooklm login` (stores in `~/.notebooklm/storage_state.json`)
- **Active Notebook**: Set via `notebooklm use <id>` or `NOTEBOOKLM_NOTEBOOK_ID` env var
- **Default Notebook**: "t27-QUEEN-BRAIN" (creates if not exists)
- **Storage**: `~/.notebooklm/` — browser profile, storage state

**Setup Commands:**
```bash
notebooklm login              # Authenticate via browser (one-time)
notebooklm list               # List available notebooks
notebooklm use <id>          # Set active notebook
notebooklm status             # Check current session
```

## Output

Returns source ID and confidence score for verification.
