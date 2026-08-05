# Wave Loop 877 — current

- Issue: #1702
- Branch: `wave-loop-877`
- Variant: `[573][2]^6 Pt` module-scope AoS variable from call with indexed signed writes
- Target: 36,672 elements × 32 bits = 1,173,504 bits (~1.120 MiBit)
- Plan: `.claude/plans/wave-loop-877.md`
- Status: branch created, ready to implement

## What to do next

1. Create W877 issue and branch `wave-loop-877` from `wave-loop-876` HEAD.
2. Copy `scripts/gen_w876.py` → `scripts/gen_w877.py`, fix copy hazard (`w877`, `OUTER = 573`, `MID_IDX = 286`).
3. Generate witness, run validation gates, add integration test.
4. Closeout report + W878 cooperation variants + skills/memory updates.

---

# Wave Loop 876 — close-out / Wave Loop 877 setup (2026-08-05)

Last updated: 2026-08-05

## Wave Loop 876 — module-scope `[571][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1701)

- Branch: `wave-loop-876`
- Parent branch: `wave-loop-875` HEAD
- Issue: #1701
- PR: TBD (GitHub-assigned on open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W876_2026-08-05.md`
- Plan: `.claude/plans/wave-loop-877.md`
- Autopilot: `.claude/skills/wave-loop-autopilot.md`
- Master plan: `.claude/skills/wave-loop-master-plan.md`

### What landed

- `specs/scratch/w876_bench_module_571x2p6_aos_var_call_write.t27`
  - 36,544 elements, 1,169,408-bit packed vector (~1.116 MiBit).
  - Module-scope `pub var dst : [571][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w876.py`
  - Generator for the W876 witness; `OUTER = 571`, `MID_IDX = 285`.
  - Copy hazard fixed: destination path, module header f-string, and `MID_IDX`
    comment updated from stale `w875` / `569` / `284` references.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added integration test `accepts_w876_bench_module_571x2p6_aos_var_call_write`.
- `.trinity/seals/scratch_w876_bench_module_571x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification

- `cargo build --release -p t27c`: OK (warnings, 0 errors).
- `cargo test --release --test icarus_lowerable accepts_w876_bench_module_571x2p6_aos_var_call_write`: 1/0.
- `cargo test --release --test icarus_lowerable` (full suite): 336/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W876: PASS.

### Research / weak points

- **Icarus Verilog:** the standard suggests 2^16 bits as a packed-dimension limit,
  but modern Icarus treats it as a soft guideline and allocates until memory is
  exhausted. At ~1.116 MiBit we are still far from any practical hard boundary.
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

### Cooperation variants for Wave Loop 877

- **A (recommended):** `[573][2]^6 Pt`, outer += 2, `MID_IDX = 286`.
- **B:** `[571][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[571][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

