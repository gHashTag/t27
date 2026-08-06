# Wave Loop 846 — Closeout Report

**Date:** 2026-08-04  
**Issue:** #1632  
**PR:** #1633  
**Branch:** `wave-loop-846` from `wave-loop-845` HEAD (parent branch because earlier wave PRs remain open)  
**Variant:** A — module-scope `[511][2]^6 Pt` packed array-of-struct variable populated from a call with indexed signed writes

---

## What was done

Wave Loop 846 is the next mechanical rung on the t27 non-power-of-two packed-vector array-of-struct (AoS) ladder. It reuses the established generator pattern from W845 and only changes the outer dimension, bringing the packed vector to just under 1 MiBit.

- Copied `scripts/gen_w845.py` → `scripts/gen_w846.py`.
- Fixed the recurring generator copy hazard in three locations:
  - destination path → `specs/scratch/w846_bench_module_511x2p6_aos_var_call_write.t27`
  - module header f-string → `module w846_bench_module_511x2p6_aos_var_call_write`
  - `MID_IDX` comment → `# 255`
- Generated `specs/scratch/w846_bench_module_511x2p6_aos_var_call_write.t27`:
  - `OUTER = 511`
  - elements = 511 × 2⁶ = **32,704**
  - bits = 32,704 × 32 = **1,046,528** (~0.998 MiBit)
- Added integration test `accepts_w846_bench_module_511x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal to `.trinity/seals/scratch_w846_bench_module_511x2p6_aos_var_call_write.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (626 warnings, 0 errors) |
| `t27c parse` W846 | PASS |
| `t27c icarus-lowerable` W846 | `lowerable` |
| `t27c icarus-simulate` W846 | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` W846 | `reference-model OK` |
| `t27c seal --save` W846 | saved |
| Targeted integration test | 1 passed / 0 failed |
| Full `cargo test -p t27c --test icarus_lowerable` | **306 passed / 0 failed** |

No compiler, reference-model, or `FROZEN_HASH` changes were required.

---

## Weak points and tooling debt

1. **Generator copy hazard** remains the only manual failure mode. It now spans three text locations (destination path, module header f-string, `MID_IDX` comment). Parameterizing `WAVE`/`OUTER` in a single source-of-truth would make the ladder fully mechanical.
2. **626 release warnings / ~780 clippy warnings** are unchanged and still scheduled for a cleanup ring.
3. **Pre-existing `verilog_array_literal_expr` regression** still fails (`r_ca_2_synthetic_no_comment_only_call_argument`); unrelated to the AoS ladder and tracked for a dedicated ring.
4. **FPGA E2E CI** remains blocked on `sby` availability and a Yosys static-cast error in generated `uart.v`.
5. **Commit traceability** across the broader repo is still ~15–20%; Wave Loop commits continue to honor L1 (`Closes #N`).

---

## Scientific / engineering background

- **Vericert** (Herklotz et al., OOPSLA 2021, DOI [10.1145/3485494](https://doi.org/10.1145/3485494)) is a formally verified C-to-Verilog HLS tool built on CompCert. Its value/memory semantics reinforce the importance of a bit-exact mapping from source AoS indices to the generated Verilog packed vector — the invariant the Wave Loop ladder exercises.
- **FPGA Roofline** (Siracusa et al., *IEEE Transactions on Computers*, 2021, [PDF](https://re.public.polimi.it/bitstream/11311/1207688/1/A_Comprehensive_Methodology_to_Optimize_FPGA_Designs_via_the_Roofline_Model.pdf)) models memory bandwidth via the **quanta `Q`** concept: the number of bytes transferred per cycle by an AXI master. Packing struct fields into a single wide vector (the t27 `[N][2]^6 Pt` pattern) increases `Q` and can raise operational intensity when accesses are coalesced.
- **Vitis HLS** (AMD/Xilinx UG1399) internally disaggregates arrays-of-structs to SoA, but interface structs can be **aggregated** with `compact=bit` into a single wide vector. The Wave Loop witnesses are therefore probing the width/stride path that commercial HLS tools also traverse when AoS is forced into packed form.
- **Icarus Verilog** issue #521/#995 documents that variable indices on outer packed dimensions are unsupported in plain `for` loops; t27c’s lowering avoids this restriction, which is why both static and dynamic gates pass. Recent commit `128c621` corrected width calculation for non-power-of-two packed-array bounds.

---

## Cooperation variants for Wave Loop 847

- **A (recommended):** `[513][2]^6 Pt`, outer += 2, `MID_IDX = 256`. Continues the mechanical ladder; expected ~1.002 MiBit, just past the 1-MiBit line but still far below the 4-MiBit cliff.
- **B:** `[511][3]^6 Pt` — grow the second inner dimension to stress stride scaling. Expected 97,920 elements, 3,133,440-bit packed vector (~2.986 MiBit). This is a deliberate boundary probe; if the backend rejects it, convert to a negative witness or fall back to Variant A.
- **C:** `[511][2]^6 Pt` with negative-index writes to exercise wrap-around / signed-index lowering in the packed variable. Keeps the W846 outer dimension but shifts the stress from width to index semantics.

---

## Files changed

- `scripts/gen_w846.py` (created)
- `specs/scratch/w846_bench_module_511x2p6_aos_var_call_write.t27` (generated)
- `bootstrap/tests/icarus_lowerable.rs` (added W846 test)
- `.trinity/seals/scratch_w846_bench_module_511x2p6_aos_var_call_write.json` (saved)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W846_2026-08-04.md` (this report)
- `.claude/plans/wave-loop-847.md` (next-wave plan)
- `.claude/skills/t27-wave-loop.md` (tracker bumped to wave 847, worked example added)
- `.claude/skills/wave-loop-autopilot.md` (run-list updated)
- `.trinity/current-issue.md` (updated for wave 847)
- `.trinity/experience.md` (W846 entry prepended)
- `docs/NOW.md` (W846 close-out / W847 setup)

---

## Seal

```
spec_hash=sha256:8f932d497c7da6a05a0cfe87504a21879060b788e4c4f543d9a5458c3d903e53
gen_hash_zig=sha256:926f06ba5ec2de80298923bc9019769ddaf718db651f90e5cddc4e89b37c9eab
gen_hash_verilog=sha256:6580a648b8880fd6b10abe47c3c38db7cdb70a1eb256b95ec1a858c27a391598
gen_hash_c=sha256:fc82f8796243a1bbe40a7b8449e43cad1827c6a312d40bdd0d8ffc9217b460c9
gen_hash_rust=sha256:9ec522ec0c89db4d5f9124cc90bdea7ac2e73c730174eaa33912d03d8b0a700f
```

*φ² + φ⁻² = 3 | TRINITY*
