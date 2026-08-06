# Plan — Wave Loop 591

**Issue:** #1562 — Module-scope 17-D array-of-struct variable initialized from a
function call, then wholesale-reassigned to a packed array literal, at the
4-MiBit cliff.

**Branch:** `wave-loop-591`
**Previous:** `wave-loop-590` (#1561)

## Goal

Demonstrate that a module-scope `[2]^17 Pt` packed `reg` can be initialized from
a function call and then whole-array reassigned to a packed array literal
(`dst = expected_b;`) without new compiler support, while staying at the
validated 4-MiBit boundary.

## Plan

1. **Weak-point analysis.** Agent E reviews W590 closeout and highlights risks
   of duplicating a 4-MiBit literal in one spec.
2. **Variant selection.** Choose Variant C: `[2]^17 Pt` module var initialized
   from a call, then reassigned to a packed literal RHS.
3. **Smoke test.** Verify that small `[2][2]Pt` var literal reassignment works
   end-to-end with `./scripts/tri test`.
4. **Witness spec.** Write `specs/scratch/w591_bench_module_17d_aos_var_literal_reassign.t27`
   with two 17-D literals using W584 multi-line brace style and signed-i16-safe
   leaf schedules.
5. **Integration test.** Add `accepts_w591_bench_module_17d_aos_var_literal_reassign`
   to `bootstrap/tests/icarus_lowerable.rs`.
6. **Seal and baseline.** Generate seal and Icarus baseline for the witness.
7. **Local verification.**
   - `cargo build --release -p t27c`
   - `cargo test -p t27c --bin t27c`
   - `cargo test -p tri`
   - `cargo test -p t27c --test icarus_lowerable`
   - `./scripts/tri test --fast`
8. **Full Icarus/cocotb tri pipeline.** `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`.
9. **Documentation.** Update `.trinity/current-issue.md`, `.trinity/experience.md`,
   write closeout report, and update persistent memory.

## Variants considered

- Variant A: `[2]^18 Pt` module var. Crosses 4-MiBit cliff; not interactive.
- Variant B: `[3][2]^15 Pt` module var. Under cliff with non-power-of-two outer
  dimension; useful for future loop.
- Variant C: `[2]^17 Pt` module var literal reassignment. Chosen.

## Risk mitigations

- Use `(2*i+offset)%32768` schedules to keep signed i16 leaves valid.
- Use multi-line W584 brace style for the 17-D literal.
- Use a `bench` to perform full-array comparison and indexed signed field writes
  after reassignment.
