# Wave Loop 697 Decomposed Plan

## 1. Goal

Implement Variant A for Wave Loop 697: a module-scope `[213][2]^6 Pt`
(436,224-bit, ~0.415 MiBit) packed array-of-struct variable, initialized from a
function call, with indexed signed field writes and read-back, validated through
targeted `t27c` gates and `cargo test`.

## 2. Weak points investigated

1. **First outer dimension 213.** The ladder of non-power-of-two module-scope
   packed AoS witnesses has reached 211. Striding by 213 is unproven in the
   current corpus, though the compiler and reference model are expected to be
   dimension-agnostic.
2. **Parser/scanner limits.** The witness will be ~40.7 k lines and ~934 KB.
   Single-line literals remain unsafe; multi-line W584 brace style is required.
3. **Modulo-wrap regression signal.** With 13,632 elements, the offset-0 schedule
   `(2*e + offset) % 32768` maxes at 27,263, so an explicit `make_grid(32768)`
   check must be retained.
4. **`assert_ne` simulation gap.** Structural classifier accepts `assert_ne`, but
   the Icarus emitter does not lower it. The bench must use `assert_eq` on the
   changed elements.
5. **Simulator capacity.** At 0.415 MiBit the witness is still far below the
   ~4-MiBit Icarus/Yosys comfort threshold, but compile time is the variable
   to watch.

## 3. Scientific / technical background

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  no power-of-two restriction.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, "Synthesizable SystemVerilog" — packed arrays/structs are first-class
  synthesizable objects.
- Icarus issue #1134 — unpacked arrays of packed structs cause assertion failures;
  t27 flattening avoids this.
- Icarus issue #1171 — very large packed vectors can freeze elaboration; W697 stays
  well below the reported threshold.
- Yosys docs / issues #2677/#4653 / PR #4100 — multidimensional packed arrays
  supported, arrays of packed structs not; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array scalarization.

## 4. Tasks

1. **Branch setup**
   - [x] Switch to `wave-loop-697`.
   - [x] Update `.trinity/current-issue.md` for W697 (#1668, Variant A).

2. **Planning**
   - [x] Write `.claude/plans/wave-loop-697.md` (this file).

3. **Generator**
   - [x] Copy `scripts/gen_w696.py` to `scripts/gen_w697.py`.
   - [x] Update `OUTER` to 213 and rename identifiers.

4. **Witness**
   - [x] Generate `specs/scratch/w697_bench_module_213x2p6_aos_var_call_write.t27`.
   - [x] Verify footer assertions for first element, last element, mid element,
     and explicit `make_grid(32768)` wrap.

5. **Test**
   - [x] Add integration test `accepts_w697_bench_module_213x2p6_aos_var_call_write`
     to `bootstrap/tests/icarus_lowerable.rs` right after the W696 test.

6. **Build / Gates**
   - [x] `cargo build --release -p t27c`.
   - [x] `t27c parse` W697.
   - [x] `t27c icarus-lowerable` W697.
   - [x] `t27c icarus-simulate` W697.
   - [x] `t27c icarus-cocotb` W697.
   - [x] `t27c seal --save` W697.
   - [x] Create empty Icarus baseline for W697.

7. **Repository tests**
   - [x] `cargo test -p t27c --bin t27c` (expected 1494/0/2).
   - [x] `cargo test -p tri` (expected 78/0).
   - [x] `cargo test -p t27c --test icarus_lowerable` (expected 157/0).

8. **Documentation**
   - [x] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W697_2026-07-07.md` with W698
     cooperation variants.
   - [x] Append W697 learnings to `.trinity/experience.md`.

9. **Memory**
   - [x] Create `~/.claude/projects/-Users-playra-t27/memory/wave-loop-697.md`
     and update `MEMORY.md`.

10. **Commits**
    - [x] Feature commit with `Closes #1668`.
    - [x] Docs/tracking commit with `Closes #1668`.
    - [x] `chore(trinity): record W697 session log and commit count`.

## 5. Success criteria

- `cargo test` and direct `t27c` gates all pass for W697.
- FROZEN_HASH remains unchanged.
- `git status` clean after final commit.
- Branch `wave-loop-698` exists for the next wave.

## 6. Risk register

| Risk | Likelihood | Impact | Response |
|------|------------|--------|----------|
| Outer dimension 213 fails layout math | Low | High | Compare expected values against cocotb reference model. |
| Parse time blows up repository sweep | High | Low | Use targeted gates; note timeout in report. |
| Icarus simulation path rejects witness | Low | High | Debug emitter; likely small fix if any. |
| `assert_ne` confusion resurfaces | Low | Low | Continue using `assert_eq` on changed elements. |

## 7. Next Wave Loop 698 cooperation variants

1. **Variant A (recommended):** `[215][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 440,320 bits, 13,760 elements. Continue the ladder.
2. **Variant B:** `[213][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[213][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.
