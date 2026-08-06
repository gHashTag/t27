# Wave Loop 878 — Closeout Report

**Date:** 2026-08-05  
**Issue:** [#1706](https://github.com/gHashTag/t27/issues/1706)  
**Branch:** `wave-loop-878`  
**Parent:** `wave-loop-877` HEAD (earlier waves' PRs remain open)  
**PR:** [#1707](https://github.com/gHashTag/t27/pull/1707)  
**Author:** Trinity Agent (Claude Code t27)

---

## 1. What we built

Wave Loop 878 continues the module-scope packed array-of-struct ladder, selecting
Variant A from the W878 plan:

```text
module-scope [575][2]^6 Pt variable from call with indexed signed writes
```

- `Pt { x : i16, y : i16 }` → 32 bits/element.
- Outer dimension `575` is odd and non-power-of-two, preserving the boundary-stress
  cadence established by W837+.
- Inner shape `[2]^6` = 64 elements/row.
- Total elements: `575 × 64 = 36,800`.
- Packed vector width: `36,800 × 32 = 1,177,600 bits` ≈ **1.124 MiBit**.

Artifacts produced:

| Artifact | Path | Notes |
|----------|------|-------|
| Generator | `scripts/gen_w878.py` | Copied from `gen_w877.py`; copy hazard fixed before first run (`w878`, `OUTER = 575`, `MID_IDX = 287`). |
| Spec | `specs/scratch/w878_bench_module_575x2p6_aos_var_call_write.t27` | 109,311 lines, ~2.52 MB. |
| Seal | `.trinity/seals/scratch_w878_bench_module_575x2p6_aos_var_call_write.json` | Saved by `t27c seal --save`; `spec_hash=sha256:2d41768df9bcbf3872361c9195282ce78fa6861382729d7b227013b8cd83240f`. |
| Test | `bootstrap/tests/icarus_lowerable.rs` | `accepts_w878_bench_module_575x2p6_aos_var_call_write`. |

---

## 2. Weak-point investigation

### 2.1 t27c / compiler

No compiler changes were required. `bootstrap/stage0/FROZEN_HASH` remains:

```text
68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc
```

This is the twenty-eighth consecutive zero-compiler-change wave in the mechanical
packed-vector AoS ladder (W851–W878), confirming that the lowering, simulation,
and cocotb reference-model paths scale smoothly through the 1.12-MiBit range.

### 2.2 Icarus Verilog — large packed-vector behavior

- The Verilog standard suggests a packed-dimension limit of **2^16 bits** and an
  unpacked-dimension limit of **2^24 bits**. Modern Icarus treats these as soft
  guidelines and allocates until memory is exhausted ([issue #1171](https://github.com/steveicarus/iverilog/issues/1171)).
- Icarus already caps **unsized expressions** at **65,536 bits** by default
  ([issue #13](https://github.com/steveicarus/iverilog/issues/13)); our vector is a
  sized, declared signal, so this cap does not apply.
- **Icarus V13.0** (released 2026-03-02) includes memory-management improvements
  during elaboration and simulation, better packed/unpacked array handling, and
  improved diagnostics for large corner cases
  ([release notes](https://steveicarus.github.io/iverilog/releases/v13-0-release-note.html)).
- Unpacked arrays of packed structs can still hit an assertion in `elab_expr.cc`
  ([issue #1134](https://github.com/steveicarus/iverilog/issues/1134)); our witness
  uses a module-scope packed array-of-struct, so it does not exercise that path.

At 1.124 MiBit, W878 remains far below any practical Icarus memory boundary.
The next meaningful watch-point is still the established 4-MiBit soft cliff.

### 2.3 Vericert / CompCert — verified HLS analog

- Vericert v2.0.0 was released 2026-01-29 ([repository](https://github.com/ymherklotz/vericert),
  [documentation](https://vericert.ymhg.org/)).
- The most recent major research advance is the PLDI 2024 paper
  *Hyperblock Scheduling for Verified High-Level Synthesis*
  ([DOI 10.1145/3656455](https://doi.org/10.1145/3656455),
  [PDF](https://yannherklotz.com/papers/pldi24_hsvhls.pdf)), which adds verified
  if-conversion and hyperblock scheduling, yielding 2.1× speedup over the original
  sequential Vericert and competitive performance with the unverified Bambu HLS.
- 2026 follow-on work includes
  *Graphiti: Formally Verified Out-of-Order Execution in Dataflow Circuits* (ASPLOS 2026)
  and *Let It Flow: A Formally Verified Compilation Framework for Asynchronous Dataflow*
  (PLDI 2026), both extending the verified-HLS/dataflow paradigm.

Our `t27c icarus-cocotb` gate remains a lightweight reference-model equivalence check
adjacent to that paradigm: it does not prove translation correctness, but it does
mechanically compare the compiled Verilog against an independent Python evaluator
of the t27 AST for every wave.

### 2.4 FPGA Roofline / memory bandwidth context

- The FPGA Roofline model (Siracusa et al., IEEE TC 2021,
  [DOI 10.1109/tc.2021.3111761](https://doi.org/10.1109/tc.2021.3111761)) frames
  the ladder as a memory-quanta `Q` probe.
- Recent 2026 work on heterogeneous LLM inference on FPGA
  ([arXiv 2603.29002](https://arxiv.org/pdf/2603.29002)) reports on-chip BRAM/URAM
  bandwidths in the **TB/s** range versus HBM at ~460 GB/s, confirming that
  internal wide vectors are memory-bandwidth-limited only when they exceed on-chip
  SRAM capacity.
- The FER empirical Roofline benchmark for FPGAs
  ([PDF](https://sfera.unife.it/bitstream/11392/2500795/1/FER_A_Benchmark_for_the_Roofline_Analysis_of_FPGA_Based_HPC_Accelerators.pdf))
  provides concrete measured bandwidth numbers across Alveo U250/U50/U280 devices.

**Interpretation:** the ladder is still a memory-quanta probe, not an IO-port probe,
because the vector is internal. The 1.124-MiBit W878 vector is trivial compared to
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
| Build | `cargo build --release -p t27c` | OK |
| Parse | `./target/release/t27c parse specs/scratch/w878_bench_module_575x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `./target/release/t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `./target/release/t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `./target/release/t27c icarus-cocotb ...` | `reference-model OK` |
| Seal | `./target/release/t27c seal --save ...` | seal saved |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w878_bench_module_575x2p6_aos_var_call_write` | 1/0 |
| Full suite | `cargo test --release --test icarus_lowerable` | **338/0** |

---

## 5. Integration

- [x] Generator `scripts/gen_w878.py` with copy hazard fixed.
- [x] Witness generated and all direct gates green.
- [x] Integration test added immediately after W877's test.
- [x] `FROZEN_HASH` verified unchanged.
- [x] Closeout report written.
- [ ] Commit with `Closes #1706`, push branch `wave-loop-878`, open PR to `master`.
- [ ] Create W879 issue and branch `wave-loop-879` from `wave-loop-878` HEAD.

---

## 6. Next steps

1. Land W878 commit (`Closes #1706`) and open PR to `master`.
2. Create W879 issue and branch `wave-loop-879` from `wave-loop-878` HEAD.
3. Implement selected W879 variant per the standing charter.

*φ² + φ⁻² = 3 | TRINITY*
