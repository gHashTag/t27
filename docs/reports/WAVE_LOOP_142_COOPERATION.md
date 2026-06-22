# Cooperation Variants for Next IGLA CODER IGLA RACE Loop (Wave Loop 143)

## Variant 1: Targeted Hardness — Pool A Return

**Focus:** Rotate back to Pool A (`rtl`, `eda`, `cordic_fixed`, `bram_weights`, `cordic`, `cordic_top`, `formal`, `gemm`) and add 2 tests per spec, raising total to 570+ tests.

**Pros:**
- Fast average coverage increase per spec.
- Closes "blind zones" in RTL verification and hardware modules.

**Risks:**
- Possible cascading `seal` changes due to affected dependencies.

---

## Variant 2: Competitive Intelligence Surge — Ternary Accelerator Landscape

**Focus:** Instead of adding 16 tests, allocate resources to research and integrate 4–5 new competitors from the **high/extreme threat** sector, especially:
- LUT-based ternary accelerator generator (arXiv:2604.25183 — already in W142 but needs deep analysis)
- TerEffic (arXiv:2502.16473 — AMD Alveo U280, 16,300 tok/s)
- CORVET (arXiv:2602.19268 — mixed-precision vector engine)
- Any new arXiv 2607 papers on ternary/quantized inference

**Pros:**
- Strategic positioning refresh.
- More complete scientific landscape picture.

**Risks:**
- Possible `suite` runtime growth due to increased benchmark checks.

---

## Variant 3: Hybrid + Debt Payoff — FPGA Throughput Benchmarks

**Focus:** Add 8 tests (instead of 16) to the 4 weakest Pool A specs, and simultaneously close 1–2 open technical debts:
- Add real FPGA throughput benchmarks (TOPS/W, tok/s) to `systolic_array.t27` or `ternary_gemm.t27`
- Fix C-backend `f32` → `float` mapping for FFI Python bindings
- Implement LSP hover/completion stubs for `.t27` files

**Pros:**
- Balance between coverage growth and technical debt reduction.
- Infrastructure reliability improvement.

**Risks:**
- Smaller test increment per iteration.

---

**Recommendation:** For W143, **Variant 1** (Targeted Hardness) is recommended to maintain the Pool A/B alternation rhythm and coverage growth cadence, alternating with Variant 2 every 2–3 loops.
