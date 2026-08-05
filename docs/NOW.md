# NOW — Wave Loop 867 close-out / Wave Loop 868 setup (2026-08-05)

Last updated: 2026-08-05

## Wave Loop 867 — module-scope `[553][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1682)

- Branch: `wave-loop-867`
- Parent branch: `wave-loop-866` HEAD
- Issue: #1682
- PR: TBD
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W867_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-868.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w867_bench_module_553x2p6_aos_var_call_write.t27`
  - 35,392 elements, 1,132,544-bit packed vector (~1.080 MiBit).
  - Module-scope `pub var dst : [553][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w867.py`
  - Generator for the W867 witness; `OUTER = 553`, `MID_IDX = 276`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w866` / `551` / `275` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w867_bench_module_553x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w867_bench_module_553x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w867_bench_module_553x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 327/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W867: PASS.

### Research / weak points

- **Icarus Verilog:** the standard suggests 2^16 bits as a packed-dimension limit,
  but modern Icarus treats it as a soft guideline and allocates until memory is
  exhausted. At ~1.080 MiBit we are still far from any practical hard boundary.
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
  the ladder is a memory-quanta `Q` probe; each wider vector grows the working
  set along the bandwidth axis while the compute roof stays flat. We remain on
  the soft, memory-bandwidth-limited side of the wall.

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

## Wave Loop 868 — module-scope `[555][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-868`
- Parent branch: `wave-loop-867` HEAD (after closeout)
- Issue: #1684 (expected)
- PR: TBD
- Plan: `.claude/plans/wave-loop-868.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### Goal

Continue the odd outer-dimension module-scope AoS ladder with `[555][2]^6 Pt`.
Expected 35,520 elements, 1,136,640-bit packed vector (~1.084 MiBit), still well
under the 4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants

- **A (recommended):** `[555][2]^6 Pt` module-scope var from call.
- **B:** `[553][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[553][2]^6 Pt` with negative-index writes to exercise wrap-around.

---

*φ² + φ⁻² = 3 | TRINITY*
