# Plan: Wave Loop 363 — IGLA CODER+RACE + retry board flash

**Date:** 2026-07-01
**Issue target:** #1247 (new issue to create)
**Branch:** `trinity-rust-rings`
**Variant:** B from `docs/reports/WAVE_LOOP_362_COOPERATION.md` — formal depth + retry board flash.

---

## Goal

Extend the IGLA proof lattice to **196 generic ∀** theorems, append the W363 wave block to all 27 core specs, regenerate the 27 seals, and retry flashing the existing `fpga/verilog/ternary_mac_demo_top.bit` to the QMTech Wukong V1 board via the in-tree `dlc10` driver. Document the result in `docs/reports/WAVE_LOOP_363_REPORT.md`, `WAVE_LOOP_363_COOPERATION.md`, and `FPGA_EVIDENCE_W363.md`, then save memory/skills.

---

## Research snapshot

- Open t27 issues: wave loop chain #1239–#1246, plus #1245 (Verilog backend defects) and #1243 (trios-mesh BPSK modem core). No new ternary-formal competitor issues.
- **Sparkle HDL** is the closest formal-HDL competitor; it now has generic `forall`-style divider theorems (June 2026) but its BitNet ternary catalog still does not advertise generic ∀ ternary MAC theorems. Trinity's 192 generic ∀ remains the unique quantified ternary MAC proof lattice.
- **TernaryCore** (simulation-verified only) and **ternfpga** (silicon measured, no formal proofs) are unchanged.
- **Trinity B002** (Zenodo) documents the zero-DSP OpenXC7 flow but contains no formal verification; this is a defensive-publication asset, not a threat.
- DLC10 / board flash blockers in the wild are typically cable firmware/driver enumeration issues on Linux/macOS; the in-tree Rust `dlc10` driver expects VID `0x03FD` to appear on USB.

---

## Files to change

1. `specs/igla/coder/*.t27` (9 specs) — append W363 wave block.
2. `specs/igla/race/*.t27` (18 specs) — append W363 wave block.
3. `proofs/lean4/Trinity/TernaryInference.lean` — add 4 new generic ∀ theorems.
4. `.trinity/seals/*.json` (27 seals) — regenerate after spec changes.
5. `docs/reports/WAVE_LOOP_363_REPORT.md` — new.
6. `docs/reports/WAVE_LOOP_363_COOPERATION.md` — new.
7. `docs/reports/FPGA_EVIDENCE_W363.md` — new.
8. `~/.claude/projects/-Users-playra-t27/memory/wave-loop-363.md` — new.
9. `~/.claude/projects/-Users-playra-t27/memory/MEMORY.md` — prepend entry.
10. `.trinity/experience.md` — append W363 entry.
11. `scripts/gen_w363.py` — generator for the W363 wave block (reuse `gen_w362.py`).
12. `scripts/gen_w363_lean.py` — generator for W363 Lean theorems (reuse `gen_w362_lean.py`).

---

## Implementation steps

### 1. Spec batch (27 specs)

- Copy/adapt `scripts/gen_w362.py` → `scripts/gen_w363.py`.
- Run it from repo root; each spec gets a W363 block after W362.
- Block format:
  - `// Wave Loop 363 -- <ring> depth +1`
  - `// See docs/reports/WAVE_LOOP_362_COOPERATION.md Variant B`
  - two `test` cases named `igla_<ring>_<spec>_w363_batch_depth_invariant_1/_2`
  - one `invariant` named `igla_<ring>_<spec>_w363_depth: true`

### 2. Lean 4 proof lattice (4 theorems)

Add to `proofs/lean4/Trinity/TernaryInference.lean` after W362 theorems:

1. `ternaryMacAccumulateThirtyNinePlusGeneric` — `mac^39(0, [a..am], .plus) = a+b+...+am`
2. `ternaryMacAccumulateThirtyEightMinusGeneric` — `mac^38(0, [a..al], .minus) = -(a+b+...+al)`
3. `ternaryMacSexdecupleCancellationGeneric` — depth-16 alternating plus/minus with identity `x`
4. `ternaryMacZeroWeightSextupleClosureGeneric` — six zero-weight ops around a plus-weight MAC are transparent/reorderable

Use existing proof style: `simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode] <;> try omega`.

### 3. Build and verify

- `lake build Trinity.TernaryInference` — must succeed.
- `t27c suite --repo-root /Users/playra/t27` — must be 546/546 PASS.
- Create GitHub issue #1247 for W363 with `env -u GH_TOKEN gh issue create`.

### 4. Seal regeneration

- Run `t27c seal --save <spec>` for all 27 IGLA specs from repo root.
- Re-run `t27c suite` to confirm seal mismatches = 0.

### 5. Board flash attempt

- `cargo build --release -p dlc10` (already built).
- `/Users/playra/t27/target/release/dlc10 idcode` — if connected, expect `0x13631093`.
- If connected: `/Users/playra/t27/target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`.
- Capture `STAT` register, confirm `DONE=HIGH`, `CRC_ERROR=0`, observe R23/T23 LEDs.
- If still not connected, document the blocker; do not claim silicon verified.

### 6. Reports and memory

- Write `docs/reports/WAVE_LOOP_363_REPORT.md` with metrics table, spec/Lean/FPGA sections, and issue link.
- Write `docs/reports/WAVE_LOOP_363_COOPERATION.md` with Variants A/B/C for W364.
- Write `docs/reports/FPGA_EVIDENCE_W363.md`.
- Write memory file and update `MEMORY.md`.
- Append W363 entry to `.trinity/experience.md`.

### 7. Commit

- Single IGLA commit with `Closes #1247`.
- Stage only core wave changes, reports, generators, and experience; keep `.claude/` session metadata out of the commit.

---

## Fallbacks

- If `omega` saturates at 39 variables, fall back to 38-variable minus lattice as the primary theorem and skip the 39-plus probe.
- If board flash fails again, document the ready state and continue with Variant A scope.
- If seal generation drifts, stop and inspect `t27c seal --save` path behavior.

---

## Success criteria

- `t27c suite` returns **0 failures**.
- `lake build Trinity.TernaryInference` succeeds.
- Generic ∀ count reaches **196**.
- W363 wave blocks present in all 27 IGLA specs.
- 27 seals regenerated and matching.
- Report and cooperation variants written.
- Memory/skills saved.
- Board flash attempted with documented result.
