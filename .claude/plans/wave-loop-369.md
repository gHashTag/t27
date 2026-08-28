# Wave Loop 369 — IGLA CODER+RACE Plan

**Tracking issue:** #1257  
**Branch:** `trinity-rust-rings`  
**Date:** 2026-07-01  
**Target:** 220 generic ∀ theorems, 45-variable accumulation, board-flash retry, one safe `gen-verilog` sub-fix or CI smoke gate.

---

## 0. Context from W368 Close-out

W368 landed as commit `705b24128` on `trinity-rust-rings` with `Closes #1256`:
- 216 generic ∀ in `proofs/lean4/Trinity/TernaryInference.lean`
- 44-variable plus accumulation, 43-variable minus lattice
- Vigintiunuple (depth-21) residual cancellation
- Zero-weight undecuple closure (corrected to 10 zero-weight MACs around 1 plus-weight MAC)
- 7,780 tests / 2,991 invariants across 27 IGLA specs
- `gen-verilog` hex-width padding extended to scalar `var`, `let`, and `return` contexts via new `current_fn_return_type` codegen state
- Conformance: 547/547 PASS (546 canonical specs + 1 scratch regression spec)
- Board flash still blocked by missing DLC10 cable

**Key external finding:** `master` already has the full #1245 gen-verilog fix set (commit `701d79b3b`), but `trinity-rust-rings` has diverged and W369 will continue applying narrow, regression-free sub-fixes on the wave-loop branch.

---

## 1. Issue Triage (GitHub `gHashTag/t27`)

| Issue | Status | Relevance |
|---|---|---|
| **#1257** | OPEN | Canonical W369 tracking issue (created in this plan). |
| **#1256** | OPEN | W368 predecessor. Should remain open or be closed by W369 commit if workflow keeps wave issues live. |
| **#1254** | CLOSED | Reseal all specs + disambiguate duplicate module names. Baseline seal gate now clean. |
| **#1245** | CLOSED on `master` via #1250 | Full gen-verilog fix set exists on `master`; not merged into `trinity-rust-rings`. |
| **#1246 / #1242** | OPEN | Historical board-flash tracking issues. No standalone JTAG issue exists. |
| **#1243** | OPEN | BPSK modem PHY — optional parallel track, not a W369 blocker. |
| **#1219** | OPEN | Language roadmap EPIC — strategic context, not a W369 blocker. |
| **#1215** | OPEN | Promote gf10/gf256 to bitexact — conformance track, orthogonal to IGLA. |

**Strategic implication:** W369 works on `trinity-rust-rings`. The main external risks are hardware availability (DLC10 cable/board) and the remaining gen-verilog defects on the wave-loop branch.

---

## 2. Weak Points & Research Synthesis

### Weak points ranked by W369 actionability

1. **Silicon evidence gap / board flash blocker** — `dlc10 idcode` still fails with `DLC10 cable not found (VID=0x03FD)`. The 3.6 MB `ternary_mac_demo_top.bit` from W361 is ready but unvalidated. This is a hardware-availability blocker, not a code blocker.
2. **Remaining `gen-verilog` defects on `trinity-rust-rings`** — defects 1 (only first const emits), 3 (early return in bare if), 4 (as cast + bitwise drops body), and 5 (struct-field reg name mismatch) remain per `GEN_VERILOG_DEFECTS_REPRO.md`. Defect 2 (`0x` width) is only partially fixed. Full fix set already on `master`.
3. **No automated yosys/iverilog CI smoke gate for generated Verilog** — defects 1/3/4/5 are not exercised by canonical specs; only scratch repros exist. A CI smoke test would catch future regressions.
4. **Proof-lattice / Lean build-time boundary** — `lake build Trinity.TernaryInference` was 4.5 s at 44 variables. W369 probes 45-variable plus accumulation; fall back if `omega` saturates.
5. **RTL-to-Lean traceability gap** — no automated pipeline links generated Verilog back to the Lean 4 proof lattice. Too large for one wave; document roadmap issue.
6. **Stale workspace artifacts** — old `gen_w362_lean.py`, `.claude/plans/wave-loop-362.md` through 364, abandoned agent worktrees. Safe to clean up.

### Scientific-paper / competitor landscape

- **Closest formal competitor:** Sparkle HDL / Verilean — Lean 4-native, 60+ BitNet theorems, 102 RV32IMA SoC theorems, but **no public generic ∀ ternary-MAC accumulation proofs** at 40+ variables or 20+ cancellation depth.
- **Implementation competitors:** TerEffic, TOM, TENET, TeLLMe v2, VitaLLM, ternfpga, TernaryCore, KU Leuven ternary-lut-dse — all ternary LLM accelerators or open generators with silicon/FPGA metrics but **no theorem-prover verification**.
- **New/updated references:** Wen et al. 2024 (ternary MAC truth-table formal model), RSR/RSR++ efficient binary/ternary matmul, BSTCIM ternary in-MRAM CIM macro, KU Leuven open Chisel generator.
- **Benchmarks:** RTL-BenchLS (10k verified Verilog designs), NotSoTiny (Tiny Tapeout living benchmark), RealBench (real-world IP formal checker), OpenLLM-RTL — useful future targets, not W369 blockers.
- **Moat:** t27 remains unchallenged in generic quantified ternary accumulation depth. W369 pushes this to 220 generic ∀.

---

## 3. Decomposed Implementation Plan

