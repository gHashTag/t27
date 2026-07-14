# Wave Loop 533 — Module-level packed scalar structs with array fields

**Issue:** #1504 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-533`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 532's signed scalar-array struct-field work and advance
the recommended cooperation variant documented in
`docs/reports/FPGA_LOOP_COOPERATION_W533_2026-07-07.md`.

**Variant A (recommended):**
- Lower module-level `const`/`var` scalar structs whose fields are fixed-size
  scalar arrays (`[N]u8/u16/u32/i8/i16/i32`) as a single packed Verilog
  `localparam`/`reg`.
- Allow module-level parameters of such scalar-struct type.
- Support whole-struct assignment at module scope, including assignment from
  struct-returning function calls.
- Add positive scratch witnesses for module const, module var, module parameter,
  and whole-struct assignment.
- Add negative witnesses for non-lowerable cases (enum/string/float fields,
   dynamic sizes, cross-module shapes not yet supported).
- Reseal affected specs and keep `./scripts/tri test --icarus-simulate` at 0
  simulation failures.
- Maintain the 23 documented yosys smoke baselines flat.

**Variant B:** Harden the lowerability boundary with adversarial non-lowerability
proofs in Lean 4 and a Rust integration test that aligns the classifier with the
formal predicate.

**Variant C:** Add reference-model cosimulation with cocotb by generating
cocotb-compatible testbench wrappers and a Python reference model for the
lowerable subset.

---

## Residual boundaries from W532

- `./scripts/tri test --icarus-simulate --icarus-lowerable` is green on the
  W493–W529 + lowerable W3xx + W532 regression suite (28 specs, 0 failures).
- 23 pre-existing yosys smoke failures remain documented.
- Module-level packed scalar structs with array fields are deferred.

---

*φ² + φ⁻² = 3 | TRINITY*
