# Wave Loop 371 — IGLA CODER+RACE Plan

**Tracking issue:** #1260 (to be created)  
**Branch:** `trinity-rust-rings`  
**Date:** 2026-07-02  
**Target:** 228 generic ∀ theorems, 47-variable accumulation, board-flash retry, one safe `gen-verilog` sub-fix.

---

## 0. Context from W370 Close-out

W370 landed as commit `d5d6250ab` on `trinity-rust-rings` with `Closes #1259`:
- **224 generic ∀** in `proofs/lean4/Trinity/TernaryInference.lean`
- 46-variable plus accumulation, 45-variable minus lattice
- Tresvigintuple (depth-23) residual cancellation
- Zero-weight tredecuple closure (6 zero + 1 plus + 7 zero = 13 zero-weight MACs, 14 variables)
- **12,696 tests / 5,549 invariants** across all specs
- `gen-verilog` defect 1 fixed: all `const` declarations now emit
- Conformance: **549/549 PASS**
- Board flash still blocked by missing DLC10 cable

**Key external finding:** `master` already has the full #1245 gen-verilog fix set (commit `701d79b3b`), but `trinity-rust-rings` has diverged and W371 will continue applying narrow, regression-free sub-fixes on the wave-loop branch.

---

## 1. Issue Triage (GitHub `gHashTag/t27`)

| Issue | Status | Relevance |
|---|---|---|
| **#1260** | NEW | Canonical W371 tracking issue (to be created in this wave). |
| **#1259** | CLOSED by W370 | Historical W370 record. |
| **#1258** | OPEN | `gen-verilog` array/RAM lowering for datapath specs — orthogonal to IGLA, too large for W371. |
| **#1257–#1239** | OPEN | Historical wave issues; kept open as a record. |
| **#1243** | OPEN | BPSK modem PHY — optional parallel track, not a W371 blocker. |
| **#1219** | OPEN | Language roadmap EPIC — strategic context, not a W371 blocker. |
| **#1215** | OPEN | Promote gf10/gf256 to bitexact — conformance track, orthogonal to IGLA. |

**Strategic implication:** W371 works on `trinity-rust-rings`. The main external risks remain hardware availability (DLC10 cable/board) and the remaining gen-verilog defects 3/4/5.

---

## 2. Weak Points & Research Synthesis

### Weak points ranked by W371 actionability

1. **Silicon evidence gap / board flash blocker** — `dlc10 idcode` still fails with `DLC10 cable not found (VID=0x03FD)`. The 3.6 MB `ternary_mac_demo_top.bit` from W361 is ready but unvalidated. Hardware-availability blocker.
2. **Remaining `gen-verilog` defects on `trinity-rust-rings`** — defects 3 (bare-if early return), 4 (`as`/bitwise body drop), and 5 (struct-field reg name mismatch) remain. W371 will pick one safe sub-fix (defect 3 recommended as next in triage order).
3. **No automated yosys CI smoke gate** — defects 3/4/5 are not exercised by canonical specs; scratch repros are the only guard. A smoke gate is a future option but L7 UNITY prohibits new shell scripts on the critical path.
4. **Proof-lattice / Lean build-time boundary** — `lake build Trinity.TernaryInference` was ~4.8 s at 46 variables. W371 probes 47-variable plus accumulation; fall back if `omega` saturates.
5. **RTL-to-Lean traceability gap** — still a roadmap item, too large for W371.
6. **Stale workspace artifacts** — old `.claude/plans/wave-loop-362.md` through 364 and `scripts/gen_w362_lean.py` remain untracked. Safe to leave or clean in a separate hygiene commit.

### Scientific-paper / competitor landscape (2025–2026)

- **Closest formal competitor:** Sparkle HDL / Verilean — Lean 4-native, BitNet theorems, RV32IMA SoC theorems, but **no public generic ∀ ternary-MAC accumulation proofs** at 46+ variables or 24+ cancellation depth.
- **Implementation competitors:** TerEffic, TOM, TENET, TeLLMe v2, VitaLLM, ternfpga, TernaryCore, KU Leuven ternary-lut-dse — ternary LLM accelerators / open generators with silicon/FPGA metrics but **no theorem-prover verification**.
- **Updated references:** Wen et al. 2024 ternary MAC truth-table model; RSR/RSR++ efficient binary/ternary matmul; BSTCIM ternary in-MRAM CIM; KU Leuven open Chisel ternary-lut generator.
- **Formal verification benchmarks:** RTL-BenchLS (10k verified Verilog designs), NotSoTiny, RealBench, OpenLLM-RTL — future targets, not W371 blockers.
- **Moat:** t27 remains unchallenged in generic quantified ternary accumulation depth. W371 pushes this to 228 generic ∀.

---

## 3. Decomposed Implementation Plan

