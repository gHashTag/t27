---
name: spec-first-ternary-nn
description: Author and verify a spec-first ternary (BitNet-class) neural-network datapath in .t27 that lowers to synthesizable Verilog. Use when building ternary MAC / neuron / layer / MLP specs under specs/ternary/, or extending the spec-first BitNet inference stack.
---

# Spec-first ternary NN cookbook (t27)

The accelerator's compute path can be written as **spec-first `.t27`** (not hand-written RTL): `t27c gen-verilog` lowers pure functions to synthesizable Verilog functions, and every node is verified **bit-exact** against an independent reference in iverilog+vvp. This is a differentiator no competitor (Ternary-NanoCore, TerEffic, bitnet.cpp) has.

The stack on master (`specs/ternary/`): `activation_quantizer.t27` (#1738) → `ternary_mac.t27` `dot27` (#1743) → `bitnet_neuron.t27` (#1747) → `bitnet_neuron_nchunk.t27` `neuronN` (#1752) → `bitnet_layer.t27` `layer2` (#1754) → `bitnet_mlp.t27` `mlp2` (#1756) → `bitnet_mlp3.t27` `mlp3` (#1759).

## Encoding & core idioms

- **Packed trit:** `{N = 0b00 = -1, Z = 0b01 = 0, P = 0b10 = +1}`. Trit `i` occupies bits `[2i+1:2i]` of a 54-bit (27-trit) chunk. Useful u64 constants for one uniform chunk: `P = 12009599006321322`, `Z = 6004799503160661`, `N = 0`.
- **Ternary multiply = sign logic, NEVER `*`.** gen-verilog lowers `*` to the *unsigned* `__mul_noop`, which is wrong for signed trits. Write:
  ```
  fn tmul(ta: u8, tb: u8) -> i8 {
      if (ta == 1) { return 0; }      // Z
      if (tb == 1) { return 0; }
      if (ta == tb) { return 1; }     // same sign -> +1
      return -1;
  }
  ```
- **dot27 (MAC) with a real loop** — extract each trit by shift+mask, accumulate:
  ```
  fn dot27(a: u64, b: u64) -> i16 {
      var acc : i16 = 0; var i : u32 = 0;
      while (i < 27) {
          var ta : u8 = ((a >> (i << 1)) & 3) as u8;
          var tb : u8 = ((b >> (i << 1)) & 3) as u8;
          acc = acc + tmul(ta, tb) as i16;
          i = i + 1;
      }
      return acc;                     // range [-27, +27]
  }
  ```
- **quantize** (activation re-ternarize): `v > +t -> P(2)`, `v < -t -> N(0)`, else `Z(1)`.
- **Inter-layer repacking** (`pack3`): a layer's output trits become the next layer's activation chunk. Start from all-Z, clear the low lanes, OR the trits in:
  ```
  fn pack3(t0: u8, t1: u8, t2: u8) -> u64 {
      var z : u64 = 6004799503160661;              // 27 Z lanes
      var cleared : u64 = z & 18446744073709551552; // & ~0x3F, clears lanes 0..2
      return cleared | (t0 as u64) | ((t1 as u64) << 2) | ((t2 as u64) << 4);
  }
  ```
- **N-chunk neuron** takes packed-array params `acts: [8]u64, weights: [8]u64` and loops `dot27` over `nchunks`. **Single-chunk `neuron1(act: u64, weight: u64, threshold)`** is for hidden layers whose activation is one packed chunk.

## Hard-won gotchas (all verified this session)

1. **Array-literal syntax is `[N]Type{e0, e1, ...}`** (e.g. `[4]u64{1,2,3,4}`), NOT `[1,2,3,4]`. The bare-bracket form parses the values as the *dimension string* and packs zeros. This was misdiagnosed as a compiler bug (#1749, closed not-a-bug). With correct syntax, array-literal `test`-block args simulate correctly.
2. **Local arrays don't pass to array-param functions.** `var h : [8]u64; h[0]=x;` lowers to an *unpacked* `reg [63:0] h[0:7]`, which cannot feed a function expecting a packed `[511:0]` input. Give hidden layers a single-chunk `neuron1(u64, u64, ...)` instead of building a local array.
3. gen-verilog backend fixes that unblocked all of this (already on master): **#1741** hoists function-local `reg` decls to the body-block top (loops/multi-locals now lower); **#1748** indexes packed-array params by part-select `xs[i*W +: W]` (was a bit-select).

## Verify every node two ways

- **In-spec `test` blocks → `t27c icarus-simulate`.** Scalar asserts always work (`assert_eq(dot27(0,0), 27)`); array-literal asserts work with the `[N]Type{...}` syntax (`assert_eq(neuronN([8]u64{...}, [8]u64{...}, 8, 10), 2)`). L4 (constitution) requires ≥1 test/invariant block.
- **Rust cross-check** (`bootstrap/tests/bitnet_*.rs`): a hand testbench packs uniform chunks directly into wide vectors (`{27{2'b10}}` = all-P) and cross-checks against an **independent** reference model (decode+MAC recomputed from scratch) over direct-packed cases + a discriminating low-threshold case that propagates a +1 through every layer. Skip gracefully when iverilog/vvp are absent. Keep sim TBs SMALL (a few deterministic cases, or ≤80 random) — interpreted nested-loop sim is slow; call the DUT once per case into a `reg`.

## Ship discipline (t27)

- `t27c seal <spec> --save` after every spec change; `--verify` must MATCH. Trust CI `validate` (the seal oracle), not local `seal --verify` (local binary can differ).
- Every PR: add one `docs/now/<YYYY-MM-DD>-<slug>.md` entry file dated today-UTC (`docs/NOW.md` is a frozen archive -- do not add entries there) + `Refs #N` in the PR body. **One clean commit** — the `Check L1 TRACEABILITY` gate fails *any* commit (incl. merge/docs) lacking an issue ref.
- master has a ruleset: merge with `gh pr merge --merge` (or `--auto`), **never** `--admin`, never force-push.

## What's next (roadmap)

Phase 1 (spec-first compute, bit-exact) is DONE. The path to an on-hardware MVP: **Phase 2** clocked streaming datapath (weights from BRAM, activations streamed) — the crux; **Phase 3** synthesize a small net via openXC7 → bitstream (Artix-7 AX7203, timing closure); **Phase 4** real BitNet weights, run on the board over UART vs a software reference = MVP.
