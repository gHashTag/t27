# BitNet v2 and the TRI-NET datapath — where the premise still holds, and where it does not

> **Question this answers.** Nine waves of verification work carried an open
> design question: *"[BitNet v2](https://arxiv.org/abs/2504.18415) moves the
> binding constraint from weight width to activation width — is a ternary-weight
> datapath still the right target?"*
>
> The short answer is **the question was posed wrongly**, and answering it
> properly surfaced something more urgent than the answer.

**φ² + 1/φ² = 3 | TRINITY**

Evidence class per section, as in [`FORMAL_FOUNDATIONS.md`](FORMAL_FOUNDATIONS.md):
`MEASURED` (reproducible observation over a stated domain) or `SOURCED` (from a
cited paper's own abstract, fetched not recalled).

---

## 1. What BitNet v2 actually changes — `SOURCED`

Titles and abstracts fetched from arXiv on 2026-08-09, not quoted from memory.

| Work | Weights | Activations |
|---|---|---|
| [BitNet](https://arxiv.org/abs/2310.11453) | 1-bit | not the subject |
| [BitNet b1.58](https://arxiv.org/abs/2402.17764) | **ternary `{-1,0,+1}`** | not the subject |
| [BitNet v2](https://arxiv.org/abs/2504.18415) | **still 1-bit** | **native 4-bit** |

BitNet v2's abstract is explicit that the obstacle is **activation outliers**,
which "complicate quantization to low bit-widths", and that its contribution is
`H-BitLinear` — "applying an online Hadamard transformation prior to activation
quantization" to smooth "sharp activation distributions into more Gaussian-like
forms". It reports 8-bit activations matching b1.58, and "minimal performance
degradation when trained with **native 4-bit activations**".

**Correction to the premise this project has been carrying.** BitNet v2 does
**not** move away from ternary weights — weights stay 1-bit. So the phrasing
"the binding constraint moved from weight width to activation width" is right
about *where the research effort went* and wrong if read as *ternary weights are
superseded*. **The ternary-weight premise is validated by BitNet v2, not
threatened by it.**

---

## 2. What this repo's datapath actually commits to — `MEASURED`

From the emitted RTL (`t27c gen-bitnet-bundle`), not from the design notes.

### 2.1 Activations are ternary too

```verilog
module trit27_dot_product (
    input  wire [53:0]       input_vec,     // 27 packed trits
    input  wire [53:0]       weight_vec,    // 27 packed trits
    output wire signed [5:0] result         // [-27, +27]
);
```

Both operands are 54-bit packed trits — 27 trits each — and the stdlib describes
the operation as "27 parallel **sign-only** multiplies". `pipeline_stage2_compute`
feeds it `input_chunk [53:0]` and `weight_chunk [53:0]`.

So the datapath performs **ternary × ternary**. Placed against the table above:

| | Weights | Activations |
|---|---|---|
| BitNet b1.58 | ternary | higher precision (8-bit in v2's comparison) |
| BitNet v2 | ternary | **4-bit**, and only reachable via a Hadamard transform |
| **this repo** | ternary | **ternary (~1.58-bit)** |

**This is more aggressive on activations than any published BitNet variant**, and
it is the axis the field has found *hard*. BitNet v2's entire contribution is
machinery for surviving 4-bit activations. No result cited here supports ~1.58-bit
activations, and this repository has never claimed accuracy evidence for it —
correctly, per [`BENCHMARKS.md`](../BENCHMARKS.md). But the RTL encodes the
assumption regardless.

### 2.2 There is no requantization stage

`pipeline_stage2_compute` produces `result` as `signed [15:0]`. The next layer's
input port is `[53:0]` packed trits. **Nothing in the bundle converts between
them.** Grepping the emitted RTL for `quant`, `requant`, `hadamard`, or `scale`
returns no module — only a `threshold` register in the CSR aperture.

That gap is exactly where `H-BitLinear` lives. Whatever the eventual activation
width, a layer boundary needs *something* there, and today it is absent rather
than simplified.

> **Closed 2026-08-09.** `t27c gen-activation-requant` emits
> `activation_requant`, wired into `bitnet_engine_top`: `signed [15:0]` →
> symmetric dead-zone on the `threshold` CSR → packed trits, 27 per word.
> **The activation-width fork now lives in one output port** rather than in the
> absence of a module. A 4-bit variant changes `trit [1:0]` to `act [3:0]` and
> swaps the dead-zone for a scale-and-round; nothing else in the datapath moves.
> Properties in [`FORMAL_FOUNDATIONS.md`](FORMAL_FOUNDATIONS.md) Prop. 15,
> including `a_trit_never_invalid` — the reserved `2'b11` code is proved
> unreachable.

### 2.3 The top level does not instantiate the datapath — `MEASURED`

`bitnet_engine_top` instantiates **three** of the nine emitted modules:

| Instantiated | Not instantiated |
|---|---|
| `multilayer_sequencer` | `pipeline_stage2_compute` ← *the MAC* |
| `layer_sequencer` | `weight_bram` |
| `double_buffer_ctrl` | `weight_prefetch_ctrl` |
| | `dma_controller` |
| | `axi_lite_slave` |
| | `interrupt_controller` |

and inside it:

```verilog
assign prefetch_done = 1'b1;   // "tied off until weight_prefetch_ctrl is wired"
assign mem_addr      = 32'd0;
assign mem_rd_en     = 1'b0;
input wire signed [15:0] threshold,   // declared, never referenced
```

Reproduce:

```bash
t27c gen-bitnet-bundle --output-dir build/rtl
sed -n '/^module bitnet_engine_top/,/^endmodule/p' build/rtl/bitnet_engine_top.sv \
  | grep -oE "^    [a-z_0-9]+ [a-z_0-9]+ \(" | sort -u
```

**The nine modules are nine independently emitted blocks, not an assembled
engine.** The top level wires the control plane and ties the data plane off.

> **Updated 2026-08-09 (same day).** Step 1 of §4 is done: `bitnet_engine_top`
> now instantiates `pipeline_stage2_compute`, a weight `weight_bram` and an
> activation `weight_bram`, with a one-cycle control skew matching the BRAM read
> latency, and `threshold` connected to gate `neuron_out`. **5 of 9** modules are
> now wired; `dma_controller`, `axi_lite_slave`, `interrupt_controller` and
> `weight_prefetch_ctrl` remain standalone. The first multi-module properties are
> proved in CI — see [`FORMAL_FOUNDATIONS.md`](FORMAL_FOUNDATIONS.md) Prop. 14.
> §3d's bound is correspondingly narrowed: `formal-yosys` now certifies
> module-level properties **and** the sequencer→BRAM→MAC composition, but still
> not the memory or host-interface path.

---

## 3. Consequences

**3a. The design question cannot be decided yet, and that is the answer.**
Choosing an activation width is a datapath decision, and there is no assembled
datapath to decide it in. Ternary-vs-4-bit activations is a real fork, but it
becomes actionable only once a layer boundary exists to put a quantizer on.

**3b. The ternary-weight premise is safe.** No change is warranted there, and
BitNet v2 is better cited as *support* than as pressure. `COMPETITORS.md` §2.5
already cites it correctly as motivation-not-evidence; that stands.

**3c. The claim that needs correcting is an integration claim, not a numerics
one.** `README.md` has carried *"BitNet HLS · RTL pipeline · GREEN · 9/9
modules"*. Nine modules **are** emitted, so the sentence is true — and it reads
as *a nine-module pipeline exists*, which is not. This is the same failure shape
the audit campaign has hit repeatedly: a metric that is accurate about the thing
it counts and misleading about the thing a reader infers. Corrected in this
commit to distinguish **emitted** from **integrated**.

**3d. Six verified RTL defects sit in blocks that are not yet wired together.**
That does not devalue them — the blocks are the deliverable, and defects found
pre-integration are the cheapest kind. But it does bound what
`formal-yosys` currently certifies: **module-level properties, not system
behaviour.** No end-to-end property can exist until 3c does.

---

## 4. Recommendation

Integration before numerics. Concretely, in order:

1. **Wire `pipeline_stage2_compute` + `weight_bram` into `bitnet_engine_top`**, so
   a single neuron's chunks actually flow. This is the smallest change that turns
   "9 modules" into "a datapath".
2. **Add the layer-boundary requantizer** — the module that consumes
   `signed [15:0]` and emits packed trits. Its interface is where the activation
   width question becomes concrete, and it is the natural home for an
   `H-BitLinear`-style transform if one is ever wanted.
3. **Only then** decide ternary vs 4-bit activations, with a real port to
   change and a benchmark to move.

Until step 2 exists, "should activations be 4-bit?" has no place in the design to
be answered.

---

## 5. What this document does not claim

- No accuracy or perplexity claim for ternary activations, in either direction.
  This repo has no model-level evaluation and [`BENCHMARKS.md`](../BENCHMARKS.md)
  governs.
- No claim that the ternary-activation choice is *wrong* — only that it is
  **unvalidated by any cited result** and more aggressive than the published
  state of the art on the axis the field finds hardest.
- No claim about silicon. [`STATUS.md`](../STATUS.md) still governs readiness.

---

**φ² + 1/φ² = 3 | TRINITY**
