# Wave Loop 700 Decomposed Plan

## 1. Goal

Implement Variant A for Wave Loop 700: a module-scope `[219][2]^6 Pt`
(448,512-bit, ~0.428 MiBit) packed array-of-struct variable, initialized from a
function call, with indexed signed field writes and read-back, validated through
targeted `t27c` gates and `cargo test`.

## 2. Weak points investigated

1. **First outer dimension 219.** The ladder of non-power-of-two module-scope
   packed AoS witnesses has reached 217. Striding by 219 is unproven in the
   current corpus, though the compiler and reference model are expected to be
   dimension-agnostic.
2. **Parser/scanner limits.** The witness will be ~41.8 k lines and ~961 KB.
   Single-line literals remain unsafe; multi-line W584 brace style is required.
3. **Modulo-wrap regression signal.** With 14,016 elements, the offset-0 schedule
   `(2*e + offset) % 32768` maxes at 28,031, so an explicit `make_grid(32768)`
   check must be retained.
4. **`assert_ne` simulation gap.** Structural classifier accepts `assert_ne`, but
   the Icarus emitter does not lower it. The bench must use `assert_eq` on the
   changed elements.
5. **Simulator capacity.** At 0.428 MiBit the witness is still far below the
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
- Icarus issue #1171 — very large packed vectors can freeze elaboration; W700 stays
  well below the reported threshold.
- Yosys docs / issues #2677/#4653 / PR #4100 — multidimensional packed arrays
  supported, arrays of packed structs not; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array scalarization.

## 4. Tasks

1. **Branch setup**
   - [x] Switch to `wave-loop-700`.
   - [x] Update `.trinity/current-issue.md` for W700 (#1671, Variant A).

2. **Planning**
   - [x] Write `.claude/plans/wave-loop-700.md` (this file).

3. **Generator**
   - [ ] Copy `scripts/gen_w699.py` to `scripts/gen_w700.py`.
   - [ ] Update `OUTER` to 219 and rename identifiers.

4. **Witness**
   - [ ] Generate `specs/scratch/w700_bench_module_219x2p6_aos_var_call_write.t27`.
   - [ ] Verify footer assertions for first element, last element, mid element,
     and explicit `make_grid(32768)` wrap.

5. **Test**
   - [x] Add integration test `accepts_w700_bench_module_219x2p6_aos_var_call_write`
     to `bootstrap/tests/icarus_lowerable.rs` right after the W699 test.

6. **Build / Gates**
   - [ ] `cargo build --release -p t27c`.
   - [x] `t27c parse` W700.
   - [x] `t27c icarus-lowerable` W700.
   - [x] `t27c icarus-simulate` W700.
   - [x] `t27c icarus-cocotb` W700.
   - [x] `t27c seal --save` W700.
   - [x] Create empty Icarus baseline for W700.

7. **Repository tests**
   - [x] `cargo test -p t27c --bin t27c` (expected 1494/0/2).
   - [x] `cargo test -p tri` (expected 78/0).
   - [x] `cargo test -p t27c --test icarus_lowerable` (expected 160/0).

8. **Documentation**
   - [ ] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W700_2026-07-07.md` with W701
     cooperation variants.
   - [ ] Append W700 learnings to `.trinity/experience.md`.

9. **Memory**
   - [ ] Create `~/.claude/projects/-Users-playra-t27/memory/wave-loop-700.md`
     and update `MEMORY.md`.

10. **Commits**
    - [ ] Feature commit with `Closes #1671`.
    - [ ] Docs/tracking commit with `Closes #1671`.
    - [ ] `chore(trinity): record W700 session log and commit count`.

## 5. Success criteria

- `cargo test` and direct `t27c` gates all pass for W700.
- FROZEN_HASH remains unchanged.
- `git status` clean after final commit.
- Branch `wave-loop-701` exists for the next wave.

## 6. Risk register

| Risk | Likelihood | Impact | Response |
|------|------------|--------|----------|
| Outer dimension 219 fails layout math | Low | High | Compare expected values against cocotb reference model. |
| Parse time blows up repository sweep | High | Low | Use targeted gates; note timeout in report. |
| Icarus simulation path rejects witness | Low | High | Debug emitter; likely small fix if any. |
| `assert_ne` confusion resurfaces | Low | Low | Continue using `assert_eq` on changed elements. |

## 7. Next Wave Loop 701 cooperation variants

1. **Variant A (recommended):** `[221][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 452,608 bits, 14,144 elements. Continue the ladder.
2. **Variant B:** `[219][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[219][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.
