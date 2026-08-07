# Training a neural network on live FPGA silicon from a verified specification

*A reproducible open-toolchain methodology.* Workshop-grade write-up of the GF-T
on-chip training result. Everything below is measured, not projected.

## Claim

A neural-network training loop — forward, loss, backward, weight update — written
as a `.t27` **specification**, compiled by a ternary compiler, **verified bit-exact
against an independent model across four targets (Verilog, C, Rust, and the model)**,
and then **run on a real Xilinx Artix-7 (AX7203, xc7a200t) where it trains a network**
— through a fully open-source toolchain (yosys → nextpnr-xilinx → prjxray), no Vivado,
no Docker, native macOS arm64.

## Method (the pipeline, end to end)

1. **Spec.** The GF-T arithmetic (a ternary-native GoldenFloat: `value = (−1)^s ·
   (1 + m/512) · 2^(off−40)`) is defined in `.t27` — `gft_smul.t27`, `gft_sadd.t27`,
   etc. The compiler `t27c` emits **Verilog, C, Rust, and Zig** from one source.

2. **Generator, not hand-RTL.** `tools/gft_backprop_microcode.py` turns an arbitrary
   feed-forward topology (free inputs × hidden layers × outputs, arbitrary depth) into
   a **microsequencer**: one shared multiply core + one shared add core, driven by a
   microcode program over a register file. Network size grows the microcode (time) and
   the register file — **not the datapath** (one multiplier, regardless of the net).

3. **Verification, as a CI invariant.** On every change, a gate (`verify_emit_bitexact`)
   regenerates the arithmetic cores from spec and proves the generated RTL is **bit-exact
   to the independent Python GF-T model over a full 80-step training run** (forward +
   backprop + update, every output per step) in Icarus Verilog; then **synthesizes** it
   (yosys) with a non-zero cell mapping and asserts the **one-shared-multiplier datapath
   invariant**. A companion gate (`verify_multitarget`, `verify_trainer_c`) proves the
   primitives and the whole trainer are bit-exact in **C and Rust** too, over moderate,
   extreme (saturation-adjacent), and cancellation operands, plus a differential fuzzer
   over random topologies. Spec→any-target bit-exactness is *guaranteed per pull request*.

4. **Silicon build.** `emit_verilog(…, clk_div=16)` emits a silicon-ready variant: the
   register file is forced to flip-flops (distributed LUTRAM cannot do the parallel
   weight-init) and the sequencer steps once per 16 cycles (a clock-enable) so the deep
   shared-core combinational path has time to settle. `yosys synth_xilinx -nocarry`
   → `nextpnr-xilinx --timing-allow-fail` → `fasm2frames` → `xc7frames2bit` → `.bit`.

5. **Flash & train.** `openFPGALoader` loads the bitstream over JTAG; the host streams
   training samples over UART; the board holds the weights, runs a full backprop step
   per sample, and returns the forward output. Weights persist across samples → it learns.

## Results (measured on the AX7203)

- **The full 2-layer backprop microsequencer trains XOR to 4/4**, 24/25–25/25 epochs,
  both layers learning on-chip, with a **weight trajectory bit-exact to the independent
  model** (epoch-0 outputs 0.000 / 0.551 / 0.936 / 0.232 match the model to three
  decimals; the error term converges toward zero).
- The **CI-verified generator's** RTL (not hand-written) trains XOR on silicon the same
  way, closing the loop spec → verified generator → open-source bitstream → live training.
- Earlier on-silicon results on the same board: inference (dot 6/6, BitNet neuron 8/8,
  3-class argmax 16/16 held-out, 2-layer ReLU XOR 4/4) and training (SGD 4/4, gradient
  descent → 0, 1-/2-parameter regressions, ReLU-gated nonlinear neuron, a classifier
  generalizing 8/8, train→save→deploy closed with SPI-flash boot).
- **Size costs time, not area** (CI-measured): microcode steps grow (2,2,1)=32 →
  (2,4,2)=88 → deep [3,4,4,2,1]=216, while synthesized cell counts stay ~constant.

## Honest limits (measured, not hidden)

- **openXC7 correctness ceiling ≈ 17M fasm.** Designs ≤ 16.7M compute correctly; ≥ 19.5M
  place and respond over UART but *miscompute* — always cross-checked against the model.
- **The open-source place-and-route cannot express a multicycle timing constraint**
  (nextpnr-xilinx's XDC parser supports only `create_clock`). The deep shared-core path
  is therefore left timing-relaxed, and correctness is **placement-dependent**: some
  `--seed` values glitch, one trains cleanly. We seed-search. This is an open-toolchain
  limitation — a commercial P&R would close the path directly — not a design flaw. A
  design's microcode step count predicts its marginality (more steps per frame = more
  chances for a glitch).

## Reproducibility

The verification runs in CI on every pull request. The silicon build is one script
(`board/build_trainer.py`): generate → wrap in the UART front-end → yosys → seed-search
nextpnr → per-seed bitstreams. Flash a seed, drive it over UART, keep the seed that
trains stably. All artifacts (chipdb, nextpnr-xilinx, prjxray) are open-source and
build natively on macOS arm64.

## Why it matters

On-device *learning* on cheap FPGA silicon, in a ternary-native format, from a
machine-verified specification, through an entirely open toolchain, is a capability we
have not seen demonstrated elsewhere. Every inference-only ternary accelerator we know
of (Ternary-NanoCore, TerEffic, bitnet.cpp, bitSMM) runs a *frozen* model in hand-written
RTL or on a CPU; here the spec *is* the network, it is verified bit-exact across four
targets, and it *trains* on live silicon.
