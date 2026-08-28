# Wave Loop 370 — IGLA CODER+RACE Plan

**Tracking issue:** #1259  
**Branch:** `trinity-rust-rings`  
**Date:** 2026-07-02  
**Target:** 224 generic ∀ theorems, 46-variable accumulation, board-flash retry, one safe `gen-verilog` sub-fix.

---

## 0. Context from W369 Close-out

W369 landed as commit `b1dfce7bf` on `trinity-rust-rings` with `Closes #1257`:
- **220 generic ∀** in `proofs/lean4/Trinity/TernaryInference.lean`
- 45-variable plus accumulation, 44-variable minus lattice
- Duovigintuple (depth-22) identity cancellation
- Zero-weight duodecuple closure (12 zero + 1 plus = 13 variables)
- **12,641 tests / 5,522 invariants** across 27 IGLA specs
- `gen-verilog` binary (`0b`) width padding extended to scalar `const`/`var`/`let`/`return`
- Conformance: **548/548 PASS**
- Board flash still blocked by missing DLC10 cable

**Key external finding:** `master` already has the full #1245 gen-verilog fix set (commit `701d79b3b`), but `trinity-rust-rings` has diverged and W370 will continue applying narrow, regression-free sub-fixes on the wave-loop branch.

---

## 1. Issue Triage (GitHub `gHashTag/t27`)

| Issue | Status | Relevance |
|---|---|---|
| **#1259** | OPEN | Canonical W370 tracking issue (created in this plan). |
| **#1258** | OPEN | `gen-verilog` array/RAM lowering for datapath specs — orthogonal to IGLA, not a W370 blocker. |
| **#1257** | OPEN | W369 predecessor; will be closed by the W370 commit referencing it indirectly, or left open as a historical wave issue. |
| **#1256 / #1253 / #1252 / #1251 / #1249 / #1246 / #1242 / #1241 / #1240 / #1239** | OPEN | Historical wave issues; kept open as a record unless maintainers close them. |
| **#1243** | OPEN | BPSK modem PHY — optional parallel track, not a W370 blocker. |
| **#1219** | OPEN | Language roadmap EPIC — strategic context, not a W370 blocker. |
| **#1215** | OPEN | Promote gf10/gf256 to bitexact — conformance track, orthogonal to IGLA. |

**Strategic implication:** W370 works on `trinity-rust-rings`. The main external risks are hardware availability (DLC10 cable/board) and the remaining gen-verilog defects on the wave-loop branch.

---

## 2. Weak Points & Research Synthesis

### Weak points ranked by W370 actionability

1. **Silicon evidence gap / board flash blocker** — `dlc10 idcode` still fails with `DLC10 cable not found (VID=0x03FD)`. The 3.6 MB `ternary_mac_demo_top.bit` from W361 is ready but unvalidated. Hardware-availability blocker.
2. **Remaining `gen-verilog` defects on `trinity-rust-rings`** — defects 1 (only first const emits), 3 (bare-if early return), 4 (`as`/bitwise body drop), and 5 (struct-field reg name mismatch) remain. W370 will pick one safe sub-fix (B1 recommended as highest impact).
3. **No automated yosys CI smoke gate** — defects 1/3/4/5 are not exercised by canonical specs; scratch repros are the only guard. A smoke gate is a W370 option (B5) but B1 is higher impact on emitted code.
4. **Proof-lattice / Lean build-time boundary** — `lake build Trinity.TernaryInference` was ~5.0 s at 45 variables. W370 probes 46-variable plus accumulation; fall back if `omega` saturates.
5. **RTL-to-Lean traceability gap** — still a roadmap item, too large for W370.
6. **Stale workspace artifacts** — old `.claude/plans/wave-loop-362.md` through 364 and `scripts/gen_w362_lean.py` remain untracked. Safe to leave or clean in a separate hygiene commit.

### Scientific-paper / competitor landscape (2025–2026)

- **Closest formal competitor:** Sparkle HDL / Verilean — Lean 4-native, 60+ BitNet theorems, 102 RV32IMA SoC theorems, but **no public generic ∀ ternary-MAC accumulation proofs** at 40+ variables or 20+ cancellation depth.
- **Implementation competitors:** TerEffic, TOM, TENET, TeLLMe v2, VitaLLM, ternfpga, TernaryCore, KU Leuven ternary-lut-dse — ternary LLM accelerators / open generators with silicon/FPGA metrics but **no theorem-prover verification**.
- **Updated references:** Wen et al. 2024 ternary MAC truth-table model; RSR/RSR++ efficient binary/ternary matmul; BSTCIM ternary in-MRAM CIM; KU Leuven open Chisel ternary-lut generator.
- **Formal verification benchmarks:** RTL-BenchLS (10k verified Verilog designs), NotSoTiny, RealBench, OpenLLM-RTL — future targets, not W370 blockers.
- **Moat:** t27 remains unchallenged in generic quantified ternary accumulation depth. W370 pushes this to 224 generic ∀.

