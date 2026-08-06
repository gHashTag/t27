# Wave Loop 682 Decomposed Plan

## 1. Goal

Implement Variant A for Wave Loop 682: a module-scope `[183][2]^6 Pt`
(374,784-bit, ~0.358 MiBit) packed array-of-struct variable, initialized from a
function call, with indexed signed field writes and read-back, validated through
targeted `t27c` gates and `cargo test`.

## 2. Weak points investigated

1. **First outer dimension 183.** The ladder of non-power-of-two module-scope
   packed AoS witnesses has reached 181. Striding by 183 is unproven in the
   current corpus, though the compiler and reference model are expected to be
   dimension-agnostic.
2. **Parser/scanner limits.** The witness will be ~35.5 k lines and ~790 KB.
   Single-line literals remain unsafe; multi-line W584 brace style is required.
3. **Modulo-wrap regression signal.** With 11,712 elements, the offset-0 schedule
   `(2*e + offset) % 32768` maxes at 23,423, so an explicit `make_grid(32768)`
   check must be retained.
4. **`assert_ne` simulation gap.** Structural classifier accepts `assert_ne`, but
   the Icarus emitter does not lower it. The bench must use `assert_eq` on the
   changed elements.
5. **Simulator capacity.** At 0.358 MiBit the witness is still far below the
   Icarus/Yosys comfort threshold (~4 MiBit), but compile time is the variable
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
- Icarus issue #1171 — very large packed vectors can freeze elaboration; W682 stays
  well below the reported threshold.
- Yosys docs / issues #2677/#4653 / PR #4100 — multidimensional packed arrays
  supported, arrays of packed structs not; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array scalarization.

## 4. Tasks

1. **Branch setup**
   - [x] Switch to `wave-loop-682`.
   - [x] Update `.trinity/current-issue.md` for W682 (#1653, Variant A).

2. **Planning**
   - [x] Write `.claude/plans/wave-loop-682.md` (this file).

3. **Generator**
   - [x] Copy `scripts/gen_w681.py` to `scripts/gen_w682.py`.
   - [x] Update `OUTER` to 183 and rename identifiers.

4. **Witness**
   - [x] Generate `specs/scratch/w682_bench_module_183x2p6_aos_var_call_write.t27`.
   - [x] Verify footer assertions for first element, last element, mid element,
     and explicit `make_grid(32768)` wrap.

5. **Test**
   - [x] Add integration test `accepts_w682_bench_module_183x2p6_aos_var_call_write`
     to `bootstrap/tests/icarus_lowerable.rs` right after the W681 test.

6. **Build / Gates**
   - [x] `cargo build --release -p t27c`.
   - [x] `t27c parse` W682.
   - [x] `t27c icarus-lowerable` W682.
   - [x] `t27c icarus-simulate` W682.
   - [x] `t27c icarus-cocotb` W682.
   - [x] `t27c seal --save` W682.
   - [x] Create empty Icarus baseline for W682.

7. **Repository tests**
   - [x] `cargo test -p t27c --bin t27c` (expected 1494/0/2).
   - [x] `cargo test -p tri` (expected 78/0).
   - [x] `cargo test -p t27c --test icarus_lowerable` (expected 142/0).

8. **Documentation**
   - [ ] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W682_2026-07-07.md` with W683
     cooperation variants.
   - [ ] Append W682 learnings to `.trinity/experience.md`.

9. **Memory**
   - [ ] Create `~/.claude/projects/-Users-playra-t27/memory/wave-loop-682.md`
     and update `MEMORY.md`.

10. **Commits**
    - [ ] Feature commit with `Closes #1653`.
    - [ ] Docs/tracking commit with `Closes #1653`.
    - [ ] `chore(trinity): record W682 session log and commit count`.

## 5. Success criteria

- `cargo test` and direct `t27c` gates all pass for W682.
- FROZEN_HASH remains unchanged.
- `git status` clean after final commit.
- Branch `wave-loop-683` exists for the next wave.

## 6. Risk register

| Risk | Likelihood | Impact | Response |
|------|------------|--------|----------|
| Outer dimension 183 fails layout math | Low | High | Compare expected values against cocotb reference model. |
| Parse time blows up repository sweep | High | Low | Use targeted gates; note timeout in report. |
| Icarus simulation path rejects witness | Low | High | Debug emitter; likely small fix if any. |
| `assert_ne` confusion resurfaces | Low | Low | Continue using `assert_eq` on changed elements. |

## 7. Next Wave Loop 683 cooperation variants

1. **Variant A (recommended):** `[185][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 378,880 bits, 11,840 elements. Continue the ladder.
2. **Variant B:** `[183][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[183][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.
