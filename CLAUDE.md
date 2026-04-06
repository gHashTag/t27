# CLAUDE.md — Instructions for Claude Code and autonomous agents (t27)

Use this file **together with** `[AGENTS.md](AGENTS.md)`. Repo-specific law always overrides generic tooling defaults.

---

## 1. Mandatory read order for this repository

1. `[AGENTS.md](AGENTS.md)` — entry point and constitutional stack.
2. `[SOUL.md](SOUL.md)` — canonical law (TDD, language, validation).
3. `[docs/T27-CONSTITUTION.md](docs/T27-CONSTITUTION.md)` — **SSOT-MATH**, **LANG-EN**, **DOCS-TREE**.
4. `[TASK.md](TASK.md)` and `[docs/coordination/TASK_PROTOCOL.md](docs/coordination/TASK_PROTOCOL.md)` — if the task touches coordination, locks, or shared hot paths.
5. Nearest `[OWNERS.md](OWNERS.md)` for the directories you edit.

Do **not** add parallel math/physics implementations in ad-hoc scripts when the same belongs in `*.t27` and the **`tri`** pipeline (`./scripts/tri`).

---

## 2. Engineering workflow

- **Bootstrap compiler:** `cd bootstrap && cargo build --release` (runs `build.rs` language checks).
- **Local sweep (CI-like):** from repo root, `./scripts/tri test` or `./bootstrap/target/release/t27c suite --repo-root .` (Rust runner; no shell test harness under `tests/`).
- **Generated code:** under `gen/` — do not hand-edit for routine fixes; change specs and regenerate.
- **Pull requests:** follow project Issue Gate and linking policy; **do not approve** PRs unless explicitly authorized.

---

## 3. Autonomous subagent behavior (when spawned unattended)

- Finish the assigned task without waiting for clarification unless the repo’s own rules require human input.
- If blocked after reasonable retries, stop and report what failed (logs, commands, file paths).
- Prefer small, reviewable diffs; match existing style and naming in touched files.
- **Output persistence:** when the parent workflow requires it, write the full final report to `/tmp/claude_code_output.md` (analysis, commands, diffs summary).

---

## 4. Skills and tooling (optional)

If your environment exposes **skills** (e.g. coding-workflow, commit-push-pr), load what matches the task. After cloning any repo, discover project-specific skills per host conventions. **This repository’s normative text remains in `AGENTS.md`, `SOUL.md`, and `docs/`.**

---

## 5. Security and secrets

- Never commit secrets. See `[SECURITY.md](SECURITY.md)`. Root `.env` patterns are gitignored; use `.env.example` patterns only in docs.

---

**Repository:** Trinity S³AI — **t27** (spec-first ternary / TRI-27). **φ² + 1/φ² = 3 | TRINITY**