---

## 3. Decomposed Implementation Plan

### Phase A — Preparation (Issue / Spec)
1. Create GitHub tracking issue #1259 (done).
2. Copy `scripts/gen_w369.py` → `scripts/gen_w370.py` and `scripts/gen_w369_lean.py` → `scripts/gen_w370_lean.py`.
3. In `gen_w370_lean.py`, keep the corrected `zero_weight_closure` helper (`total = before + 1 + after`) and set shape to 6 zero before + 1 plus + 7 zero after = 13 zero-weight MACs (14 variables).

### Phase B — Spec Extension (TDD)
4. Run `gen_w370.py` on all 27 IGLA specs.
   - Each spec gains a W370 block: +2 tests, +1 invariant.
   - Expected totals: ~12,695 tests, ~5,549 invariants.

### Phase C — Lean Proof Lattice (Code)
5. Run `gen_w370_lean.py` to append four new generic ∀ theorems:
   - `ternaryMacAccumulateFortySixPlusGeneric` — 46-variable plus accumulation (a..at).
   - `ternaryMacAccumulateFortyFiveMinusGeneric` — 45-variable minus accumulation.
   - `ternaryMacTresvigintupleCancellationGeneric` — depth-23 residual cancellation (`= mac(x,a,.plus)`).
   - `ternaryMacZeroWeightTredecupleClosureGeneric` — 6 zero before + 1 plus + 7 zero after (13 zero-weight MACs).
6. Build `Trinity.TernaryInference` with `lake build`. If 46-variable theorem times out, fall back to 45-variable plus / 44-variable minus and adjust targets.

### Phase D — Gen-Verilog Sub-fix (Code)
7. Implement **B1:** emit all `const` declarations, not only the first, in `bootstrap/src/compiler.rs` `gen_verilog_module`.
8. Add scratch spec `specs/scratch/w370_const_order.t27` with multiple `const` declarations and verify emitted Verilog with `yosys read_verilog`.

### Phase E — Seal & Conformance (Verify)
9. Regenerate all affected seals (27 IGLA + any non-IGLA specs whose Verilog output shifts).
10. Run full conformance: `./target/release/t27c suite --repo-root .`.
    - Gate: 548/548 PASS.

### Phase F — Board Flash (Verify)
11. Build `dlc10`: `cargo build --release -p dlc10`.
12. Run `./target/release/dlc10 idcode`.
    - If success: capture IDCODE and proceed to `sram`/`flash`.
    - If failure: document in `docs/reports/FPGA_EVIDENCE_W370.md`.

### Phase G — Reports & Cooperation (Synthesize)
13. Write `docs/reports/WAVE_LOOP_370_REPORT.md`.
14. Write `docs/reports/WAVE_LOOP_370_COOPERATION.md` with three W371 variants:
    - **Variant A:** Formal-only (safe) — 228 generic ∀ target.
    - **Variant B:** Formal + board flash retry + one gen-verilog sub-fix (recommended) — 228 generic ∀ + silicon evidence + backend hardening.
    - **Variant C:** Formal + RTL-to-Lean traceability prototype + board flash — higher risk, deeper moat.
15. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` to mark defect 1 fixed (if B1 succeeds).

### Phase H — Land & Learn
16. Stage all W370 files, commit with message containing `Closes #1259`.
17. Run final conformance after commit to confirm PASS.
18. Update `.trinity/experience.md` with W370 learnings.
19. Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-370.md` and update `MEMORY.md` index.

---

## 4. Risk Register

| Risk | Likelihood | Mitigation |
|---|---|---|
| 46-variable theorem times out | Medium | Fall back to 45-variable plus / 44-variable minus. |
| B1 sub-fix breaks seals | Low–Medium | Test on scratch spec first; run full suite before commit. |
| Board still missing | High | Document and move on; keep retry in W371. |
| Full conformance count drifts | Low | Reseal any mismatched spec immediately. |
| `trinity-rust-rings` drift from master | High | Do not merge master; keep changes narrow and branch-local. |

---

## 5. Success Criteria

- [ ] 224 generic ∀ theorems in `TernaryInference.lean`.
- [ ] 46-variable plus accumulation theorem builds (or documented fallback).
- [ ] Zero-weight tredecuple closure theorem is correctly 13 zeros + 1 plus.
- [ ] 27 IGLA specs extended, all 27 seals regenerated.
- [ ] Conformance suite: PASS (548/548).
- [ ] One safe `gen-verilog` sub-fix with scratch-spec test.
- [ ] Board flash attempted and documented.
- [ ] `WAVE_LOOP_370_REPORT.md` and `WAVE_LOOP_370_COOPERATION.md` written.
- [ ] Memory and experience updated.
- [ ] Commit on `trinity-rust-rings` with `Closes #1259`.
