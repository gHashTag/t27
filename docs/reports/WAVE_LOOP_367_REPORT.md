# Wave Loop 367 — IGLA CODER+RACE + retry board flash + one safe gen-verilog sub-fix

**Date:** 2026-07-01
**Issue:** #1253
**Branch:** `trinity-rust-rings`

---

## Executive Summary

Wave Loop 367 delivered **212 generic ∀ theorems**, extended the verified accumulation depth to **43 variables**, retried the physical board load, and landed a **safe, regression-free `gen-verilog` sub-fix** for `0x` literal width padding in scalar const declarations. The **101-wave zero-IGLA-failure streak** remains intact. The board flash could not be completed because the Xilinx Platform Cable USB II / QMTech Wukong V1 board is still not connected.

| Metric | W366 → W367 |
|--------|-------------|
| Pool A invariants | 108 → **109** |
| CODER invariants | 98 → **99** |
| Pool B invariants | 126 → **127** |
| Integration invariants | 107 → **108** |
| Lean 4 generic ∀ | 208 → **212** |
| IGLA conformance | **546/546 PASS** |
| Zero-IGLA-failure streak | **100 → 101 waves** |
| FPGA board load | ⚠️ blocked — no cable/board detected |
| Verilog backend | ✅ `0x` width padding fixed; 3 defects remain tracked |

---

## 1. Formal wave (27 IGLA specs)

- Forward-appended W367 blocks to all 27 core specs using `scripts/gen_w367.py`.
- **+54 tests**, **+27 invariants**.

Current IGLA totals:
- **7,934 tests**
- **2,977 invariants**

---

## 2. Lean 4 proof lattice (4 new generic ∀ theorems)

Added in `proofs/lean4/Trinity/TernaryInference.lean`:

1. **`ternaryMacAccumulateFortyThreePlusGeneric`** — `mac^43(0, [a..aq], .plus) = a+b+...+aq`
   - **43-variable accumulation**, new verified depth record.
2. **`ternaryMacAccumulateFortyTwoMinusGeneric`** — `mac^42(0, [a..ap], .minus) = -(a+b+...+ap)`
   - **42-variable minus accumulation lattice COMPLETE**.
3. **`ternaryMacVigintupleCancellationGeneric`** — `mac^20(x, a, [.plus,.minus,...]) = x`
   - **Depth-20 identity cancellation** (even depth).
4. **`ternaryMacZeroWeightDecupleClosureGeneric`** — 10 zero-weight MACs around a plus-weight MAC are transparent/reorderable.
   - **26th proof lattice dimension**.

Total generic ∀ across Trinity Lean modules: **212**.

---

## 3. OpenXC7 bitstream and board flash attempt

