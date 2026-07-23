# Wave Loop 701 Decomposed Plan

## 1. Goal

Implement Variant A for Wave Loop 701: a module-scope `[221][2]^6 Pt`
(452,608-bit, ~0.432 MiBit) packed array-of-struct variable, initialized from a
function call, with indexed signed field writes and read-back, validated through
targeted `t27c` gates and `cargo test`.

## 2. Weak points investigated

1. **First outer dimension 221.** The ladder of non-power-of-two module-scope
   packed AoS witnesses has reached 219. Striding by 221 is unproven in the
   current corpus, though the compiler and reference model are expected to be
   dimension-agnostic.
2. **Parser/scanner limits.** The witness will be ~42.0 k lines and ~972 KB.
   Single-line literals remain unsafe; multi-line W584 brace style is required.
3. **Modulo-wrap regression signal.** With 14,144 elements, the offset-0 schedule
   `(2*e + offset) % 32768` maxes at 28,287, so an explicit `make_grid(32768)`
   check must be retained.
4. **`assert_ne` simulation gap.** Structural classifier accepts `assert_ne`, but
   the Icarus emitter does not lower it. The bench must use `assert_eq` on the
   changed elements.
5. **Simulator capacity.** At 0.432 MiBit the witness is still far below the
   ~4-MiBit Icarus/Yosys comfort threshold, but compile time is the variable
   to watch.

## 3. Scientific / technical background

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  no power-of-two restriction.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, "Synthesizable SystemVerilog" — packed arrays/structs are
  synthesizable first-class objects.
- Icarus issue #1134 — unpacked arrays of packed structs cause assertion failures;
  t27 flattening avoids this.
- Icarus issue #1171 — very large packed vectors can freeze elaboration; W701 stays
  well below the reported threshold.
- Yosys docs / issues #2677/#4653 / PR #4100 — multidimensional packed arrays
  supported, arrays of packed structs unsupported; t27 flattening avoids the gap.
- cocotb PR #3608 / discussion #2933 — Python reference models compare whole
  packed-vector VCD probes against flattened expectations; W701 reuses this.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## 4. Tasks

1. **Generator script**
   - [x] Copy `scripts/gen_w700.py` to `scripts/gen_w701.py` and set `OUTER = 221`.
   - [x] Run `python3 scripts/gen_w701.py` to produce the witness.

2. **Witness**
   - [x] Generate `specs/scratch/w701_bench_module_221x2p6_aos_var_call_write.t27`.
   - [x] Verify footer assertions match expected element indices.

3. **Integration test**
   - [ ] Add `accepts_w701_bench_module_221x2p6_aos_var_call_write` to
     `bootstrap/tests/icarus_lowerable.rs` after the W700 test.

4. **Direct t27c gates**
   - [x] `t27c parse` W701.
   - [x] `t27c icarus-lowerable` W701.
   - [x] `t27c icarus-simulate` W701.
   - [x] `t27c icarus-cocotb` W701.
   - [x] `t27c seal --save` W701.
   - [x] Create empty Icarus baseline for W701.

5. **Repository tests**
   - [x] `cargo test -p t27c --bin t27c` (expected 1494/0/2).
   - [x] `cargo test -p tri` (expected 78/0).
   - [x] `cargo test -p t27c --test icarus_lowerable` (expected 161/0).

6. **Documentation**
   - [x] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W701_2026-07-07.md` with W702
     cooperation variants.
   - [x] Append W701 learnings to `.trinity/experience.md`.

7. **Memory**
   - [ ] Create `~/.claude/projects/-Users-playra-t27/memory/wave-loop-701.md`
     and update `MEMORY.md`.

8. **Commits**
   - [x] Feature commit with `Closes #1672`.
   - [x] Docs/tracking commit with `Closes #1672`.
   - [x] `chore(trinity): record W701 session log and commit count`.

## 5. Success criteria

- `cargo test` and direct `t27c` gates all pass for W701.
- FROZEN_HASH remains unchanged.
- `git status` clean after final commit.
- Branch `wave-loop-702` exists for the next wave.

## 6. Risk register

| Risk | Likelihood | Impact | Response |
|------|------------|--------|----------|
| Outer dimension 221 fails layout math | Low | High | Compare expected values against cocotb reference model. |
| Parse time blows up repository sweep | High | Low | Use targeted gates; note timeout in report. |
| Icarus simulation path rejects witness | Low | High | Debug emitter; likely small fix if any. |
| `assert_ne` confusion resurfaces | Low | Low | Continue using `assert_eq` on changed elements. |

## 7. Next Wave Loop 702 cooperation variants

1. **Variant A (recommended):** `[223][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 456,704 bits, 14,272 elements. Continue the ladder.
2. **Variant B:** `[221][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[221][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.