### Phase A — Preparation (Issue / Spec)
1. Create GitHub tracking issue #1260.
2. Copy `scripts/gen_w370.py` → `scripts/gen_w371.py` and `scripts/gen_w370_lean.py` → `scripts/gen_w371_lean.py`.
3. In `gen_w371_lean.py`, keep keyword skip set `{at, by, do, if, in, or, to}` and set shape for zero-weight quattuordecuple closure to 7 zero before + 1 plus + 7 zero after (14 zero-weight MACs, 15 variables).

### Phase B — Spec Extension (TDD)
4. Run `gen_w371.py` on all 27 IGLA specs.
   - Each spec gains a W371 block: +2 tests, +1 invariant.
   - Expected totals: ~12,750 tests, ~5,576 invariants.

### Phase C — Lean Proof Lattice (Code)
5. Run `gen_w371_lean.py` to append four new generic ∀ theorems:
   - `ternaryMacAccumulateFortySevenPlusGeneric` — 47-variable plus accumulation (a..z, aa..as, au, av; skip `at`).
   - `ternaryMacAccumulateFortySixMinusGeneric` — 46-variable minus accumulation (skip `at`).
   - `ternaryMacQuattuorvigintupleCancellationGeneric` — depth-24 alternating plus/minus identity cancellation (`= x`).
   - `ternaryMacZeroWeightQuattuordecupleClosureGeneric` — 7 zero before + 1 plus + 7 zero after (14 zero-weight MACs).
6. Build `Trinity.TernaryInference` with `lake build`. If 47-variable theorem times out, fall back to 46-variable plus / 45-variable minus and adjust targets.

### Phase D — Gen-Verilog Sub-fix (Code)
7. Implement **defect 3:** preserve fall-through control flow for bare `if` with early `return` in `bootstrap/src/compiler.rs` `gen_verilog_if_stmt`.
8. Add scratch spec `specs/scratch/w371_early_return.t27`:
   ```t27
   fn sign(x : i8) -> i8 {
       if (x < 0) { return -1; }
       if (x > 0) { return 1; }
       return 0;
   }
   ```
9. Verify emitted Verilog with `yosys read_verilog` before mass resealing.

### Phase E — Seal & Conformance (Verify)
10. Regenerate all affected seals (27 IGLA + any non-IGLA specs whose Verilog output shifts).
11. Run full conformance: `./target/release/t27c suite --repo-root /Users/playra/t27`.
    - Gate: 549/549 PASS minimum.

### Phase F — Board Flash (Verify)
12. Build `dlc10`: `cargo build --release -p dlc10`.
13. Run `./target/release/dlc10 idcode`.
    - If success: capture IDCODE and proceed to `sram`/`flash`.
    - If failure: document in `docs/reports/FPGA_EVIDENCE_W371.md`.

### Phase G — Reports & Cooperation (Synthesize)
14. Write `docs/reports/WAVE_LOOP_371_REPORT.md`.
15. Write `docs/reports/WAVE_LOOP_371_COOPERATION.md` with three W372 variants:
    - **Variant A:** Formal-only (safe) — 229 generic ∀ target.
    - **Variant B:** Formal + board flash retry + one gen-verilog sub-fix (recommended) — 232 generic ∀ + silicon evidence + backend hardening.
    - **Variant C:** Formal + RTL-to-Lean traceability prototype + board flash — higher risk, deeper moat.
16. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to mark defect 3 fixed (if it succeeds).

### Phase H — Land & Learn
17. Stage all W371 files, commit with message containing `Closes #1260`.
18. Run final conformance after commit to confirm PASS.
19. Update `.trinity/experience.md` with W371 learnings.
20. Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-371.md` and update `MEMORY.md` index.

---

## 4. Risk Register

| Risk | Likelihood | Mitigation |
|---|---|---|
| 47-variable theorem times out | Medium | Fall back to 46-variable plus / 45-variable minus. |
| Defect 3 sub-fix breaks seals | Low–Medium | Test on scratch spec first; run full suite before commit. |
| Board still missing | High | Document and move on; keep retry in W372. |
| Full conformance count drifts | Low | Reseal any mismatched spec immediately. |
| `trinity-rust-rings` drift from master | High | Do not merge master; keep changes narrow and branch-local. |

---

## 5. Success Criteria

- [ ] 228 generic ∀ theorems in `TernaryInference.lean`.
- [ ] 47-variable plus accumulation theorem builds (or documented fallback).
- [ ] Zero-weight quattuordecuple closure theorem is correctly 14 zeros + 1 plus.
- [ ] 27 IGLA specs extended, all 27 seals regenerated.
- [ ] Conformance suite: PASS (549/549 or current count).
- [ ] One safe `gen-verilog` sub-fix with scratch-spec test.
- [ ] Board flash attempted and documented.
- [ ] `WAVE_LOOP_371_REPORT.md` and `WAVE_LOOP_371_COOPERATION.md` written.
- [ ] Memory and experience updated.
- [ ] Commit on `trinity-rust-rings` with `Closes #1260`.

---

*phi² + 1/phi² = 3 | TRINITY*