### Phase A — Preparation (Issue / Spec)
1. Ensure `scripts/gen_w369.py` and `scripts/gen_w369_lean.py` are created from the W368 generator pattern.
2. In `gen_w369_lean.py`, keep the corrected `zero_weight_closure` helper (`total = before + 1 + after`) from W368.
3. Update W369 cooperation doc references from #1256 → #1257 in generated text.

### Phase B — Spec Extension (TDD)
4. Run `gen_w369.py` on all 27 IGLA specs under `specs/igla/coder/` and `specs/igla/race/`.
   - Each spec gains a W369 block: +2 tests, +1 invariant.
   - Expected totals: ~7,834 tests, ~3,018 invariants.

### Phase C — Lean Proof Lattice (Code)
5. Run `gen_w369_lean.py` to append four new generic ∀ theorems to `proofs/lean4/Trinity/TernaryInference.lean`:
   - `ternaryMacAccumulateFortyFivePlusGeneric` — 45-variable plus accumulation.
   - `ternaryMacAccumulateFortyFourMinusGeneric` — 44-variable minus accumulation lattice.
   - `ternaryMacDuovigintupleCancellationGeneric` — depth-22 identity cancellation (even depth → identity).
   - `ternaryMacZeroWeightDuodecupleClosureGeneric` — 6 zero-weight MACs before and 6 zero-weight MACs after a plus-weight MAC (12 zero + 1 plus = 13 variables).
6. Build `Trinity.TernaryInference` with `lake build` and measure time. If 45-variable theorem times out, fall back to 44-variable plus + 43-variable minus and adjust targets.

### Phase D — Gen-Verilog Sub-fix or CI Smoke Gate (Code)
7. Choose **one** of the following safe deliverables:
   - **Option D1:** Extend binary `0b` literal width padding to scalar `const`/`var`/`let`/`return` contexts, mirroring the W368 `0x` fix.
   - **Option D2:** Add a `yosys read_verilog` CI smoke gate that runs `t27c gen-verilog` on `specs/scratch/w369_hex_width.t27` (or another scratch spec) and fails on parse/elaboration errors.
   - **Option D3:** If neither is safe, write `docs/reports/GEN_VERILOG_BACKEND_ROADMAP.md` documenting the remaining defects and safe triage order.
8. Use a scratch spec under `specs/scratch/` to verify the chosen path and ensure no new `// TODO: implement` markers are emitted.

### Phase E — Seal & Conformance (Verify)
9. Regenerate all affected seals (27 IGLA + any non-IGLA specs whose Verilog output shifts).
10. Run full conformance: `target/release/t27c suite --repo-root .`.
    - Gate: 547/547 PASS (or 546/546 if scratch spec is removed).

### Phase F — Board Flash (Verify)
11. Build `dlc10`: `cargo build --release -p dlc10`.
12. Run `./target/release/dlc10 idcode` on QMTech Wukong V1.
    - If success: capture IDCODE and proceed to `sram` / `flash` with existing `ternary_mac_demo_top.bit`.
    - If failure: document in `docs/reports/FPGA_EVIDENCE_W369.md` and keep hardware issue open.

### Phase G — Reports & Cooperation (Synthesize)
13. Write `docs/reports/WAVE_LOOP_369_REPORT.md`.
14. Write `docs/reports/WAVE_LOOP_369_COOPERATION.md` with three W370 variants:
    - **Variant A:** Formal-only (safe, no board dependency) — 224 generic ∀ target.
    - **Variant B:** Formal + board flash retry + one gen-verilog sub-fix (recommended) — 224 generic ∀ + silicon evidence + backend hardening.
    - **Variant C:** Formal + RTL-to-Lean traceability prototype + board flash — higher risk, deeper moat.
15. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` or create `GEN_VERILOG_BACKEND_ROADMAP.md` for remaining defects.

### Phase H — Land & Learn
16. Stage all W369 files, commit with message containing `Closes #1257`.
17. Run final conformance after commit to confirm PASS.
18. Update `.trinity/experience.md` with W369 learnings.
19. Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-369.md` and update `MEMORY.md` index.

---

## 4. Risk Register

| Risk | Likelihood | Mitigation |
|---|---|---|
| 45-variable theorem times out | Medium | Fall back to 44-variable plus / 43-variable minus; keep cancellation/closure targets. |
| Gen-verilog sub-fix breaks seals | Low | Test on scratch spec first; run full suite before commit. |
| Board still missing | High | Document and move on; keep retry in W370. |
| Full conformance count drifts | Low | Reseal any mismatched spec immediately. |
| `trinity-rust-rings` drift from master | High | Do not merge master; keep changes narrow and branch-local. |

---

## 5. Success Criteria

- [ ] 220 generic ∀ theorems in `TernaryInference.lean`.
- [ ] 45-variable plus accumulation theorem builds.
- [ ] Zero-weight duodecuple closure theorem is correctly 12 zeros + 1 plus.
- [ ] 27 IGLA specs extended, all 27 seals regenerated.
- [ ] Conformance suite: PASS (547/547 or 546/546).
- [ ] One safe `gen-verilog` sub-fix or CI smoke gate with scratch-spec test.
- [ ] Board flash attempted and documented.
- [ ] `WAVE_LOOP_369_REPORT.md` and `WAVE_LOOP_369_COOPERATION.md` written.
- [ ] Memory and experience updated.
- [ ] Commit on `trinity-rust-rings` with `Closes #1257`.
