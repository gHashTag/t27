# CLAUDE.md — Instructions for Claude Code and autonomous agents (t27)

Use this file **together with** `[AGENTS.md](AGENTS.md)`. Repo-specific law always overrides generic tooling defaults.

---

## Autonomous Execution Loop (AEL v2.0)

When operating as the Trinity Agent (Queen), follow this 6-phase loop:

```
┌─────────────────────────────────────────────────────────────┐
│  OBSERVE → PLAN → DELEGATE → VERIFY → SYNTHESIZE → LEARN   │
│         ↓       ↓        ↓        ↓         ↓         ↓    │
│  [E]     [T]     [C/V]    [V]      [L]      [L]           │
└─────────────────────────────────────────────────────────────┘
```

### Phase 1: OBSERVE
- Call Experience Agent (E) for context
- Read `.trinity/current-issue.md` for issue details
- Check ring and phase state from branch name
- Gather relevant files and context

### Phase 2: PLAN
- Break down task into subtasks
- Identify required skills: `/phi-loop`, `/tri-pipeline`, `/experience-save`
- Determine which agents to delegate to
- Estimate complexity and dependencies

### Phase 3: DELEGATE
- Delegate implementation to Creator Agent (C)
- Delegate validation to Verifier Agent (V)
- Coordinate parallel execution where possible
- Monitor agent progress

### Phase 4: VERIFY
- Review agent outputs
- Run conformance tests via `/tri-pipeline`
- Check L1-L7 law compliance
- Ensure quality standards

### Phase 5: SYNTHESIZE
- Combine agent results
- Resolve conflicts
- Create cohesive solution
- Prepare for integration

### Phase 6: LEARN
- Call Learner Agent (L) for pattern extraction
- Update `.trinity/experience.md` via `/experience-save`
- Save ring-specific learnings
- Improve future execution

---

## 1. Mandatory read order for this repository

1. `[AGENTS.md](AGENTS.md)` — entry point and constitutional stack.
2. `[SOUL.md](SOUL.md)` — canonical law (TDD, language, validation).
3. `[docs/T27-CONSTITUTION.md](docs/T27-CONSTITUTION.md)` — **SSOT-MATH**, **LANG-EN**, **DOCS-TREE**.
4. `[NOW.md](NOW.md)` and `[docs/coordination/TASK_PROTOCOL.md](docs/coordination/TASK_PROTOCOL.md)` — if the task touches coordination, locks, or shared hot paths.
5. Nearest `[OWNERS.md](OWNERS.md)` for the directories you edit.

Do **not** add parallel math/physics implementations in ad-hoc scripts when the same belongs in `*.t27` and the **`tri`** pipeline (`./scripts/tri`).

---

## 2. Engineering workflow

- **Bootstrap compiler:** `cd bootstrap && cargo build --release` (runs `build.rs` language checks).
- **Local sweep (CI-like):** from repo root, `./scripts/tri test` or `./bootstrap/target/release/t27c --repo-root . suite` (Rust runner; no shell test harness under `tests/`).
- **Generated code:** under `gen/` — do not hand-edit for routine fixes; change specs and regenerate.
- **Pull requests:** follow project Issue Gate and linking policy; **do not approve** PRs unless explicitly authorized.

---

## 3. PHI LOOP Execution

Follow the 9-phase PHI LOOP for ring-based development:

1. **Issue** - Define problem or requirement
2. **Spec** - Write .t27 specification
3. **TDD** - Write tests in spec before implementation
4. **Code/Impl** - Implement according to spec
5. **Gen** - Run `tri gen` to generate code from spec
6. **Seal** - Verify generated code and seal hash
7. **Verify** - Run `tri test` or conformance checks
8. **Land** - Merge changes to main branch
9. **Learn** - Capture learnings and update knowledge base

### Phase Completion Marker

When a phase is complete, include in your output:
```
Phase complete: [phase name]
→ Phase [next phase number]: [next phase name]
```

This triggers automatic branch creation for the next phase.

---

## 4. Autonomous subagent behavior (when spawned unattended)

- Finish the assigned task without waiting for clarification unless the repo's own rules require human input.
- If blocked after reasonable retries, stop and report what failed (logs, commands, file paths).
- Prefer small, reviewable diffs; match existing style and naming in touched files.
- **Output persistence:** when the parent workflow requires it, write the full final report to `/tmp/claude_code_output.md` (analysis, commands, diffs summary).

---

## 5. Skills and tooling

### Available Skills

- `/phi-loop` - Execute 9-phase PHI LOOP
- `/tri-pipeline` - Execute tri commands (gen, test, verify, seal)
- `/experience-save` - Save learnings to persistent memory

Load these skills when their functionality matches the task.

---

## 6. Security and secrets

- Never commit secrets. See `[SECURITY.md](SECURITY.md)`. Root `.env` patterns are gitignored; use `.env.example` patterns only in docs.

---

## The 7 Invariant Laws

| Law | Name | Description |
|------|------|-------------|
| L1 | TRACEABILITY | No code merged without `Closes #N` |
| L2 | GENERATION | Files under `gen/` are generated; edit specs instead |
| L3 | PURITY | Source files must be ASCII-only with English identifiers |
| L4 | TESTABILITY | Every `.t27` spec must contain `test`/`invariant`/`bench` |
| L5 | IDENTITY | φ² = φ + 1; φ² + φ⁻² = 3; IEEE f64 checks use tolerance |
| L6 | CEILING | `FORMAT-SPEC-001.json` + `gf16.t27` are numeric SSOT |
| L7 | UNITY | No new `*.sh` on critical path; use `tri`/`t27c` |

See [`docs/T27-CONSTITUTION.md`](docs/T27-CONSTITUTION.md#2--invariant-laws-never-change-without-constitutional-amendment) for full details.

---

**Repository:** Trinity S³AI — **t27** (spec-first ternary / TRI-27). **φ² + 1/φ² = 3 | TRINITY**
