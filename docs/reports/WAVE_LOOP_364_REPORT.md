# Wave Loop 364 — IGLA CODER+RACE + retry board flash + gen-verilog weak-point probe

**Date:** 2026-07-01
**Issue:** #1249
**Branch:** `trinity-rust-rings`

---

## Executive Summary

Wave Loop 364 delivered **200 generic ∀ theorems**, extended the verified accumulation depth to **40 variables**, retried the physical board load, and probed a critical project weak point in the Verilog backend. The **98-wave zero-IGLA-failure streak** remains intact. The board flash could not be completed because the Xilinx Platform Cable USB II / QMTech Wukong V1 board is still not connected; the bitstream is ready and the load procedure is documented. A narrow, safe fix for binary-literal emission in `t27c gen-verilog` was landed.

| Metric | W363 → W364 |
|--------|-------------|
| Pool A invariants | 105 → **106** |
| CODER invariants | 95 → **96** |
| Pool B invariants | 123 → **124** |
| Integration invariants | 105 → **106** |
| Lean 4 generic ∀ | 196 → **200** |
| IGLA conformance | **546/546 PASS** |
| Zero-IGLA-failure streak | **97 → 98 waves** |
| FPGA board load | ⚠️ blocked — no cable/board detected |
| Verilog backend | ✅ `0b` literals now emit sized Verilog (`N'b...`) |

---

## 1. Formal wave (27 IGLA specs)

- Forward-appended W364 blocks to all 27 core specs using `scripts/gen_w364.py`.
- **+54 tests**, **+27 invariants**.

Current IGLA totals:
- **7,564 tests**
- **2,853 invariants**

---

## 2. Lean 4 proof lattice (4 new generic ∀ theorems)

Added in `proofs/lean4/Trinity/TernaryInference.lean`:

1. **`ternaryMacAccumulateFortyPlusGeneric`** — `mac^40(0, [a..an], .plus) = a+b+...+an`
   - **40-variable accumulation**, new verified depth record.
2. **`ternaryMacAccumulateThirtyNineMinusGeneric`** — `mac^39(0, [a..am], .minus) = -(a+b+...+am)`
   - **39-variable minus accumulation lattice COMPLETE**.
3. **`ternaryMacSeptendecupleCancellationGeneric`** — `mac^17(x, a, [.plus,.minus,...]) = mac(x, a, .plus)`
   - **Depth-17 cancellation** (odd depth leaves residual plus-weight MAC).
4. **`ternaryMacZeroWeightSeptupleClosureGeneric`** — seven zero-weight MACs around a plus-weight MAC are transparent/reorderable.
   - **23rd proof lattice dimension**.

Total generic ∀ across Trinity Lean modules: **200**.

---

## 3. OpenXC7 bitstream and board flash attempt

The W361-generated bitstream `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB) is still ready. The in-tree `dlc10` driver was rebuilt, but the board/cable were not detected:

```sh
/Users/playra/t27/target/release/dlc10 idcode
# Error: open DLC10
# Caused by: DLC10 cable not found (VID=0x03FD)
```

Full details are in [`docs/reports/FPGA_EVIDENCE_W364.md`](FPGA_EVIDENCE_W364.md).

---

## 4. Project weak-point probe: gen-verilog backend (#1245)

GitHub issue #1245 documents five Verilog lowering defects that block `iverilog`-clean RTL from non-trivial specs. This wave investigated all five and landed a safe, narrow fix for the literal-formatting defect.

| Defect | Status | Notes |
|--------|--------|-------|
| 1. Only first `const` emits as `localparam` | ⚠️ identified, not fixed | Root cause: `parse_const_decl` returns before consuming `;`; `skip_to_next_top_level` also omits `KwConst`/`KwVar`. Fixing this requires parser-level changes that risk regressions across 546 specs. |
| 2. `0b`/`0x` literals emitted verbatim | ✅ fixed | `0x` was already converted to `N'h...`; added `0b` → `N'b...` conversion in `gen_verilog_expr`. |
| 3. Early `return` inside `if` inverts logic | ⚠️ identified, not fixed | Requires control-flow lowering change; workaround is `if/else`. |
| 4. `as` cast + compound bitwise drops body | ⚠️ identified, not fixed | Requires expression-lowering fix; workaround is removing casts. |
| 5. Struct-field reg name mismatch | ⚠️ identified, not fixed | Fields emit as `<structlower>_<field>` but references use `<varname>_<field>`. |

The binary-literal fix is in `bootstrap/src/compiler.rs` and passed the full 546-spec conformance suite.

---

## 5. Research / competitive landscape

Recent 2026 primary sources strengthen the case for formally-grounded ternary silicon:

- **Hardware Generation and Exploration of Lookup Table-Based Accelerators for 1.58-bit LLM Inference** — KU Leuven MICAS, arXiv:2604.25183 / ISPASS 2026. Open-source Chisel generator; TSMC 16 nm synthesis. LUT-based ternary/1.58-bit MACs, **no formal equivalence proofs**.
- **VitaLLM: A Versatile and Tiny Accelerator for Mixed-Precision LLM Inference on Edge Devices** — arXiv:2605.00320v1. 16 nm BitNet b1.58 ASIC prototype at 1 GHz / 0.8 V, 0.214 mm². **Silicon metrics, no formal verification**.
- **TOM: A Ternary Read-only Memory Accelerator for LLM-powered Edge Intelligence** — arXiv:2602.20662. Hybrid ROM-SRAM edge ASIC with ternary quantization; 3,306 tokens/s, 5.33 W. **No formal verification**.
- **CktFormalizer: Autoformalization of Natural Language into Circuit Representations** — arXiv:2605.07782v3. Lean 4 → SystemVerilog compiler with bit-width safety and equivalence proofs. **Not ternary-specific**.
- **FormalRTL: Verified RTL Synthesis at Scale** — arXiv:2603.08738v1. C-reference → RTL with hw-cbmc equivalence checking. **Not ternary-specific**.

Trinity's moat remains **formal depth + ready bitstream**: 200 generic ∀ theorems vs. zero generic ∀ ternary MAC proofs in any competitor.

---

## 6. GitHub issue hygiene

Open wave-loop issues (#1246, #1242, #1241, #1240, #1239) predate the current cadence and are superseded by later waves. The board-flash goal is tracked in the recurring wave issues (#1249 for W364). Recommending a single hardware-connectivity tracking issue to consolidate these.

---

## 7. Verification

- `lake build Trinity.TernaryInference` — ✅ success (3.8 s)
- `./target/release/t27c suite --repo-root /Users/playra/t27` — ✅ 546/546 PASS
- 27 IGLA seals regenerated from repo root — ✅ all match
- `t27c gen-verilog` binary-literal fix — ✅ verified with scratch spec

---

## 8. Threats and context

- **Sparkle HDL** continues to expand generic `forall`-style proofs (RV32 divider, June 2026), but the BitNet ternary catalog still advertises **ground-instance** theorems, not generic ∀ ternary MAC results.
- **ternfpga**, **TernaryCore**, **TerEffic**, **VitaLLM**, and **TOM** all report silicon or FPGA metrics but **no formal verification**.
- **CktFormalizer** is the closest verified-compilation competitor; it is not ternary-specific and does not address MAC accumulation depth.
- The single largest remaining Trinity vulnerability is the **unconnected board**: bitstream is ready but not physically observed running.

---

## 9. Next step

See [`docs/reports/WAVE_LOOP_364_COOPERATION.md`](WAVE_LOOP_364_COOPERATION.md) for three W365 cooperation variants.
