# Wave Loop 873 — current

- Issue: #1693
- Branch: `wave-loop-873`
- Variant: `[565][2]^6 Pt` module-scope AoS variable from call with indexed signed writes
- Target: 36,160 elements × 32 bits = 1,157,120 bits (~1.104 MiBit)
- Plan: `.claude/plans/wave-loop-873.md`
- Status: branch created, ready to implement

## What to do next

1. ~~Create W873 issue and branch `wave-loop-873` from `wave-loop-872` HEAD.~~ Done.
2. Copy `scripts/gen_w872.py` → `scripts/gen_w873.py`, fix copy hazard (`w873`, `OUTER = 565`, `MID_IDX = 282`).
3. Generate witness, run validation gates, add integration test.
4. Closeout report + W874 cooperation variants + skills/memory updates.

---

# Wave Loop 872 — close-out / Wave Loop 873 setup (2026-08-05)

Last updated: 2026-08-05

## Wave Loop 872 — module-scope `[563][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1691)

- Branch: `wave-loop-872`
- Parent branch: `wave-loop-871` HEAD
- Issue: #1691
- PR: TBD
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W872_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-873.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w872_bench_module_563x2p6_aos_var_call_write.t27`
  - 36,032 elements, 1,153,024-bit packed vector (~1.100 MiBit).
  - Module-scope `pub var dst : [563][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w872.py`
  - Generator for the W872 witness; `OUTER = 563`, `MID_IDX = 281`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w871` / `561` / `280` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w872_bench_module_563x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w872_bench_module_563x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w872_bench_module_563x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 332/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W872: PASS.

### Research / weak points

- **Icarus Verilog:** the standard suggests 2^16 bits as a packed-dimension limit,
  but modern Icarus treats it as a soft guideline and allocates until memory is
  exhausted. At ~1.100 MiBit we are still far from any practical hard boundary.
  Upstream discussions in 2024–2025 (issues #1171, #1134, #1180) highlight that
  the real limits are memory-allocation and expression-width edge cases, not a
  1-MiBit hard cap. Historical Icarus 0.8 had a ~256 K-entry allocator assertion
  for huge packed vectors; modern versions do not hit it at this scale.
- **Vitis HLS UG1399 `compact=bit`:** commercial analog for packing structs into
  wide vectors. Maximum packed *port* width is 8192 bits (4096 for `axis`), but
  our vector is an internal module variable, so the comparison is about internal
  representation fidelity, not IO pin width.
- **Vericert / CompCert:** verified C-to-Verilog HLS framework. The original
  Vericert paper is OOPSLA 2021 (DOI 10.1145/3485494); the 2024 PLDI paper
  *Hyperblock Scheduling for Verified High-Level Synthesis* (DOI 10.1145/3656455)
  adds verified if-conversion and scheduling. Our `t27c icarus-cocotb` gate is a
  lightweight reference-model equivalence check adjacent to that paradigm.
- **FPGA Roofline (Siracusa et al., IEEE TC 2021, DOI 10.1109/tc.2021.3111761):**
  the ladder is a memory-quanta `Q` probe; each wider vector grows the working set
  along the bandwidth axis while the compute roof stays flat. We remain on the soft,
  memory-bandwidth-limited side of the wall.

### Cooperation variants for Wave Loop 873

- **A (recommended):** `[565][2]^6 Pt`, outer += 2, `MID_IDX = 282`.
- **B:** `[563][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[563][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

# Wave Loop 871 — close-out / Wave Loop 872 setup (2026-08-05)

Last updated: 2026-08-05

## Wave Loop 871 — module-scope `[561][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1690)

- Branch: `wave-loop-871`
- Parent branch: `wave-loop-870` HEAD
- Issue: #1690
- PR: #1692
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W871_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-872.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w871_bench_module_561x2p6_aos_var_call_write.t27`
  - 35,904 elements, 1,148,928-bit packed vector (~1.096 MiBit).
  - Module-scope `pub var dst : [561][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w871.py`
  - Generator for the W871 witness; `OUTER = 561`, `MID_IDX = 280`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w870` / `559` / `279` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w871_bench_module_561x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w871_bench_module_561x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w871_bench_module_561x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 331/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W871: PASS.

### Research / weak points

- **Icarus Verilog:** the standard suggests 2^16 bits as a packed-dimension limit,
  but modern Icarus treats it as a soft guideline and allocates until memory is
  exhausted. At ~1.096 MiBit we are still far from any practical hard boundary.
  Upstream discussions in 2024–2025 (issues #1171, #1134, #1180) highlight that
  the real limits are memory-allocation and expression-width edge cases, not a
  1-MiBit hard cap. Historical Icarus 0.8 had a ~256 K-entry allocator assertion
  for huge packed vectors; modern versions do not hit it at this scale.
- **Vitis HLS UG1399 `compact=bit`:** commercial analog for packing structs into
  wide vectors. Maximum packed *port* width is 8192 bits (4096 for `axis`), but
  our vector is an internal module variable, so the comparison is about internal
  representation fidelity, not IO pin width.
- **Vericert / CompCert:** verified C-to-Verilog HLS framework. The original
  Vericert paper is OOPSLA 2021 (DOI 10.1145/3485494); the 2024 PLDI paper
  *Hyperblock Scheduling for Verified High-Level Synthesis* (DOI 10.1145/3656455)
  adds verified if-conversion and scheduling. Our `t27c icarus-cocotb` gate is a
  lightweight reference-model equivalence check adjacent to that paradigm.
- **FPGA Roofline (Siracusa et al., IEEE TC 2021, DOI 10.1109/tc.2021.3111761):**
  the ladder is a memory-quanta `Q` probe; each wider vector grows the working set
  along the bandwidth axis while the compute roof stays flat. We remain on the soft,
  memory-bandwidth-limited side of the wall.

### Cooperation variants for Wave Loop 872

- **A (recommended):** `[563][2]^6 Pt`, outer += 2, `MID_IDX = 281`.
- **B:** `[561][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[561][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
