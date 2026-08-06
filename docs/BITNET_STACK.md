# Spec-first ternary compute stack (BitNet-class)

A complete ternary (BitNet-class) compute path written as **spec-first `.t27`** — pure functions that `t27c gen-verilog` lowers to **synthesizable Verilog**, with every node verified **bit-exact** against an independent reference in iverilog + vvp. From a single neuron up to a multi-bit arithmetic datapath, with zero hand-written RTL.

This is a differentiator no competitor has: a neural network (and a small ALU) generated from a **ternary-native spec-first compiler**, not hand-written RTL or a CPU kernel.

See also: [`.claude/skills/spec-first-ternary-nn.md`](../.claude/skills/spec-first-ternary-nn.md) (authoring cookbook) and the visual roadmap artifact.

## The stack (`specs/ternary/`)

Encoding: packed trit `{N = 0b00 = -1, Z = 0b01 = 0, P = 0b10 = +1}`; trit `i` at bits `[2i+1:2i]` of a 54-bit (27-trit) chunk.

| Category | Spec | Function | Computes | Verified | PR |
|---|---|---|---|---|---|
| primitive | `activation_quantizer.t27` | `quantize` | re-ternarize accumulator by threshold | icarus 7/7 | #1738 |
| primitive | `ternary_mac.t27` | `dot27` | 27-trit ternary dot product (loop) | 300-vec cross-check vs `trit27_dot_product` | #1743 |
| neuron | `bitnet_neuron.t27` | `neuron4` | 4-chunk dot accumulation + quantize | 200-vec cross-check | #1747 |
| neuron | `bitnet_neuron_nchunk.t27` | `neuronN` | N-chunk neuron over `[8]u64` arrays | in-spec array-literal + direct-packed | #1752 / #1758 |
| layer | `bitnet_layer.t27` | `layer2` | 2 neurons → packed trits | direct-packed | #1754 |
| network | `bitnet_mlp.t27` | `mlp2` | 2-layer inference (repack between layers) | 5-case vs independent 2-layer ref | #1756 |
| network | `bitnet_mlp3.t27` | `mlp3` | 3-layer inference | 6-case vs independent 3-layer ref | #1760 |
| named fn | `bitnet_majority.t27` | `maj3`, `weighted_vote` | `sign(a+b+c)` / `sign(a+b-c)` (weights define the function) | exhaustive 27+27 | #1766 / #1768 |
| named fn | `ternary_xor.t27` | `ternary_xor` | XOR — **not linearly separable**, a genuine 2-layer net | exhaustive 9 + true-XOR | #1770 |
| arithmetic | `ternary_full_adder.t27` | `full_adder` | binary full adder = XOR ⊕ majority | exhaustive 8 | #1772 |
| arithmetic | `ternary_ripple_adder.t27` | `add2` | 2-bit ripple-carry adder (2 full adders, carry chain) | exhaustive 16 | #1776 |

**Verification methodology (two-way).** Every node is checked (1) in-spec via `t27c icarus-simulate` `test` blocks, and (2) by a Rust integration test (`bootstrap/tests/bitnet_*.rs`, `ternary_*.rs`) that generates the Verilog and cross-checks it against a **fully independent reference model** (decode + recompute from scratch) over direct-packed / exhaustive inputs. `t27c seal --verify` confirms all three backends (verilog/rust/c) hash-match; CI `validate` is the authoritative seal oracle.

**Backend fixes that unblocked the stack** (both in gen-verilog, both seal-neutral): **#1741** hoists function-local `reg` declarations to the body-block top (loops + multi-locals now lower); **#1748** indexes packed-array params by part-select `xs[i*W +: W]` (was a bit-select). Together they make `while`-loop MACs and `[N]u64` array parameters lower correctly.

## Where we are, honestly

The **combinational** spec-first ternary compute path is comprehensively demonstrated: classification (neurons, layers, MLPs), interpretable named functions (majority, weighted vote), the canonical perceptron→MLP result (XOR), and arithmetic (full adder, multi-bit adder). Further combinational demos are diminishing returns.

The next real step toward an **on-hardware MVP** (a network running on the AX7203 FPGA with real weights) is **Phase 2: a clocked, streaming datapath** — and every path to it is a compiler/architecture change that warrants a deliberate, reviewed effort:

### Frontier (decomposed plans filed as issues)

1. **Clocked/sequential construct — #1764 (the MVP gate).** The spec-first path is combinational-only: `gen-verilog` emits no `always @(posedge clk)`, module-level `var` state is never registered, and the direct interface is fixed `(clk,rst_n,en,ready)` with **no data ports**. `HirModule::convert` lowers every `fn` combinationally and never populates `always_blocks` (though the HIR emitter `emit_always_block` exists). *Plan:* add a `SeqBlock` to the grammar → lower it to `HirAlwaysBlock` in `convert` → route stateful specs through the HIR path (which models ports/memories/fifos) → registered-`var` semantics with reset/`en`. Then the bit-exact `dot27` can be wrapped in a streaming pipeline stage.
2. **Cross-module imports — #1773 (composability).** Each spec re-defines the same ~40 lines of primitives because the Verilog path compiles a single source string with no file context. *Plan:* give `compile_verilog` a repo-path context, resolve `use ternary::mac` to a file, parse + merge the imported `fn`s. Makes the stack a real reusable library.
3. **Hand-written engine — #1726 (alternative Phase-2 vehicle).** The clocked datapath already exists in the hand-written SV emitters, and the spec-first `dot27` is bit-exact equal to their `trit27_dot_product` (#1743). *Plan:* complete `bitnet_engine_top` — fix the input≡weight aliasing (it computes w·w), wire activation write-back and the quantizer, and resolve the stale `busy`/`mem` test contract (owner decision needed).

## Competitors

| Project | What it is | Spec-first? |
|---|---|---|
| Ternary-NanoCore (Artix-7) | hand-written ternary NN accelerator | no — hand RTL |
| TerEffic (2025) | ternary LLM FPGA design | no — hand RTL |
| bitnet.cpp | edge inference for ternary LLMs | no — CPU |
| bitSMM | bit-serial matmul accelerator | no — hand RTL |

None generates a network from a ternary-native spec-first compiler. That is this stack's unique position — the compute core is proven; the on-hardware MVP is Phases 2–4 of the roadmap, with the clocked datapath (#1764) as the crux.