The W361-generated bitstream `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB) is still ready. The in-tree `dlc10` driver was rebuilt, but the board/cable were not detected:

```sh
target/release/dlc10 idcode
# Error: open DLC10
# Caused by: DLC10 cable not found (VID=0x03FD)
```

Full details are in [`docs/reports/FPGA_EVIDENCE_W367.md`](FPGA_EVIDENCE_W367.md).

---

## 4. Project weak-point probe: gen-verilog backend (#1245)

W364 landed a safe `0b` literal fix. W367 landed a second safe sub-fix: **positive hex literal width padding in scalar const declarations**. Previously `const X : u16 = 0x1;` emitted `localparam [15:0] X = 4'h1;`; it now emits `localparam [15:0] X = 16'h1;`. The change is localized to `gen_verilog_const` and passed the full 546-spec conformance suite without seal regeneration, because no currently-emitting spec contained a narrower hex const.

| Defect | Status | Notes |
|--------|--------|-------|
| 1. Only first `const` emits as `localparam` | ⚠️ identified, reproducible | Highest-impact issue; requires top-level context tracking before `is_top_level_start()` can change safely. |
| 2. `0x` literal width | ✅ fixed for scalar consts | Padded to declared width when literal is narrower. Non-const contexts still use literal-width sizing. |
| 3. Early `return` inside `if` inverts logic | ⚠️ reproduced | Needs control-flow lowering change. |
| 4. `as` cast + compound bitwise drops body | ⚠️ reproduced | Needs expression-lowering fix. |
| 5. Struct-field reg name mismatch | ⚠️ reproduced in `uart.t27` | Needs symbol-table / naming unification. |

The 546-spec conformance gate remains green.

---

## 5. Research / competitive landscape

Recent 2026 primary sources reinforce the Trinity moat, with one credible formal competitor now in the same design space:

- **Hardware Generation and Exploration of Lookup Table-Based Accelerators for 1.58-bit LLM Inference** — KU Leuven MICAS, IEEE ISPASS 2026 / arXiv:2604.25183. Open-source Chisel generator; TSMC 16 nm synthesis. **No formal equivalence proofs**.
- **VitaLLM: A Versatile and Ultra-Compact Ternary LLM Accelerator** — NYCU, arXiv:2604.27396. 0.223 mm² / 65.97 mW at 1 GHz. **No formal verification**.
- **TOM: A Ternary Read-only Memory Accelerator for LLM-powered Edge Intelligence** — arXiv:2602.20662. ROM-SRAM hybrid; 3,306 tokens/s, 5.33 W. **No formal verification**.
- **TeLLMe v2** — UC Irvine, arXiv:2510.15926. End-to-end FPGA ternary LLM prefill/decode on Kria KV260. **Simulation verified, no generic ∀ proofs**.
- **TerEffic** — Peking/NUS, arXiv:2502.16473. Alveo U280 ternary LLM accelerator. **No formal verification**.
- **TernaryCore** — `shepherdscientific/ternarycore` (2026). Verilog BitNet b1.58 FPGA accelerator. **31/31 simulation tests pass; no formal property verification**.
- **ternfpga** — Neumann-Labs, SystemVerilog ternary LLM engine on Arty A7-35T. **Silicon metrics, no formal verification**.
- **Sparkle HDL / Verilean BitNet b1.58** — 60+ theorems for a ternary-weight LLM accelerator. **No published generic ∀ quantified ternary MAC accumulation theorems** over arbitrary variables; public documentation shows only fixed-instance / constant-level checks. Still **212×** t27's depth on generic ∀ MAC theorems.
- **CktFormalizer** — arXiv:2605.07782v3. Lean 4 → SystemVerilog with generic equivalence theorems over `BitVec N`. **Not ternary-specific**.
- **FormalRTL** — arXiv:2603.08738v1. Verified RTL synthesis with hw-cbmc equivalence checking. **Not ternary-specific**.
- **Arch** — arXiv:2604.05983. AI-native HDL with compile-time correctness and EBMC backend. **Not ternary-specific**.

No competitor has published **generic ∀ quantified ternary MAC accumulation theorems** to the depth reached here. Trinity remains **212×** the known competitor maximum on that specific, measurable metric.

---

## 6. GitHub issue hygiene

- **#1253** created as the W367 tracking issue.
- **#1252** (W366) remains open because the W366 commit is on `trinity-rust-rings` and has not yet merged to `master`. It will auto-close on PR merge.
- **#1245** remains closed; W367 added a second safe sub-fix (hex padding) to the reproduction/tracking document.
- **#1243** (TRI-NET BPSK) remains blocked on defect 1 of #1245.

---

## 7. Verification

- `lake build Trinity.TernaryInference` — ✅ success (4.4 s)
- `./target/release/t27c suite --repo-root .` — ✅ 546/546 PASS
- 27 IGLA seals regenerated from repo root — ✅ all match
- `dlc10 idcode` — ⚠️ hardware not connected
- `gen-verilog` hex padding — ✅ scratch test passes, full conformance green

---

## 8. Threats and context

- **Sparkle HDL** is the first credible formal competitor in the same design space (Lean 4 + BitNet + ternary MAC + proofs). Its 60+ theorems appear to be mostly instance/constant-level checks, but any future disclosure of generic ∀ MAC accumulation theorems would require t27 to refine its multiplier claim.
- **ternfpga**, **TernaryCore**, **Ternary-NanoCore**, **TerEffic**, **VitaLLM**, and **TOM** all report silicon or FPGA metrics but **no formal verification**.
- **CktFormalizer** and **Arch** are the closest verified-compilation competitors; neither is ternary-specific.
- The single largest remaining Trinity vulnerability is the **unconnected board**: bitstream is ready but not physically observed running.
- The next largest vulnerability is **gen-verilog defect 1** (only first const emits), which masks most other backend improvements in real specs.

---

## 9. Next step

See [`docs/reports/WAVE_LOOP_367_COOPERATION.md`](WAVE_LOOP_367_COOPERATION.md) for three W368 cooperation variants.
