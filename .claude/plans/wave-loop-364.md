# Wave Loop 364 — Decomposed Plan

**Date:** 2026-07-01
**Issue:** #1249
**Branch:** `trinity-rust-rings`
**Target:** Extend IGLA CODER+RACE formal moat to **200 generic ∀**, 40-variable accumulation, retry board flash, probe gen-verilog weak point, update research/threat survey.

---

## 1. Scope

### 1.1 Formal wave (must)
- Append W364 blocks to all 27 core IGLA specs (`specs/igla/coder/*.t27`, `specs/igla/race/*.t27`): +54 tests, +27 invariants.
- Add 4 new generic ∀ theorems in `proofs/lean4/Trinity/TernaryInference.lean`:
  1. `ternaryMacAccumulateFortyPlusGeneric` — 40-variable plus accumulation.
  2. `ternaryMacAccumulateThirtyNineMinusGeneric` — 39-variable minus accumulation.
  3. `ternaryMacSeptendecupleCancellationGeneric` — depth-17 alternating plus/minus residual `mac(x, a, .plus)`.
  4. `ternaryMacZeroWeightSeptupleClosureGeneric` — seven zero-weight MACs around a plus-weight MAC are transparent.
- Regenerate 27 IGLA seals from repo root.
- Run `lake build Trinity.TernaryInference` and `./target/release/t27c suite --repo-root .` → 546/546 PASS.

### 1.2 Silicon retry (must attempt)
- Build `dlc10` (`cargo build --release -p dlc10`).
- Run `dlc10 idcode`; if board/cable present, run `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`.
- Document result in `docs/reports/FPGA_EVIDENCE_W364.md`.
- Do not let a hardware blocker fail the wave; ship formal deliverables and document blocker.

### 1.3 Weak-point research & fix (probe)
- Study #1245 (gen-verilog lowering defects). Identify which defects are small, safe fixes in the Rust backend.
- Attempt a narrow fix for the **literal formatting** defect (second item: `0b`/`0x` literals emitted verbatim). This is the most self-contained and high-impact defect for FPGA specs like `bpsk.t27` and `uart.t27`.
- If fix is too large for one wave, document findings and a recommended patch in the W364 report.

### 1.4 Research / threat survey update (must)
- Refresh competitive landscape: Sparkle HDL, ternfpga, TernaryCore, Balanced_Ternary, Trinity CLARA, CktFormalizer, etc.
- Add 2-3 recent papers/arXiv entries relevant to ternary/1.58-bit AI silicon or verified HDL compilation.
- Cite primary sources only; no overclaim.

### 1.5 Reporting (must)
- Write `docs/reports/WAVE_LOOP_364_REPORT.md`.
- Write `docs/reports/WAVE_LOOP_364_COOPERATION.md` with 3 variants for W365.
- Update `.trinity/experience.md` with W364 learnings.
- Update `~/.claude/projects/-Users-playra-t27/memory/MEMORY.md` and write `wave-loop-364.md`.

---

## 2. Order of execution

1. Create generator scripts `scripts/gen_w364.py` and `scripts/gen_w364_lean.py`.
2. Apply W364 blocks to specs and append Lean theorems.
3. Build `t27c` if needed and run conformance pre-check.
4. Regenerate seals.
5. Run full `t27c suite` and `lake build Trinity.TernaryInference`.
6. Attempt board flash and document.
7. Investigate gen-verilog backend defect; attempt narrow fix or document.
8. Write reports, cooperation variants, evidence doc.
9. Update memory and experience log.
10. Stage, commit with `Closes #1249`, push if appropriate.

---

## 3. Risks & mitigations

| Risk | Mitigation |
|------|------------|
| 40-variable `simp+omega` times out | Use `ternaryMac_eq_acc_plus_mul` + `omega`; if timeout, fall back to 39-variable plus and document boundary. |
| Depth-17 cancellation residual mismatch | Verify algebraically before building; odd depth leaves residual `mac(x,a,.plus)`. |
| Board still missing | Document and proceed; do not block wave. |
| gen-verilog fix breaks other backends | Run full suite after any backend change; scope fix narrowly. |
| Report grows too large | Keep concise; move deep research into dedicated sections. |

---

## 4. Definition of done

- [ ] 27 IGLA specs have W364 blocks.
- [ ] 4 new Lean theorems build and push generic ∀ to 200.
- [ ] 27 seals regenerated and suite is 546/546 PASS.
- [ ] Board flash attempted and documented.
- [ ] gen-verilog weak point investigated (fix or documented recommendation).
- [ ] W364 report + W365 cooperation variants written.
- [ ] Memory and experience log updated.
- [ ] Commit with `Closes #1249` landed.
