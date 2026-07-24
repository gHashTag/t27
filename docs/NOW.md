# NOW — Wave Loop 786 close-out / Wave Loop 787 setup (2026-07-24)

Last updated: 2026-07-24

## Wave Loop 786 — module-scope `[391][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1501)

- Branch: `wave-loop-786`
- Parent branch: `wave-loop-785` HEAD (`af5c29cca`)
- Issue: #1501
- PR: #1502
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W786_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-786.md`
- Cooperation W787: `.claude/plans/wave-loop-787.md`

### What landed
- `specs/scratch/w786_bench_module_391x2p6_aos_var_call_write.t27`
  - 25,024 elements, 800,768-bit packed vector (~0.763 MiBit).
  - Module-scope `pub var dst : [391][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w786.py`
  - Generator for the W786 witness; `OUTER = 391`, `MID_IDX = 195`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w786_bench_module_391x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-787.md`
  - W786 learnings saved and W787 plan/cooperation variants created.

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
- `cargo test -p t27c --test icarus_lowerable`: 246/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W786: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).

---

## Wave Loop 787 — module-scope `[393][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-787`
- Parent branch: `wave-loop-786` HEAD (after closeout)
- Issue: TBD after W786 PR opened
- PR: (to open)
- Plan: `.claude/plans/wave-loop-787.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[393][2]^6 Pt`.
Expected 25,152 elements, 804,864-bit packed vector (~0.767 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[393][2]^6 Pt` module-scope var from call.
- **B:** `[391][2]^6 Pt` bench/function-scope packed var from call.
- **C:** `[391][2]^6 Pt` module-scope var with `if`-guarded writes.

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
