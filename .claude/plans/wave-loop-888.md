# Wave Loop 888 Plan — [595][2]^6 Pt packed AoS witness

## Objective
Continue the mechanical packed-vector array-of-struct ladder one step past the 1.16 MiBit line, keeping the same pattern and zero compiler changes.

## Shape
- Outer dimension: `595` (non-power-of-two)
- Inner struct: `[2]^6 Pt` → 2 fields × 6 trits × 32 bits = 384 bits per element
- Total elements: `595 × 64 = 38,080` structs
- Packed vector width: `38,080 × 32 = 1,218,560` bits (~1.162 MiBit)

## Pattern
Module-scope variable `dst : [595][2]^6 Pt` initialized from a function call `make_grid(0)`, with indexed signed field writes and `assert_eq` read-back inside a `bench` block.

## Variants
- **A (recommended)**: `[595][2]^6 Pt` — continue the outer-dimension ladder.
- **B**: `[593][3]^6 Pt` — keep outer dimension near W887 but scale field count to 3, increasing memory-quanta density while testing a different stride.
- **C**: `[593][2]^6 Pt` with explicit negative-index wrap-around writes — exercise signed-index bound normalization one step beyond the baseline W887 shape.

## Procedure
1. Branch `wave-loop-888` from `wave-loop-887` HEAD (earlier wave PRs remain open).
2. Copy `scripts/gen_w887.py` → `scripts/gen_w888.py` and clear the copy-hazard checklist:
   - destination path,
   - module header f-string,
   - `MID_IDX` comment.
3. Run `python3 scripts/gen_w888.py` to produce `specs/scratch/w888_bench_module_595x2p6_aos_var_call_write.t27`.
4. Validate:
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save` and `seal --verify`
   - targeted `cargo test --release --test icarus_lowerable accepts_w888_...`
5. Add the W888 integration test to `bootstrap/tests/icarus_lowerable.rs`.
6. Commit with `Closes #1836`, push `wave-loop-888`, open PR.
7. Update trackers, skill, experience, and persistent memory.

## Invariants
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- All generated files must pass ASCII and `seal --verify`.
