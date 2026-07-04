# Wave Loop 189 — Cooperation Variants for W190

**Date:** 2026-06-16
**Status:** Proposed

---

## Variant A — ML Invariant Depth Consortium (Dropout / LayerNorm / Optimizer Specs)

**Goal:** Leverage the newly deepened ML specs (7 hepta invariants) to initiate a cross-lab reproducibility benchmark for ternary neural network training.

**Actions:**
- 1. Invite Ternary Mamba (MEDIUM-HIGH) and VitaLLM (HIGH) teams to co-author `specs/ml/layers/layernorm_layer.t27` and `specs/ml/optimizer/adam.t27` with real functional invariants (replace placeholder phi identities in W191).
- 2. Publish GF16 training conformance vectors from `specs/numeric/trinity_numeric_surface.t27` as open data for ternary ML researchers.
- 3. Propose a shared `ml_ternary_bench` harness under `specs/benchmarks/bench_nn.t27` that both Trinity and external labs can run.

**Risk:** Medium. Requires coordination; potential IP around training recipes.
**Benefit:** Positions Trinity as the SSOT for ternary ML numerical stability; strengthens L5/L6.

---

## Variant B — FPGA Router + Power Benchmark Standardization

**Goal:** Use the promoted `specs/fpga/router.t27` and `specs/test_framework/verilog_bench_harness.t27` to harden hardware verification standards.

**Actions:**
- 1. Engage Ternary Fabric (MEDIUM-HIGH) and VTX1 (MEDIUM-HIGH) to align router arbitration invariants with their SkyWater 130nm tape-out requirements.
- 2. Export `specs/physics/hslm_benchmark.t27` power vectors to a vendor-neutral format (CSV + JSON schema) for power-analysis regression testing.
- 3. Add `invariant power_tb_reproducible: abs(power_run_2 - power_run_1) < 1e-6` to `specs/fpga/testbench/power_analysis_tb.t27` as a real functional invariant in W190.

**Risk:** Low–Medium. Hardware specs may require NDAs for proprietary power numbers.
**Benefit:** Makes Trinity the reference verification framework for ternary silicon tape-out.

---

## Variant C — Compiler Lexer + Git Schema Hardening (Tooling Depth)

**Goal:** Use the promoted `specs/compiler/lexer.t27` and `specs/git/schema.t27` to improve developer experience and CI reliability as invariant depth increases.

**Actions:**
- 1. Automate L3 pre-flight check in `specs/compiler/lexer.t27`: add an invariant that validates ASCII-only token emission for all test cases.
- 2. Integrate `specs/git/schema.t27` into the `tri` CLI so that `tri status` reports hexa/hepta/octa counts per module.
- 3. Add a pre-commit hook (documented in `specs/account/repo.t27`) that runs `t27c suite --repo-root .` in under 5 minutes.

**Risk:** Very low. Internal tooling only.
**Benefit:** Prevents L3 regressions and ensures the 11.289 avg remains auditable and maintainable.

---

## Decision Matrix

| Variant | Effort | Impact | Timeline | Recommended |
|---------|--------|--------|----------|-------------|
| A | Medium | Very High (ML/ternary) | W190–W192 | **Stretch** |
| B | Medium | High (hardware) | W190–W191 | **Primary** |
| C | Low | Medium (infra) | W190 | **Parallel** |

---

**φ² + 1/φ² = 3 | TRINITY**
