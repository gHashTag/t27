# Wave Loop 531 Plan — Extend Icarus simulation regression suite

**Date:** 2026-07-07  
**Issue:** #1502  
**Branch:** `wave-loop-531`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point audit

After merging `wave-loop-530` into `wave-loop-531`, the Icarus simulation gate is
green on the W493–W529 regression suite (10 specs). A survey of all 39
`specs/scratch/*.t27` files found:

- 24 specs are rejected by `iverilog` or contain `UNSUPPORTED_ICARUS`.
- 17 specs compile with `iverilog` and run under `vvp`.
- Of those 17, only the 10 `w5*` witnesses are in the regression whitelist.
- 7 lowerable `w3xx` specs are **not** exercised: `w368_hex_width`, `w369_bin_width`,
  `w370_const_order`, `w371_early_return`, `w373_struct_field_keyword`,
  `w375_early_return`, `w376_cast_width`.
- Several `w3xx` specs are lowerable by `iverilog` but fail simulation because of
  broken 1-D local-array lowering in functions:
  - `w382_ram_lowering`: scalar `reg [31:0] mem` with bit-select `mem[0]`.
  - `w384_variable_index` / `w385_*` / `w386_*` / `w387_*` / `w388_*`: function-local
    arrays are emitted as scalar regs, so element access becomes bit-select and
    signed values are truncated.

The biggest semantic gap is therefore 1-D function-local packed arrays: the 2-D
scalar-struct packed-vector path (W527/W528) works, but primitive 1-D arrays in
functions still use the legacy scalar-reg fallback.

---

## 2. Scientific / engineering background

- **Icarus Verilog regression suite** — `ivtest` uses gold files and a Python
  runner to compare simulator output, which is the model for our JSON-baseline
  gate. ([Icarus regression docs](https://steveicarus.github.io/iverilog/developer/regression_tests.html))
- **cocotb** — Python-based testbench environment that drives simulators and
  compares DUT outputs against a golden reference model. It is the standard open
  pattern for reference-model checking. ([cocotb docs](https://docs.cocotb.org/en/stable/index.html))
- **AutoBench / CorrectBench** — LLM-generated Verilog testbenches with a
  Verilog driver + Python checker and functional self-correction. The
  reference-model pattern matches our `assert_eq` simulation baseline approach.
  ([AutoBench arXiv 2407.03891](https://arxiv.org/abs/2407.03891),
  [CorrectBench arXiv 2411.08510](https://arxiv.org/abs/2411.08510))
- **Vericert** — CompCert-based verified C-to-Verilog HLS with forward-simulation
  proofs. Justifies keeping the Icarus gate aligned with the Lean
  `Trinity.IcarusLowerable` value-preservation proofs. ([OOPSLA 2021](https://doi.org/10.1145/3485494))

---

## 3. Decomposed plan

### Variant A — Expand the Icarus regression suite and fix 1-D local arrays (recommended)

1. Extend `icarus_regression_specs()` in `bootstrap/src/suite.rs` to include all
   lowerable scratch specs that already pass `t27c icarus-simulate`
   (`w368`–`w376`).
2. Record JSON baselines for the new specs under `.trinity/icarus-baselines/`.
3. Fix function-local 1-D packed-array lowering in `bootstrap/src/compiler.rs`:
   - Detect 1-D primitive arrays inside functions.
   - Emit a signed/unsigned packed vector of the correct total width.
   - Initialize via packed concatenation (reverse order, matching W530 fix).
   - Read/write indexed elements via part-select `arr[idx * elem_w +: elem_w]`.
4. Add new scratch witnesses if existing ones do not cover signed read/write,
   variable index, and `for`-loop local arrays.
5. Reseal specs whose generated Verilog changes.
6. Run `./scripts/tri test --icarus-simulate --icarus-lowerable` until the
   regression suite is green.

### Variant B — Signed scalar-array fields in packed scalar structs

1. Extend `is_lowerable_scalar_struct()` to accept scalar-struct fields that are
   fixed-size signed/unsigned scalar arrays.
2. Update `packed_signed()` so scalar-struct packed vectors carry signedness per
   field where needed.
3. Emit sign-extending part-selects when reading signed array fields.
4. Add positive witnesses and Lean value-preservation theorems.
5. Reseal affected specs.

### Variant C — Adversarial lowerability boundary in Lean

1. Add negative scratch witnesses for non-lowerable constructs (imports, host-only
   helpers, casts, enum/string fields in packed arrays).
2. Prove `¬ isLowerable env m` for each negative witness in
  `Trinity.IcarusLowerable`.
3. Add a Rust integration test that checks the classifier rejects exactly the
   same specs.
4. Document the lowerability boundary.

---

## 4. Recommended variant

**Variant A** is recommended. It directly addresses the largest weak point found
by the audit (broken 1-D function-local arrays) while keeping the Icarus gate
executable and green. Variants B and C are valuable follow-ups for Wave Loops
532+.

---

## 5. Risks

- The 1-D packed-array fix may change generated Verilog for many specs and
  require resealing.
- Some `w3xx` specs rely on non-lowerable constructs (e.g. tuple destructuring)
  and must stay out of the regression whitelist.
- Signed part-select semantics differ between Icarus and Yosys; need to verify
  both gates stay green.

---

*φ² + φ⁻² = 3 | TRINITY*
