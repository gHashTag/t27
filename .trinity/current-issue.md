# Wave Loop 546 — function-local primitive scalar array return initializers and reassignments for independent VCD cross-check

**Issue:** #1517 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-546`  
**Source:** `docs/reports/FPGA_LOOP_COOPERATION_W546_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend Wave Loop 545's primitive scalar array function-return support from
module-level `const`/`var` initializers into function-local `let` bindings and
assignments.  This completes the primitive scalar array return shape matrix
and makes small array-returning helpers usable inside functions in the
Icarus-lowerable subset.

## Scope

1. Extend the Verilog backend in `bootstrap/src/compiler.rs` so that a function
   returning `[N]T` (primitive scalar `T`) can initialize a function-local
   `let` binding or be assigned to a local packed primitive array.
2. Add positive scratch witnesses for:
   - function-local `let a : [3]u8 = seq();` used in assertions,
   - reassignment `a = seq();` after an earlier initializer.
3. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
4. Reseal affected specs, record Icarus baselines, and run the full validation
   matrix.

## Acceptance criteria

- The new function-local primitive scalar array return witnesses pass Icarus
  simulation and the cocotb reference-model cross-check.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  0 cocotb failures, 0 Icarus failures, 0 seal mismatches.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
  `cargo test -p t27c --test icarus_lowerable`, and
  `lake build Trinity.IcarusLowerable.Soundness` remain green / 0 `sorry`.
- The 24 pre-existing yosys smoke baseline failures remain unchanged.

---

*Next: Wave Loop 547 cooperation variants will be proposed in the W546 closeout.*
