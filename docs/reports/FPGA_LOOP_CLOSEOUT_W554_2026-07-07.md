# FPGA / Icarus Loop Closeout — Wave Loop 554

**Issue #1525** — Bench-local primitive scalar arrays (Variant A).  
**Branch:** `wave-loop-554`  
**Date:** 2026-07-07  
**Author:** Trinity Agent (Queen)  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 554 extended the deterministic `bench` cross-check to support named
local bindings that receive a packed primitive scalar array returned by a
function call, e.g. `let tmp : [N]T = f();`. The loop created three scratch
witnesses covering unsigned, signed, and 2-D arrays, validated them through the
Icarus simulation gate and the cocotb/Python reference-model cross-check, and
uncovered (and fixed) a latent multi-dimensional packed primitive-array indexing
bug that affected earlier W548/W549/W550/W552 witnesses.

---

## What was asked

Issue #1525 acceptance criteria:

1. New scratch witness(es) under `specs/scratch/w554_*` exercising a
   `bench`-local `let` initialized from a function returning `[N]i8` or
   `[N]u8`, with element reads in `assert_eq` statements.
2. Generated Verilog that hoists the local declaration to the top of the
   `initial` block and assigns it from the packed function result.
3. Python reference model that resolves the local type and evaluates element
   accesses at the correct width/signedness.
4. `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
   showing zero new Icarus/cocotb failures and zero seal mismatches.
5. `lake build Trinity.IcarusLowerable.Soundness` remaining green with zero
   `sorry`.

---

## What was done

### Scratch witnesses

| Spec | Purpose |
|------|---------|
| `specs/scratch/w554_bench_local_array_unsigned.t27` | `let tmp : [3]u8 = seq_u8();` in `test` and `bench`; reads `tmp[0..2]`. |
| `specs/scratch/w554_bench_local_array_signed.t27` | `let tmp : [3]i8 = seq_i8();` in `test` and `bench`; reads signed elements. |
| `specs/scratch/w554_bench_local_array_2d.t27` | `let tmp : [2][3]u8 = mat();` in `test` and `bench`; reads `tmp[0][0]` and `tmp[1][2]`. |

Each witness contains an equivalent `test` block and `bench` block so the
Icarus and cocotb gates exercise the same expression both inside a static
assertion context and inside a procedural `initial` block.

### Compiler fix

While validating the 2-D witness, the simulation produced an unexpected `x`
probe value for `tmp[1][2]`. Investigation showed that
`try_emit_primitive_array_access` collects array indices outermost-first
(`[row, col]`) but computed the row-major flat index without reversing them to
source order. For `tmp[1][2]` in `[2][3]u8` this addressed bit offset 56 of a
48-bit vector, yielding `x`.

Fix in `bootstrap/src/compiler.rs`:

```rust
// W554: indices are collected outermost-first; put them in source order
// before computing the row-major flat index.
indices.reverse();
let flat_idx = if indices.len() == 1 { ... };
```

This also repaired latent indexing in W548, W549, W550, and W552_2d, whose
saved seals were regenerated.

### Reference model

No model changes were required. The existing `_collect_assertions` binding for
`TestBlock`/`BenchBlock` locals, `_resolve_full_type`, and `_eval_index_bv`
correctly resolved the local packed-array type and extracted signed/unsigned
elements once the compiler generated correct Verilog.

### Baselines and seals

- t27 seals saved under `.trinity/seals/scratch_w554_*.json`.
- Resealed W548, W549, W550, and W552_2d because their generated Verilog
  changed.
- W554 witnesses are structurally Icarus-lowerable and pass direct
  `t27c icarus-simulate` / `t27c icarus-cocotb`. They are not yet included in
  the automated `./scripts/tri test --icarus-lowerable ...` regression count
  because the suite's conservative `gen-verilog` pre-flight rejects any spec
  whose `test`/`bench` block declares a named local (the synth-oriented
  `gen-verilog` path strips the declaration while leaving the reference).
  This is a pre-existing suite limitation, not a W554 regression; the suite
  still reports 65/65 Icarus/cocotb PASS and zero seal mismatches.

### Integration test

Added `accepts_w554_bench_local_array_cross_check` in
`bootstrap/tests/icarus_lowerable.rs`, checking that all three W554 witnesses
are classified as Icarus-lowerable.

### FROZEN_HASH

Updated `bootstrap/stage0/FROZEN_HASH` to the new compiler hash after the
multi-D indexing fix.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 14 passed; 0 failed |
| Direct `t27c icarus-simulate` / `t27c icarus-cocotb` on W554 witnesses | 3/3 PASS |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 65 Icarus PASS, 65 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` (in `proofs/lean4`) | 8572 jobs, 0 `sorry` |

---

## Cooperation variants for Wave Loop 555

The next loop should tackle one of the following, in decreasing order of
recommendation:

1. **Variant A — Recommended: whole-array bench assignments.** Support
   `assert_eq` on a complete 2-D primitive scalar array value inside a `bench`,
   using the W540 multi-slice probe path for wide signed packed arrays.
2. **Variant B: multi-site call-return array deduplication.** When the same
   `f()` packed-array expression is indexed at multiple sites in one bench,
   reuse a single packed temporary and emit only one assignment.
3. **Variant C: timed/non-deterministic bench classifier.** Introduce an AST
   classifier that rejects (or skips) `bench` blocks containing `#` delays or
   unbounded loops from the deterministic cocotb gate, and document the
   boundary.

---

## Skills learned

- A `bench`-local primitive scalar array initialized from a function call is
  just a packed-vector `reg` in Verilog; the compiler's existing packed-array
  local lowering and the Python evaluator's `_resolve_full_type` handle it once
  the witness exists.
- When computing a row-major flat index from indices collected AST-outermost-
  first, reverse the vector to source order before scaling by dimensions.
- Symmetric multi-D test cases (`tmp[0][0]`, `tmp[1][1]`) can hide indexing
  order bugs; always include at least one asymmetric read (`tmp[1][2]`).

---

## Closes

Closes #1525
