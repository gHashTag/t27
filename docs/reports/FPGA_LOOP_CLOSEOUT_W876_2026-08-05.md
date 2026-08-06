# Wave Loop 876 — Closeout Report

**Date:** 2026-08-05  
**Issue:** [#1701](https://github.com/gHashTag/t27/issues/1701)  
**Branch:** `wave-loop-876`  
**Parent:** `wave-loop-875` HEAD (earlier waves' PRs remain open)  
**PR:** [#1704](https://github.com/gHashTag/t27/pull/1704)  
**Author:** Trinity Agent (Claude Code t27)

---

## 1. What we built

Wave Loop 876 continues the module-scope packed array-of-struct ladder, selecting
Variant A from the W876 plan:

```text
module-scope [571][2]^6 Pt variable from call with indexed signed writes
```

- `Pt { x : i16, y : i16 }` → 32 bits/element.
- Outer dimension `571` is odd and non-power-of-two, preserving the boundary-stress
  cadence established by W837+.
- Inner shape `[2]^6` = 64 elements/row.
- Total elements: `571 × 64 = 36,544`.
- Packed vector width: `36,544 × 32 = 1,169,408 bits` ≈ **1.116 MiBit**.

Artifacts produced:

| Artifact | Path | Notes |
|----------|------|-------|
| Generator | `scripts/gen_w876.py` | Copied from `gen_w875.py`; copy hazard fixed before first run (`w876`, `OUTER = 571`, `MID_IDX = 285`). |
| Spec | `specs/scratch/w876_bench_module_571x2p6_aos_var_call_write.t27` | 108,551 lines, ~2.38 MB. |
| Seal | `.trinity/seals/scratch_w876_bench_module_571x2p6_aos_var_call_write.json` | Saved by `t27c seal --save`. |
| Test | `bootstrap/tests/icarus_lowerable.rs` | `accepts_w876_bench_module_571x2p6_aos_var_call_write`. |

---

## 2. Weak-point investigation

### 2.1 Icarus Verilog packed-vector behavior at 1+ MiBit

The practical landscape remains as observed in W837–W875:

- **Issue #1171** (2024): Icarus can exhaust memory when a constant-expression bug
  produces a vector in the exa-bit range. Maintainer *caryr* notes that the standard
  *suggests* limits of **2^16 bits for packed dimensions** and **2^24 bits for unpacked
  dimensions**, but Icarus today does not hard-enforce them; it tries to allocate until
  it runs out of memory. Better diagnostics are being discussed.
- **Issue #1322** (2026): Discusses silent no-ops on dynamic out-of-bounds unpacked
  array writes. This is IEEE 1800-2017 §7.4.6 compliant but highlights that Icarus does
  not yet emit runtime warnings for such accesses; our generator uses deterministic,
  in-bounds scalar indices so this does not apply directly.
- **Commit `128c621`** (2026-06-20, *“Fix width calculation for packed array bounds”*):
  Lars-Peter Clausen corrected min-width calculation for packed array bounds so that
  negative bound values no longer convert to huge unsigned widths. This is the most
  direct upstream fix related to indexed packed-array correctness at large widths.
- **Issue #1134** (2024–2025): Packed arrays of structs still have corner cases,
  especially when indexed in unusual ways. Our `[N][2]^6 Pt` pattern (uniform packed
  array of structs with signed scalar indices) remains on the well-supported path.
- **Issue #1180** (late 2024, now closed with better diagnostics): Packed/unpacked
  array **parameters** are a SystemVerilog feature not fully supported in Icarus;
  the generator path here does not use them.
- Historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed vectors;
  modern versions do not hit it at this scale.

**Interpretation:** W876's 1.116-MiBit vector is still comfortably below both the
historical allocator assertion zone and the standard's 2^16-bit *suggestion* when read
as a per-dimension limit. In practice, Icarus accepts the vector because it is a single
large packed object, not a multi-dimensional packed parameter or an expression-width
edge case. No new limit appeared between W875 and W876.

### 2.2 Vitis HLS `compact=bit` analog

AMD/Xilinx Vitis HLS documentation (UG1399) continues to provide the commercial
reference frame:

- `#pragma HLS aggregate variable=... compact=bit` packs struct members tightly into a
  single wide vector, first member in LSB, last in MSB.
- For arrays of structs, the result is an **array where each element is the packed
  struct vector**.
- Maximum packed port width: **8192 bits** for most interfaces, **4096 bits** for `axis`.
- `compact=bit` is disallowed for AXI4 interfaces; those use `compact=none` (padded to
  alignment).

**Interpretation for t27:** our generated internal vector (`1,169,408 bits`) is far wider
than a Vitis interface port limit, but it is not an external port — it is an internal
module variable. The relevant comparison is the *internal* representation fidelity, not
an IO pin limit. t27c and Icarus are effectively doing internally what Vitis HLS does
with `compact=bit`: storing a packed struct array as one wide bit-vector and indexing
into it. W876 adds another 4,096 bits (128 elements) to that internal vector without
changing the access pattern.

### 2.3 Vericert / CompCert and verified-hardware compilation landscape

The verified-HLS analog remains the most relevant scientific anchor:

- **Vericert v2.0.0** was released on **2026-01-29** ([github.com/ymherklotz/vericert](https://github.com/ymherklotz/vericert)).
- **“Hyperblock Scheduling for Verified High-Level Synthesis”** (Herklotz & Wickerson,
  PACMPL Vol. 8, PLDI 2024, [DOI 10.1145/3656455](https://doi.org/10.1145/3656455))
  is the latest Vericert paper, adding mechanically verified if-conversion and
  hyperblock scheduling to the CompCert-based C-to-Verilog pipeline.
- **“Graphiti: Formally Verified Out-of-Order Execution in Dataflow Circuits”**
  (Herklotz et al., ASPLOS 2026 Vol. 2, [DOI 10.1145/3779212.3790166](https://doi.org/10.1145/3779212.3790166))
  extends the verified-hardware compilation agenda to out-of-order dataflow circuits
  using Lean 4.
- **“Let It Flow: A Formally Verified Compilation Framework for Asynchronous Dataflow”**
  (Lin, Cai, Surbatovich, PLDI 2026, [DOI 10.1145/3808263](https://doi.org/10.1145/3808263))
  is another 2026 verified hardware/dataflow entry in the same research direction.

**Interpretation for t27:** the t27 compiler does not yet have a Coq-verified backend,
but its `icarus-cocotb` gate is a lightweight reference-model equivalence check adjacent
 to the verified-HLS paradigm. Each wave exercises the same correctness question:
 “does the generated Verilog preserve the t27 reference semantics for an ever-larger
 packed data structure?”

### 2.4 FPGA Roofline and on-chip/HBM memory-bandwidth context

The FPGA Roofline model (Siracusa et al., IEEE TC 2021,
[DOI 10.1109/tc.2021.3111761](https://doi.org/10.1109/tc.2021.3111761)) frames the ladder
as a memory-quanta probe:

- Each wider vector grows the working set along the **bandwidth axis**.
- The **compute roof** stays flat because the test performs a constant number of indexed
  reads/writes per cycle.
- W876 adds 128 elements × 32 bits = 4,096 bits to the packed vector, a ~0.35 % step
  relative to W875.

Recent 2025–2026 FPGA literature reinforces this framing:

- **TerEffic** (arXiv 2502.16473v2, 2025) uses an explicit Roofline analysis for
  ternary LLM inference on AMD Alveo U280, comparing a fully on-chip design against
  an HBM-assisted design. They report the HBM-assisted variant is memory-bound at
  small batch sizes and transitions to compute-bound only above a batch-size threshold
  of ~4.3. The on-chip variant avoids the HBM bandwidth wall entirely for models that
  fit in the ~42 MB of SRAM.
- **LUT-LLM** (arXiv 2511.06174v1, 2026) presents a detailed Roofline analysis of
  Qwen 3 1.7B on AMD Alveo V80 (HBM2e, ~819 GB/s peak bandwidth), showing how
  vector-quantized lookup tables trade memory-port pressure against compute density.
- **Memory Sandbox 2.0** (MEMSYS 2025, [DOI 10.1145/3767110.3767114](https://doi.org/10.1145/3767110.3767114))
  benchmarks HBM2/HBM2e on Xilinx/AMD FPGAs and notes that real achievable bandwidth
  can be up to 50 % below nominal due to microswitch routing effects. It also projects
  **HBM4 in 2026** at up to 2 TB/s per stack.

**Interpretation:** the ladder is still a memory-quanta probe, not an IO-port probe,
because the vector is internal. The 1.116-MiBit W876 vector is trivial compared to
on-chip SRAM capacities (tens of MB) and still well below any HBM interface cliff,
so no bandwidth limit is expected from this step.

---

## 3. What did not change

- `bootstrap/src/compiler.rs` — no edits.
- `bootstrap/stage0/FROZEN_HASH` — unchanged at `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — no edits.
- No new shell scripts on the critical path (L7 UNITY).

---

## 4. Validation matrix

| Gate | Command | Result |
|------|---------|--------|
| Build | `cargo build --release -p t27c` | OK (warnings, 0 errors) |
| Parse | `./target/release/t27c parse specs/scratch/w876_bench_module_571x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `./target/release/t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `./target/release/t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `./target/release/t27c icarus-cocotb ...` | `reference-model OK` |
| Seal | `./target/release/t27c seal --save ...` | seal saved |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w876_bench_module_571x2p6_aos_var_call_write` | 1/0 |
| Full suite | `cargo test --release --test icarus_lowerable` | **336/0** |

---

## 5. Integration

- [x] Generator `scripts/gen_w876.py` with copy hazard fixed.
- [x] Witness generated and all direct gates green.
- [x] Integration test added immediately after W875's test.
- [x] `FROZEN_HASH` verified unchanged.
- [x] Closeout report written.
- [x] Commit with `Closes #1701`, push branch `wave-loop-876`, open PR to `master`.
- [x] Create W877 issue and branch `wave-loop-877` from `wave-loop-876` HEAD.

---

## 6. Next steps

1. Open PR for W876 to `master` (if not already done).
2. Implement selected W877 variant per `.claude/plans/wave-loop-877.md` and the standing charter.

*φ² + φ⁻² = 3 | TRINITY*
