# Wave Loop 545 — primitive scalar array function returns for independent VCD cross-check

**Issue:** #1516 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-545`  
**Source:** `docs/reports/FPGA_LOOP_COOPERATION_W545_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Remove the W544 classifier boundary and lower functions that return fixed-size
primitive scalar arrays (e.g. `[3]u8`) into module-level `const`/`var`
initializers.  This makes a broad class of small-array helpers usable in the
Icarus-lowerable subset and completes the call-initializer shape matrix.

## Scope

1. Extend the Verilog backend in `bootstrap/src/compiler.rs` so that a function
   returning `[N]T` (primitive scalar `T`) emits a packed vector result and the
   caller's module `const`/`var` receives the full concatenation correctly.
2. Remove the W544 rule in `Compiler::ast_is_icarus_lowerable` that rejects
   primitive scalar array function return types.
3. Update `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` so that
   `Function.isLowerable` accepts primitive scalar array returns again.
4. Convert `specs/scratch/w544_negative_call_init_returns_array.t27` to a
   positive witness (`w545_call_init_returns_array.t27`) and add
   lowerability/sequential/value-preservation theorems in
   `Lemmas.lean` / `Soundness.lean`.
5. Reseal affected specs, record Icarus baselines, and run the full validation
   matrix.

## Acceptance criteria

- The converted primitive scalar array return witness passes Icarus simulation
  and the cocotb reference-model cross-check.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  0 cocotb failures, 0 Icarus failures, 0 seal mismatches.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
  `cargo test -p t27c --test icarus_lowerable`, and
  `lake build Trinity.IcarusLowerable.Soundness` remain green / 0 `sorry`.
- The 24 pre-existing yosys smoke baseline failures remain unchanged.

---

*Next: Wave Loop 546 cooperation variants will be proposed in the W545 closeout.*
