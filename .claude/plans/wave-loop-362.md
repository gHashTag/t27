# Plan: Wave Loop 362 — IGLA CODER+RACE + board flash

**Date:** 2026-07-01
**Issue target:** #1243 (open) or create #1244 for W362
**Branch:** `trinity-rust-rings`
**Variant:** B from `docs/reports/WAVE_LOOP_361_COOPERATION.md` — formal depth + board flash.

---

## Goal

Extend the IGLA proof lattice to **192 generic ∀** theorems, append the W362 wave block to all 27 core specs, regenerate the 27 seals, and flash the existing `fpga/verilog/ternary_mac_demo_top.bit` to the QMTech Wukong V1 board via the in-tree `dlc10` driver. Document the result in `docs/reports/WAVE_LOOP_362_REPORT.md`, `WAVE_LOOP_362_COOPERATION.md`, and `FPGA_EVIDENCE_W362.md`, then save memory/skills.

---

## Files to change

1. `specs/igla/coder/*.t27` (9 specs) — append W362 wave block.
2. `specs/igla/race/*.t27` (18 specs) — append W362 wave block.
3. `proofs/lean4/Trinity/TernaryInference.lean` — add 4 new generic ∀ theorems.
4. `.trinity/seals/*.json` (27 seals) — regenerate after spec changes.
5. `docs/reports/WAVE_LOOP_362_REPORT.md` — new.
6. `docs/reports/WAVE_LOOP_362_COOPERATION.md` — new.
7. `docs/reports/FPGA_EVIDENCE_W362.md` — new or update W361 evidence.
8. `~/.claude/projects/-Users-playra-t27/memory/wave-loop-362.md` — new.
9. `~/.claude/projects/-Users-playra-t27/memory/MEMORY.md` — prepend entry.
10. `.trinity/experience.md` — append W362 entry.
11. `scripts/gen_w362.py` — optional generator for the wave block (matches `gen_w328.py` pattern).

---

## Implementation steps

### 1. Spec batch (27 specs)

- Append a W362 wave block after the last W361 block in each of the 27 IGLA specs.
- Each block:
  - comment header: `// Wave Loop 362 -- <ring> depth +1`
  - cross-reference: `// See docs/reports/WAVE_LOOP_361_COOPERATION.md Variant B`
  - two `test` cases named `igla_<ring>_<spec>_w362_batch_depth_invariant_1/_2`
  - one `invariant` named `igla_<ring>_<spec>_w362_depth: true`
- Use a generator script `scripts/gen_w362.py` and run it once from repo root, then verify with `git diff --stat`.

### 2. Lean 4 proof lattice (4 theorems)

Add to `proofs/lean4/Trinity/TernaryInference.lean` after the W361 theorems:

1. `ternaryMacAccumulateThirtyEightPlusGeneric` — `mac^38(0, [a..al], .plus) = a+b+...+al`
2. `ternaryMacAccumulateThirtySevenMinusGeneric` — `mac^37(0, [a..ak], .minus) = -(a+b+...+ak)`
3. `ternaryMacQuindecupleCancellationGeneric` — depth-15 alternating plus/minus with residual `mac(x,a,.plus)`
4. `ternaryMacZeroWeightQuintupleClosureGeneric` — five zero-weight ops around a plus-weight MAC are transparent/reorderable

Use the existing proof style: `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode] <;> try omega`.

### 3. Build and verify

- `lake build Trinity.TernaryInference` — must succeed.
- `/Users/playra/t27/target/release/t27c suite --repo-root /Users/playra/t27` — must be 546/546 PASS.
- If a new issue is needed, create it with `env -u GH_TOKEN gh issue create --title "Wave Loop 362 ..." --body "..."`.

### 4. Seal regeneration

- Run seal regeneration from repo root for all 27 IGLA specs. The exact command pattern is `t27c seal --save <spec>` or the wrapper `./scripts/tri seal <spec>`.
- Verify all 27 `.trinity/seals/*.json` are updated and the suite still passes.

### 5. Board flash attempt

- `cargo build --release -p dlc10` (already built).
- `/Users/playra/t27/target/release/dlc10 idcode` — if cable is connected, expect `0x13631093`.
- If the board is available: `/Users/playra/t27/target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`
- Capture `STAT` register, confirm `DONE=HIGH`, `CRC_ERROR=0`, observe R23/T23 LEDs.
- If the board is not available, document the blocker and the ready-to-load bitstream state; do not claim silicon verified.

### 6. Reports and memory

- Write `docs/reports/WAVE_LOOP_362_REPORT.md` with metrics table, spec/Lean/FPGA sections, and issue link.
- Write `docs/reports/WAVE_LOOP_362_COOPERATION.md` with Variants A/B/C for W363.
- Write or update `docs/reports/FPGA_EVIDENCE_W362.md`.
- Write memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-362.md` and prepend one-line pointer to `MEMORY.md`.
- Append W362 entry to `.trinity/experience.md`.

### 7. Commit

- Single IGLA commit with `Closes #N` referencing the W362 issue.
- Separate docs/memory commits if desired.
- Ensure commit messages pass L1 via the hook.

---

## Fallbacks

- If `omega` saturates at 38 variables, fall back to 37-variable minus lattice as the primary theorem and skip the 38-plus probe.
- If board flash fails due to missing cable/board, document the ready state and continue with Variant A scope.
- If seal generation drifts, stop and inspect `t27c seal --save` path behavior.

---

## Success criteria

- `t27c suite` returns **0 failures**.
- `lake build Trinity.TernaryInference` succeeds.
- Generic ∀ count reaches **192** (188 + 4).
- W362 wave blocks present in all 27 IGLA specs.
- 27 seals regenerated and matching.
- Report and cooperation variants written.
- Memory/skills saved.
- Board flash attempted with documented result.
