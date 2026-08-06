# Wave Loop 880 — Closeout Report

**Date:** 2026-08-05  
**Issue:** [#1712](https://github.com/gHashTag/t27/issues/1712)  
**Branch:** `wave-loop-880`  
**Parent:** `wave-loop-879` HEAD (earlier waves' PRs remain open)  
**PR:** [#1720](https://github.com/gHashTag/t27/pull/1720)  
**Author:** Trinity Agent (Claude Code t27)

---

## 1. What we built

Wave Loop 880 continues the module-scope packed array-of-struct ladder, selecting
Variant A from the W880 plan:

```text
module-scope [579][2]^6 Pt variable from call with indexed signed writes
```

- `Pt { x : i16, y : i16 }` → 32 bits/element.
- Outer dimension `579` is odd and non-power-of-two, preserving the boundary-stress
  cadence established by W837+.
- Inner shape `[2]^6` = 64 elements/row.
- Total elements: `579 × 64 = 37,056`.
- Packed vector width: `37,056 × 32 = 1,185,792 bits` ≈ **1.131 MiBit**.

Artifacts produced:

| Artifact | Path | Notes |
|----------|------|-------|
| Generator | `scripts/gen_w880.py` | Copied from `gen_w879.py`; copy hazard fixed before first run (`w880`, `OUTER = 579`, `MID_IDX = 289`). |
| Spec | `specs/scratch/w880_bench_module_579x2p6_aos_var_call_write.t27` | 110,071 lines, ~2.54 MB. |
| Seal | `.trinity/seals/scratch_w880_bench_module_579x2p6_aos_var_call_write.json` | Saved by `t27c seal --save`. |
| Test | `bootstrap/tests/icarus_lowerable.rs` | `accepts_w880_bench_module_579x2p6_aos_var_call_write`. |

---

## 2. Weak-point investigation

### 2.1 t27c / compiler

No compiler changes were required. `bootstrap/stage0/FROZEN_HASH` remains:

```text
68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc
```

This is the thirtieth consecutive zero-compiler-change wave in the mechanical
packed-vector AoS ladder (W851–W880), confirming that the lowering, simulation,
and cocotb reference-model paths scale smoothly through the 1.13-MiBit range.

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
- **Issue #1134** (`elab_expr.cc` assertion for unpacked arrays of packed structs)
  remains open, with **PR #1292** (opened 2026-01-23) aiming to fix several
  assertion failures and elaboration issues. Our witness uses a module-scope
  **packed** array-of-struct, so it does not exercise the #1134 path.

At 1.131 MiBit, W880 remains far below any practical Icarus memory boundary.
The next meaningful watch-point is still the established 4-MiBit soft cliff.

### 2.3 Vericert / Graphiti — verified HLS analog

- **Vericert** (v2.0.0 released 2026-01-29) is the CompCert-based verified HLS
  toolchain ([repository](https://github.com/ymherklotz/vericert),
  [documentation](https://vericert.ymhg.org/)).
- The most recent major adjacent result is **Graphiti** at **ASPLOS 2026**
  ([DOI 10.1145/3779212.3790166](https://doi.org/10.1145/3779212.3790166),
  [PDF](https://yannherklotz.com/papers/asplos26_fvo.pdf)): a Lean 4 formalization
  of dataflow circuits with a verified rewriting algorithm and a verified loop
  rewrite that introduces out-of-order execution. It reports **2.1× speedup**
  over in-order dataflow HLS and **5.8× speedup over Vericert**, while remaining
  provably sound — it even caught an unsound transformation in the original
  unverified Dynamatic/DF-OoO `bicg` rewrite.
- The 2024 PLDI paper *Hyperblock Scheduling for Verified High-Level Synthesis*
  ([DOI 10.1145/3656455](https://doi.org/10.1145/3656455)) remains the canonical
  reference for verified if-conversion and hyperblock scheduling in Vericert.

Our `t27c icarus-cocotb` gate is a lightweight reference-model equivalence check
adjacent to that paradigm: it does not prove translation correctness, but it does
mechanically compare the compiled Verilog against an independent Python evaluator
of the t27 AST for every wave.

### 2.4 FPGA Roofline / memory bandwidth context

- The FPGA Roofline model (Siracusa et al., IEEE TC 2021,
  [DOI 10.1109/tc.2021.3111761](https://doi.org/10.1109/tc.2021.3111761)) frames
  the ladder as a memory-quanta `Q` probe.
- Recent 2026 work on heterogeneous LLM inference on FPGA
  ([arXiv 2603.29002](https://arxiv.org/pdf/2603.29002)) reports on-chip
  **BRAM (21.8 TB/s) and URAM (10.4 TB/s)** versus HBM at **~460 GB/s** on
  AMD Alveo U55C, confirming that internal wide vectors are
  memory-bandwidth-limited only when they exceed on-chip SRAM capacity.
- A 2026 persistent-state dataflow accelerator for Gated DeltaNet decode
  ([arXiv 2603.05931](https://arxiv.org/pdf/2603.05931)) keeps the full 2 MB
  recurrent state in on-chip BRAM, achieving **4.5× lower latency** than an
  NVIDIA H100 and **60× better energy efficiency** per token at ~10 W.
- PD-Swap (arXiv 2512.11550) uses dynamic partial reconfiguration on the AMD
  Kria KV260 to swap prefill/decode attention modules, exploiting BRAM/URAM
  lookup tables for ternary weights and reaching **27 tokens/s** decode.

**Interpretation:** the ladder is still a memory-quanta probe, not an IO-port probe,
because the vector is internal. The 1.131-MiBit W880 vector is trivial compared to
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
| Parse | `./target/release/t27c parse specs/scratch/w880_bench_module_579x2p6_aos_var_call_write.t27` | PASS |
| Lowerable | `./target/release/t27c icarus-lowerable ...` | `lowerable` |
| Simulate | `./target/release/t27c icarus-simulate ...` | `PASSED` (17 cycles) |
| Cocotb | `./target/release/t27c icarus-cocotb ...` | `reference-model OK` |
| Seal | `./target/release/t27c seal --save ...` | seal saved |
| Targeted test | `cargo test --release --test icarus_lowerable accepts_w880_bench_module_579x2p6_aos_var_call_write` | 1/0 |
| Full suite | `cargo test --release --test icarus_lowerable` | **340/0** |

---

## 5. Integration

- [x] Generator `scripts/gen_w880.py` with copy hazard fixed.
- [x] Witness generated and all direct gates green.
- [x] Integration test added immediately after W879's test.
- [x] `FROZEN_HASH` verified unchanged.
- [x] Closeout report written.
- [ ] Commit with `Closes #1712`, push branch `wave-loop-880`, open PR to `master`.
- [ ] Create W881 issue and branch `wave-loop-881` from `wave-loop-880` HEAD.

---

## 6. Next steps

1. Land W880 commit (`Closes #1712`) and open PR to `master`.
2. Create W881 issue and branch `wave-loop-881` from `wave-loop-880` HEAD.
3. Implement selected W881 variant per the standing charter.

*φ² + φ⁻² = 3 | TRINITY*
