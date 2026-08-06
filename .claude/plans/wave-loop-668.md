# Wave Loop 668 — Decomposed Plan

**Issue #1639** | Branch `wave-loop-668` | Date 2026-07-07
**Variant A**: module-scope `[155][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable from a function call, with indexed signed field writes.

---

## 1. Objective

Extend the module-scope packed array-of-struct (AoS) witness ladder to outer
dimension 155. Validate that a 317,440-bit packed vector (9,920 `Pt` elements)
can be declared as a module-level mutable `reg`, initialized from a function call,
read with full-index paths, partially updated via signed-index field writes,
and cross-checked with the cocotb/Python reference model. No compiler changes
are expected.

---

## 2. Weak points / risks

| # | Weak point | Mitigation |
|---|------------|------------|
| 1 | **Non-power-of-two outer dimension 155.** Compiler and cocotb model must multiply/stride by 155. Earlier waves up to 153 passed; W668 is the next rung. | Run targeted `t27c` parse, lowerable, simulate, cocotb, and seal gates. |
| 2 | **Element count still below modulo-wrap point.** 9,920 elements → max raw value 19,839, so `% 32768` never wraps at offset 0. | Keep explicit `make_grid(32768)` check to preserve regression signal. |
| 3 | **Large literal parse time.** A 155× 6-D nested literal is ~29,600 lines / ~679 KB. | Reuse multi-line W584 brace style; do not rely on `./scripts/tri test --fast` for the giant literal. |
| 4 | **`assert_ne` not emitted by Icarus simulation path.** Structural classifier accepts it but simulation does not lower it. | Use `assert_eq` checks on changed elements only. |
| 5 | **Simulator capacity / elaboration time.** At 0.303 MiBit the vector is still small, but 155 rows is a new local maximum. | Direct Icarus simulation gate + cocotb cross-check. |

---

## 3. Scientific / technical background

- **IEEE Std 1800-2017 §7.4.1 / §7.4.3**: packed array width is the product of
  dimensions; no power-of-two restriction. The emitted vector is 317,440 bits.
- **Sutherland, “Synthesizable SystemVerilog”**: packed arrays and packed structs
  are synthesizable first-class values.
- **Lutsig (CPP 2021)** and **CIRCT `HWLegalizeModules.cpp`**: flattening nested
  arrays to wide packed vectors is a verified/production compiler discipline,
  even with non-power-of-two dimensions.
- **Icarus issue #1134**: unpacked arrays of packed structs trigger assertion
  failures; t27 scalarizes the struct and uses a single packed vector, avoiding
  the construct.
- **Icarus issue #1171**: very large packed vectors can freeze elaboration;
  W668 is far below the reported threshold.
- **Yosys issues #2677 / #4653**: native frontend does not support arrays of
  packed structs; t27 flattening avoids the gap.
- **cocotb PR #3608 / discussion #2933**: packed structs and flat `LogicArray`
  are the correct reference-model representations.

---

## 4. Tasks

1. **Issue/branch setup**
   - [x] Update `.trinity/current-issue.md` for W668 (#1639).
   - [x] Confirm branch `wave-loop-668` exists and is checked out.

2. **Generator**
   - [ ] Copy `scripts/gen_w667.py` → `scripts/gen_w668.py` and set `OUTER = 155`.

3. **Witness spec**
   - [ ] Run generator to produce
         `specs/scratch/w668_bench_module_155x2p6_aos_var_call_write.t27`.
   - [ ] Verify header/footer, dimensions, and expected values.

4. **Integration test**
   - [ ] Add `accepts_w668_bench_module_155x2p6_aos_var_call_write` to
         `bootstrap/tests/icarus_lowerable.rs` right after W667 test.

5. **Build + targeted gates**
   - [ ] `cargo build --release -p t27c`.
   - [ ] `t27c parse` W668.
   - [ ] `t27c icarus-lowerable` W668.
   - [ ] `t27c icarus-simulate` W668.
   - [ ] `t27c icarus-cocotb` W668.
   - [ ] `t27c seal --save` W668.

6. **Baselines**
   - [ ] Create empty Icarus baseline
         `.trinity/icarus-baselines/specs/scratch/w668_bench_module_155x2p6_aos_var_call_write.json`.

7. **Cargo test suites**
   - [ ] `cargo test -p t27c --bin t27c` (expected 1494/0/2).
   - [ ] `cargo test -p tri` (expected 78/0).
   - [ ] `cargo test -p t27c --test icarus_lowerable` (expected 128/0).

8. **Repository-wide sweep (best effort)**
   - [ ] Run `./scripts/tri test --fast` with a timeout. Expected outcome: timeout
         in Phase 1 Parse due to the giant literal, as in W665–W667.

9. **Closeout + experience + memory**
   - [ ] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W668_2026-07-07.md`.
   - [ ] Append W668 entry to `.trinity/experience.md`.
   - [ ] Save
         `~/.claude/projects/-Users-playra-t27/memory/wave-loop-668.md` and update
         `MEMORY.md`.

10. **Commits**
    - [ ] Feature commit with `Closes #1639`.
    - [ ] Docs/tracking commit with `Closes #1639`.
    - [ ] `chore(trinity): record W668 session log and commit count`.

---

## 5. Success criteria

- `cargo test` and direct `t27c` gates all pass for W668.
- FROZEN_HASH remains unchanged.
- `git status` clean after final commit.
- Branch `wave-loop-669` exists for the next wave.

---

## 6. Risk register

| Risk | Likelihood | Impact | Response |
|------|------------|--------|----------|
| Outer dimension 155 fails layout math | Low | High | Compare expected values against cocotb reference model. |
| Parse time blows up repository sweep | High | Low | Use targeted gates; note timeout in report. |
| Icarus simulation path rejects witness | Low | High | Debug emitter; likely small fix if any. |
| `assert_ne` confusion resurfaces | Low | Low | Continue using `assert_eq` on changed elements. |

---

## 7. Next Wave Loop 669 cooperation variants

1. **Variant A (recommended):** `[157][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 321,536 bits, 10,048 elements. Continue the ladder.
2. **Variant B:** `[155][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[155][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.
