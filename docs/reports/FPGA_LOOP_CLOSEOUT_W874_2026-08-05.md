# Wave Loop 874 — Closeout Report

**Date:** 2026-08-05  
**Issue:** [#1696](https://github.com/gHashTag/t27/issues/1696)  
**Branch:** `wave-loop-874`  
**Parent:** `wave-loop-873` HEAD (earlier waves' PRs remain open)  
**PR:** [#1698](https://github.com/gHashTag/t27/pull/1698)  
**Author:** Trinity Agent (Claude Code t27)

---

## 1. What we built

Wave Loop 874 continues the module-scope packed array-of-struct ladder, selecting
Variant A from the W874 plan:

```text
module-scope [567][2]^6 Pt variable from call with indexed signed writes
```

- `Pt { x : i16, y : i16 }` → 32 bits/element.
- Outer dimension `567` is odd and non-power-of-two, preserving the boundary-stress
  cadence established by W837+.
- Inner shape `[2]^6` = 64 elements/row.
- Total elements: `567 × 64 = 36,288`.
- Packed vector width: `36,288 × 32 = 1,161,216 bits` ≈ **1.108 MiBit**.

Artifacts produced:

| Artifact | Path | Notes |
|----------|------|-------|
| Generator | `scripts/gen_w874.py` | Copied from `gen_w873.py`; copy hazard fixed before first run (`w874`, `OUTER = 567`, `MID_IDX = 283`). |
| Spec | `specs/scratch/w874_bench_module_567x2p6_aos_var_call_write.t27` | 107,791 lines, ~2.37 MB. |
| Seal | `.trinity/seals/scratch_w874_bench_module_567x2p6_aos_var_call_write.json` | Saved by `t27c seal --save`. |
| Test | `bootstrap/tests/icarus_lowerable.rs` | `accepts_w874_bench_module_567x2p6_aos_var_call_write`. |

---

## 2. Weak-point investigation

### 2.1 Icarus Verilog packed-vector behavior at 1+ MiBit

The practical landscape remains as observed in W837–W873:

- **Issue #1171** (2024): Icarus can exhaust memory when a constant-expression bug
  produces a vector in the exa-bit range. Maintainer *caryr* notes that the standard
  *suggests* limits of **2^16 bits for packed dimensions** and **2^24 bits for unpacked
  dimensions**, but Icarus today does not hard-enforce them; it tries to allocate until
  it runs out of memory. Better diagnostics are being discussed.
- **Issue #1180** (late 2024): Packed/unpacked array **parameters** are a SystemVerilog
  feature not fully supported in Icarus; the generator path here does not use them.
- **Issue #1134** (2024–2025): Packed arrays of structs still have corner cases,
  especially when indexed in unusual ways. Our `[N][2]^6 Pt` pattern (uniform packed
  array of structs with signed scalar indices) remains on the well-supported path.
- Historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed vectors;
  modern versions do not hit it at this scale.

**Interpretation:** W874's 1.108-MiBit vector is still comfortably below both the
historical allocator assertion zone and the standard's 2^16-bit *suggestion* when read
as a per-dimension limit. In practice, Icarus accepts the vector because it is a single
large packed object, not a multi-dimensional packed parameter or an expression-width
edge case. No new limit appeared between W873 and W874.

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

**Interpretation for t27:** our generated internal vector (`1,161,216 bits`) is far wider
than a Vitis interface port limit, but it is not an external port — it is an internal
module variable. The relevant comparison is the *internal* representation fidelity, not
an IO pin limit. t27c and Icarus are effectively doing internally what Vitis HLS does
for `compact=bit` on internal arrays: bit-packing an AoS.

### 2.3 Verified-compilation analog: Vericert / CompCert

- **Vericert** is a formally verified C-to-Verilog HLS tool built on CompCert/Coq.
- The original framework was introduced at **OOPSLA 2021**
  (Herklotz et al., *Formal Verification of High-Level Synthesis*, DOI:10.1145/3485494).
- The 2024 PLDI paper *Hyperblock Scheduling for Verified High-Level Synthesis*
  (Herklotz & Wickerson, DOI:10.1145/3656455) adds verified if-conversion and
  hyperblock scheduling, making Vericert competitive with unverified Bambu HLS.

**Interpretation:** the t27 Wave Loop exercises the same correctness surface that
Vericert targets — bit-exact lowering of source-level arrays and structs to
hardware-level vectors. Our `t27c icarus-cocotb` gate is a lightweight reference-model
equivalence check, conceptually adjacent to the verified translation-validation
approach used in Vericert.

### 2.4 FPGA Roofline model

Siracusa et al. (IEEE TC 2021, DOI:10.1109/tc.2021.3111761) and the precursor ICCAD
2020 paper (DOI:10.1145/3400302.3415730) frame the ladder as a **memory-quanta probe**:

- Each wider packed vector increases the working set `Q` along the memory-bandwidth
  axis while the compute roof stays flat.
- **Locality walls** on the Roofline plot show how caching inner-loop data in on-chip
  memory (BRAM/URAM) shifts operational intensity rightward.
- **Bandwidth ceilings** distinguish random access, gather-scatter, and peak configured
  AXI bandwidth.

**Interpretation:** W874 is still on the "soft" side of the memory wall. The kernel
performs the same arithmetic per element; only the vector width grows. Until a tool
limit appears, the ladder measures how large `Q` can become before lowering, simulation,
or sealing breaks.

---

## 3. Validation matrix

| Gate | Command / Test | Result |
|------|----------------|--------|
| Parse | `t27c parse specs/scratch/w874_bench_module_567x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `t27c icarus-cocotb ...` | `reference-model OK` |
| Seal | `t27c seal --save ...` | seal saved |
| Targeted test | `cargo test --release --test icarus_lowerable -- accepts_w874_bench_module_567x2p6_aos_var_call_write` | 1/0 |
| Full suite | `cargo test --release --test icarus_lowerable` | **334/0** |
| FROZEN_HASH | `cat bootstrap/stage0/FROZEN_HASH` | unchanged |

No compiler source changes, no reference-model changes, no FROZEN_HASH change.

---

## 4. Remaining weak points (non-blocking)

- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, separate
  from the Wave Loop ladder, Closes #749).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- Release warning cleanup sprint (~626 release warnings, ~780 clippy warnings).
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day commit traceability remains low outside wave-loop commits.
- Generator copy hazard persists; the durable fix is a parameterized template where
  `WAVE`/`OUTER`/`PREV` come from a single config block.
- Full `./scripts/tri test` suite still stalls on the pre-existing
  `w589_bench_module_17d_aos_var_call_write.t27` parse phase and was not completed
  this wave.

---

## 5. Cooperation variants for Wave Loop 875

| Variant | Shape | Outer | Inner | Elements | Bits | MiBit | Purpose |
|---------|-------|-------|-------|----------|------|-------|---------|
| **A (recommended)** | `[569][2]^6 Pt` | 569 | `[2]^6` | 36,416 | 1,165,312 | ~1.112 | Continue mechanical `outer += 2` ladder. |
| **B** | `[567][3]^6 Pt` | 567 | `[3]^6` | 54,432 | 1,741,824 | ~1.662 | Grow second inner dimension, stress stride scaling (~1.6 MiBit). |
| **C** | `[567][2]^6 Pt` (neg-index writes) | 567 | `[2]^6` | 36,288 | 1,161,216 | ~1.108 | Negative-index / wrap-around writes. |

Variant A is recommended. It preserves the established mechanical cadence and keeps
the next rung well under the 4-MiBit soft cliff while continuing the non-power-of-two
outer-dimension stress pattern.

---

## 6. Next steps

1. ~~Land W874 commit (`Closes #1696`) and open PR #1698 to `master`.~~ Done.
2. Create W875 issue and branch `wave-loop-875` from `wave-loop-874` HEAD.
2. Create W875 issue and branch `wave-loop-875` from `wave-loop-874` HEAD.
3. Implement selected W875 variant per the standing charter.

*φ² + φ⁻² = 3 | TRINITY*
