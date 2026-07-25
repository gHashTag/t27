# NOW — Wave Loop 797 close-out / Wave Loop 798 setup (2026-07-24)

Last updated: 2026-07-24

## Wave Loop 797 — module-scope `[413][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1523)

- Branch: `wave-loop-797`
- Parent branch: `wave-loop-796` HEAD (`72cb23d77`)
- Issue: #1523
- PR: #1524 (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W797_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-797.md`
- Cooperation W798: `.claude/plans/wave-loop-798.md`

### What landed
- `specs/scratch/w797_bench_module_413x2p6_aos_var_call_write.t27`
  - 26,432 elements, 845,824-bit packed vector (~0.807 MiBit).
  - Module-scope `pub var dst : [413][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w797.py`
  - Generator for the W797 witness; `OUTER = 413`, `MID_IDX = 206`.
  - Note: both the destination path and the module header f-string had to be
    manually fixed after copying from W796 (generator copy hazard).
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w797_bench_module_413x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w797_bench_module_413x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-798.md`
  - W797 learnings saved and W798 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 257/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W797: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.

---

## Wave Loop 798 — module-scope `[415][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-798`
- Parent branch: `wave-loop-797` HEAD (after closeout)
- Issue: TBD after W797 PR opened
- PR: (to open)
- Plan: `.claude/plans/wave-loop-798.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[415][2]^6 Pt`.
Expected 26,560 elements, 849,920-bit packed vector (~0.810 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[415][2]^6 Pt` module-scope var from call.
- **B:** `[413][2]^6 Pt` bench/function-scope packed var from call.
- **C:** `[413][2]^6 Pt` module-scope var with `if`-guarded writes.

---

# NOW — Wave Loop 795 close-out / Wave Loop 796 setup (2026-07-24)

Last updated: 2026-07-24

## Wave Loop 795 — module-scope `[409][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1519)

- Branch: `wave-loop-795`
- Parent branch: `wave-loop-794` HEAD (`0d7475997`)
- Issue: #1519
- PR: #1520
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W795_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-795.md`
- Cooperation W796: `.claude/plans/wave-loop-796.md`

### What landed
- `specs/scratch/w795_bench_module_409x2p6_aos_var_call_write.t27`
  - 26,176 elements, 837,632-bit packed vector (~0.799 MiBit).
  - Module-scope `pub var dst : [409][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w795.py`
  - Generator for the W795 witness; `OUTER = 409`, `MID_IDX = 204`.
  - Note: both the destination path and the module header f-string had to be
    manually fixed after copying from W794 (generator copy hazard).
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w795_bench_module_409x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-796.md`
  - W795 learnings saved and W796 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (627 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 255/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W795: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 627 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject ~9.4% (99/1058), down from earlier waves; keep closing
  references in commit subjects.

---

## Wave Loop 796 — module-scope `[411][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-796`
- Parent branch: `wave-loop-795` HEAD (after closeout)
- Issue: TBD after W795 PR opened
- PR: (to open)
- Plan: `.claude/plans/wave-loop-796.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[411][2]^6 Pt`.
Expected 26,304 elements, 841,728-bit packed vector (~0.803 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[411][2]^6 Pt` module-scope var from call.
- **B:** `[409][2]^6 Pt` bench/function-scope packed var from call.
- **C:** `[409][2]^6 Pt` module-scope var with `if`-guarded writes.

---

# NOW — Wave Loop 794 close-out / Wave Loop 795 setup (2026-07-24)

Last updated: 2026-07-24

## Wave Loop 794 — module-scope `[407][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1517)

- Branch: `wave-loop-794`
- Parent branch: `wave-loop-793` HEAD (`d92cc7dfb`)
- Issue: #1517
- PR: #1518
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W794_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-794.md`
- Cooperation W795: `.claude/plans/wave-loop-795.md`

### What landed
- `specs/scratch/w794_bench_module_407x2p6_aos_var_call_write.t27`
  - 26,048 elements, 833,536-bit packed vector (~0.795 MiBit).
  - Module-scope `pub var dst : [407][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w794.py`
  - Generator for the W794 witness; `OUTER = 407`, `MID_IDX = 203`.
  - Note: both the destination path and the module header f-string had to be
    manually fixed after copying from W793 (generator copy hazard).
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w794_bench_module_407x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-795.md`
  - W794 learnings saved and W795 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 254/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W794: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 627 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject ~9.4% (99/1058), down from W793; keep closing
  references in commit subjects.

---

## Wave Loop 795 — module-scope `[409][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-795`
- Parent branch: `wave-loop-794` HEAD (after closeout)
- Issue: TBD after W794 PR opened
- PR: (to open)
- Plan: `.claude/plans/wave-loop-795.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[409][2]^6 Pt`.
Expected 26,176 elements, 837,632-bit packed vector (~0.799 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[409][2]^6 Pt` module-scope var from call.
- **B:** `[407][2]^6 Pt` bench/function-scope packed var from call.
- **C:** `[407][2]^6 Pt` module-scope var with `if`-guarded writes.

---

# NOW — Wave Loop 793 close-out / Wave Loop 794 setup (2026-07-24)

Last updated: 2026-07-24

## Wave Loop 793 — module-scope `[405][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1515)

- Branch: `wave-loop-793`
- Parent branch: `wave-loop-792` HEAD (`c327d1aaa`)
- Issue: #1515
- PR: #1516
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W793_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-793.md`
- Cooperation W794: `.claude/plans/wave-loop-794.md`

### What landed
- `specs/scratch/w793_bench_module_405x2p6_aos_var_call_write.t27`
  - 25,920 elements, 829,440-bit packed vector (~0.791 MiBit).
  - Module-scope `pub var dst : [405][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w793.py`
  - Generator for the W793 witness; `OUTER = 405`, `MID_IDX = 202`.
  - Note: the generator destination path had to be manually fixed from `403` to `405`
    after copying from W792, then regenerated to match the correct module name.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w793_bench_module_405x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-794.md`
  - W793 learnings saved and W794 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 253/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W793: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject ~16.7% (15/90), up slightly from W792; keep
  closing references in commit subjects.

---

## Wave Loop 794 — module-scope `[407][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-794`
- Parent branch: `wave-loop-793` HEAD (after closeout)
- Issue: TBD after W793 PR opened
- PR: (to open)
- Plan: `.claude/plans/wave-loop-794.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[407][2]^6 Pt`.
Expected 26,048 elements, 833,536-bit packed vector (~0.795 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[407][2]^6 Pt` module-scope var from call.
- **B:** `[405][2]^6 Pt` bench/function-scope packed var from call.
- **C:** `[405][2]^6 Pt` module-scope var with `if`-guarded writes.

---

# NOW — Wave Loop 789 close-out / Wave Loop 790 setup (2026-07-24)

Last updated: 2026-07-24

## Wave Loop 789 — module-scope `[397][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1507)

- Branch: `wave-loop-789`
- Parent branch: `wave-loop-788` HEAD (`44fa559e7`)
- Issue: #1507
- PR: #1508
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W789_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-789.md`
- Cooperation W790: `.claude/plans/wave-loop-790.md`

### What landed
- `specs/scratch/w789_bench_module_397x2p6_aos_var_call_write.t27`
  - 25,408 elements, 813,056-bit packed vector (~0.775 MiBit).
  - Module-scope `pub var dst : [397][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w789.py`
  - Generator for the W789 witness; `OUTER = 397`, `MID_IDX = 198`.
  - Note: the generator header had a hardcoded `w788` prefix inside an f-string,
    which required a manual fix and regeneration before the module name matched
    the wave number.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w789_bench_module_397x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-790.md`
  - W789 learnings saved and W790 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 249/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W789: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).

---

## Wave Loop 790 — module-scope `[399][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1509)

- Branch: `wave-loop-790`
- Parent branch: `wave-loop-789` HEAD (`228e1d850`)
- Issue: #1509
- PR: #1510
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W790_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-790.md`
- Cooperation W791: `.claude/plans/wave-loop-791.md`

### What landed
- `specs/scratch/w790_bench_module_399x2p6_aos_var_call_write.t27`
  - 25,536 elements, 817,152-bit packed vector (~0.779 MiBit).
  - Module-scope `pub var dst : [399][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w790.py`
  - Generator for the W790 witness; `OUTER = 399`, `MID_IDX = 199`.
  - Note: the generator header had a hardcoded `w789` prefix inside an f-string,
    which required a manual fix and regeneration before the module name matched
    the wave number.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w790_bench_module_399x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-791.md`
  - W790 learnings saved and W791 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 250/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W790: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by commit subject dropped to 0.0% (0/87); closing refs are
  in commit bodies, not subjects.

---

## Wave Loop 791 — module-scope `[401][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-791`
- Parent branch: `wave-loop-790` HEAD (after closeout)
- Issue: TBD after W790 PR opened
- PR: (to open)
- Plan: `.claude/plans/wave-loop-791.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[401][2]^6 Pt`.
Expected 25,664 elements, 821,248-bit packed vector (~0.783 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[401][2]^6 Pt` module-scope var from call.
- **B:** `[399][2]^6 Pt` bench/function-scope packed var from call.
- **C:** `[399][2]^6 Pt` module-scope var with `if`-guarded writes.

---

## Wave Loop 784 — module-scope `[387][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1497)

- Branch: `wave-loop-784`
- Parent branch: `wave-loop-783` HEAD (`7f2c7afb4`)
- Issue: #1497
- PR: (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W784_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-784.md`
- Cooperation W785: `.claude/plans/wave-loop-785.md`

### What landed
- `specs/scratch/w784_bench_module_387x2p6_aos_var_call_write.t27`
  - 24,768 elements, 792,576-bit packed vector (~0.756 MiBit).
  - Module-scope `pub var dst : [387][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w784.py`
  - Generator for the W784 witness; `OUTER = 387`, `MID_IDX = 193`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w784_bench_module_387x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-785.md`
  - W784 learnings saved and W785 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 244/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W784: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).

---

## Wave Loop 783 — module-scope `[385][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1495)

- Branch: `wave-loop-783`
- Parent branch: `wave-loop-782` HEAD (`753197599`)
- Issue: #1495
- PR: (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W783_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-783.md`
- Cooperation W784: `.claude/plans/wave-loop-784.md`

### What landed
- `specs/scratch/w783_bench_module_385x2p6_aos_var_call_write.t27`
  - 24,640 elements, 788,480-bit packed vector (~0.752 MiBit).
  - Module-scope `pub var dst : [385][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w783.py`
  - Generator for the W783 witness; `OUTER = 385`, `MID_IDX = 192`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w783_bench_module_385x2p6_aos_var_call_write`.
- Weak-point fix in this closeout:
  - `bootstrap/tests/verilog_const_array.rs:166` — relaxed stale TODO expectation
    to accept any `TODO: array literal` or `TODO: struct literal` substring, matching
    the richer emitter diagnostic format.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-784.md`
  - W783 learnings saved and W784 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 243/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W783: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).

---

## Wave Loop 782 — module-scope `[383][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1493)

- Branch: `wave-loop-782`
- Parent branch: `wave-loop-781` HEAD (`a61465608`)
- Issue: #1493
- PR: (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W782_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-782.md`
- Cooperation W783: `.claude/plans/wave-loop-783.md`

### What landed
- `specs/scratch/w782_bench_module_383x2p6_aos_var_call_write.t27`
  - 24,512 elements, 784,384-bit packed vector (~0.748 MiBit).
  - Module-scope `pub var dst : [383][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w782.py`
  - Generator for the W782 witness; `OUTER = 383`, `MID_IDX = 191`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w782_bench_module_383x2p6_aos_var_call_write`.
- Weak-point fix in this closeout:
  - `bootstrap/src/host/telemetry.rs:242` — replaced literal `3.14` with
    `std::f64::consts::PI` to keep `cargo clippy -p t27c` green.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-783.md`
  - W782 learnings saved and W783 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 242/0.
- Direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W782: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing).
- FPGA E2E CI red.
- 626 release / 780 clippy warnings.
- Vivado-in-Docker CI gap.

---

## Standing process debt

- Open PR stack awaiting review: W774-W785.
- 30-day commit traceability is low (~15–20% of commit subjects carry
  `Closes #N` / `Fixes #N`).
- FPGA synthesis CI is blocked on the Yosys static-cast issue in `uart.v`.

---

φ² + 1/φ² = 3 | TRINITY
