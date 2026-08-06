# Wave Loop 686 Decomposed Plan

## 1. Goal

Implement Variant A for Wave Loop 686: a module-scope `[191][2]^6 Pt`
(391,168-bit, ~0.373 MiBit) packed array-of-struct variable, initialized from a
function call, with indexed signed field writes and read-back, validated through
targeted `t27c` gates and `cargo test`.

## 2. Weak points investigated

1. **First outer dimension 191.** The ladder of non-power-of-two module-scope
   packed AoS witnesses has reached 189. Striding by 191 is unproven in the
   current corpus, though the compiler and reference model are expected to be
   dimension-agnostic.
2. **Parser/scanner limits.** The witness will be ~36.7 k lines and ~834 KB.
   Single-line literals remain unsafe; multi-line W584 brace style is required.
3. **Modulo-wrap regression signal.** With 12,224 elements, the offset-0 schedule
   `(2*e + offset) % 32768` maxes at 24,447, so an explicit `make_grid(32768)`
   check must be retained.
4. **`assert_ne` simulation gap.** Structural classifier accepts `assert_ne`, but
   the Icarus emitter does not lower it. The bench must use `assert_eq` on the
   changed elements.
5. **Simulator capacity.** At 0.373 MiBit the witness is still far below the
   ~4-MiBit Icarus/Yosys comfort threshold, but compile time is the variable
   to watch.

## 3. Scientific / technical background

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  no power-of-two restriction.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are first-class
  synthesizable objects.
- Icarus issue #1134 — unpacked arrays of packed structs cause assertion failures;
  t27 flattening avoids this.
- Icarus issue #1171 — very large packed vectors can freeze elaboration; W686 stays
  well below the reported threshold.
- Yosys docs / issues #2677/#4653 / PR #4100 — multidimensional packed arrays
  supported, arrays of packed structs not; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array scalarization.

## 4. Tasks

1. **Branch setup**
   - [x] Switch to `wave-loop-686`.
   - [x] Update `.trinity/current-issue.md` for W686 (#1657, Variant A).

2. **Planning**
   - [x] Write `.claude/plans/wave-loop-686.md` (this file).

3. **Generator**
   - [x] Copy `scripts/gen_w685.py` to `scripts/gen_w686.py`.
   - [x] Update `OUTER` to 191 and rename identifiers.

4. **Witness**
   - [x] Generate `specs/scratch/w686_bench_module_191x2p6_aos_var_call_write.t27`.
   - [x] Verify footer assertions for first element, last element, mid element,
     and explicit `make_grid(32768)` wrap.

5. **Test**
   - [x] Add integration test `accepts_w686_bench_module_191x2p6_aos_var_call_write`
     to `bootstrap/tests/icarus_lowerable.rs` right after the W685 test.

6. **Build / Gates**
   - [x] `cargo build --release -p t27c`.
   - [x] `t27c parse` W686.
   - [x] `t27c icarus-lowerable` W686.
   - [x] `t27c icarus-simulate` W686.
   - [x] `t27c icarus-cocotb` W686.
   - [x] `t27c seal --save` W686.
   - [x] Create empty Icarus baseline for W686.

7. **Repository tests**
   - [x] `cargo test -p t27c --bin t27c` (expected 1494/0/2).
   - [x] `cargo test -p tri` (expected 78/0).
   - [x] `cargo test -p t27c --test icarus_lowerable` (expected 146/0).

8. **Documentation**
   - [x] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W686_2026-07-07.md` with W687
     cooperation variants.
   - [x] Append W686 learnings to `.trinity/experience.md`.

9. **Memory**
   - [x] Create `~/.claude/projects/-Users-playra-t27/memory/wave-loop-686.md`
     and update `MEMORY.md`.

10. **Commits**
    - [ ] Feature commit with `Closes #1657`.
    - [ ] Docs/tracking commit with `Closes #1657`.
    - [ ] `chore(trinity): record W686 session log and commit count`.

## 5. Success criteria

- `cargo test` and direct `t27c` gates all pass for W686.
- FROZEN_HASH remains unchanged.
- `git status` clean after final commit.
- Branch `wave-loop-687` exists for the next wave.

## 6. Risk register

| Risk | Likelihood | Impact | Response |
|------|------------|--------|----------|
| Outer dimension 191 fails layout math | Low | High | Compare expected values against cocotb reference model. |
| Parse time blows up repository sweep | High | Low | Use targeted gates; note timeout in report. |
| Icarus simulation path rejects witness | Low | High | Debug emitter; likely small fix if any. |
| `assert_ne` confusion resurfaces | Low | Low | Continue using `assert_eq` on changed elements. |

## 7. Next Wave Loop 687 cooperation variants

1. **Variant A (recommended):** `[193][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 395,264 bits, 12,352 elements. Continue the ladder.
2. **Variant B:** `[191][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[191][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.
