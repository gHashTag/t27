# Wave Loop 878 — current

- Issue: #1704
- Branch: `wave-loop-878`
- Variant: `[575][2]^6 Pt` module-scope AoS variable from call with indexed signed writes
- Target: 36,800 elements × 32 bits = 1,177,600 bits (~1.124 MiBit)
- Plan: `.claude/plans/wave-loop-878.md`
- Status: branch created, ready to implement

## What to do next

1. Create W878 issue and branch `wave-loop-878` from `wave-loop-877` HEAD.
2. Copy `scripts/gen_w877.py` → `scripts/gen_w878.py`, fix copy hazard (`w878`, `OUTER = 575`, `MID_IDX = 287`).
3. Generate witness, run validation gates, add integration test.
4. Closeout report + W879 cooperation variants + skills/memory updates.

---

# Wave Loop 877 — close-out / Wave Loop 878 setup (2026-08-05)

Last updated: 2026-08-05

## Wave Loop 877 — module-scope `[573][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1703)

- Branch: `wave-loop-877`
- Parent branch: `wave-loop-876` HEAD
- Issue: #1703
- PR: TBD (GitHub-assigned on open)
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

