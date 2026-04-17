# Plan: PHI LOOP Infrastructure Bootstrap

## Context

**What's done:**
- Created state files (`.trinity/state/active-skill.json`, `issue-binding.json`, `episodes.jsonl`)
- Created basic CLI server T27 (`specs/compiler/parser.t27`)
- Updated SKILL.md to v1.2.0

**Current state:**
- NO active skill (status: closed)
- NO issue binding
- Uncommitted changes in infrastructure (state files, SKILL.md, parser spec)

---

## Phase 1: Current status — completing infrastructure

### Task

Commit current uncommitted changes as "infrastructure bootstrap" without opening skill session.

### Changes

```
git add .trinity/state/active-skill.json
git add .trinity/state/issue-binding.json
git add .trinity/experience/episodes.jsonl
git add .claude/skills/tri/SKILL.md
git add specs/compiler/parser.t27
```

### Commit message

```
feat: bootstrap PHI LOOP state infrastructure

Parser spec created for T27 language.
State files: active-skill, issue-binding, episodes.jsonl
- SKILL.md updated to v1.2.0
```

---

## Phase 2: What's next?

### Option A: CLI `tri` (full implementation)
- Create Bash/CLI script `tri` which:
  - Reads/writes state files
  - Runs PHI LOOP commands (begin, gen, test, verdict, etc.)
  - Works as real wrapper over `.trinity/`

**Pros:**
  - Strong guard enforcement at CLI level
  - Clear control flow
  - Automated state machine

**Cons:**
  - Requires full implementation
  - Needs runtime for execution

### Option B: Specifications and git commit (simplest path)
- Continue writing specifications in Markdown (`PHI_LOOP_CONTRACT.md`, `PHI_LOOP_REGO.md`)
- Use `git commit` directly for infrastructure

**Pros:**
  - Can start immediately
  - Works with current T27 CLI
  - Good for documenting guard policies

**Cons:**
  - CLI guard enforcement only at documentation level
  - No automated state machine

---

## Question

Which path do we prefer?

**Option A** — CLI `tri` (full implementation)
**Option B** — Specifications and git commit (simplest path)

---

## Summary

**Option 1** — CLI `tri` (full implementation)
  - Full guard enforcement at CLI level
  - Clear control flow
  - Automated state machine

**Option 2** — Specifications and git commit (simplest path)
  - Can start immediately
  - Works with current T27 CLI
  - Good for documenting guard policies
