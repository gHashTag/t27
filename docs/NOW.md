# Wave Loop 881 — current

- Issue: #1713 (expected)
- Branch: `wave-loop-881`
- Variant: `[581][2]^6 Pt` module-scope AoS variable from call with indexed signed writes
- Target: 37,184 elements × 32 bits = 1,189,888 bits (~1.135 MiBit)
- Plan: `.claude/plans/wave-loop-881.md`
- Status: plan ready; issue to create, branch to create

## What to do next

1. Create W881 issue and branch `wave-loop-881` from `wave-loop-880` HEAD.
2. Copy `scripts/gen_w880.py` → `scripts/gen_w881.py`, fix copy hazard (`w881`, `OUTER = 581`, `MID_IDX = 290`).
3. Generate witness, run validation gates, add integration test.
4. Closeout report + W882 cooperation variants + skills/memory updates.

---

# Wave Loop 880 — close-out / Wave Loop 881 setup (2026-08-05)

Last updated: 2026-08-05

## Wave Loop 877 — module-scope `[573][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1703)

- Branch: `wave-loop-877`
- Parent branch: `wave-loop-876` HEAD
- Issue: #1703
- PR: #1705
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W877_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-878.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w877_bench_module_573x2p6_aos_var_call_write.t27`
  - 36,672 elements, 1,173,504-bit packed vector (~1.120 MiBit).
  - Module-scope `pub var dst : [573][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w877.py`
  - Generator for the W877 witness; `OUTER = 573`, `MID_IDX = 286`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w876` / `571` / `285` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w877_bench_module_573x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w877_bench_module_573x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w877_bench_module_573x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 337/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W877: PASS.

### Research / weak points

- **Icarus Verilog:** the standard suggests 2^16 bits as a packed-dimension limit,
  but modern Icarus treats it as a soft guideline and allocates until memory is
  exhausted. At ~1.120 MiBit we are still far from any practical hard boundary.
  Icarus V13.0 (2026-03-02) improves memory management and packed/unpacked array
  handling. Open issue #1134 tracks unpacked arrays of packed structs; our
  module-scope packed array-of-struct witness does not exercise that path.
- **Vitis HLS UG1399 `compact=bit`:** commercial analog for packing structs into
  wide vectors. Maximum packed *port* width is 8192 bits (4096 for `axis`), but
  our vector is an internal module variable, so the comparison is about internal
  representation fidelity, not IO pin width.
- **Vericert / CompCert:** verified C-to-Verilog HLS framework. Vericert v2.0.0
  released 2026-01-29; the 2024 PLDI paper *Hyperblock Scheduling for Verified
  High-Level Synthesis* (DOI 10.1145/3656455) adds verified if-conversion and
  scheduling. 2026 follow-ons include Graphiti (ASPLOS) and Let It Flow (PLDI).
- **FPGA Roofline (Siracusa et al., IEEE TC 2021, DOI 10.1109/tc.2021.3111761):**
  the ladder is a memory-quanta `Q` probe; each wider vector grows the working set
  along the bandwidth axis while the compute roof stays flat. 2026 work on FPGA
  LLM inference reports on-chip BRAM/URAM bandwidths in the TB/s range versus
  HBM ~460 GB/s, confirming that internal vectors at this scale remain
  comfortably inside the on-chip memory bandwidth regime.

### Cooperation variants for Wave Loop 878

- **A (recommended):** `[575][2]^6 Pt`, outer += 2, `MID_IDX = 287`.
- **B:** `[573][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[573][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Wave Loop 878 — module-scope `[575][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1706)

- Branch: `wave-loop-878`
- Parent branch: `wave-loop-877` HEAD
- Issue: #1706
- PR: #1707
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W878_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-879.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w878_bench_module_575x2p6_aos_var_call_write.t27`
  - 36,800 elements, 1,177,600-bit packed vector (~1.124 MiBit).
  - Module-scope `pub var dst : [575][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w878.py`
  - Generator for the W878 witness; `OUTER = 575`, `MID_IDX = 287`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w877` / `573` / `286` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w878_bench_module_575x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w878_bench_module_575x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w878_bench_module_575x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 338/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W878: PASS.

### Research / weak points

- **Icarus Verilog:** the standard suggests 2^16 bits as a packed-dimension limit,
  but modern Icarus treats it as a soft guideline and allocates until memory is
  exhausted. At ~1.124 MiBit we are still far from any practical hard boundary.
  Icarus V13.0 (2026-03-02) improves memory management and packed/unpacked array
  handling. Open issue #1134 tracks unpacked arrays of packed structs; our
  module-scope packed array-of-struct witness does not exercise that path.
- **Vitis HLS UG1399 `compact=bit`:** commercial analog for packing structs into
  wide vectors. Maximum packed *port* width is 8192 bits (4096 for `axis`), but
  our vector is an internal module variable, so the comparison is about internal
  representation fidelity, not IO pin width.
- **Vericert / CompCert:** verified C-to-Verilog HLS framework. Vericert v2.0.0
  released 2026-01-29; the 2024 PLDI paper *Hyperblock Scheduling for Verified
  High-Level Synthesis* (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS)
  and Let It Flow (PLDI) provide the verified-HLS context.
- **FPGA Roofline (Siracusa et al., IEEE TC 2021, DOI 10.1109/tc.2021.3111761):**
  the ladder is a memory-quanta `Q` probe; each wider vector grows the working set
  along the bandwidth axis while the compute roof stays flat. 2026 work on FPGA
  LLM inference reports on-chip BRAM/URAM bandwidths in the TB/s range versus
  HBM ~460 GB/s, confirming that internal vectors at this scale remain
  comfortably inside the on-chip memory bandwidth regime.

### Cooperation variants for Wave Loop 881

- **A (recommended):** `[581][2]^6 Pt`, outer += 2, `MID_IDX = 290`.
- **B:** `[579][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[579][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Wave Loop 880 — module-scope `[579][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1712)

- Branch: `wave-loop-880`
- Parent branch: `wave-loop-879` HEAD
- Issue: #1712
- PR: #1720
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W880_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-881.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w880_bench_module_579x2p6_aos_var_call_write.t27`
  - 37,056 elements, 1,185,792-bit packed vector (~1.131 MiBit).
  - Module-scope `pub var dst : [579][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w880.py`
  - Generator for the W880 witness; `OUTER = 579`, `MID_IDX = 289`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w879` / `577` / `288` references. The bare
    outer-dimension number in the destination path and the stale `MID_IDX` comment
    (`286` carried from earlier waves) required a second replacement pass,
    reinforcing the checklist + post-generation sanity check.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w880_bench_module_579x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w880_bench_module_579x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w880_bench_module_579x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 340/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W880: PASS.

### Research / weak points

- **Icarus Verilog:** the standard suggests 2^16 bits as a packed-dimension limit,
  but modern Icarus treats it as a soft guideline and allocates until memory is
  exhausted. At ~1.131 MiBit we are still far from any practical hard boundary.
  Icarus V13.0 (2026-03-02) improves memory management and packed/unpacked array
  handling. Open issue #1134 tracks unpacked arrays of packed structs; our
  module-scope packed array-of-struct witness does not exercise that path.
  PR #1292 (opened 2026-01-23) is a broader fix effort for elaboration assertions.
- **Vitis HLS UG1399 `compact=bit`:** commercial analog for packing structs into
  wide vectors. Maximum packed *port* width is 8192 bits (4096 for `axis`), but
  our vector is an internal module variable, so the comparison is about internal
  representation fidelity, not IO pin width.
- **Vericert / Graphiti:** verified C-to-Verilog HLS framework. Vericert v2.0.0
  released 2026-01-29; the 2024 PLDI paper *Hyperblock Scheduling for Verified
  High-Level Synthesis* (DOI 10.1145/3656455) and 2026 ASPLOS paper *Graphiti:
  Formally Verified Out-of-Order Execution in Dataflow Circuits* (DOI 10.1145/3779212.3790166)
  provide the verified-HLS context. Graphiti reports 2.1× speedup over in-order
  dataflow HLS and 5.8× over Vericert, and caught an unsound transformation in
  the original unverified Dynamatic/DF-OoO `bicg` rewrite.
- **FPGA Roofline (Siracusa et al., IEEE TC 2021, DOI 10.1109/tc.2021.3111761):**
  the ladder is a memory-quanta `Q` probe; each wider vector grows the working set
  along the bandwidth axis while the compute roof stays flat. 2026 work on FPGA
  LLM inference reports on-chip BRAM bandwidths up to **21.8 TB/s** and URAM up to
  **10.4 TB/s** versus HBM at **~460 GB/s** on AMD Alveo U55C, confirming that
  internal vectors at this scale remain comfortably inside the on-chip memory
  bandwidth regime.

### Cooperation variants for Wave Loop 881

- **A (recommended):** `[581][2]^6 Pt`, outer += 2, `MID_IDX = 290`.
- **B:** `[579][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[579][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

