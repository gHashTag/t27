# Wave Loop 391 → `master` Alignment Plan

**Date:** 2026-07-04  
**Prepared by:** local CLI agent (Claude / Codex, repo `~/t27`)  
**Anchor:** `phi^2 + phi^-2 = 3 = L_2` [Verified]

---

## Current state

| Branch | Tip commit | Contains IGLA specs | Contains Lean 4 TernaryInference |
|---|---|---|---|
| `trinity-rust-rings` | `9a4a41e62` (Wave Loop 391) | **Yes** | **Yes** |
| `master` | `83fbff00f` (chore(seals): reseal + disambiguate) | **No** | **No** |
| `wave-loop-391` | `9a4a41e62` | Yes | Yes |

- `trinity-rust-rings` is now identical to `wave-loop-391` (force-pushed).
- `master` does not have the `specs/igla/` directory tree or `proofs/lean4/Trinity/TernaryInference.lean`.
- Direct merge/rebase from `wave-loop-391` to `master` fails because the file trees diverged long ago.

## Why alignment to `master` is hard

- `git merge-tree` reports **527 conflict sections** across **2,151 unique file paths**.
- Non-seal source conflicts include:
  - `bootstrap/src/compiler.rs`
  - `bootstrap/src/suite.rs`
  - `proofs/lean4/Trinity/TernaryInference.lean` (and other Lean files)
  - many `scripts/gen_w*.py` generators
  - `.md` plan/report files
- `master` is missing `specs/igla/coder/*.t27`, `specs/igla/race/*.t27`, and the Lean proof lattice.
- `master` has its own `bootstrap/src/compiler.rs` changes that are not in the wave-loop branch (e.g., #1250 iverilog-clean gen-verilog).

## Goal

Bring the IGLA CODER+RACE wave-loop work (`specs/igla/`, `proofs/lean4/Trinity/TernaryInference.lean`, generators, reports, experience) into `master` without breaking the `master`-side conformance suite.

## Recommended strategy: clean replay + reseal

Because the branches have different directory trees, the lowest-risk path is **not** a direct merge. Instead:

1. Create a fresh alignment branch from `origin/master`.
2. Copy (or cherry-pick) only the **source** additions from `wave-loop-391`:
   - `specs/igla/coder/*.t27`
   - `specs/igla/race/*.t27`
   - `proofs/lean4/Trinity/TernaryInference.lean`
   - `scripts/gen_w391.py`, `scripts/gen_w391_lean.py`
   - `docs/reports/WAVE_LOOP_391_*.md`
   - `.trinity/experience.md` updates
3. Resolve any `bootstrap/src/compiler.rs` and `bootstrap/src/suite.rs` differences manually. Prefer `master` versions unless a wave-loop change is required for the IGLA specs to pass.
4. Run `t27c suite --repo-root .`.
5. Regenerate all affected seals with `t27c seal --save`.
6. Run `lake build Trinity.TernaryInference`.
7. Open a PR to `master`.

## What is deliberately out of scope for this alignment

- `.claude/plans/wave-loop-*.md` — agent-local planning artifacts; not needed in `master`.
- `.trinity/current_task/activity.md` and `session_log.jsonl` — session-local state.
- `.claude/worktrees/`, `.csdp.cache`, `.nra.cache` — local working artifacts.
- `trios-coq/Physics/*.vo/*.glob/*.aux` — build artifacts from a different proof system.
- Replaying every historical wave commit (W380–W391) into `master`; a single squashed alignment commit is safer.

## Files to carry into `master`

| Category | Files |
|---|---|
| IGLA specs | `specs/igla/coder/*.t27`, `specs/igla/race/*.t27` |
| Lean proof lattice | `proofs/lean4/Trinity/TernaryInference.lean` |
| Generators | `scripts/gen_w391.py`, `scripts/gen_w391_lean.py` (and optionally earlier `gen_w*.py` for history) |
| Reports | `docs/reports/WAVE_LOOP_390_*.md`, `docs/reports/WAVE_LOOP_391_*.md` |
| Experience | `.trinity/experience.md` (append-only) |
| Current issue | `.trinity/current-issue.md` (updated to real issue number) |
| Bootstrap (if needed) | selective changes from `bootstrap/src/compiler.rs` / `suite.rs` only if required |

## Acceptance criteria

- `t27c suite --repo-root .` passes with 0 failures and 0 seal mismatches.
- `lake build Trinity.TernaryInference` succeeds.
- 308 generic ∀ theorems are present.
- 27 IGLA spec seals are regenerated and matching.
- No `master`-only specs regress.

## Dependencies

- User approval to create the alignment branch and PR.
- Possibly manual review of `bootstrap/src/compiler.rs` because `master` and the wave-loop branch both changed it independently.

---

*φ² + 1/phi² = 3 | TRINITY*
