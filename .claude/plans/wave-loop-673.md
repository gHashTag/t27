# Wave Loop 673 — Decomposed Plan

**Issue #1644** | Branch `wave-loop-673` | Date 2026-07-07
**Variant A**: module-scope `[165][2]^6 Pt` non-power-of-two outer-dimension
array-of-struct variable from a function call, with indexed signed field writes.

---

## 1. Objective

Extend the module-scope packed array-of-struct (AoS) witness ladder to outer
dimension 165. Validate that a 337,920-bit packed vector (10,560 `Pt` elements)
can be declared as a module-level mutable `reg`, initialized from a function call,
read with full-index paths, partially updated via signed-index field writes,
and cross-checked with the cocotb/Python reference model. No compiler changes
are expected.

---

## 2. Weak points / risks

| # | Weak point | Mitigation |
|---|------------|------------|
| 1 | **Non-power-of-two outer dimension 165.** Compiler and cocotb model must multiply/stride by 165. Earlier waves up to 163 passed; W673 is the next rung. | Run targeted `t27c` parse, lowerable, simulate, cocotb, and seal gates. |
| 2 | **Element count still below modulo-wrap point.** 10,560 elements → max raw value 21,119, so `% 32768` never wraps at offset 0. | Keep explicit `make_grid(32768)` check to preserve regression signal. |
| 3 | **Large literal parse time.** A 165× 6-D nested literal is ~31,400 lines / ~725 KB. | Reuse multi-line W584 brace style; do not rely on `./scripts/tri test --fast` for the giant literal. |
| 4 | **`assert_ne` not emitted by Icarus simulation path.** Structural classifier accepts it but simulation does not lower it. | Use `assert_eq` checks on changed elements only. |
| 5 | **Simulator capacity / elaboration time.** At 0.323 MiBit the vector is still small, but 165 rows is a new local maximum. | Direct Icarus simulation gate + cocotb cross-check. |

---

## 3. Scientific / technical background

- **IEEE Std 1800-2017 §7.4.1 / §7.4.3**: packed array width is the product of
  dimensions; no power-of-two restriction. The emitted vector is 337,920 bits.
- **Sutherland, “Synthesizable SystemVerilog”**: packed arrays and packed structs
  are synthesizable first-class values.
- **Lutsig (CPP 2021)** and **CIRCT `HWLegalizeModules.cpp`**: flattening nested
  arrays to wide packed vectors is a verified/production compiler discipline,
  even with non-power-of-two dimensions.
- **Icarus issue #1134**: unpacked arrays of packed structs trigger assertion
  failures; t27 scalarizes the struct and uses a single packed vector, avoiding
  the construct.
- **Icarus issue #1171**: very large packed vectors can freeze elaboration;
  W673 is far below the reported threshold.
- **Yosys issues #2677 / #4653**: native frontend does not support arrays of
  packed structs; t27 flattening avoids the gap.
- **cocotb PR #3608 / discussion #2933**: packed structs and flat `LogicArray`
  are the correct reference-model representations.

---

## 4. Tasks

1. **Issue/branch setup**
   - [x] Update `.trinity/current-issue.md` for W673 (#1644).
   - [x] Confirm branch `wave-loop-673` exists and is checked out.

2. **Generator**
   - [ ] Copy `scripts/gen_w672.py` → `scripts/gen_w673.py` and set `OUTER = 165`.

3. **Witness spec**
   - [ ] Run generator to produce
         `specs/scratch/w673_bench_module_165x2p6_aos_var_call_write.t27`.
   - [ ] Verify header/footer, dimensions, and expected values.

4. **Integration test**
   - [ ] Add `accepts_w673_bench_module_165x2p6_aos_var_call_write` to
         `bootstrap/tests/icarus_lowerable.rs` right after W672 test.

5. **Build + targeted gates**
   - [ ] `cargo build --release -p t27c`.
   - [ ] `t27c parse` W673.
   - [ ] `t27c icarus-lowerable` W673.
   - [ ] `t27c icarus-simulate` W673.
   - [ ] `t27c icarus-cocotb` W673.
   - [ ] `t27c seal --save` W673.

6. **Baselines**
   - [ ] Create empty Icarus baseline
         `.trinity/icarus-baselines/specs/scratch/w673_bench_module_165x2p6_aos_var_call_write.json`.

7. **Cargo test suites**
   - [ ] `cargo test -p t27c --bin t27c` (expected 1494/0/2).
   - [ ] `cargo test -p tri` (expected 78/0).
   - [ ] `cargo test -p t27c --test icarus_lowerable` (expected 133/0).

8. **Repository-wide sweep (best effort)**
   - [ ] Run `./scripts/tri test --fast` with a timeout. Expected outcome: timeout
         in Phase 1 Parse due to the giant literal, as in W665–W672.

9. **Closeout + experience + memory**
   - [ ] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W673_2026-07-07.md`.
   - [ ] Append W673 entry to `.trinity/experience.md`.
   - [ ] Save
         `~/.claude/projects/-Users-playra-t27/memory/wave-loop-673.md` and update
         `MEMORY.md`.

10. **Commits**
    - [ ] Feature commit with `Closes #1644`.
    - [ ] Docs/tracking commit with `Closes #1644`.
    - [ ] `chore(trinity): record W673 session log and commit count`.

---

## 5. Success criteria

- `cargo test` and direct `t27c` gates all pass for W673.
- FROZEN_HASH remains unchanged.
- `git status` clean after final commit.
- Branch `wave-loop-674` exists for the next wave.

---

## 6. Risk register

| Risk | Likelihood | Impact | Response |
|------|------------|--------|----------|
| Outer dimension 165 fails layout math | Low | High | Compare expected values against cocotb reference model. |
| Parse time blows up repository sweep | High | Low | Use targeted gates; note timeout in report. |
| Icarus simulation path rejects witness | Low | High | Debug emitter; likely small fix if any. |
| `assert_ne` confusion resurfaces | Low | Low | Continue using `assert_eq` on changed elements. |

---

## 7. Next Wave Loop 674 cooperation variants

1. **Variant A (recommended):** `[167][2]^6 Pt` module-scope var from a call with
   indexed signed writes — 342,016 bits, 10,688 elements. Continue the ladder.
2. **Variant B:** `[165][2]^6 Pt` bench-local packed array var from a call with
   indexed signed writes — same size, different scope.
3. **Variant C:** `[165][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes — same size, control-flow coverage.

Recommended: **Variant A**.
