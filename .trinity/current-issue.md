# Wave Loop 548 — multi-dimensional primitive scalar array function returns for independent VCD cross-check

**Issue:** #1519 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-548`  
**Source:** `docs/reports/FPGA_LOOP_COOPERATION_W548_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend Wave Loop 547's signed primitive scalar array function-return support to
multi-dimensional arrays (e.g. `[2][3]u8`).  This makes the feature usable for
small matrix helpers in the Icarus-lowerable subset.

## Scope

1. Extend the Verilog backend in `bootstrap/src/compiler.rs` so that a function
   returning `[N][M]T` emits a packed vector of total width `N*M*W` and the
   caller's local/variable receives the full concatenation.
2. Extend `try_emit_primitive_array_access` to compute multi-dimensional
   bit-slices for packed locals.
3. Add positive scratch witnesses for 2-D primitive array return initializers and
   element reads (e.g. `let m : [2][3]u8 = grid(); assert_eq(m[1][2], 42)`).
4. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
5. Reseal affected specs, record Icarus baselines, and run the full validation
   matrix.

## Acceptance criteria

- The new 2-D primitive scalar array return witnesses pass Icarus simulation
  and the cocotb reference-model cross-check.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  0 cocotb failures, 0 Icarus failures, 0 seal mismatches.
- `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
  `cargo test -p t27c --test icarus_lowerable`, and
  `lake build Trinity.IcarusLowerable.Soundness` remain green / 0 `sorry`.
- The 24 pre-existing yosys smoke baseline failures remain unchanged.

---

*Next: Wave Loop 549 cooperation variants will be proposed in the W548 closeout.*
