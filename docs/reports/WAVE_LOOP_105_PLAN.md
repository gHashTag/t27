# Wave Loop 105 Plan
## Synthetic Dataset + Benchmark Baseline + New Competitor Documentation

**Date:** 2026-06-17
**Target:** 562/562 PASS, 0 clippy, L3 clean
**Focus:** Close dataset scale gap; establish honest benchmark baseline; document EXTREME new competitors.

---

## Phase 1: OBSERVE (Complete)

- **Suite:** 562/562 PASS, 0 failures
- **Lean 4:** 8567/8567 PASS
- **Open issues:** 5 (all IGLA roadmap #1037-#1041)
- **New competitors discovered:**
  - **CHIPCRAFTBRAIN** (arXiv:2604.19856) — 98.7% pass@1 VerilogEval-Human, 6-agent PPO orchestration, **EXTREME threat**
  - **VeriGraphi** (arXiv:2604.14550v2) — hierarchical RTL via Knowledge Graph, RISC-V 32I generation, **HIGH threat**
  - **SK_EFT_Hawking** (GitHub, June 2026) — Lean 4, 9944 theorems, Standard Model fingerprints, **HIGH threat**
  - **Krippendorf & Tooby-Smith** (arXiv:2603.28406) — SU(5) GUT in Lean 4, anomaly cancellation proofs

---

## Track A: Hierarchical Template Composition (dataset.t27)

**Problem:** 12 flat templates produce ~400 samples. Real designs are hierarchical.
**Solution:**
- Add `compose_templates(tmpl_a, tmpl_b)` — concatenate two RTL modules with wrapper.
- Add `generate_uart_rx()` — counter + shift_register composition.
- Add `generate_alu_slice()` — adder + divider + fsm composition.
- Add `generate_memory_controller()` — counter + fsm composition.
- Add tests for composed modules.

---

## Track B: Honest Benchmark Baseline (new file: bench/verilog_eval_proxy.t27)

**Problem:** Trinity has no Pass@K numbers. Competitors publish 0.857+.
**Solution:**
- Create proxy benchmark spec with 20 VerilogEval-style problems (simple combinational + sequential).
- Define `evaluate_template_on_benchmark(template, problem)` — returns pass/fail.
- Compute Pass@1 proxy metric for current templates.
- Document result honestly (likely low; sets baseline for improvement).

---

## Track C: Competitor Documentation (COMPETITIVE_POSITIONING.md)

**Problem:** 3 new EXTREME/HIGH competitors not yet documented.
**Solution:**
- Add CHIPCRAFTBRAIN section with 98.7% pass@1 and differentiation matrix.
- Add VeriGraphi section with Knowledge Graph approach.
- Add SK_EFT_Hawking section with 9944 theorems and Lean 4 positioning.
- Add Krippendorf & Tooby-Smith SU(5) section.

---

## Verification Checklist

- [ ] cargo build --release (0 errors)
- [ ] ./target/release/t27c suite --repo-root . (562/562 PASS)
- [ ] cargo clippy --workspace --all-features (0 warnings)
- [ ] ./target/release/t27c lint --ascii for all modified specs (clean)
- [ ] Regenerate seals for all modified specs
- [ ] Write WAVE_LOOP_105_REPORT.md
- [ ] Write WAVE_LOOP_105_COOPERATION.md
- [ ] Increment .commit_count

---

phi^2 + 1/phi^2 = 3 | TRINITY
