# FPGA Loop Closeout — Wave Loop 768

**Date:** 2026-07-23  
**Issue:** #1739  
**Branch:** `wave-loop-768`  
**Next branch:** `wave-loop-769`  
**Witness:** `specs/scratch/w768_bench_module_355x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w768.py`

---

## 1. Summary

Wave Loop 768 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[355][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [355][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 355 |
| Total elements | 355 × 64 = 22,720 |
| Packed vector width | 22,720 × 32 = 727,040 bits |
| Approximate size | ~0.694 MiBit |
| Mid index | `MID_IDX = 177` |
| Frame-condition element | `[177][1][0][0][0][0][0]` → element 11,360 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w767.py` → `scripts/gen_w768.py`.
2. Updated constants: `OUTER = 355`, `MID_IDX = 177`.
3. Manually fixed the f-string module header so the literal expands to
   `w768_bench_module_355x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w768.py`.
5. Added integration test `accepts_w768_bench_module_355x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[177][1][0][0][0][0][0]`:

```
MID_E = 177*64 + 32 = 11,360
MID_X = (2 * 11,360) % 32768 = 22,720
MID_Y = 22,721
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 22,720 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*22719) % 32768 = 12,670`).

`assert_ne` is structurally accepted by the classifier but not emitted on the
Icarus simulation path; the bench therefore uses `assert_eq` checks on the
changed elements.

---

## 3. Validation results

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 228 passed; 0 failed |
| `t27c parse` W768 | PASS |
| `t27c icarus-lowerable` W768 | PASS (`lowerable`) |
| `t27c icarus-simulate` W768 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W768 | PASS (`reference-model OK`) |
| `t27c seal --save` W768 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` | 42 of 51 commits (≈82%) include `Closes #N` in the current 30-day window. Feature/closeout commits for W768 explicitly reference `Closes #1739`. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 53 of 895 non-worktree `.t27` files still lack any test/invariant/bench block (≈5.9%). |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
packed dimensions with no power-of-two restriction. The `[355][2]^6 Pt` witness
becomes a single 727,040-bit SystemVerilog packed vector, which is legal.
t27's scalar-flattening discipline avoids the Icarus/Yosys gaps around arrays of
packed structs.

2025–2026 ternary / MVL / emerging-device landscape relevant to t27:

- **The Era of 1-bit LLMs: All Large Language Models are in 1.58 Bits** — foundational BitNet b1.58 ternary {-1,0,+1} LLM with absmean weight quantization and 8-bit absmax activations, matching FP16/BF16 Transformers at ≥3B scale (arXiv 2024, [arXiv:2402.17764](https://arxiv.org/abs/2402.17764)).
- **BitNet: 1-bit Pre-training for Large Language Models** — JMLR 2025 journal version covering BitNet b1/b1.58, BitLinear layers, straight-through estimators, mixed-precision latent-weight training, and two-stage LR/weight-decay recipe ([JMLR 26/24-2050](https://www.jmlr.org/papers/volume26/24-2050/24-2050.pdf)).
- **BitNet b1.58 2B4T Technical Report** — first open-source native 1-bit 2B-parameter LLM trained from scratch on 4T tokens, released with optimized GPU/CPU inference (arXiv 2025, [arXiv:2504.12285](https://arxiv.org/abs/2504.12285)).
- **Bitnet.cpp: Efficient Edge Inference for Ternary LLMs** — TL LUT-based and lossless I2_S MAD mpGEMM kernels, up to 6.25× speedup over FP16 (arXiv 2025, [arXiv:2502.11880](https://arxiv.org/abs/2502.11880)).
- **Sparse-BitNet: 1.58-bit LLMs are Naturally Friendly to Semi-Structured Sparsity** — ternary BitNet converges to ~42% zero weights and combines cleanly with N:M semi-structured sparsity, up to 1.30× end-to-end speedup (arXiv 2026, [arXiv:2603.05168](https://arxiv.org/abs/2603.05168)).
- **Multi-value Probabilistic Computing with current-controlled Skyrmion Diffusion** — first multi-value probabilistic computing using thermally activated skyrmion diffusion, softmax operation, invertible OR gate, MTJ-compatible readout (arXiv 2025, [arXiv:2508.19623](https://arxiv.org/pdf/2508.19623)).
- **Ternary computing using a novel spintronic multi-operator logic-in-memory architecture** — MTJ+FinFET ternary LiM array executing AND/NAND, OR/NOR, XNOR/XOR, ~78% delay and ~86% power reduction vs. CMOS in image-processing simulations (Results in Engineering 2025, [DOI 10.1016/j.rineng.2025.104011](https://doi.org/10.1016/j.rineng.2025.104011)).
- **Intelligent Reconfigurable Skyrmion-Based Multi-Port Logic Device for In-Memory Computing** — seven basic logic gates in one skyrmion structure with VCMA reconfigurability, <1 fJ/bit (Chinese Phys. Lett. 2026, [DOI 10.1088/0256-307X/43/3/030802](https://doi.org/10.1088/0256-307X/43/3/030802)).
- **Cascading reconfigurable skyrmion logic devices** — VCMA-gated AND/OR/NAND/NOR skyrmion gates with cascaded architectures, robustness sweep via micromagnetic simulations (Nanotechnology 2026, [DOI 10.1088/1361-6528/ae4ef0](https://doi.org/10.1088/1361-6528/ae4ef0)).
- **Ternary Digital Output Data Link From SFQ Circuits** — ternary output voltage-level data link from RSFQ to room-temperature electronics, 3-bit→2-trit encoder in MIT Lincoln Lab SFQ5ee process, +50% data rate or −33% cryogenic cables (IEEE Trans. Appl. Supercond. 2025, [NSF PAR](https://par.nsf.gov/servlets/purl/10674449)).
- **Generalizable Verilog Modeling Framework for Synchronous and Asynchronous Superconducting Pulse-Based Logic Gates** — Verilog/SDF modeling for SFQ/RSFQ/ASFQ gates supporting mixed synchronous–asynchronous superconducting design (arXiv 2026, [arXiv:2603.25885](https://www.arxiv.org/pdf/2603.25885)).
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx 7-series toolchain used for QMTech XC7A100T ternary projects without Vivado.

---

## 6. Three cooperation variants for Wave Loop 769

1. **Variant A (recommended): `[357][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (357 → 22,848 elements, 731,136
     bits, ~0.698 MiBit) and confirms non-power-of-two stride 357.

2. **Variant B: `[355][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W768 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[355][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.694 MiBit and tests control-flow-guarded writes on a packed reg,
     verifying conditional write emission in the Icarus path.

---

## 7. Definition of done

- [x] Witness generated and under version control.
- [x] Integration test added and passing.
- [x] Icarus lowerability, simulation, cocotb, and seal gates green.
- [x] Cargo suites green.
- [x] FROZEN_HASH unchanged.
- [x] Closeout report written.
- [x] `.trinity/experience.md` updated.
- [x] Next-wave cooperation variants defined.

---

phi^2 + 1/phi^2 = 3 | TRINITY
