# Wave Loop 583 Plan — Module-scope 3-D array-of-struct constants with computed-field bench cross-check

## Issue

Closes #1554.

## Context

Wave Loops 566–582 systematically scaled **function-local / call-return**
array-of-struct (AoS) packed-vector lowering from 2-D to 16-D, culminating in
a 2,097,152-bit `[2]^16 Pt` witness that passed Icarus simulation and cocotb
VCD cross-check without compiler changes. Module-scope AoS, however, has only
been exercised at small scale (2-D `[2][3]Pt` const/var in W528/W529 and
single scalar structs in W533). Module-scope 3-D AoS and whole-array comparison
against a function return were untested.

## Weak-spot analysis

1. **Scope gap:** high-rank / whole-array behavior has only been proven for
   function-local and call-return paths. Module-level parameters/registers
   share some code paths (`module_types`, `param_types`) but were not stressed
   with bench-block whole-array assertions or computed-field initializers.
2. **Indefinite-width literals in packed concatenations:** when a scalar struct
   field or array element is a non-literal expression such as `base + 1`, the
   compiler emitted the raw Verilog expression inside a concatenation. Icarus
   12.0 rejected this with:
   ```
   error: Concatenation operand "(base)+('sd1)" has indefinite width.
   ```
   Plain literals worked because `emit_packed_scalar_value` sized them
   explicitly; non-literals did not.
3. **Tooling limits:** IEEE 1800-2017 guarantees only 65,536-bit packed arrays.
   Icarus has no documented hard cap but can hang/oom on pathological widths.
   Yosys synthesis smoke already warns on large signed literals (`16'sd131071`)
   for the 16-D witness. A 17-D 4-MiBit vector would double W582's already
   heavy resource footprint and risk CI timeout, while adding little new
   coverage of a different code path.

## Scientific / engineering precedents

- IEEE Std 1800-2017, clause 7.4.1: packed-array minimum size 2^16 bits.
- Icarus maintainer caryr (Sep 2024): standard *suggests* 2^16 packed dimension
  floor; Icarus does not enforce it but very large packed vectors can exhaust
  memory (steveicarus/iverilog#1171).
- CIRCT `HWLegalizeModules.cpp`: recursively legalizes multi-dimensional packed
  arrays one dimension at a time, confirming that t27's rank-agnostic lowering
  strategy matches production compiler infrastructure.
- Yosys frontend: literal width mismatch warnings are a known, non-fatal
  synthesis-smoke artifact (StackOverflow / Yosys internals docs).

## Variant choice

Select **Variant C — module-scope 2-D/3-D array-of-struct constants/variables**,
but push it to **3-D with computed-field initializers and a bench whole-array
comparison**, because this is the smallest variant that:
- exercises a previously untested scope (module-level 3-D AoS),
- triggers the indefinite-width concatenation bug,
- fixes it with a minimal, semantically equivalent SystemVerilog width cast,
- adds only one new integration test and one new witness instead of a
  multi-megabyte 17-D spec.

Variants A and B are documented as alternative cooperation options for Wave
Loop 584.

## Risk assessment

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Width-cast change breaks existing literal rendering | Low | Cast only applies to non-literal branch of `emit_packed_scalar_value`; literal branch unchanged. |
| Existing seals mismatch because generated code changes | Medium | Reseal affected specs after running `./scripts/tri test`. |
| `$signed(width'(expr))` rejected by Yosys | Low | Verified with Yosys 0.63 on a minimal example. |
| `width'(expr)` rejected by older Icarus | Low | Icarus 12.0 `-g2012` accepts it; `-g2012` already required for packed arrays. |
| Lean lowerability proofs need update | Low | No predicate change; only generated expression syntax changes. |

## Implementation steps

1. Fix `emit_packed_scalar_value` in `bootstrap/src/compiler.rs` to emit
   `width'(expr)` for non-literal expressions (and `$signed(width'(expr))` for
   signed fields).
2. Run freeze ceremony: update `bootstrap/stage0/FROZEN_HASH` to the new
   SHA-256 of `bootstrap/src/compiler.rs`.
3. Create deterministic witness `specs/scratch/w583_bench_module_3d_aos_call_dedup.t27`:
   - module-level `pub const expected : [2][2][2]Pt` with literal values,
   - function `make_cube(offset : u16) -> [2][2][2]Pt` returning a 3-D AoS with
     computed fields (`offset + N`),
   - bench block that calls `make_cube(0)`, assigns to a local `actual`, and
     `assert_eq(actual, expected)`.
4. Add integration test `accepts_w583_bench_module_3d_aos_call_dedup` to
   `bootstrap/tests/icarus_lowerable.rs`.
5. Run verification matrix:
   - `cargo build --release -p t27c`
   - `cargo test -p t27c --bin t27c`
   - `cargo test -p tri`
   - `cargo test -p t27c --test icarus_lowerable`
   - `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
   - direct `t27c icarus-simulate` and `t27c icarus-cocotb` on the W583 witness.
6. Reseal any specs whose generated output changed.
7. Write closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W583_2026-07-07.md`.
8. Update `.trinity/current-issue.md` with Wave Loop 584 variants.
9. Update `.trinity/experience.md` with W583 learnings.
10. Save persistent memory `~/.claude/projects/-Users-playra-t27/memory/wave-loop-583.md`
    and `MEMORY.md` index.
11. Commit with `Closes #1554` and create `wave-loop-584` branch.

## Verification criteria

- `cargo build --release -p t27c` green.
- All cargo test suites pass with previous counts or explainable changes.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  Icarus PASS count >= 73, cocotb PASS count >= 73, 0 seal mismatches, 24
  pre-existing yosys smoke baselines unchanged.
- Direct `t27c icarus-simulate` on W583 witness: PASS.
- Direct `t27c icarus-cocotb` on W583 witness: PASS.

## Three cooperation variants for Wave Loop 584

### Variant A — Continue rank scaling (recommended if CI budget allows)
17-D `[2]^17 Pt` (4,194,304 bits, 131,072 elements). Validates whether the
function-local/call-return path accepts a 4-MiBit vector and whether the
Icarus `$display` workaround scales. Risk: witness ~22 MB / ~1.2M lines,
direct simulation may approach 8–10 min.

### Variant B — Non-power-of-two extreme
16-D with outer dimension 3: `[3][2]^16 Pt` (3,932,160 bits, 196,608 elements).
Tests non-power-of-two outer dimension at the boundary of practical file size.
Witness ~34 MB / ~1.8M lines.

### Variant C — Module-scope multi-site / larger constants
Extend W583 to a module-level 3-D AoS **variable** initialized from a
function-call return, or multiple call sites reading the module const,
exercising CSE/deduplication across module-scope and function-return boundaries.
This keeps file size small while probing interaction between W557 call-array
CSE and module-level constants.
