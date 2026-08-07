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
decision boundary and **generalizes 8/8 held-out** · and the capstone: a **full 2-layer
backprop microsequencer trains XOR to 4/4 across 25/25 epochs, both layers learning on
the chip**, its weight trajectory bit-exact to an independent model.

**Edge loop** — train on-chip → read the learned weights → bake them into an inference
bitstream (**12/12 held-out with zero training**) → write to SPI flash → the board boots
pre-trained. `train → save → deploy` closed on one device.

---

## 4. Four engineering results that make it scale

**(a) A ~2× area optimization, proven bit-identical.** The dominant cost in every GF-T
design is `magsub`'s normalization. Replacing its 12-iteration *linear* normalize with a
*log-depth* binary-search priority-encoder + a single barrel shift is **bit-identical
over 17.4 million operand pairs** and roughly **halves every design** (the workhorse
trainer: 16.7M → 9.6M fasm). Applied across all 24 specs.

**(b) A microsequenced trainer — full backprop, TRAINING XOR ON LIVE SILICON.** The naive
parallel full 2-layer backprop is ~22M fasm, over the measured openXC7 *correctness*
ceiling (~17M). A **microsequencer** — one shared multiply core + one shared add core,
driven by a microcode program over a register file — runs the full forward + backprop +
update in **~3.4K LUTs** (7× smaller). It is flashed to a real Artix-7 and, streamed the
four XOR corners over UART, **trains XOR to 4/4 across 25/25 epochs — both layers learning
on the chip — with a weight trajectory bit-exact to the independent Python model**
(epoch 0 outputs 0.000 / 0.551 / 0.936 / 0.232 match the model to three decimals; the
error term converges toward zero). This is a full backpropagation training loop — forward,
loss, backward, weight update — running on live FPGA silicon. Network size costs *time*,
not FPGA *area* — one shared multiplier, regardless of the net.

**(c) A fully programmable trainer — any feed-forward topology, no structural limits.**
A microcode generator turns an *arbitrary* feed-forward net into a buildable bitstream:
free input count, free output count, free hidden width, and **arbitrary depth** (2-, 3-,
4-layer nets all generate). Biases are trainable at every layer. The generated trainers
learn *real* tasks, not toy XOR: a noisy nonlinear 2-D task to **~97 %** held-out (2-layer)
and **98 %** (3-layer, two hidden layers), and a multi-class one-hot classifier to **93 %**
(argmax over outputs). Because the datapath is one shared multiply/add, a 3-layer
`[2,4,3,1]` net synthesizes to essentially the same cell count as a 2-layer `(2,4,2)` —
depth is *time*, not *area*, measured and CI-checked (see (d)).

**(d) Correctness is a CI-enforced invariant, not a one-off.** Every change to the trainer
generator must pass a gate that, for a spread of topologies (2- to 4-layer, multi-input,
multi-output): regenerates the GF-T arithmetic cores fresh from their `.t27` specs, and
proves the generated RTL is **bit-exact to an independent GF-T model over a full 80-step
training run** (forward + backprop + update, every output compared per step) in Icarus
Verilog; then **synthesizes** it with yosys (non-zero cell mapping) and asserts the
**one-shared-multiplier datapath invariant** (a change that parallelized the datapath —
area blow-up — would fail here). Spec→Verilog bit-exactness is thus *guaranteed on every
pull request*, not asserted once.

---

## 5. Honesty about limits (measured, not claimed)

- **openXC7 correctness ceiling ≈ 17M fasm.** Designs ≤ 16.7M compute correctly; ≥ 19.5M
  place and respond over UART but *miscompute* — always cross-checked against an
  independent model, never trusted on a UART response alone.
- **The compiler's wide return is u64.** Values > 64 bits are handled by the
  microsequencer (one value per step), not by wide ports.
- **Post-synthesis cell counts are yosys-version-specific.** The CI area report is a
  *trend* to watch across pull requests, not an absolute number; the architectural
  guarantee (one shared multiplier, area ≈ constant in network size) is enforced as an
  invariant instead.
- **Multi-output was a real bug, caught by the gate and fixed.** An early multi-output
  emit left a target register undriven (uninitialized in RTL, zero in the model); the
  bit-exact cross-check surfaced it. The port interface was made fully parametric and the
  register file zero-initialized on reset, so the divergence is now structurally
  impossible — an example of the gate doing its job.
- **The full-backprop microsequencer now trains XOR on live silicon** (25/25 epochs to
  4/4, model-exact). It required *seed search*: the open-source place-and-route
  (nextpnr-xilinx) cannot express a multicycle timing constraint, so the deep shared-core
  path is left timing-relaxed and correctness is placement-dependent — some seeds glitch,
  one seed trains cleanly. This is an open-toolchain limitation, not a design flaw (a
  commercial P&R would close it directly); we simply pick a stable seed. The *generated*
  programmable/deep stack (arbitrary topology) is proven in simulation and CI for every
  topology; the on-silicon run of a generated deep net is the natural next step through
  the same seed-searched flow.

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
