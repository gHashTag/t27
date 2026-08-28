# Wave Loop 368 — IGLA CODER+RACE Plan

**Tracking issue:** #1256  
**Branch:** `trinity-rust-rings`  
**Date:** 2026-07-01  
**Target:** 216 generic ∀ theorems, 44-variable accumulation, board-flash retry, one safe `gen-verilog` sub-fix.

---

## 0. Context from W367 Close-out

W367 landed as commit `1f3897b29` on `trinity-rust-rings` with `Closes #1253`:
- 212 generic ∀ in `proofs/lean4/Trinity/TernaryInference.lean`
- 43-variable plus accumulation, 42-variable minus lattice
- Vigintuple (depth-20) cancellation
- Zero-weight decuple closure
- 7,934 tests / 2,977 invariants across 27 IGLA specs
- Safe `0x` hex-width padding sub-fix in `bootstrap/src/compiler.rs`
- Conformance: 546/546 PASS

**Important finding during W368 planning:** the `zero_weight_closure` helper in `scripts/gen_w367_lean.py` computes `total = before + after` but the plus-weight activation itself is not counted. As a result, the W367 "decuple closure" theorem actually contains 9 zero-weight MACs (5 before + 4 after) rather than the advertised 10. The theorem is still true and type-checks, but the depth label is off by one. W368 will correct the helper and produce a true zero-weight undecuple closure.

---

## 1. Issue Triage (GitHub `gHashTag/t27`)

| Issue | Status | Relevance |
|---|---|---|
| **#1256** | OPEN | Canonical W368 tracking issue. Contains target 216 generic ∀, board-flash retry, one gen-verilog sub-fix. |
| **#1254** | OPEN | Reseal all specs + disambiguate duplicate module names. Must complete before the 546/546 seal gate. |
| **#1245** | CLOSED on `master` via #1250 | `iverilog-clean gen-verilog` already fixed on `master` (commit `701d79b3b`). The fix is **not** on `trinity-rust-rings`. W368 should therefore land a fresh, narrow `gen-verilog` sub-fix directly on `trinity-rust-rings`, not try to merge the whole `master` refactor. |
| **#1246 / #1242** | OPEN | Historical board-flash tracking issues. No standalone JTAG issue exists. |
| **#1243** | OPEN | BPSK modem PHY — optional parallel track, not a W368 blocker. |

**Strategic implication:** Because `master` has a much diverged history, W368 will keep working on `trinity-rust-rings` and apply small, regression-free improvements there.

---

## 2. Weak Points & Research Synthesis

### Weak points ranked by W368 actionability

1. **Gen-Verilog backend (trinity-rust-rings branch)** — remaining gaps from `GEN_VERILOG_DEFECTS_REPRO.md`: array-LUT aggregate lowering, `TODO` fallback debt, generated-Verilog lacks `yosys`/`iverilog` CI gate. Since #1245 is closed on master, W368 picks the next safe sub-fix: **extend `0x`/`0b` width padding to non-const scalar expressions** (narrow, no seal break observed in W367).
2. **Board flash / silicon evidence** — `dlc10 idcode` still fails with missing cable. Retry each wave; document if still blocked.
3. **Proof-lattice build time** — `lake build Trinity.TernaryInference` grew from 3.1 s (W360, 36 vars) to 4.4 s (W367, 43 vars). Probe 44-variable plus accumulation first; fall back if `omega` saturates.
4. **RTL-to-Lean traceability** — no automated link between proven specs and emitted Verilog. Too large for one wave; document roadmap.
5. **Stale untracked artifacts** — old `gen_w362_lean.py`, `.claude/plans/wave-loop-362.md` through 364. Clean up or archive.

### Scientific-paper / competitor landscape

- **Closest formal competitor:** Sparkle HDL / Verilean — Lean 4-native, 60+ BitNet theorems, 102 RV32IMA SoC theorems, but **no public generic ∀ ternary-MAC accumulation proofs** at 40+ variables or 20+ cancellation depth.
- **Implementation competitors:** TerEffic, TOM, TENET, TeLLMe v2, VitaLLM, ternfpga, TernaryCore — all ternary LLM accelerators with silicon/FPGA metrics but **no theorem-prover verification**.
- **Benchmarks:** RTL-BenchLS, RealBench, ChipBench, ArchXBench — useful reference for future LLM-for-RTL work; not blockers for W368.
- **Moat:** t27 remains unchallenged in generic quantified ternary accumulation depth. W368 pushes this to 216 generic ∀.

---

## 3. Decomposed Implementation Plan

### Phase A — Preparation (Issue / Spec)
1. Create `scripts/gen_w368.py` from `scripts/gen_w367.py` pattern.
2. Create `scripts/gen_w368_lean.py` from `scripts/gen_w367_lean.py`, **fixing `zero_weight_closure` helper:**
   - Change `total = before + after` to `total = before + 1 + after`.
   - This makes a true undecuple closure: `before=5, after=5` ⇒ 10 zero-weight MACs around 1 plus-weight MAC (11 variables).
