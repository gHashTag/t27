# Wave Loop 875 — Closeout Report

**Date:** 2026-08-05  
**Issue:** [#1699](https://github.com/gHashTag/t27/issues/1699)  
**Branch:** `wave-loop-875`  
**Parent:** `wave-loop-874` HEAD (earlier waves' PRs remain open)  
**PR:** [#1700](https://github.com/gHashTag/t27/pull/1700)  
**Author:** Trinity Agent (Claude Code t27)

---

## 1. What we built

Wave Loop 875 continues the module-scope packed array-of-struct ladder, selecting
Variant A from the W875 plan:

```text
module-scope [569][2]^6 Pt variable from call with indexed signed writes
```

- `Pt { x : i16, y : i16 }` → 32 bits/element.
- Outer dimension `569` is odd and non-power-of-two, preserving the boundary-stress
  cadence established by W837+.
- Inner shape `[2]^6` = 64 elements/row.
- Total elements: `569 × 64 = 36,416`.
- Packed vector width: `36,416 × 32 = 1,165,312 bits` ≈ **1.112 MiBit**.

Artifacts produced:

| Artifact | Path | Notes |
|----------|------|-------|
| Generator | `scripts/gen_w875.py` | Copied from `gen_w874.py`; copy hazard fixed before first run (`w875`, `OUTER = 569`, `MID_IDX = 284`). |
| Spec | `specs/scratch/w875_bench_module_569x2p6_aos_var_call_write.t27` | 108,171 lines, ~2.37 MB. |
| Seal | `.trinity/seals/scratch_w875_bench_module_569x2p6_aos_var_call_write.json` | Saved by `t27c seal --save`. |
| Test | `bootstrap/tests/icarus_lowerable.rs` | `accepts_w875_bench_module_569x2p6_aos_var_call_write`. |

---

## 2. Weak-point investigation

### 2.1 Icarus Verilog packed-vector behavior at 1+ MiBit

The practical landscape remains as observed in W837–W874:

- **Issue #1171** (2024): Icarus can exhaust memory when a constant-expression bug
  produces a vector in the exa-bit range. Maintainer *caryr* notes that the standard
  *suggests* limits of **2^16 bits for packed dimensions** and **2^24 bits for unpacked
  dimensions**, but Icarus today does not hard-enforce them; it tries to allocate until
  it runs out of memory. Better diagnostics are being discussed.
- **Issue #1134** (2024–2025): Packed arrays of structs still have corner cases,
  especially when indexed in unusual ways. Our `[N][2]^6 Pt` pattern (uniform packed
  array of structs with signed scalar indices) remains on the well-supported path.
  Commit [`128c621`](https://github.com/steveicarus/iverilog/commit/128c621e8540b0a68145094fa876dc5de073c9a6)
  (*“Fix width calculation for packed array bounds”*, 2026-06-20) corrected min-width
  calculation for packed array bounds so negative bounds no longer accidentally
  become huge widths — directly relevant to indexed packed-array correctness.
- **Issue #1180** (late 2024, now closed with better diagnostics): Packed/unpacked
  array **parameters** are a SystemVerilog feature not fully supported in Icarus;
  the generator path here does not use them.
- Historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed vectors;
  modern versions do not hit it at this scale.

**Interpretation:** W875's 1.112-MiBit vector is still comfortably below both the
historical allocator assertion zone and the standard's 2^16-bit *suggestion* when read
as a per-dimension limit. In practice, Icarus accepts the vector because it is a single
large packed object, not a multi-dimensional packed parameter or an expression-width
edge case. No new limit appeared between W874 and W875.

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

**Interpretation for t27:** our generated internal vector (`1,165,312 bits`) is far wider
than a Vitis interface port limit, but it is not an external port — it is an internal
module variable. The relevant comparison is the *internal* representation fidelity, not
an IO pin limit. t27c and Icarus are effectively doing internally what Vitis HLS does
with `compact=bit`: storing a packed struct array as one wide bit-vector and indexing
into it. W875 adds another 4,096 bits (128 elements) to that internal vector without
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
  extends the verified-hardware compilation agenda to out-of-order dataflow circuits.
- **“Let It Flow: A Formally Verified Compilation Framework for Asynchronous Dataflow”**
  (Lin, Cai, Surbatovich, PLDI 2026, [DOI 10.1145/3808263](https://doi.org/10.1145/3808263))
  is another 2026 verified hardware/dataflow entry in the same research direction.

**Interpretation for t27:** the t27 compiler does not yet have a Coq-verified backend,
but its `icarus-cocotb` gate is a lightweight reference-model equivalence check adjacent
 to the verified-HLS paradigm. Each wave exercises the same correctness question:
 “does the generated Verilog preserve the t27 reference semantics for an ever-larger
 packed data structure?”

### 2.4 FPGA Roofline model

The FPGA Roofline model (Siracusa et al., IEEE TC 2021,
[DOI 10.1109/tc.2021.3111761](https://doi.org/10.1109/tc.2021.3111761)) frames the ladder
as a memory-quanta probe:

- Each wider vector grows the working set along the **bandwidth axis**.
- The **compute roof** stays flat because the test performs a constant number of indexed
  reads/writes per cycle.
- W875 adds 128 elements × 32 bits = 4,096 bits to the packed vector, a ~0.35 % step
  relative to W874.

**Interpretation:** we remain on the soft, memory-bandwidth-limited side of the wall.
There is no indication that the additional 4 Kbits pushes us into a new tool limit or
changes the asymptotic behavior.

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
| Parse | `./target/release/t27c parse specs/scratch/w875_bench_module_569x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `./target/release/t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `./target/release/t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `./target/release/t27c icarus-cocotb ...` | `reference-model OK` |
| Seal | `./target/release/t27c seal --save ...` | seal saved |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w875_bench_module_569x2p6_aos_var_call_write` | 1/0 |
| Full suite | `cargo test --release --test icarus_lowerable` | **335/0** |

---

## 5. Integration

- [x] Generator `scripts/gen_w875.py` with copy hazard fixed.
- [x] Witness generated and all direct gates green.
- [x] Integration test added immediately after W874's test.
- [x] `FROZEN_HASH` verified unchanged.
- [x] Closeout report written.
- [x] Commit with `Closes #1699`, push branch `wave-loop-875`, open PR #1700 to `master`.
- [x] Create W876 issue #1701 and branch `wave-loop-876` from `wave-loop-875` HEAD.

---

## 6. Next steps

1. ~~Land W875 commit (`Closes #1699`) and open PR #1700 to `master`.~~ Done.
2. ~~Create W876 issue #1701 and branch `wave-loop-876` from `wave-loop-875` HEAD.~~ Done.
3. Implement selected W876 variant per the standing charter.

*φ² + φ⁻² = 3 | TRINITY*
