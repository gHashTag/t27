# Wave Loop 547 — signed primitive scalar array function returns for independent VCD cross-check

**Issue:** #1518 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-547`  
**Source:** `docs/reports/FPGA_LOOP_COOPERATION_W547_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend Wave Loop 546's primitive scalar array function-return support to signed
element types (e.g. `[3]i8`).  This makes the feature usable for DSP-style
fixed-point helpers in the Icarus-lowerable subset.

## Scope

1. Extend the Verilog backend in `bootstrap/src/compiler.rs` so that a function
   returning `[N]i8`/`[N]i16`/`[N]i32` emits a signed packed vector and the
   caller's local/variable receives the full concatenation with correct sign
   extension.
2. Add positive scratch witnesses for:
   - signed array return initializers (`let a : [3]i8 = seq();`),
   - signed element comparison/assertion (`assert_eq(a[0], -1)`).
3. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
4. Reseal affected specs, record Icarus baselines, and run the full validation
   matrix.

## Acceptance criteria

- The new signed primitive scalar array return witnesses pass Icarus simulation
  and the cocotb reference-model cross-check.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  0 cocotb failures, 0 Icarus failures, 0 seal mismatches.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
  `cargo test -p t27c --test icarus_lowerable`, and
  `lake build Trinity.IcarusLowerable.Soundness` remain green / 0 `sorry`.
- The 24 pre-existing yosys smoke baseline failures remain unchanged.

---

*Next: Wave Loop 548 cooperation variants will be proposed in the W547 closeout.*
