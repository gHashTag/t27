---
name: agent-coord
description: Agent Coordination Skill — Enables ALL agents to share knowledge via NotebookLM semantic memory. Use when ANY agent needs to query what other agents have done, or before starting work to avoid duplication. This is the universal knowledge bus for autonomous agents.
version: 1.0.0
---

# AGENT-COORD Skill — Universal Agent Knowledge Bus

Every agent MUST query this skill before starting work. This prevents session amnesia and duplicate work across all agents.

## Constitutional Law: KNOW-BEFORE-WORK

**MANDATORY PRE-WORK CHECK:**

1. **ALL agents** must query existing knowledge before making changes
2. **ALL agents** must upload wrap-up after completing work
3. **No agent works in isolation** — knowledge is shared via NotebookLM

## Quick Reference

```bash
# Before ANY work — check what's already done
agent-coord query "<topic or task>"

# After completing work — share with all agents
agent-coord wrapup --summary "completed X" \
                   --decisions "used Y approach" \
                   --files "changed A, B, C" \
                   --next "step Z"
```

## Standard Query Patterns

All agents use these patterns to retrieve knowledge:

| Pattern | Returns | Example |
|---------|---------|---------|
| "status of <feature>" | Completion state, blockers | "status of GoldenFloat format" |
| "decisions for <task>" | Design decisions made | "decisions for E8 integration" |
| "architecture of <module>" | Design context | "architecture of VSA engine" |
| "known issues with <spec>" | Bugs, blockers | "known issues with quantum.t27" |
| "what agents did <topic>" | Agent work history | "what agents did for CI" |
| "files changed for <task>" | Changed file list | "files changed for P2 sprint" |

## Universal Wrap-Up Schema

After ANY work, ALL agents upload wrap-up with:

```json
{
  "agent": "agent-name-or-id",
  "timestamp": "ISO-8601",
  "summary": "one-line what was done",
  "decisions": ["decision 1", "decision 2"],
  "files": ["file1.t27", "file2.zig"],
  "next": "what should happen next",
  "issue_ref": "issue-N or PR-N",
  "verdict": "clean|toxic|blocked"
}
```

## Agent Types and Coordination

### Code Agents (Coder, Reviewer)
- **Pre-work:** Query for existing implementations, patterns used
- **Post-work:** Upload code changes, test results, verdicts
- **Query:** "what patterns used for <feature>"

### Research Agents (Explorer, Analyst)
- **Pre-work:** Query what's already investigated
- **Post-work:** Upload findings, documentation, recommendations
- **Query:** "what's known about <topic>"

### CI/Build Agents
- **Pre-work:** Query CI history, known failures
- **Post-work:** Upload CI results, fixes applied
- **Query:** "CI status for <branch> or <PR>"

### Doctor Agent (Self-Healing)
- **Pre-work:** Query health history, recovery attempts
- **Post-work:** Upload health events, recovery actions
- **Query:** "health anomalies for <domain>"

## NotebookLM Configuration

Storage: `~/.notebooklm/storage_state.json`
Active Notebook: `t27-QUEEN-BRAIN` (default)
Auth: Cookie-based via `notebooklm login` CLI

## Integration with Other Skills

This skill coordinates with:

- **tri skill** — PHI LOOP mutations, hash seals
- **fpga-synth** — FPGA synthesis workflows
- **vsa-verify** — VSA verification tasks
- **vibee-gen** — VIBEE code generation

All skills feed into the same NotebookLM notebook for unified knowledge.

## Avoiding Session Amnesia

**Problem:** Each agent session starts blank, forgetting previous work.

**Solution:** Always query first, wrap-up last.

```bash
# WRONG — session amnesia
agent starts → makes changes → commits → exits

# RIGHT — knowledge preserved
agent starts → queries → learns context → works → wraps up → exits
```

## Example Workflow

```bash
# Agent starts new session
agent-coord query "status of P2 Sprint 2 benchmarks"
# Returns: "6 specs created, PR #326 open, CI failing on docs/NOW.md"

# Agent knows what to do
agent-coord query "why is CI failing on PR #326"
# Returns: "docs/NOW.md not in PR diff, needs to be included"

# Agent fixes issue
# ... makes changes ...

# Agent uploads wrap-up
agent-coord wrapup --summary "fixed PR #326 CI by adding docs/NOW.md" \
                   --decisions "added NOW.md to commit" \
                   --files "docs/NOW.md" \
                   --next "merge PR #326"
```

## Knowledge Persistence

All wrap-ups are stored in NotebookLM and persist across:
- Different agent sessions
- Different agent types
- Different time periods
- Different tasks

This creates a living knowledge base that grows with every agent action.

## Emergency Recovery

If NotebookLM is unavailable:

1. Check `.trinity/experience/episodes.jsonl` — local episode log
2. Check `.trinity/events/akashic-log.jsonl` — event history
3. Check git log for recent commits

These are fallback knowledge sources when NotebookLM is down.

## Agent Coordination Laws

1. **Law 1: Query First** — Never work without checking existing knowledge
2. **Law 2: Wrap-Up Last** — Never exit without uploading what you did
3. **Law 3: Shared Context** — All agents use the same notebook
4. **Law 4: Explicit Next** — Always specify what happens next
5. **Law 5: Issue Binding** — Always reference issue/PR numbers

## Scripts and Tools

```bash
# Query NotebookLM (via contrib/backend/notebooklm/)
notebooklm query "<search query>"

# Upload wrap-up
notebooklm wrapup --summary "..." --decisions "..." --files "..." --next "..."

# List available notebooks
notebooklm list

# Switch active notebook
notebooklm use <notebook-id>

# Check connection status
notebooklm status
```

## Testing

```bash
# Verify NotebookLM connection
python contrib/backend/notebooklm/test_client.py

# Test wrap-up upload
python test_notebooklm.py
```

---

**Constitutional Basis:** This skill implements L1 TRACEABILITY across all agents by ensuring every action is recorded and discoverable via semantic search.
