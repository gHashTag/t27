# Trinity · GF-T: a spec-first ternary neural network that **trains on FPGA silicon**

**Thesis.** A neural network written as a *specification* is compiled to synthesizable
Verilog by a ternary compiler, verified bit-exact against an independent model, and
its **full training loop — forward → loss → backprop → weight update — runs on a live
Xilinx Artix-7**, through a fully open-source toolchain (no Vivado, no Docker). Every
competitor we know of does *inference only*, in hand-written RTL or on a CPU. We train.

---

## 1. What is new

| | Competitors (Ternary-NanoCore, TerEffic, bitnet.cpp, bitSMM) | Trinity / GF-T |
|---|---|---|
| Network source | hand-written RTL / CPU kernels | a `.t27` **specification** → `t27c` → Verilog |
| Correctness | ad-hoc | **bit-exact** vs an independent model (iverilog + a faithful Python model) |
| On-chip capability | inference | **inference AND training** (backprop + weight update on the FPGA) |
| Toolchain | Vivado (proprietary) | **open-source** (yosys → nextpnr-xilinx → prjxray), native macOS arm64 |
| Number format | fp / int | **GF-T** — ternary-native GoldenFloat |

---

## 2. The GF-T format

GF-T16 packs a signed ternary-native GoldenFloat value into a u32:
`value = (−1)^sign · (1 + mant/512) · 2^(offset − 40)`, with a 7-bit offset (BIAS 40)
and 9-bit mantissa. Weights are the ternary set {−1, 0, +1}; activations are GF-T16.
The whole arithmetic library (multiply, add, subtract, reciprocal, exp2, log2, softmax,
argmax, relu) is defined in `.t27` and verified bit-exact.

---

## 3. Proven on live silicon (AX7203, xc7a200t)

14 distinct bitstreams, each driven over UART and cross-checked against RTL/an
independent model:

**Inference** — dot product 6/6 · BitNet neuron 8/8 · 2-layer MLP 8/8 · 3-class argmax
classifier **16/16 held-out** · a 2-layer ReLU network computes **XOR 4/4** plus the
correct nonlinear surface (impossible for a single linear layer).

**Training (on the FPGA itself)** — SGD weight update 4/4 · vector SGD 2/2 · gradient
descent converges (loss → 0) · 1- and 2-parameter regressions discover hidden weights ·
a nonlinear neuron with a working ReLU-derivative gate · a binary classifier learns a
decision boundary and **generalizes 8/8 held-out** · the output layer of an XOR network
**learns to solve XOR**.

**Edge loop** — train on-chip → read the learned weights → bake them into an inference
bitstream (**12/12 held-out with zero training**) → write to SPI flash → the board boots
pre-trained. `train → save → deploy` closed on one device.

---

## 4. Two engineering results that make it scale

**(a) A ~2× area optimization, proven bit-identical.** The dominant cost in every GF-T
design is `magsub`'s normalization. Replacing its 12-iteration *linear* normalize with a
*log-depth* binary-search priority-encoder + a single barrel shift is **bit-identical
over 17.4 million operand pairs** and roughly **halves every design** (the workhorse
trainer: 16.7M → 9.6M fasm). Applied across all 24 specs.

**(b) A microsequenced trainer — full backprop with near-constant area.** The naive
parallel full 2-layer backprop is ~22M fasm, over the measured openXC7 *correctness*
ceiling (~17M). A **microsequencer** — one shared multiply core + one shared add core,
driven by a microcode program over a register file — runs the full forward + backprop +
update in **~3.4K LUTs / 2.93M fasm** (7× smaller), meets timing at 12 MHz, and **trains
XOR to 4/4** (both layers learn) in simulation, with the silicon bitstream built and
validated. A microcode generator turns *any* 2-layer topology into a buildable bitstream:
the (2,3,1) network builds to **2.92M fasm — essentially identical to the XOR net's
2.93M**. Network size costs *time*, not FPGA *area*. This is a **programmable ternary
neural-network trainer**.

---

## 5. Honesty about limits (measured, not claimed)

- **openXC7 correctness ceiling ≈ 17M fasm.** Designs ≤ 16.7M compute correctly; ≥ 19.5M
  place and respond over UART but *miscompute* — always cross-checked against an
  independent model, never trusted on a UART response alone.
- **The compiler's wide return is u64.** Values > 64 bits are handled by the
  microsequencer (one value per step), not by wide ports.
- The microsequencer's forward-both-layers-trainable step is proven in simulation and
  built to a valid, timing-clean bitstream; the on-silicon training run is pending only a
  physical JTAG re-connect on the board.

---

## 6. The pipeline

`.t27 spec → t27c parse/typecheck/gen-verilog → yosys synth_xilinx → nextpnr-xilinx
(chipdb built natively) → prjxray fasm2frames → xc7frames2bit → openFPGALoader → live
Artix-7`. All open-source, native macOS arm64, no Vivado and no Docker. The invariant
`φ² + φ⁻² = 3` anchors the ternary compiler.

---

## 7. Why it matters

On-device *learning* on cheap FPGA silicon, in a ternary-native format, from a
verifiable specification, is a capability nobody else demonstrates. It points at edge
devices that adapt in the field — not just run a frozen model — at a fraction of the
power and area of floating-point, with a correctness story (bit-exact, spec-first) that
inference-only hand-RTL cannot match.