3. Update W368 cooperation doc reference from #1253 → #1256 in generated text.

### Phase B — Spec Extension (TDD)
4. Run `gen_w368.py` on all 27 IGLA specs under `specs/igla/coder/` and `specs/igla/race/`.
   - Each spec gains a W368 block: +2 tests, +1 invariant.
   - Expected totals: ~7,988 tests, ~3,004 invariants.

### Phase C — Lean Proof Lattice (Code)
5. Run `gen_w368_lean.py` to append four new generic ∀ theorems to `proofs/lean4/Trinity/TernaryInference.lean`:
   - `ternaryMacAccumulateFortyFourPlusGeneric` — 44-variable plus accumulation.
   - `ternaryMacAccumulateFortyThreeMinusGeneric` — 43-variable minus accumulation lattice.
   - `ternaryMacVigintiunupleCancellationGeneric` — depth-21 identity cancellation.
   - `ternaryMacZeroWeightUndecupleClosureGeneric` — corrected 10 zero-weight MACs around a plus-weight MAC.
6. Build `Trinity.TernaryInference` with `lake build` and measure time. If 44-variable theorem times out, fall back to 43-variable plus + 42-variable minus and adjust targets.

### Phase D — Gen-Verilog Sub-fix (Code)
7. Land one safe sub-fix in `bootstrap/src/compiler.rs`: extend the W367 `0x` width-padding logic to non-const scalar expressions where a declared type width is known (e.g., assignments, return values, explicit casts). Use a scratch spec under `specs/scratch/` to verify the emitted Verilog contains no new `// TODO` markers and passes `yosys read_verilog` syntax check.

### Phase E — Seal & Conformance (Verify)
8. Regenerate all 27 IGLA seals and address #1254 duplicate-module-name disambiguation if needed.
9. Run full conformance: `target/release/t27c suite --repo-root .`.
   - Gate: 546/546 PASS.

### Phase F — Board Flash (Verify)
10. Build `dlc10`: `cargo build --release -p dlc10`.
11. Run `./target/release/dlc10 idcode` on QMTech Wukong V1.
    - If success: capture IDCODE and proceed to `sram` / `flash` with existing `ternary_mac_demo_top.bit`.
    - If failure: document in `docs/reports/FPGA_EVIDENCE_W368.md` and keep hardware issue open.

### Phase G — Reports & Cooperation (Synthesize)
12. Write `docs/reports/WAVE_LOOP_368_REPORT.md`.
13. Write `docs/reports/WAVE_LOOP_368_COOPERATION.md` with three W369 variants:
    - **Variant A:** Formal-only (safe, no board dependency) — 220 generic ∀ target.
    - **Variant B:** Formal + board flash retry + one gen-verilog sub-fix (recommended) — 220 generic ∀ + silicon evidence + backend hardening.
    - **Variant C:** Formal + RTL-to-Lean traceability prototype + board flash — higher risk, deeper moat.
14. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` or create `GEN_VERILOG_BACKEND_ROADMAP.md` for remaining defects.

### Phase H — Land & Learn
15. Stage all W368 files, commit with message containing `Closes #1256`.
16. Run final conformance after commit to confirm 546/546 PASS.
17. Update `.trinity/experience.md` with W368 learnings.
18. Save memory file `~/.claude/projects/-Users-playra-t27/memory/wave-loop-368.md` and update `MEMORY.md` index.

---

## 4. Risk Register

| Risk | Likelihood | Mitigation |
|---|---|---|
| 44-variable theorem times out | Medium | Fall back to 43-variable plus / 42-variable minus; keep cancellation/closure targets. |
| Gen-verilog sub-fix breaks seals | Low | Test on scratch spec first; run full suite before commit. |
| Board still missing | High | Document and move on; keep retry in W369. |
| #1254 reseal blocker | Medium | Resolve duplicate module names before final seal gate. |
| `trinity-rust-rings` drift from master | High | Do not merge master; keep changes narrow and branch-local. |

---

## 5. Success Criteria

- [ ] 216 generic ∀ theorems in `TernaryInference.lean`.
- [ ] 44-variable plus accumulation theorem builds.
- [ ] Zero-weight undecuple closure theorem is **correctly** 10 zeros + 1 plus.
- [ ] 27 IGLA specs extended, all 27 seals regenerated.
- [ ] Conformance suite: 546/546 PASS.
- [ ] One safe `gen-verilog` sub-fix with scratch-spec test.
- [ ] Board flash attempted and documented.
- [ ] `WAVE_LOOP_368_REPORT.md` and `WAVE_LOOP_368_COOPERATION.md` written.
- [ ] Memory and experience updated.
- [ ] Commit on `trinity-rust-rings` with `Closes #1256`.
