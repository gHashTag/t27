# NOW — Wave Loop 859 close-out / Wave Loop 860 setup (2026-08-05)

Last updated: 2026-08-05

## Wave Loop 859 — module-scope `[537][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1662)

- Branch: `wave-loop-859`
- Parent branch: `wave-loop-858` HEAD
- Issue: #1662
- PR: TBD
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W859_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-860.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w859_bench_module_537x2p6_aos_var_call_write.t27`
  - 34,368 elements, 1,099,776-bit packed vector (~1.049 MiBit).
  - Module-scope `pub var dst : [537][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w859.py`
  - Generator for the W859 witness; `OUTER = 537`, `MID_IDX = 268`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w858` / `535` / `267` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w859_bench_module_537x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w859_bench_module_537x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w859_bench_module_537x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 319/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W859: PASS.

### Research / weak points

- Icarus Verilog has no documented 1-MiBit hard cap; the LRM only requires 65,536-bit
  packed-array support and Icarus warns around 1 Gbit. Upstream commit `128c621`
  fixed a bound-normalization path that could accidentally create billion-bit vectors.
  Historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed vectors;
  modern versions do not hit it.
- Siracusa et al. (IEEE TC 2021) FPGA Roofline model frames the ladder as a probe
  of memory quanta `Q` growth.
- Vericert/CompCert provide verified-compilation analogs for bit-exact source-to-hardware
  mapping.
- Vitis HLS UG1399 provides the commercial analog for packed interface structs.

### Remaining weak points

- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- Release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit subjects.
- Generator copy hazard persists; parameterize `WAVE`/`OUTER` in the template.
- Full `./scripts/tri test` suite stalls on the pre-existing `w589_bench_module_17d_aos_var_call_write.t27` parse phase and was not completed this wave.

---

## Wave Loop 860 — module-scope `[539][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-860`
- Parent branch: `wave-loop-859` HEAD (after closeout)
- Issue: #1664 (expected)
- PR: TBD
- Plan: `.claude/plans/wave-loop-860.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### Goal

Continue the odd outer-dimension module-scope AoS ladder with `[539][2]^6 Pt`.
Expected 34,496 elements, 1,103,872-bit packed vector (~1.052 MiBit), still well
under the 4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants

- **A (recommended):** `[539][2]^6 Pt` module-scope var from call.
- **B:** `[537][3]^6 Pt` — grow second inner dimension to stress stride scaling.
- **C:** `[537][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
