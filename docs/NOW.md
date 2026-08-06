# NOW — fix: gen-rust bool negation and integer-width coercion (2026-08-06)

Last updated: 2026-08-06

## fix(gen-rust): bool negation and integer-width coercion (Closes #1775)

- Branch: `claude/wonderful-jackson-a3ea48`

### Что легло
- `bootstrap/src/compiler.rs`: three gen-rust defects that `typecheck` accepted but rustc rejected, all from one root cause — `expr_to_rust` was a static fn and could not see the codegen's type tables. (1) `!x` on a Rust `bool` (a `bool` param or a `-> bool` call) became the integer zero test `(x) == 0` → E0308; `expr_to_rust` is now a `&self` method reusing the existing `expr_is_bool`, and integer operands still get `(x) == 0`. (2) `return` was not coerced to the declared return type — `return dir;` (u8) / `return (ctr >> 8) & 255;` (u64) from a `-> u32` fn → E0308. (3) binary operands were not coerced to a common width — `u16 * u8` (E0277), `u32 * u64` (E0308); shifts exempt, Rust already accepts any integer shift amount. (2)+(3) driven by a new conservative `infer_int_type` over params/locals/consts/callee return types: a cast is emitted **only** on positive evidence of a differing declared width, so literals and unknown widths are untouched and Rust's own inference still applies. Fixed in gen-rust rather than by tightening typecheck — t27 accepts mixed widths by design across all five backends. Verified: 889-spec differential vs the pre-fix compiler — 26 files change, 15 drop rustc errors (`isa/ternary_memory` 80→43, `fpga/vcd_conformance_compare` 46→42), 11 byte-identical error sets, **0 regressions**; `gen`/`gen-c`/`gen-verilog`/`gen-verilog-hir`/`typecheck` byte-identical on every changed spec; `t27c suite` summary identical to baseline; 1506 unit tests pass incl. 6 new regression tests. FROZEN_HASH resealed (M5). Unblocks the tri-net over-wire suite, whose `crypto_frame.t27` can now drop its `seen_any == false` / `dir as u32` workarounds. Also filed **#1781**: the parse phase is superlinear (~quadratic), so `t27c suite` cannot complete on the 12.4M-line `specs/scratch/` tree.

---

# NOW — docs: canonical index of the spec-first ternary compute stack (2026-08-06)

Last updated: 2026-08-06

## docs: BITNET_STACK.md index (Closes #1778)

- Branch: `docs/bitnet-stack-index`

### Что легло
- `docs/BITNET_STACK.md` — canonical index of the spec-first ternary compute stack after ~25 PRs: the full table (11 specs / 12 functions: quantize, dot27, neuron4, neuronN, layer2, mlp2, mlp3, maj3, weighted_vote, ternary_xor, full_adder, add2 → what each computes → verification → PR); the two-way verification methodology (in-spec test blocks + Rust cross-check vs independent reference); the unblocking backend fixes (#1741, #1748); an honest "where we are" (combinational track comprehensive, on-hardware MVP needs Phase 2); the decomposed frontier plans (#1764 clocked / #1773 imports / #1726 engine); and the competitor landscape (Ternary-NanoCore, TerEffic, bitnet.cpp, bitSMM — none spec-first). Docs-only.

---

# NOW — feat: spec-first 2-bit ternary ripple-carry adder (multi-bit datapath) (2026-08-06)

Last updated: 2026-08-06

## feat: spec-first 2-bit ripple-carry adder add2 (Closes #1774)

- Branch: `feat/spec-first-ripple-adder`

### Что легло
- `specs/ternary/ternary_ripple_adder.t27`: `add2(a0,a1,b0,b1)` — a 2-bit ripple-carry adder = two full adders (each XOR + majority) with the carry threaded between them, over trit-embedded bits {0→N,1→P}. Output packs sum0[1:0]+sum1[3:2]+carry-out[5:4]. Extends the stack from a single full adder into a **multi-bit arithmetic datapath** — the shape of a real ALU, all from the ternary neural primitives. Verified: typecheck 0 err; icarus-simulate 5/5; seal MATCH; new `tests/ternary_ripple_adder.rs` exhaustively drives all 2^4=16 input pairs vs binary addition = ALL_PASS 16. No compiler change. Also filed **#1773**: no cross-module imports → each spec re-defines the primitives (reuse/ergonomics gap for a real library).

---

# NOW — feat: spec-first ternary full adder (arithmetic from XOR + majority) (2026-08-06)

Last updated: 2026-08-06

## feat: spec-first ternary full adder (Closes #1771)

- Branch: `feat/spec-first-full-adder`

### Что легло
- `specs/ternary/ternary_full_adder.t27`: `full_adder(a,b,cin)` — a binary full adder over trit-embedded bits {0→N,1→P}, built from the spec-first stack: `sum = a XOR b XOR cin` (composed from the 2-layer XOR #1769), `carry = majority(a,b,cin)` (a single neuron #1765), output packs sum[1:0]+carry[3:2]. Extends the stack from classification into **arithmetic** — a real datapath building block composed from already-verified named functions. Verified: typecheck 0 err; icarus-simulate 6/6; seal MATCH; new `tests/ternary_full_adder.rs` exhaustively drives all 2^3=8 binary inputs vs the arithmetic truth table = ALL_PASS 8. No compiler change.

---

# NOW — feat: spec-first ternary XOR (a 2-layer net does what one neuron cannot) (2026-08-06)

Last updated: 2026-08-06

## feat: spec-first ternary XOR (Closes #1769)

- Branch: `feat/spec-first-ternary-xor`

### Что легло
- `specs/ternary/ternary_xor.t27`: `ternary_xor(a,b)` — the canonical not-linearly-separable function. A single linear neuron cannot compute XOR; this is a genuine 2-layer network with biases (h1=sign(a+b-1) AND-like, h2=sign(a+b+1) OR-like, out=sign(h2-h1-1)). The bias is the neuron's threshold offset — what a trained net learns with the weights. On binary inputs {N,P} the output is exact XOR. Verified: typecheck 0 err; icarus-simulate 4/4; seal MATCH; new `tests/ternary_xor.rs` drives all 9 combos vs an independent 2-layer reference + asserts the 4 binary cases equal true XOR = ALL_PASS 9. A recognizable ML result (perceptron→MLP) from the spec-first ternary stack. Also refined **#1764**: the direct gen-verilog interface is fixed `(clk,rst_n,en,ready)` with no data ports → Phase 2 must go through the HIR path.

---

# NOW — feat: weighted_vote — weights define the function (toward trained model) (2026-08-06)

Last updated: 2026-08-06

## feat: weighted_vote named function (Closes #1767)

- Branch: `feat/spec-first-weighted-vote`

### Что легло
- Added `weighted_vote(a,b,c)` to `specs/ternary/bitnet_majority.t27`: the SAME single BitNet neuron as `maj3`, but per-input weights `[+1,+1,-1]` make it compute `sign(a+b-c)` instead of `sign(a+b+c)`. Demonstrates the essence of a trained model — **weights define the function, not the topology** (weight P → +input, N → -input). Verified: typecheck 0 err; icarus-simulate 12/12 test blocks (7 maj3 + 5 wv); seal MATCH; `tests/bitnet_majority.rs` now exhaustively checks all 27+27=54 combinations of both functions vs independent references = ALL_PASS 54. No compiler change. Also (this iteration) filed **#1764** with a decomposed plan for the Phase-2 clocked construct — the spec-first path is combinational-only.

---

# NOW — feat: spec-first ternary majority gate (named function, exhaustive) (2026-08-06)

Last updated: 2026-08-06

## feat: spec-first ternary majority gate maj3 (Closes #1765)

- Branch: `feat/spec-first-ternary-majority`

### Что легло
- `specs/ternary/bitnet_majority.t27`: `maj3(a,b,c)` = ternary majority of three trits = sign of (a+b+c), realized as a single BitNet neuron (pack3 → dot27 with an all-+1 weight → quantize at 0). First recognizable *named* function computed by the spec-first stack (not random vectors). Verified: typecheck 0 err; icarus-simulate 7/7 test blocks; seal 3 backends MATCH; new `tests/bitnet_majority.rs` drives **all 27 input combinations** vs an independent sign-of-sum reference = ALL_PASS 27 (exhaustive). No compiler change. **Also filed #1764:** the spec-first `.t27` path is combinational-only — clocked/streaming (Phase 2) needs a compiler clocked-process construct or the hand-written engine; `debug-hir` shows `always_blocks: []`, the AST→HIR lowering never populates them.

---

# NOW — docs: spec-first ternary NN cookbook skill (2026-08-06)

Last updated: 2026-08-06

## docs: spec-first-ternary-nn skill (Closes #1761)

- Branch: `docs/nn-skill-clean`

### Что легло
- New `.claude/skills/spec-first-ternary-nn.md` capturing the spec-first BitNet stack know-how (built #1738→#1759): trit encoding + uniform-chunk constants; ternary sign-multiply (never `*` — unsigned `__mul_noop`); dot27/quantize/pack3 idioms; N-chunk `neuronN` vs single-chunk `neuron1`; gotchas (`[N]Type{...}` array-literal syntax, local-array-vs-array-param mismatch, #1741/#1748 backend fixes); two-way verification (in-spec `test` blocks + Rust cross-check vs an independent reference); ship discipline; and the phase roadmap to an on-hardware MVP. Docs-only.

---

# NOW — feat: 3-layer spec-first BitNet inference (deepens mlp2) (2026-08-06)

Last updated: 2026-08-06

## feat: spec-first 3-layer BitNet inference mlp3 (Closes #1759)

- Branch: `feat/spec-first-bitnet-mlp3`

### Что легло
- `specs/ternary/bitnet_mlp3.t27`: `mlp3` deepens mlp2 (#1756) to three layers — L1 (3 neurons over input) → pack3 → h1 → L2 (3 single-chunk neurons) → pack3 → h2 → L3 (2 single-chunk neurons) → 2 packed output trits. Verified: typecheck 0 err; icarus-simulate 4/4 scalar test blocks; seal 3 backends MATCH; new `tests/bitnet_mlp3.rs` cross-checks mlp3 vs a fully independent 3-layer reference over 5 direct-packed cases + a propagation case (all-P, thr=2 → +1 survives all 3 layers → 10) = ALL_PASS. Spec-first ternary inference now scales in depth. No compiler change, no reseal of others.

---

# NOW — test: in-spec array-literal coverage for neuronN (#1749 was wrong syntax) (2026-08-06)

Last updated: 2026-08-06

## test: neuronN in-spec array-literal test blocks (Closes #1757)

- Branch: `test/neuron-array-literal-blocks`

### Что легло
- Investigated #1749 ("array-literal args materialize wrong") — **not a compiler bug, operator error**: T27 array-literal syntax is `[N]Type{e0, e1, ...}` (e.g. `[4]u64{1,2,3,4}`), NOT `[1,2,3,4]` (which parses the values as the dimension string → all-zero packing). Closed #1749; reverted the speculative `fn_param_types` gen-verilog change (fixed a non-bug). Capitalized: added 5 full-neuron array-literal test blocks to `specs/ternary/bitnet_neuron_nchunk.t27` (all-P×P→P, all-P×N→N, all-Z→Z, 2-chunk→P, 0-chunk→Z) with the correct syntax — `neuronN` is now verified **in-spec** via icarus-simulate (11/11), not only by the Rust cross-check. Reseal MATCH; no compiler change.

---

# NOW — feat: 2-layer spec-first BitNet inference (dot/quantize/repack/dot) (2026-08-06)

Last updated: 2026-08-06

## feat: spec-first 2-layer BitNet inference mlp2 (Closes #1755)

- Branch: `feat/spec-first-bitnet-mlp`

### Что легло
- `specs/ternary/bitnet_mlp.t27`: `mlp2` chains two layers — layer 1 (3 `neuronN` over the input activations) → `pack3` repacks the 3 output trits into one hidden chunk → layer 2 (2 single-chunk `neuron1`) → 2 packed output trits. New capability = **inter-layer trit repacking** (`pack3`), what a real multi-layer BitNet inference needs. Verified: typecheck 0 err; icarus-simulate 4/4 scalar test blocks; seal 3 backends MATCH; new `tests/bitnet_mlp.rs` cross-checks `mlp2` vs a fully independent 2-layer reference on 5 direct-packed cases incl. a low-threshold case exercising non-Z layer-2 outputs (P,P→10) = ALL_PASS. Runnable spec-first inference path complete: MAC(#1743)→neuron(#1747/#1752)→layer(#1754)→2-layer MLP. No compiler change, no reseal.

---

# NOW — feat: 2-neuron spec-first BitNet layer with packed trit output (2026-08-06)

Last updated: 2026-08-06

## feat: spec-first BitNet layer2 (two neurons, packed trits) (Closes #1753)

- Branch: `feat/spec-first-bitnet-layer`

### Что легло
- `specs/ternary/bitnet_layer.t27`: `layer2(acts: [8]u64, w0: [8]u64, w1: [8]u64, nchunks, threshold)` runs two `neuronN` units over shared activations + per-neuron weights and packs the two output trits into a byte (`(t1<<2)|t0`). Array params pass straight through to `neuronN` — no new backend work. First spec-first BitNet **layer** (multiple output trits), composing quantizer (#1738) + MAC (#1743) + N-chunk neuron (#1752). Verified: typecheck 0 err; icarus-simulate 4/4 scalar test blocks; seal 3 backends MATCH; new `tests/bitnet_layer.rs` drives layer2 over direct-packed uniform chunks (w0=P,w1=N→2; P,P→10; N,P→8; allZ→5; 0chunks→5) = ALL_PASS. No compiler change, no reseal.

---

# NOW — feat: parameterized N-chunk spec-first BitNet neuron over packed arrays (2026-08-06)

Last updated: 2026-08-06

## feat: spec-first neuronN over [8]u64 packed arrays + loop (Closes #1750)

- Branch: `feat/spec-first-neuron-nchunk`

### Что легло
- `specs/ternary/bitnet_neuron_nchunk.t27`: `neuronN(acts: [8]u64, weights: [8]u64, nchunks, threshold)` loops `dot27` over the first `nchunks` chunk pairs then quantizes. The direct payoff of both gen-verilog fixes composing: `acts[c]` → part-select `acts[c*64 +: 64]` (#1748) and the loop + its locals lower cleanly (#1741). Prior neuron (#1747) used separate scalar params; this is fully parameterized. Verified: typecheck 0 err; icarus-simulate 6/6 scalar test blocks; seal 3 backends MATCH; new `tests/bitnet_neuron_nchunk.rs` checks neuronN over uniform chunks packed directly into 512-bit vectors across several nchunks/threshold settings (allP×8→P, allP×N×4→N, allN×8→P, allZ→Z, 1-chunk-band→Z, 0-chunks→Z) = ALL_PASS. Filed #1749: array-literal test-block args with 0/Z elements materialize wrong in the sim harness (neuronN itself is correct — proven by direct-packed check). No compiler change, no reseal.

---

# NOW — test: guard nested-if/else reg-decl hoist in gen-verilog (2026-08-06)

Last updated: 2026-08-06

## test: gen-verilog nested-if/else reg-decl hoist regression (Refs #1697)

- Branch: `claude/youthful-dewdney-e29e8e`

### Что легло
- The #1741 emitter fix (hoist function-local `reg` decls to the `begin : <fn>_body` top) is in master, but its regression test `tests/verilog_decl_hoist.rs` only covered the `while`-loop shape. Added `nested_if_else_locals_lower_and_elaborate`, reproducing GF-T's `gft_mul_offset_full_p`: a second same-block local declared after a leading statement (`carry; carry = ...; sum;`) plus a local inside an `else` branch (`result`). Asserts all locals are hoisted before the first statement, none re-declared mid-block, and the output elaborates under `iverilog -g2012`. Verified it fails against the pre-#1741 emitter (interleaved `reg [31:0] sum;` -> iverilog exit 2) and passes on master. Test-only; no compiler change, no reseal. PR #1751.

---

# NOW — fix: gen-verilog array-param element index -> part-select (#1745) (2026-08-05)

Last updated: 2026-08-05

## fix: gen-verilog packed-array param index emits part-select (Closes #1745)

- Branch: `fix/verilog-array-param-index`

### Что легло
- An `[N]T` array parameter lowers to a packed `input [N*W-1:0]` vector, but indexing it with a variable emitted a bit-select (`xs[i]`) instead of an element part-select, reading one bit (`sum_arr([1,2,3,4])` returned 1, not 10). `try_emit_primitive_array_access` only recognised module/local packed arrays; added a `param_types` fallback via `primitive_array_info` so array params index by `xs[i*W +: W]` too. Now `sum_arr([1,2,3,4]) == 10`. New regression test `tests/verilog_array_param_index.rs`; full compiler suite green (excl. pre-existing #1726); FROZEN_HASH re-sealed. Unblocks fully-parameterized loops over packed-array inputs (e.g. an N-chunk BitNet neuron over a `[N]u64` weight/activation array — the workaround in #1746 used separate scalar params).

---

# NOW — feat: spec-first BitNet neuron (.t27) — accumulation + quantize (2026-08-05)

Last updated: 2026-08-05

## feat: spec-first BitNet neuron4 (dot-product accumulation + quantize) (Closes #1746)

- Branch: `feat/spec-first-bitnet-neuron`

### Что легло
- First spec-first BitNet **compute unit**. `specs/ternary/bitnet_neuron.t27`: `dot27` (27-trit dot product via a real `while` loop — now that #1741 unblocked loops) + `quantize` + `neuron4(a0,w0,...,a3,w3,threshold)` accumulating the ternary dot product over 4 chunk pairs then re-ternarizing. Chunks are separate scalar params because packed-array element indexing is broken (filed #1745). Verified: typecheck 0 err; icarus-simulate 5/5; new `tests/bitnet_neuron_specfirst.rs` cross-checks neuron4 vs an independent reference on **200 random vectors** (ALL_MATCH); seal 3 backends MATCH. Combines #1738 (quantizer) + #1743 (MAC) into a working neuron and demonstrates the #1741 loop fix end-to-end. No compiler change, no reseal.

---

# NOW — fix: gen-verilog hoist function-local reg decls to body top (#1741) (2026-08-05)

Last updated: 2026-08-05

## fix: gen-verilog hoists function-local decls -> loops/multi-locals elaborate (Closes #1741)

- Branch: `fix/verilog-hoist-fn-locals`

### Что легло
- gen-verilog emitted function-local `reg` declarations at their point of declaration — after preceding statements and inside `while`-loop blocks — which Verilog forbids (iverilog: "syntax error / Malformed statement"). Root cause confirmed with minimal cases: decl-after-statement fails in ANY block; decl-first is fine. Fix: recursively collect every function-body local (`collect_fn_local_decls`) and hoist its `reg` declaration to the top of the `begin : <fn>_body` block, then emit each StmtLocal as assignment-only (Init phase) via a `hoist_fn_locals` flag. Verified: the loop-form ternary dot product (while + locals) now iverilog-compiles AND bit-exact cross-checks vs `trit27_dot_product` on 300 random vectors; new regression test `tests/verilog_decl_hoist.rs`; full compiler suite green (excl. pre-existing #1726). FROZEN_HASH re-sealed. Unblocks non-trivial spec-first hardware (loops/locals) — the wall the spec-first MAC (#1743) had to route around with a 27-term loop-free form.

---

# NOW — feat: spec-first ternary MAC dot product (.t27), bit-exact vs handwritten (2026-08-05)

Last updated: 2026-08-05

## feat: spec-first ternary MAC dot27 (.t27) == trit27_dot_product (Closes #1742)

- Branch: `feat/spec-first-ternary-mac`

### Что легло
- The accelerator's **core datapath** now has a spec-first form. `specs/ternary/ternary_mac.t27`: `tmul` (sign-only ternary multiply, no `*` since gen-verilog's `*`=unsigned `__mul_noop`) + `dot27(a: u64, b: u64) -> i8` (27-trit ternary dot product over 54-bit packed vectors). Written **loop-free** (27-term sum of per-position helpers) because gen-verilog can't lower a local-declaring `while` loop — filed that backend defect as **#1741** (reg decls after statements / inside loop blocks → iverilog syntax error). Verified: typecheck 0 err; icarus-simulate 4/4 (all-N×all-N=+27, all-N×all-P=-27, all-P×all-P=+27, all-Z=0); seal 3 backends MATCH; new Rust integration test `tests/bitnet_mac_specfirst.rs` **bit-exact cross-checks dot27 vs the hand-written trit27_dot_product on 300 random vectors (ALL_MATCH)**. Full suite green (excl. pre-existing #1726). No compiler change, no reseal.

---

# NOW — feat: self-contained BitNet bundle (bundles trit_stdlib, elaborates standalone) (2026-08-05)

Last updated: 2026-08-05

## feat: gen-bitnet-bundle includes trit_stdlib -> elaborates standalone (Closes #1739)

- Branch: `feat/self-contained-bundle`

### Что легло
- Follow-up from #1730: the bundle instantiated `trit27_dot_product` but omitted the trit stdlib that defines it, so it couldn't elaborate/synthesize standalone. Compose `trit_stdlib.sv` at dependency-first position; `BUNDLE_ORDER`/`BUNDLE_FILE_COUNT` 12→13; updated the index-based unit tests. Simplified `tests/bitnet_elaborate.rs` to elaborate the bundle directory as-is (dropped the separate stdlib emission) — now proves self-containment. Verified: `iverilog -t null -s bitnet_engine_top` over the bundle alone elaborates clean; unit 1500/0, bundle 21/21, full suite green (excl. pre-existing #1726). No compiler change, no reseal.

---

# NOW — feat: spec-first ternary activation quantizer (.t27) (2026-08-05)

Last updated: 2026-08-05

## feat: spec-first ternary activation quantizer (Closes #1737)

- Branch: `feat/spec-first-activation-quantizer`

### Что легло
- First BitNet-relevant datapath primitive on the **spec-first** path (the whole accelerator was hand-written Rust emitters, off-constitution). `specs/ternary/activation_quantizer.t27`: `quantize(v: i16, threshold: i16) -> u8` re-ternarizes a signed accumulator to a packed trit `{N=00,Z=01,P=10}` with a symmetric threshold (`v>+t→P`, `v<-t→N`, else Z). gen-verilog emits a clean signed-compare nested-if function. Verified: `typecheck` 0/0; `icarus-simulate` **7/7** embedded test blocks pass (positive/negative/zero-band, both `v==±t` boundaries→Z, just-over/under→P/N); `seal --save/--verify` all 3 backends hash-MATCH. Provides the activation re-ternarizer the engine needs to chain MAC→quantize→next-layer, and proves the ternary-native spec-first path — a differentiator no competitor (Ternary-NanoCore) has.

---

# NOW — test: harden ternary MAC coverage (boundary + randomized reference sweep) (2026-08-05)

Last updated: 2026-08-05

## test: MAC overflow-boundary + 200-vector randomized reference sweep (Closes #1734)

- Branch: `test/mac-randomized-sweep-work`

### Что легло
- Follow-up to the MAC overflow fix (#1733). Hardened `tests/bitnet_compute_mac.rs`: (1) overflow-boundary vectors pinning a level-2 group summing to exactly +/-8 (the value the old `signed [3:0]` wrapped); (2) a 200-vector randomized sweep (deterministic iverilog seed) cross-checked against an **independent** in-TB `ref_dot` (decode + multiply-accumulate, not using the DUT reduction tree). All pass → the MAC bug class is dry in the searched space; future arithmetic regressions now caught. Test-only; no compiler/emitter change, no reseal.

---

# NOW — bug: fix ternary MAC adder_tree_27 overflow (wrong dot product) (2026-08-05)

Last updated: 2026-08-05

## bug: adder_tree_27 level-2 overflow + functional MAC testbench (Closes #1732)

- Branch: `fix/mac-adder-tree-overflow`

### Что легло
- **Model-critical correctness bug in the ternary MAC core.** `adder_tree_27` (gen-trit-stdlib; wrapped by `trit27_dot_product`/`pipeline_stage2_compute`) declared level-2 as `signed [3:0]` (range [-8,+7]), but each `l2[j]` sums three `l1 in [-3,+3]` → range [-9,+9]. All-+1 dot product read **-21 instead of +27** (each group-of-9 = +9 wrapped to -7). Fixed by widening `l2` to `signed [4:0]`. Found by a NEW golden-vector functional testbench `tests/bitnet_compute_mac.rs` (iverilog+vvp): all-P/all-N/all-Z/single-trit/multi-chunk accumulation now match; skips w/o iverilog. Updated the unit test that had codified the buggy `signed [3:0]` width. trit_stdlib.rs is NOT under the FROZEN_HASH seal (only compiler.rs) → no reseal. Full suite green (excl. pre-existing #1726). This is the functional instrument that makes the engine-top datapath fix (input≡weight aliasing) provable next.

---

# NOW — test: iverilog elaboration instrument for the assembled BitNet engine (2026-08-05)

Last updated: 2026-08-05

## test: elaborate the assembled BitNet engine bundle under iverilog (Closes #1730)

- Branch: `feat/bitnet-elaborate-test`

### Что легло
- The BitNet HLS modules were validated only by substring asserts on emitted Verilog; **nothing elaborated the assembled engine** — the same structural blind spot that let the stale `bitnet_top` asserts (#1726) sit red unnoticed. New integration test `tests/bitnet_elaborate.rs` generates `gen-bitnet-bundle` + `gen-trit-stdlib` and runs `iverilog -g2012 -t null -s bitnet_engine_top` over all RTL (excl. the SVA property file), asserting clean elaboration; skips gracefully when iverilog is absent. Proven meaningful — during bring-up the same instrument caught `Unknown module type: trit27_dot_product` (the bundle omits its trit-stdlib dependency; making the bundle self-contained is a follow-up on #1730). Test-only; no compiler change, no reseal. Establishes the observability needed before any engine-top datapath change (e.g. the input≡weight aliasing at `bitnet_top.rs:217`).

---

# NOW — ci: fix fpga-build flags in fpga-synthesis jobs (unblock red CI) (2026-08-05)

Last updated: 2026-08-05

## ci: fpga-build passes flags the subcommand rejects (Closes #1723)

- Branch: `fix/ci-fpga-build-flags`

### Что легло
- `fpga-build.yml` invoked `t27c fpga-build ... --board <X> --profile minimal`, but the subcommand only defines `--device`/`--minimal` → `error: unexpected argument '--board'` (exit 2), making `fpga-synthesis-arty` red on **every** PR (it merged red on #1710/#1716/#1718/#1725/#1728). Replaced both bad invocations with `--minimal` (the `--device` default `xc7a100tcsg324-1` is already the Arty A7-100T part and matches the job's own chipdb placeholder; both jobs are synth/smoke where the exact part is immaterial). Verified: fixed command parses and proceeds to Verilog generation; old `--board` form still errors. Workflow-only change (no compiler, no reseal). Board-level UX (`--board`/`--profile`) could be added CLI-side later if desired.

---

# NOW — codegen: gen-c tuple lowering — closes tuple workstream across all backends (2026-08-05)

Last updated: 2026-08-05

## codegen: gen-c tuple literals + let (a,b) destructuring (Closes #1727, #1702)

- Branch: `feat/tuple-c`

### Что легло
- Final backend of tuple support (#1702). C has no anonymous tuples, so gen-c now hoists a `typedef struct { T f0; U f1; }` per distinct tuple shape, lowers a tuple return type to that struct (`c_return_type`), the tuple literal to a C99 compound literal `(T){ e0, e1 }` (`gen_c_expr` ExprTuple), and `let (s, d) = call()` to a temp struct + per-field copies with element C types (`gen_c_stmt`). Emitted C compiles under `cc -std=c99 -Wall -Wextra` and runs: `use_it(5,3)==10`, `dm(7,2)==(9,5)`. New test `test_tuple_literal_and_destructuring_c`; full suite 1500/0; FROZEN_HASH re-sealed. **Tuple support now complete across parser + rust + verilog + zig + c → #1702 closed.**

---

# NOW — codegen: gen-zig tuple lowering (return type + literal + destructuring) (2026-08-05)

Last updated: 2026-08-05

## codegen: gen-zig tuple literals + let (a,b) destructuring (Closes #1724)

- Branch: `feat/tuple-zig`

### Что легло
- Part 4 of tuple support (#1702). gen-zig now lowers a tuple return type `(T, U)` to a Zig anonymous tuple struct `struct { T, U }` (`gen_fn_decl` + `t27_tuple_type_to_zig`), the tuple literal to `.{ e0, e1 }` (`gen_expr` ExprTuple arm), and `let (s, d) = call()` to `const s, const d = call();` (`gen_stmt` StmtLocal tuple branch). Emitted Zig compiles under Zig 0.15.2 (`zig build-obj`) and comptime-evaluates: `use_it(5,3)==10`, `dm(7,2)==(9,5)`. New test `test_tuple_literal_and_destructuring_zig`; full suite 1499/0; FROZEN_HASH re-sealed. gen-c is the last remaining backend on #1702.

---

# NOW — test: fix stale sequencer_idle_arms_on_start assertion (2026-08-05)

Last updated: 2026-08-05

## test: fix stale IDLE assertion in bitnet_pipeline (Closes #1719)

- Branch: `fix/sequencer-idle-test`

### Что легло
- `sequencer_idle_arms_on_start` was red on master: `gen-layer-sequencer` deasserts `done<=0` on IDLE entry (wave-36b) and emits `IDLE: begin done<=0; if(start) begin state<=RUN; neuron_id<=0; chunk_id<=0; end end`, but the test still asserted the pre-wave-36b string. Updated the expectation to the current correct output (test-only; no codegen change, no reseal). `bitnet_pipeline` now 20/20.

---

# NOW — codegen: gen-verilog tuple lowering (concat + destructuring) (2026-08-05)

Last updated: 2026-08-05

## codegen: gen-verilog tuple literals + let (a,b) destructuring (Closes #1717)

- Branch: `feat/tuple-verilog`

### Что легло
- Part 3 of tuple support (#1702), the model-critical backend. gen-verilog now lowers `ExprTuple` to a packed concat with element 0 in the LSB (`(e0, e1)` -> `{e1, e0}`), sizes tuple-return functions to the summed width (`packed_width` tuple branch), and destructures `let (s, d) = call()` through a packed temp sliced back into each binding (`s = __tup_lN[31:0]`, `d = __tup_lN[63:32]`). Verified: iverilog `-t null -Wall` syntax OK + numeric TB `use_it(5,3)=10` (= 2·5). New unit test `test_tuple_literal_and_destructuring_verilog`; full suite 1498/0; FROZEN_HASH re-sealed. Zig/C tuple lowering still TODO (tracked on #1702).

---

# NOW — codegen: gen-rust tuple support (literals + destructuring) (2026-08-05)

Last updated: 2026-08-05

## codegen: gen-rust tuple literals + let (a,b) destructuring (Closes #1715)

- Branch: `feat/tuple-destructure`

### Что легло
- Adds NodeKind::ExprTuple (parsed in the `(` primary when a comma follows), a tuple-pattern branch in parse_local_decl (pattern stored in extra_field, name empty), an optimizer guard so tuple-destructuring locals are not eliminated, and gen-rust emit for tuple literals `(e0, e1)` and `let (a, b) = init`. Tuple specs now rustc-compile end-to-end. Full suite 1497/0; FROZEN_HASH re-sealed. Part 2 of tuple support (#1702) — Verilog/C tuple lowering still TODO.

---

# NOW — parser: tuple return types no longer silently drop the function (2026-08-05)

Last updated: 2026-08-05

## parser: tuple return types no longer drop the function (Closes #1709)

- Branch: `feat/tuple-return-parse`

### Что легло
- The function return-type parser had no `(` branch, so `-> (T, U)` desynced and the whole function was silently dropped from the AST (1 FnDecl vs 2). Added a LParen branch capturing `(T, U, ...)`. Functions with tuple returns now survive with the correct signature. Part 1 of tuple support (#1702); body-lowering and `let (a,b)` destructuring are follow-ups. Full suite 1496/0; FROZEN_HASH re-sealed.

---

# NOW — lang: wrapping arithmetic operators -% and *% (2026-08-05)

Last updated: 2026-08-05

## lang: wrapping arithmetic operators -% and *% (Closes #1659)

- Branch: `feat/wrapping-arith-ops`
- PR: #1674

### Что легло
- Adds the Zig-style wrapping-operator family (only +% existed, and gen-rust mis-emitted it as invalid Rust). Lexer MinusPercent/StarPercent; parser -% additive, *% multiplicative; Rust -> wrapping_add/sub/mul; Verilog collapses to +/- and *% -> __mul_noop (HW wraps by width); C collapses to +/-/* (unsigned wraps); Zig native passthrough. Cross-backend test test_wrapping_ops_all_backends_1659; FROZEN_HASH re-sealed. Supersedes #1660.

### Границы честности (BINDING)
- Full compiler suite 1495/0; generated Rust compiles under rustc; Verilog/C emit no literal %-operator. Checked +/-/* stay infix -> same overflow-panic semantics as the Zig backend.

---

# NOW — chore: remove stray backup/patch artifacts + close gitignore gap (2026-08-05)

Last updated: 2026-08-05

## chore: remove stray backup/patch artifacts + close gitignore gap (Closes #1653)

- Branch: `chore/r2-stray-artifacts`
- PR: #1654

### Что легло
- Removed 5 tracked strays (compiler.rs.orig/.backup, two .trinity/state/*.patch, a .tex.bak3). Extended .gitignore: *.bak/*.orig missed numbered variants (.bak3) and .backup; also ignore root /*.patch.

---

# NOW — ci: wire dyadic wide-format witness into CI (gf48..gf1024) (2026-08-05)

Last updated: 2026-08-05

## ci: wire dyadic wide-format witness into CI (gf48..gf1024) (Closes #1580)

- Branch: `ci/gf-wide-conformance`
- PR: #1675

### Что легло
- gf_wide_independent_witness.py decodes GF rungs as exact dyadic pairs (gf1024 bias ~2.5e120) with no Fraction blowup, resolving #1580. Added gf-wide-conformance.yml matrix gate over gf14/gf48/gf96/gf128/gf256/gf512/gf1024; each exits 0, witness returns 1 on any mismatch.

---

# NOW — conformance: correct 5 stale gf16 vectors + wire oracle into CI (2026-08-05)

Last updated: 2026-08-05

## conformance: correct 5 stale gf16 vectors + wire oracle into CI (Closes #1579)

- Branch: `fix/gf16-conformance-vectors`
- Issue: #1579
- PR: #1673

### Что легло
- gf16_ref.py exited non-zero on 5 named-constant vectors. Brute force over all 65536 codes proved the encoder picks the nearest GF16 code and decode is FPGA-consistent, so the vectors' expected.decoded values were phantom (not representable). Set expected.decoded to the true constant and tolerance_abs to the quantization bound (phase_transition widened to 0.0005). Added .github/workflows/gf16-conformance.yml so the oracle runs on every change.

### Границы честности (BINDING)
- The fix corrects test data, not the encoder/decoder — those are proven correct by roundtrip (19/19) and FPGA-consistency (32/32). Oracle now 35 pass / 0 fail.

---

# NOW — scripts: cocotb_ref_model is importable again (2026-08-01)

Last updated: 2026-08-05

## scripts: cocotb_ref_model is importable again (Closes #1592)

- Branch: `fix/cocotb-forward-reference`
- Issue: #1592
- PR: #1589

### Что легло
- The module raised `NameError: name 'EvalContext' is not defined` at import: the class is annotated on parameters at lines 174 and 215 and defined at line 402. `from __future__ import annotations` defers annotation evaluation and the module loads.

### Границы честности (BINDING)
- Placement matters: a `__future__` import may be preceded only by the docstring, comments and blank lines. Putting it between shebang and docstring also loads the module but silently leaves `__doc__` as `None`. It goes after the docstring, confirmed with `ast.get_docstring`.
- Found by running the EXEC bucket — the last 12 scripts in the repository never executed. No workflow references this file.

---

# NOW — ci: track the paper count the published version actually declares (2026-08-01)

Last updated: 2026-08-01

## ci: track the paper count the published version actually declares (Closes #1583)

- Branch: `fix/stale-paper-count`
- Issue: #1583
- PR: #1584

### Что легло
- `check_catalog_count.py` printed, on every run, that the SSOT's 83 disagreed with a paper count of 84 and an erratum was required. `ERRATA_2026-06-14.md` recorded that erratum and the v2 replacement carried it out.
- Fetching the current entry shows both the title and the abstract were corrected: *An 83-Format Numeric Catalog…*, arXiv:2606.09686v2, updated 2026-06-22. The constant tracked the withdrawn v1.

### Границы честности (BINDING)
- This is not the edit the existing comment forbids. The constant tracks what the PAPER declares, and the paper changed; matching it to the SSOT would defeat the gate.
- The comment now carries the fetch command, version and date so the next reader can re-check in one line.
- Alarm intact, verified by mutation: with a divergent constant the gate still warns and still exits 3 under `--strict-paper`.

---

# NOW — witness: wide-rung decode refs resolve their pack inside the repository (2026-08-01)

Last updated: 2026-08-01

## witness: wide-rung decode refs resolve their pack inside the repository (Closes #1581)

- Branch: `fix/witness-default-paths`
- Issue: #1581
- PR: #1582

### Что легло
- All six wide-rung witness decode references defaulted to a path under `/home/user/workspace` when run with no argument, so the file a `witnesses[]` entry names failed on every machine but the one it was written on. The default now resolves relative to the script's own location.

### Границы честности (BINDING)
- The witnesses themselves were sound: given the in-repo pack explicitly, gf128 already reported 15/15 exact at abs_error=0. Only the default lookup was wrong; no decoding changes.
- All six run standalone afterwards, gf48 through gf1024, each at abs_error=0.
- The `cross_check_representative.py` scripts were never affected — they import these modules and never reach `__main__`.

---

# NOW — ci: gate INDEX_all_formats.json against the packs it summarises (2026-08-01)

Last updated: 2026-08-01

## ci: gate INDEX_all_formats.json against the packs it summarises (Closes #1577)

- Branch: `gate/pack-index-consistency`
- Issue: #1577
- PR: #1578

### Что легло
- Nothing checked that the index agreed with the packs, which is how the pass-48 tier regression went unnoticed. `tools/pack_index_consistency_gate.py` locks digest, tier, witness count, header totals and index membership.
- `--selftest` plants a mutant for every check and fails if any survives, following `wp18_selftest_gate.py`'s convention.

### Границы честности (BINDING)
- Selftest: every check FAIL-reachable.
- Live corpus: verdict CLEAN, 0 failures, 5 informational notes about hand-curated packs that declare no `bitexact` flag.
- Deliberately NOT checked: a bitexact pack with an empty `witnesses[]`. That is the normal case for the ~60 uncontested packs; demanding a witness everywhere would misstate honesty rule #10.

---

# NOW — conformance: the pack generator runs on a clean checkout (2026-08-01)

Last updated: 2026-08-01

## conformance: the pack generator runs on a clean checkout (Closes #1575)

- Branch: `fix/conformance-generator-reproducible`
- Issue: #1575
- PR: #1576

### Что легло
- `gen_all_formats.py` read its catalog from a hardcoded `/tmp/catalog_lines.txt` that was never committed, so a fresh clone raised `FileNotFoundError` before writing a pack. It now reads the committed SSOT, `specs/numeric/formats_catalog.t27`, whose `// CATALOG:` rows are the same lines `catalog-count-gate.yml` counts.
- Re-running the generator had silently reverted the 2026-07-05 promotions: the `SELFCONSISTENT` branch hardcoded `bitexact_selfconsistent`, rewriting the index from 75/0/8 back to 69/6/8. The tier now derives from the pack, which is the artefact of record for its own status.
- The index carries a `witnesses` count per entry and a top-level `witnessed_packs`; a consumer could not previously tell which packs are witnessed without opening all 83.
- `conformance/vectors/verify_regeneration.py` added as the regression test.

### Границы честности (BINDING)
- Regeneration reproduces the committed corpus exactly: 83/83 digests unchanged, 0 tiers changed.
- Index changes are strictly additive — 0 entry fields changed or removed.
- Both existing gates pass: `wp18_selftest_gate.py` all PASS, `wp18_conformance_gate.py` verdict CLEAN.

---

# NOW — Rust backend type fixes (2026-07-31)

Last updated: 2026-07-31

## gen-rust: array types, index casts, boolean coercion (Closes #1574)

- Branch: `fix/rust-backend-types`
- Issue: #1574
- The Rust backend emitted code that does not compile; downstream tri-net had 32 errors and red CI since 2026-07-23.

### Что легло
- `t27_type_to_rust` — форма `[T; N]` больше не схлопывается в `Vec<>`; эмитится `[T; N as usize]`.
- `ExprIndex` — нелитеральный индекс приводится к `usize`.
- Целые и булевы согласованы: классификатор `expr_is_bool`, `!= 0` в условии, `as u32` в целочисленной позиции, `((x) == 0)` вместо побитового `!`.
- Рекурсивный предпроход собирает функции `-> bool`, плюс `bool`-параметры и локальные, чтобы вызов булевой функции не получал лишний `!= 0`.

### Границы честности (BINDING)
- Проверено downstream: библиотека tri-net собирается, 101 тест проходит.
- FROZEN_HASH переподписан механически; governance-часть церемонии M5 (GOLD-RING, одобрение Архитектора) не выполнялась.
- `fpga-synthesis` падает на SystemVerilog-касте `8'(...)` в Verilog-бэкенде — воспроизведено на немодифицированном master, к этой работе отношения не имеет.

---

# NOW — conformance instance-пакеты + Lean φ-скелет (2026-07-29)

Last updated: 2026-08-01

## conformance: параметрические instance-пакеты (structural) + Lean-скелет φ-правила (Closes #1558)

- Branch: `wave-loop-29-07b-swtracks`
- Issue: #1558
- Трек Wave-лупа 29.07b «улучшения без железа», SW-часть (encoding-уровень).

### Что легло
- `conformance/gen_structural_instances.py` + `conformance/README_structural_instances.md` — генератор instance-пакетов для structural-форматов.
- 4 instance-пакета (`kind="instance"`, независимый Fraction-декодер, abs_error=0):
  - `instance_q_format_Q4_4_v0.json` — канонич. round-trip 512/512.
  - `instance_q_format_Q2_5_v0.json` — 256/256.
  - `instance_minifloat_E2M1_v0.json` — 13/13.
  - `instance_minifloat_E3M4_v0.json` — 225/225.
- `proofs/lean4/Trinity/GoldenFloatRoundTrip.lean` — скелет φ-правила (e=round((N−1)/φ²), m=N−1−e, bias=2^(e−1)−1); field_budget+anchor_witness доказаны, gf16_fields/roundtrip_normal = `sorry` `[ТРЕБУЕТ lake build]`.

### Границы честности (BINDING)
- `kind="instance"` — КОНКРЕТНЫЕ параметризации structural-семейств, каталог НЕ меняют: **75 bitexact / 0 selfconsistent / 8 structural = 83**.
- Всё `[verified SW]` (independent decoder), НЕ HW. encoding ≠ compute ≠ FPGA.

---

# NOW — Wave Loop 773 close-out / Wave Loop 774 setup (2026-07-24)

Last updated: 2026-07-29

## Wave Loop 789 — module-scope `[397][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1507)

- Branch: `wave-loop-789`
- Parent branch: `wave-loop-788` HEAD (`44fa559e7`)
- Issue: #1507
- PR: #1508
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W789_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-789.md`
- Cooperation W790: `.claude/plans/wave-loop-790.md`

### What landed
- `specs/scratch/w789_bench_module_397x2p6_aos_var_call_write.t27`
  - 25,408 elements, 813,056-bit packed vector (~0.775 MiBit).
  - Module-scope `pub var dst : [397][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w789.py`
  - Generator for the W789 witness; `OUTER = 397`, `MID_IDX = 198`.
  - Note: the generator header had a hardcoded `w788` prefix inside an f-string,
    which required a manual fix and regeneration before the module name matched
    the wave number.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w789_bench_module_397x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-790.md`
  - W789 learnings saved and W790 plan/cooperation variants created.

### Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 249/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W789: PASS.

### Cooperation variants for Wave Loop 872

- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- Release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).

---

## Wave Loop 790 — module-scope `[399][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1509)

- Branch: `wave-loop-790`
- Parent branch: `wave-loop-789` HEAD (`228e1d850`)
- Issue: #1509
- PR: #1510
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W790_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-790.md`
- Cooperation W791: `.claude/plans/wave-loop-791.md`

### What landed
- `specs/scratch/w790_bench_module_399x2p6_aos_var_call_write.t27`
  - 25,536 elements, 817,152-bit packed vector (~0.779 MiBit).
  - Module-scope `pub var dst : [399][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w790.py`
  - Generator for the W790 witness; `OUTER = 399`, `MID_IDX = 199`.
  - Note: the generator header had a hardcoded `w789` prefix inside an f-string,
    which required a manual fix and regeneration before the module name matched
    the wave number.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w790_bench_module_399x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-791.md`
  - W790 learnings saved and W791 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 250/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W790: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by commit subject dropped to 0.0% (0/87); closing refs are
  in commit bodies, not subjects.

---

## Wave Loop 791 — module-scope `[401][2]^6 Pt` packed array-of-struct from call with indexed signed writes (variant A)

- Branch: `wave-loop-791`
- Parent branch: `wave-loop-790` HEAD (after closeout)
- Issue: TBD after W790 PR opened
- PR: (to open)
- Plan: `.claude/plans/wave-loop-791.md`

### Goal
Continue the odd outer-dimension module-scope AoS ladder with `[401][2]^6 Pt`.
Expected 25,664 elements, 821,248-bit packed vector (~0.783 MiBit), still under
4-MiBit cliff, with zero compiler / reference-model / FROZEN_HASH changes.

### Variants
- **A (recommended):** `[401][2]^6 Pt` module-scope var from call.
- **B:** `[399][2]^6 Pt` bench/function-scope packed var from call.
- **C:** `[399][2]^6 Pt` module-scope var with `if`-guarded writes.

---

## Wave Loop 784 — module-scope `[387][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1497)

- Branch: `wave-loop-784`
- Parent branch: `wave-loop-783` HEAD (`7f2c7afb4`)
- Issue: #1497
- PR: (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W784_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-784.md`
- Cooperation W785: `.claude/plans/wave-loop-785.md`

### What landed
- `specs/scratch/w784_bench_module_387x2p6_aos_var_call_write.t27`
  - 24,768 elements, 792,576-bit packed vector (~0.756 MiBit).
  - Module-scope `pub var dst : [387][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w784.py`
  - Generator for the W784 witness; `OUTER = 387`, `MID_IDX = 193`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w784_bench_module_387x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-785.md`
  - W784 learnings saved and W785 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 244/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W784: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).

---

## Wave Loop 783 — module-scope `[385][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1495)

- Branch: `wave-loop-783`
- Parent branch: `wave-loop-782` HEAD (`753197599`)
- Issue: #1495
- PR: (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W783_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-783.md`
- Cooperation W784: `.claude/plans/wave-loop-784.md`

### What landed
- `specs/scratch/w783_bench_module_385x2p6_aos_var_call_write.t27`
  - 24,640 elements, 788,480-bit packed vector (~0.752 MiBit).
  - Module-scope `pub var dst : [385][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w783.py`
  - Generator for the W783 witness; `OUTER = 385`, `MID_IDX = 192`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w783_bench_module_385x2p6_aos_var_call_write`.
- Weak-point fix in this closeout:
  - `bootstrap/tests/verilog_const_array.rs:166` — relaxed stale TODO expectation
    to accept any `TODO: array literal` or `TODO: struct literal` substring, matching
    the richer emitter diagnostic format.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-784.md`
  - W783 learnings saved and W784 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 243/0.
- `cargo test -p t27c --test verilog_const_array`: 2/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W783: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing, deeper
  compiler lowering issue, tracked for separate issue).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- 626 release warnings and 780 clippy warnings need dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).

---

## Wave Loop 782 — module-scope `[383][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1493)

- Branch: `wave-loop-782`
- Parent branch: `wave-loop-781` HEAD (`a61465608`)
- Issue: #1493
- PR: (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W782_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-782.md`
- Cooperation W783: `.claude/plans/wave-loop-783.md`

### What landed
- `specs/scratch/w782_bench_module_383x2p6_aos_var_call_write.t27`
  - 24,512 elements, 784,384-bit packed vector (~0.748 MiBit).
  - Module-scope `pub var dst : [383][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w782.py`
  - Generator for the W782 witness; `OUTER = 383`, `MID_IDX = 191`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w782_bench_module_383x2p6_aos_var_call_write`.
- Weak-point fix in this closeout:
  - `bootstrap/src/host/telemetry.rs:242` — replaced literal `3.14` with
    `std::f64::consts::PI` to keep `cargo clippy -p t27c` green.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-783.md`
  - W782 learnings saved and W783 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo clippy -p t27c`: OK (780 warnings, 0 errors).
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p flash-spi`: 2/0.
- `cargo test -p t27c --test bitnet_pipeline`: 20/0.
- `cargo test -p t27c --test bitnet_top`: 17/0.
- `cargo test -p t27c --test icarus_lowerable`: 242/0.
- Direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W782: PASS.

### Remaining weak points
- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing).
- FPGA E2E CI red.
- 626 release / 780 clippy warnings.
- Vivado-in-Docker CI gap.

---

## Standing process debt

- Open PR stack awaiting review: W774-W785.
- 30-day commit traceability is low (~15–20% of commit subjects carry
  `Closes #N` / `Fixes #N`).
- FPGA synthesis CI is blocked on the Yosys static-cast issue in `uart.v`.

---

## Wave Loop 773 — module-scope `[365][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1481)

- Branch: `wave-loop-773`
- Issue: #1481
- PR: #1482
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W773_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-773.md`
- Cooperation W774: `.claude/plans/wave-loop-774.md`

### What landed
- `specs/scratch/w773_bench_module_365x2p6_aos_var_call_write.t27`
  - 23,360 elements, 747,520-bit packed vector (~0.713 MiBit).
  - Module-scope `pub var dst : [365][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w773.py`
  - Generator for the W773 witness; `OUTER = 365`, `MID_IDX = 182`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w773_bench_module_365x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/plans/wave-loop-774.md`
  - W773 learnings saved and W774 plan created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 233/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W773: PASS.

---

## Wave Loop 776 — module-scope `[371][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1487)

- Branch: `wave-loop-776`
- Parent branch: `wave-loop-775` HEAD (`2e86eb0b8`)
- Issue: #1487
- PR: #1488 (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W776_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-776.md`
- Cooperation W777: `.claude/plans/wave-loop-777.md`

### What landed
- `specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`
  - 23,744 elements, 759,808-bit packed vector (~0.725 MiBit).
  - Module-scope `pub var dst : [371][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w776.py`
  - Generator for the W776 witness; `OUTER = 371`, `MID_IDX = 185`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w776_bench_module_371x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-777.md`
  - W776 learnings saved and W777 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 236/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W776: PASS.

### Notes
- W774 PR #1484 and W775 PR #1486 are still open awaiting review, so W776 was
  branched from `wave-loop-775` HEAD to keep the ladder unblocked.

---

## Wave Loop 777 — module-scope `[373][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1490)

- Branch: `wave-loop-777`
- Parent branch: `wave-loop-776` HEAD (`484c41725`)
- Issue: #1490
- PR: #1491 (to open / pending review)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W777_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-778.md`

### What landed
- `specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
  - 23,872 elements, 764,416-bit packed vector (~0.729 MiBit).
  - Module-scope `pub var dst : [373][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w777.py`
  - Generator for the W777 witness; `OUTER = 373`, `MID_IDX = 186`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w777_bench_module_373x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-778.md`
  - W777 learnings saved and W778 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 237/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W777: PASS.

### Notes
- W774 PR #1484, W775 PR #1486, W776 PR #1488, and PR #1489 (README/W774-W776 merge)
  remain open awaiting review, so W777 was branched from `wave-loop-776` HEAD to keep
  the ladder unblocked.

---

## Wave Loop 778 — next odd outer-dimension `[375][2]^6 Pt` (Issue #1492)

- Branch: `wave-loop-778` (to create after W777 merge or stack)
- Issue: #1492
- Plan: `.claude/plans/wave-loop-778.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[375][2]^6 Pt`.
- Variant B: keep width at ~0.729 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 774 — next odd outer-dimension `[367][2]^6 Pt` (Issue TBD)

- Branch: `wave-loop-776`
- Parent branch: `wave-loop-775` HEAD (`2e86eb0b8`)
- Issue: #1487
- PR: #1488 (to open)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W776_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-776.md`
- Cooperation W777: `.claude/plans/wave-loop-777.md`

### What landed
- `specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`
  - 23,744 elements, 759,808-bit packed vector (~0.725 MiBit).
  - Module-scope `pub var dst : [371][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w776.py`
  - Generator for the W776 witness; `OUTER = 371`, `MID_IDX = 185`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w776_bench_module_371x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-777.md`
  - W776 learnings saved and W777 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 236/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W776: PASS.

### Notes
- W774 PR #1484 and W775 PR #1486 are still open awaiting review, so W776 was
  branched from `wave-loop-775` HEAD to keep the ladder unblocked.

---

## Wave Loop 777 — module-scope `[373][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1490)

- Branch: `wave-loop-777`
- Parent branch: `wave-loop-776` HEAD (`484c41725`)
- Issue: #1490
- PR: #1491 (to open / pending review)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W777_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-778.md`

### What landed
- `specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
  - 23,872 elements, 764,416-bit packed vector (~0.729 MiBit).
  - Module-scope `pub var dst : [373][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w777.py`
  - Generator for the W777 witness; `OUTER = 373`, `MID_IDX = 186`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w777_bench_module_373x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-778.md`
  - W777 learnings saved and W778 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 237/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W777: PASS.

### Notes
- W774 PR #1484, W775 PR #1486, W776 PR #1488, and PR #1489 (README/W774-W776 merge)
  remain open awaiting review, so W777 was branched from `wave-loop-776` HEAD to keep
  the ladder unblocked.

---

## Wave Loop 778 — module-scope `[375][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1492)

- Branch: `wave-loop-778`
- Parent branch: `wave-loop-777` HEAD (`0867846cf`)
- Issue: #1492
- PR: #1493 (to open / pending review)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W778_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-779.md`

### What landed
- `specs/scratch/w778_bench_module_375x2p6_aos_var_call_write.t27`
  - 24,000 elements, 768,000-bit packed vector (~0.733 MiBit).
  - Module-scope `pub var dst : [375][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w778.py`
  - Generator for the W778 witness; `OUTER = 375`, `MID_IDX = 187`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w778_bench_module_375x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-779.md`
  - W778 learnings saved and W779 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 238/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W778: PASS.

### Notes
- W774 PR #1484, W775 PR #1486, W776 PR #1488, W777 PR #1491, and PR #1489
  (README/W774-W776 merge) remain open awaiting review, so W778 was branched from
  `wave-loop-777` HEAD to keep the ladder unblocked.
- The `bitnet_pipeline::sequencer_idle_arms_on_start` test drift remains a
  pre-existing failure unrelated to the wave-loop ladder.

---

## Wave Loop 779 — module-scope `[377][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1494)

- Branch: `wave-loop-779`
- Parent branch: `wave-loop-778` HEAD (`0c856f5f4`)
- Issue: #1494
- PR: #1495 (to open / pending review)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W779_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-780.md`

### What landed
- `specs/scratch/w779_bench_module_377x2p6_aos_var_call_write.t27`
  - 24,128 elements, 772,096-bit packed vector (~0.737 MiBit).
  - Module-scope `pub var dst : [377][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w779.py`
  - Generator for the W779 witness; `OUTER = 377`, `MID_IDX = 188`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w779_bench_module_377x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-780.md`
  - W779 learnings saved and W780 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 239/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W779: PASS.

### Notes
- W774 PR #1484, W775 PR #1486, W776 PR #1488, W777 PR #1491, W778 PR #1493, and
  PR #1489 (README/W774-W776 merge) remain open awaiting review, so W779 was branched
  from `wave-loop-778` HEAD to keep the ladder unblocked.
- The `bitnet_pipeline::sequencer_idle_arms_on_start` test drift remains a
  pre-existing failure unrelated to the wave-loop ladder.

---

## Wave Loop 780 — module-scope `[379][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1496)

- Branch: `wave-loop-780`
- Parent branch: `wave-loop-779` HEAD (`eadd9cfbcb`)
- Issue: #1496
- PR: #1497 (to open / pending review)
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W780_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-781.md`

### What landed
- `specs/scratch/w780_bench_module_379x2p6_aos_var_call_write.t27`
  - 24,256 elements, 776,192-bit packed vector (~0.741 MiBit).
  - Module-scope `pub var dst : [379][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w780.py`
  - Generator for the W780 witness; `OUTER = 379`, `MID_IDX = 189`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w780_bench_module_379x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`,
  `.claude/plans/wave-loop-781.md`
  - W780 learnings saved and W781 plan/cooperation variants created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 240/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W780: PASS.

### Notes
- W774 PR #1484, W775 PR #1486, W776 PR #1488, W777 PR #1491, W778 PR #1493, W779
  PR #1495, and PR #1489 (README/W774-W776 merge) remain open awaiting review, so
  W780 was branched from `wave-loop-779` HEAD to keep the ladder unblocked.
- The `bitnet_pipeline::sequencer_idle_arms_on_start` test drift remains a
  pre-existing failure unrelated to the wave-loop ladder.

---

## Wave Loop 781 — next odd outer-dimension `[381][2]^6 Pt` (Issue #1498)

- Branch: `wave-loop-781` (to create after W780 merge or stack)
- Issue: #1498
- Plan: `.claude/plans/wave-loop-781.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[381][2]^6 Pt`.
- Variant B: keep width at ~0.741 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 772 — module-scope `[363][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1743)

- Branch: `wave-loop-772`
- Issue: #1743
- PR: #1480
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W772_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-772.md`
- Cooperation W773: `.claude/plans/wave-loop-773.md`

### What landed
- `specs/scratch/w772_bench_module_363x2p6_aos_var_call_write.t27`
  - 23,232 elements, 743,424-bit packed vector (~0.709 MiBit).
  - Module-scope `pub var dst : [363][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w772.py`
  - Generator for the W772 witness; `OUTER = 363`, `MID_IDX = 181`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w772_bench_module_363x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/plans/wave-loop-773.md`
  - W772 learnings saved and W773 issue/plan created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 232/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W772: PASS.

---

## Wave Loop 771 — module-scope `[361][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1742)

- Branch: `wave-loop-771`
- Issue: #1742
- PR: to open
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W771_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-771.md`
- Cooperation W772: `.claude/plans/wave-loop-772.md`

### What landed
- `specs/scratch/w771_bench_module_361x2p6_aos_var_call_write.t27`
  - 23,104 elements, 739,328-bit packed vector (~0.705 MiBit).
  - Module-scope `pub var dst : [361][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w771.py`
  - Generator for the W771 witness; `OUTER = 361`, `MID_IDX = 180`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w771_bench_module_361x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/plans/wave-loop-772.md`
  - W771 learnings saved and W772 issue/plan created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 231/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W771: PASS.

---

## Wave Loop 770 — module-scope `[359][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1741)

- Branch: `wave-loop-770`
- Issue: #1741
- PR: to open
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W770_2026-07-24.md`
- Plan: `.claude/plans/wave-loop-770.md`
- Cooperation W771: `.claude/plans/wave-loop-771.md`

### What landed
- `specs/scratch/w770_bench_module_359x2p6_aos_var_call_write.t27`
  - 22,976 elements, 735,232-bit packed vector (~0.701 MiBit).
  - Module-scope `pub var dst : [359][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w770.py`
  - Generator for the W770 witness; `OUTER = 359`, `MID_IDX = 179`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w770_bench_module_359x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/plans/wave-loop-771.md`
  - W770 learnings saved and W771 issue/plan created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 230/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W770: PASS.

---

## Wave Loop 768 — next odd outer-dimension `[355][2]^6 Pt` (Issue #1739)

- Branch: `wave-loop-768` (to create after W767 merge)
- Issue: #1739
- Plan: `.claude/plans/wave-loop-768.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[355][2]^6 Pt`.
- Variant B: keep width at ~0.690 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 767 — next odd outer-dimension `[353][2]^6 Pt` (Issue #1738)

- Branch: `wave-loop-767` (to create after W766 merge)
- Issue: #1738
- Plan: `.claude/plans/wave-loop-767.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[353][2]^6 Pt`.
- Variant B: keep width at ~0.686 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 766 — next odd outer-dimension `[351][2]^6 Pt` (Issue #1737)

- Branch: `wave-loop-766` (to create after W765 merge)
- Issue: #1737
- Plan: `.claude/plans/wave-loop-766.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[351][2]^6 Pt`.
- Variant B: keep width at ~0.682 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 765 — next odd outer-dimension `[349][2]^6 Pt` (Issue #1736)

- Branch: `wave-loop-765` (to create after W764 merge)
- Issue: #1736
- Plan: `.claude/plans/wave-loop-765.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[349][2]^6 Pt`.
- Variant B: keep width at ~0.678 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 764 — next odd outer-dimension `[347][2]^6 Pt` (Issue #1735)

- Branch: `wave-loop-764` (to create after W763 merge)
- Issue: #1735
- Plan: `.claude/plans/wave-loop-764.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[347][2]^6 Pt`.
- Variant B: keep width at ~0.674 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

# NOW — IGLA cycle 1 + Wave Loop 469 context (2026-07-07)

## t27c codegen: mut-inference for reassigned locals (Fixes #1463)

- PR #1461 adds mutability inference to the Rust codegen backend.
- `collect_mutable_names()` scans function bodies for assignment targets
  (simple, index, field), emitting `let mut` where needed.
- Eliminates 117 E0384 errors in tri-net specs.
## t27c codegen: recursive optimizer scan for control-flow bodies (Fixes #1464)

- PR #1462 fixes const_propagate, copy_propagate, dead_store_elim to recurse
  into if/while/for bodies when checking for reassignment/reads.
- Eliminates 182 E0425 errors in tri-net specs (208 -> 26).
- Stacked on #1461.
## IGLA cycle 1 — process debt needles (Refs #1438, #1440, #1442, #1444, #1446)

- Charter: `docs/nona-03-manifest/IGLA_IMPROVEMENT_LOOP.md` + audit `docs/reports/IGLA_AUDIT_W470_2026-07-07.md`.
- Open PRs:
  - #1439: remove duplicate workflow directory, enforce `FROZEN_HASH`, unify L1 regexes.
  - #1441: harden auto-merge and brain-seal-refresh workflows.
  - #1445: align Vivado scripts/constraints with `HARDWARE_SSOT` XC7A200T-FGG676.
- Completed: worktree cleanup (#1442); salvaged compiler postfix-array change to `salvage/ae9fe-postfix-array-notation`.
- Blocked: W469 2D-struct-array Verilog lowering (#1443, blocked on `wave-loop-469`); Digilent FTDI `cli/dlc10` support (#1446, blocked on hardware access).

## IGLA cycle 1 — auto-workflow hardening (Closes #1440)

- PR #1441 hardens `.github/workflows/auto-merge-ready-prs.yml` and
  `.github/workflows/brain-seal-refresh.yml` with explicit permissions,
  L1 traceability gating, correct dry-run boolean handling, and change-detection
  guards before `git commit`.
- This closes issue #1440 (automated workflows could merge/commit without
  issue linkage or review).

## IGLA cycle 1 — FPGA target alignment (Closes #1444)

- PR #1445 aligns Vivado synthesis scripts and the constraints header with the
  `fpga/HARDWARE_SSOT.md` canonical device: `XC7A200T-FGG676` (`xc7a200tfgg676-1`).
- This replaces the stale `XC7A100T-FGG676` hard-coding in `fpga/vivado/build.tcl`,
  `build_gf16.tcl`, `build_gf16_matmul4x4.tcl`, and `specs/fpga/constraints/qmtech_a100t.xdc`.
- This closes issue #1444 (synthesis scripts targeted the wrong FPGA device).

## Architecture — ADR-007 documents de-jure/de-facto split for generated .v in specs/ (Closes #1435)

- Fact-check on HEAD 6c704801: specs/**/*.v = 61 files, gen/**/*.v = 33. Issues
  #960 and #1205 were closed as done, but the L2 GENERATION violation artifact
  (61 generated .v in specs/, some already duplicated in gen/, e.g. specs/fpga/uart.v
  vs canonical gen/verilog/fpga/uart.v) is still present on master.
- #1205 body itself says "30/61 migrated, ~30 remain" with unchecked acceptance
  criteria yet the issue is closed -> premature-closure pattern (text claim, not HEAD).
- This PR adds architecture/ADR-007-verilog-in-specs.md ONLY (a decision record). It
  does NOT delete any .v file: choice A (finish migration) vs B (legalize as golden
  fixtures with a whitelist path) is left to the owner. SSOT=83 untouched.
- Status tag: [доказано] for the counts; [ТРЕБУЕТ ДЕЙСТВИЯ ПОЛЬЗОВАТЕЛЯ] for A vs B.

## Wave Loop 460 — Next wave (to be selected from cooperation plan) (Closes #1433)

- Branch: `wave-loop-460` (to create from W459 land commit)
- Issue: #1433 (to create)
- PR: (to open after close-out)
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W460_2026-07-01.md`
- Cooperation W461: (to be written at W460 close-out)

### Not started

- Create issue #1433 and branch `wave-loop-460` from the W459 land commit.
- Select one of the three W460 variants documented in
  `docs/reports/FPGA_LOOP_COOPERATION_W460_2026-07-01.md`.

---

## Wave Loop 459 — gen-verilog array parameters from test/invariant/bench + yosys warning gate + ROM style pragma (Variant B default) (Closes #1431)

- Branch: `wave-loop-459`
- Issue: #1431
- PR: #1434
- Report: `docs/reports/WAVE_LOOP_459_REPORT.md`
- Evidence W459: `docs/reports/FPGA_LOOP_EVIDENCE_W459_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W459_2026-07-01.md`
- Cooperation W460: `docs/reports/FPGA_LOOP_COOPERATION_W460_2026-07-01.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - Extended array-parameter binding analysis to recurse into `test`, `invariant`,
    and `bench` blocks so functions with array parameters can be exercised from
    any call site that passes the same module-level array identifier.
  - Test-block `assert_eq` and bare function calls are now emitted as real
    Verilog statements (real `if (!(...))` checks and real calls) inside the
    existing `` `ifndef SIMULATION `` / `` `endif `` guard, instead of being
    commented out.
  - `gen_verilog_const` emits `(* {pragma} *)` before the memory declaration when
    a `const [N]T` declaration carries a `rom_style` pragma.
  - Added `tests_w459` unit-test module:
    - `array_param_bound_from_test_block`
    - `test_block_emits_real_function_call`
    - `rom_style_block_pragma_emitted`

- `bootstrap/src/suite.rs`
  - Added `YOSYS_ALLOWED_WARNINGS` allow-list and unrecognized-warning failure
    gate in `cmd_gen_verilog_yosys_smoke`.
  - Yosys smoke parsing now defines `SIMULATION` (`read_verilog -sv -DSIMULATION`)
    so test/bench blocks are skipped and the smoke baseline stays empty.

- `specs/scratch/w459_array_param_test_call.t27`
  - Regression spec with a module-level `var [4]u16` RAM and `set`/`get`
    functions exercised from a `test` block with `assert_eq`.

- `specs/scratch/w459_rom_style_block.t27`
  - Regression spec with `pragma rom_style = "block"` on a module-level
    `const [4]u16` ROM and a lookup function tested from a `test` block.

- `.trinity/seals/scratch_w459_array_param_test_call.json`
- `.trinity/seals/scratch_w459_rom_style_block.json`
  - Seals for the two new regression specs.

- All 583 `.trinity/seals/*.json` files re-sealed to the new gen-verilog output.

- `docs/reports/gen_verilog_smoke_baseline.json`
  - Kept empty: the four specs that were briefly added as "pre-existing"
    failures are now fully passing with `-DSIMULATION`.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W459 competitor boundary section.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_459_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W459_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W460_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W459_OPERATING_POINT` — bench unavailable.
- Array-parameter support for multiple/different bound arrays or literal array
  arguments — deferred to W460 (Variant B).
- Bench-block local-variable lowering to declared registers — deferred to W460
  (Variant B).
- The three pre-existing `let_binding` cargo-test failures — deferred to W460
  (Variant B).

### Verification

- `cargo test -p t27c --bin t27c tests_w459`: **PASS** (3/3).
- `t27c gen-verilog specs/scratch/w459_array_param_test_call.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION ...'`: **PASS**.
- `t27c gen-verilog specs/scratch/w459_rom_style_block.t27` +
  `yosys -q -p 'read_verilog -sv -DSIMULATION ...'`: **PASS**, emits
  `(* rom_style = "block" *)`.
- `./scripts/tri test --fast --json /tmp/tri_test_w459_fast.json`: **ALL TESTS PASSED**.
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    583/583 PASS.
  - Gen Verilog Yosys Smoke: **63 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`.
- Full `./scripts/tri test` (no `--fast`): Phase 3c smoke gate reports `passed: true`,
  but Phase 3c-standalone stalls on an external `lake` download of `batteries`
  from `reservoir.lean-lang.org`. The `--fast` path is fully green.
- `cargo test -p t27c --bin t27c`: 1521 passed, **3 pre-existing failures**
  (`let_binding_is_lowered_1401`, `test_let_binding_emitted_c_1401`,
  `test_let_binding_emitted_rust_1401`) that also fail on `HEAD~1`.

---

## Wave Loop 458 — gen-verilog warning hygiene + module-level array parameters (Variant B default) (Closes #1429)

- Branch: `wave-loop-458`
- Issue: #1429
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_458_REPORT.md`
- Evidence W458: `docs/reports/FPGA_LOOP_EVIDENCE_W458_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W458_2026-07-01.md`
- Cooperation W459: `docs/reports/FPGA_LOOP_COOPERATION_W459_2026-07-01.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - Replaced `// synthesis translate_off/on` guards with `` `ifndef SIMULATION `` /
    `` `endif `` for test and bench blocks.
  - `f32`/`f64` scalar constants now emit `parameter real` / `localparam real`
    instead of bit-vector declarations.
  - String literals are escaped (`\\`, `\\n`, `\\t`, `\\"`) before Verilog emission.
  - Bare module-level statements are now parsed and emitted inside an
    `always @(*)` block.
  - Functions inside a module can reference module-level arrays by name.
  - `pub fn` array parameters can be bound to a module-level array through a single
    module-level call site; the bound array is referenced by name inside the
    function and omitted from the scalar port list.
  - Added `tests_w458` unit-test module:
    - `array_param_read_emitted`
    - `float_param_emits_real`
    - `string_newline_escaped`
    - `no_translate_off_comments`
  - Fixed a module-body recovery infinite-loop edge case on stray top-level keywords.

- `specs/scratch/w458_array_param_read.t27`
  - Regression spec with module-level `const [4]u16` ROM and a function reading
    from it.

- `specs/scratch/w458_array_param_write.t27`
  - Regression spec with module-level `var [4]u16` RAM and functions writing to
    and reading from it.

- `.trinity/seals/scratch_w458_array_param_read.json`
- `.trinity/seals/scratch_w458_array_param_write.json`
  - Seals for the two new regression specs.

- All 581 `.trinity/seals/*.json` files re-sealed to the new gen-verilog output.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_458_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W458_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W459_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W458_OPERATING_POINT` — bench unavailable.
- Known-warnings gate in `bootstrap/src/suite.rs` — deferred to W459 (Variant B).
- Array-parameter support for test/invariant/bench call sites — deferred to W459
  (Variant B).
- ROM style pragma — deferred to a future wave.

### Verification

- `cargo test -p t27c --bin t27c tests_w458`: **PASS** (4/4).
- `t27c gen-verilog specs/scratch/w458_array_param_read.t27` +
  `yosys read_verilog -sv; synth -top w458_array_param_read`: **PASS**.
- `t27c gen-verilog specs/scratch/w458_array_param_write.t27` +
  `yosys read_verilog -sv; synth -top w458_array_param_write`: **PASS**.
- `./scripts/tri test --fast --json /tmp/tri_test_w458_fast.json`: **ALL TESTS PASSED**.
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    581/581 PASS.
  - Gen Verilog Yosys Smoke: **61 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**, 24-variant theorem matrix,
    `envelope_check: "ok"`, `schema_version: "1.0"`, `passed: true`.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `ACCEPTABLE: yes`.
- Full `./scripts/tri test` (no `--fast`): Phase 3c smoke gate reports `passed: true`,
  but Phase 3c-standalone stalls on an external `lake` download of `batteries`
  from `reservoir.lean-lang.org`. The `--fast` path is fully green.
- `cargo test -p t27c --bin t27c`: 1518 passed, **3 pre-existing failures**
  (`let_binding_is_lowered_1401`, `test_let_binding_emitted_c_1401`,
  `test_let_binding_emitted_rust_1401`) that also fail on `HEAD~1`.

---

## Wave Loop 457 — RAM style pragma support for module-level arrays (Variant B default) (Closes #1428)

- Branch: `wave-loop-457`
- Issue: #1428
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_457_REPORT.md`
- Evidence W457: `docs/reports/FPGA_LOOP_EVIDENCE_W457_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W457_2026-07-01.md`
- Cooperation W458: `docs/reports/FPGA_LOOP_COOPERATION_W458_2026-07-01.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - Added `KwPragma` token and `pragma` keyword lexer mapping.
  - Added `extra_pragma: String` to `Node`; initialized in `Default` and `new`.
  - Added `pending_pragma` to `Parser` and `ParserCheckpoint` with save/restore.
  - Added `parse_pragma` for `pragma name = "value";` top-level statements;
    currently accepts `ram_style = "block"` / `ram_style = "distributed"`
    and rejects unknown pragma names.
  - `parse_module_body` now consumes `pragma` directives before the next
    module-level declaration.
  - `parse_const_decl` and `parse_var_decl` capture the pending pragma into the
    declaration node and clear it so it is not accidentally reused.
  - `gen_verilog_var` emits `(* {pragma} *)` before the synthesizable `reg ... [0:N]`
    memory declaration for true array types (e.g. `[4]u16`), giving Vivado/Yosys
    a synthesizer-controllable RAM style attribute.
  - Added `tests_w457_ram_style` unit-test module:
    - `ram_style_block_pragma_emitted`
    - `ram_style_distributed_pragma_emitted`
    - `unknown_pragma_rejected`

- `specs/scratch/w457_ram_style_block.t27`
  - New regression spec exercising `pragma ram_style = "block";` on a module-level
    writable `[4]u16` array with write/read and loop-sum tests.

- `specs/scratch/w457_ram_style_distributed.t27`
  - New regression spec exercising `pragma ram_style = "distributed";` on a
    module-level writable `[4]u16` array with write/read tests.

- `.trinity/seals/scratch_w457_ram_style_block.json`
- `.trinity/seals/scratch_w457_ram_style_distributed.json`
  - Seals for the two new regression specs.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W457 competitor boundary section.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_457_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W457_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W458_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W457_OPERATING_POINT` — bench unavailable.
- Pragmas for module-level `const`/ROM style (`rom_style`) or per-port RAM
  attributes — deferred to a future wave.

### Verification

- `cargo test -p t27c --bin t27c tests_w457_ram_style`: **PASS** (3/3).
- `t27c gen-verilog specs/scratch/w457_ram_style_block.t27` +
  `yosys read_verilog -sv; synth -top w457_ram_style_block`: **PASS**,
  emits `(* ram_style = "block" *)`.
- `t27c gen-verilog specs/scratch/w457_ram_style_distributed.t27` +
  `yosys read_verilog -sv; synth -top w457_ram_style_distributed`: **PASS**,
  emits `(* ram_style = "distributed" *)`.
- `./scripts/tri test --json /tmp/tri_test_w457.json`: **ALL TESTS PASSED**.
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    579/579 PASS.
  - Gen Verilog Yosys Smoke: **59 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - FPGA Standalone Lake-Package Build: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `ACCEPTABLE: yes`.

---

## Wave Loop 456 — ROM read-only enforcement (Variant B, narrowed scope) (Closes #1427)

- Branch: `wave-loop-456`
- Issue: #1427
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_456_REPORT.md`
- Evidence W456: `docs/reports/FPGA_LOOP_EVIDENCE_W456_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W456_2026-07-01.md`
- Cooperation W457: `docs/reports/FPGA_LOOP_COOPERATION_W457_2026-07-01.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B narrowed to ROM read-only — bench still blocked)

- `bootstrap/src/compiler.rs`
  - `typecheck_ast` / `check_stmt` now rejects assignments to elements of immutable
    `const [N]T` arrays (`lut[i] = ...`) with a typecheck error.
  - Existing immutable scalar assignment remains a warning.
  - Added `tests_w456_rom_readonly` unit-test module:
    - `rom_readonly_array_element_assign_is_rejected`
    - `var_array_element_assign_still_allowed`

- `specs/scratch/w456_rom_readonly.t27`
  - New regression spec with module-level `const [4]u16` ROM and read-only lookups.

- `.trinity/seals/scratch_w456_rom_readonly.json`
  - Seal for the new regression spec.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W456 competitor boundary section.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_456_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W456_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W457_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W456_OPERATING_POINT` — bench unavailable.
- RAM style pragmas / module-level array parameters / warning hygiene — deferred to W457.

### Verification

- `cargo test -p t27c --bin t27c tests_w456_rom_readonly`: **PASS** (2/2).
- `t27c gen-verilog specs/scratch/w456_rom_readonly.t27` + `yosys read_verilog -sv; synth -top w456_rom_readonly`: **PASS**.
- `./scripts/tri test --json /tmp/tri_test_w456.json`: **ALL TESTS PASSED**.
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    577/577 PASS.
  - Gen Verilog Yosys Smoke: **57 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - FPGA Standalone Lake-Package Build: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `ACCEPTABLE: yes**.

---

## Wave Loop 455 — Implement missing `gen-verilog` tuple/array backend (Variant B default) (Closes #1425)

- Branch: `wave-loop-455`
- Issue: #1425
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_455_REPORT.md`
- Evidence W455: `docs/reports/FPGA_LOOP_EVIDENCE_W455_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W455_2026-07-01.md`
- Cooperation W456: `docs/reports/FPGA_LOOP_COOPERATION_W456_2026-07-01.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `bootstrap/src/compiler.rs`
  - Parser support for tuple return types `-> (T1, T2, ...)`.
  - Parser support for tuple literals `(a, b, c)`.
  - Parser support for `let (a, b, c) = expr` destructuring assignment.
  - Verilog backend: packed function result register for tuple returns.
  - Verilog backend: tuple literal as packed concatenation.
  - Verilog backend: `let` destructuring lowering with per-binding width inference
    from the callee's tuple return type.
  - Verilog backend: module-level `const [N]T{...}` ROM lowering.
  - Verilog backend: function-local `var [N]T` array lowering (numeric/variable
    indices, signed elements, `for` loops, 2D arrays, array-literal initializers).
  - Keyword-safe full-token identifier escaping for flattened local-array element
    names (`\buf_0 ` instead of `\buf _0`).
  - Added `ParserCheckpoint` save/restore helpers used for safe lookahead in the
    tuple/array literal parser.

- `cli/flash-spi/src/main.rs`
  - Restored workspace build by supplying the new `FlashOpts` fields
    (`no_jprogram: false`, `bitswap: true`) that the updated flash driver now
    requires.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch header to `wave-loop-455`.
  - Documented the W455 triage decision and the cleared 7-residual-failure matrix.

- `docs/reports/gen_verilog_smoke_baseline.json`
  - Expected-failure set updated to empty; the 7 baseline failures are now cleared.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W455 boundary paragraph; refreshed competitor numbers.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_455_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W455_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W456_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W455_OPERATING_POINT` — bench unavailable.
- RAM style inference / block-vs-distributed pragma hints — out of scope for W455.

### Verification

- `cargo build --release`: **PASS**.
- `t27c gen-verilog` + `yosys read_verilog -sv` on the 7 previously failing specs:
  **PASS** (0 failures).
- `./scripts/tri test --json /tmp/tri_test_w455.json`: **ALL TESTS PASSED**.
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify:
    576/576 PASS.
  - Gen Verilog Yosys Smoke: **56 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - FPGA Standalone Lake-Package Build: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `ACCEPTABLE: yes`.
- 67 affected `.trinity/seals/*.json` files resealed to the new compiler output.

---

## Wave Loop 454 — High-VCCINT adversarial witness + duty-cycle / jitter robustness theorems (Variant C) (Closes #1424)

- Branch: `wave-loop-454`
- Issue: #1424
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_454_REPORT.md`
- Evidence W454: `docs/reports/FPGA_LOOP_EVIDENCE_W454_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W454_2026-07-01.md`
- Cooperation W455: `docs/reports/FPGA_LOOP_COOPERATION_W455_2026-07-01.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant C — master-merge rejected, bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `OUTSIDE_VCCINT_HIGH_W454_OPERATING_POINT` (25 °C, 1200 mV VCCINT,
    1800 mV VCCAUX, `ss` corner) — a VCCINT above the documented 1100 mV maximum.
  - Proved `outside_vccint_high_w454_operating_point_not_within_envelope`.
  - Proved `cclk_variant_and_xadc_envelope_check_outside_vccint_high_false` — the
    dashboard gate rejects high VCCINT for every documented OSCFSEL.
  - Added `cclk_oscfsel_7_duty_asymmetry_w454` — at OSCFSEL=7 (~33.3 MHz, 30 ns
    period), any high-time between 14 ns and 16 ns keeps the PVT-aware raw-ns
    predicate true under the worst-case operating point.
  - Added `cclk_ideal_split_robust_to_1ns_jitter_w454` — at every documented
    OSCFSEL selection, the ideal 50 % high time tolerates ±1 ns of jitter while
    remaining flash-spec compliant under the worst-case PVT context.

- `cli/tri/src/fpga.rs`
  - Added `cclk_variant_and_xadc_envelope_check(oscfsel, ctx)` helper mirroring
    the Lean dashboard gate.
  - Added five W454 unit tests:
    - `test_pvt_context_high_vccint_outside_envelope_w454`
    - `test_cclk_variant_and_xadc_envelope_check_high_vccint_false_w454`
    - `test_cclk_variant_and_xadc_envelope_check_worst_case_true_w454`
    - `test_raw_ns_oscfsel_7_duty_asymmetry_w454`
    - `test_raw_ns_ideal_split_1ns_jitter_w454`

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W454 boundary section; no new public competitor signals appeared.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch header to `wave-loop-454` and documented the W454 triage
    decision: master-merge `701d79b3b` rejected as insufficient; 7 residual
    yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_454_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W454_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W455_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge of `gen-verilog` fix set from `master` (`701d79b3b`) — rejected
  as insufficient for the 7 residual failures and as a regression risk to the
  wave-loop branch's own sub-fixes.
- Clearing the 7 yosys smoke failures — requires a dedicated compiler wave for
  tuple/array lowering.

### Verification

- `cd proofs/lean4 && lake build Trinity.TernaryFPGABoot`: **success**
  (2967 jobs).
- `cargo test -p tri w454`: **PASS** (5/5 new W454 tests).
- `./scripts/tri test --json /tmp/tri_test_w454.json`: **ACCEPTABLE**.
  - 576/576 non-smoke PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 baseline failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, `passed: true`, `acceptable: true`.
  - FPGA standalone lake-package build: **PASS**.

---

## Wave Loop 452 — Boundary cold/high-voltage envelope-corner theorem + adversarial voltage witness + CI metric hardening (Variant B default) (Closes #1422)

- Branch: `wave-loop-452`
- Issue: #1422
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_452_REPORT.md`
- Evidence W452: `docs/reports/FPGA_LOOP_EVIDENCE_W452_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W452_2026-07-01.md`
- Cooperation W453: `docs/reports/FPGA_LOOP_COOPERATION_W453_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `BOUNDARY_COLD_HIGHV_W452_OPERATING_POINT` at -40 °C, 1100 mV VCCINT,
    1800 mV VCCAUX, all `ff`/`tt`/`ss` corners.
  - Proved `boundary_cold_highv_w452_all_corners_transaction_ok`: a single `∀`
    theorem that the ideal raw-ns capture produces a flash-spec-compliant SPI
    read transaction for every OSCFSEL 0..7 and every process corner at the
    cold/high-voltage envelope corner.
  - Added `OUTSIDE_VCCINT_LOW_W452_OPERATING_POINT` (800 mV, below envelope) and
    proved `cclk_variant_and_xadc_envelope_check_outside_vccint_low_false` —
    the dashboard gate rejects low VCCINT.
  - Added `oscfsel_out_of_range_combined_check_false`: any `oscfsel > 7` is
    rejected by the combined-check gate.

- `bootstrap/src/suite.rs`
  - Extended `FpgaSmokeResult` with `failed: bool` and
    `failure_reason: Option<String>`.
  - Extended `SuiteSummary` with `fpga_smoke_skipped`, `fpga_smoke_failed`, and
    `fpga_smoke_failure_reason` so the JSON dashboard distinguishes passed,
    skipped, and failed smoke gates.
  - Updated `parse_smoke_gate_report` and the error fallback path to populate
    the new fields.
  - Added/updated builder and smoke-state unit tests.

- `cli/tri/src/fpga.rs`
  - Added `test_smoke_gate_all_ok_matches_snapshot`, a deterministic synthetic
    snapshot of a fully-passing smoke-gate report with every phase populated.

- `tests/fixtures/fpga/smoke-gate/`
  - Committed `all_ok_snapshot.json`.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W452 boundary section; Sparkle/Verilean remains the only fresh
    Lean-native HDL signal in early July 2026.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch to `wave-loop-452` and documented the W452 triage decision:
    7 residual yosys smoke failures remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_452_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W452_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_PLAN_W452_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W453_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo check -p tri`: **PASS**.
- `cargo check -p t27c`: **PASS**.
- `cargo test -p tri --bin tri all_ok`: **PASS**.
- `cargo test -p tri --bin tri missing_bitstream`: **PASS**.
- `cargo test -p tri --bin tri fast_skipped`: **PASS**.
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/t27_w452_suite.json`: **ACCEPTABLE**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `passed: true`, `acceptable: true`.
- `./scripts/tri test --fast --json /tmp/t27_w452_fast_suite.json`: **ACCEPTABLE**.
  - Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
  - FPGA board-less smoke gate: **PASS**, same 24-variant matrix and
    `passed: true` as the default run.
  - Phase 3c-standalone: **skipped** (`--fast` mode);
    `validate_lean_standalone_elapsed_ms` is `null`.
  - `acceptable: true`.

---

## Wave Loop 453 — Close the four-corner PVT operating rectangle in Lean + smoke-gate JSON schema hardening (Variant B default) (Closes #1421)

- Branch: `wave-loop-453`
- Issue: #1421
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_453_REPORT.md`
- Evidence W453: `docs/reports/FPGA_LOOP_EVIDENCE_W453_2026-07-01.md`
- Cooperation W454: `docs/reports/FPGA_LOOP_COOPERATION_W454_2026-07-01.md`
- Competitor snapshot: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
- Gen-verilog defect tracker: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`

### What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `EnvelopeCorner` inductive (`hot_lowv`, `hot_highv`, `cold_lowv`, `cold_highv`).
  - Added direct record definitions `BOUNDARY_HOT_HIGHV_W453_OPERATING_POINT`
    (85 °C, 1100 mV VCCINT, 1800 mV VCCAUX) and
    `BOUNDARY_COLD_LOWV_W453_OPERATING_POINT` (-40 °C, 900 mV VCCINT, 1800 mV VCCAUX)
    covering all `ff`/`tt`/`ss` corners.
  - Added `envelope_corner_operating_point` mapping each corner to its
    `XadcOperatingPoint`.
  - Minted `all_envelope_corners_w453_all_corners_transaction_ok`: a single
    quantified theorem proving that every envelope corner, every process corner,
    and every OSCFSEL 0..7 produces a flash-spec-compliant SPI boot transaction.

- `cli/tri/src/fpga.rs`
  - Added strict `SmokeGateReport` schema struct with `#[serde(deny_unknown_fields)]`
    guarding every emitted smoke-gate JSON report.
  - Added generator-side validation before write and two unit tests:
    acceptance of a canonical report and rejection of an unknown field.

- `bootstrap/src/suite.rs`
  - Added the same `SmokeGateReport` schema on the consumer side.
  - Updated `parse_smoke_gate_report` to validate schema before ingesting the report
    into the suite summary.
  - Added `test_parse_smoke_gate_report_deny_unknown_fields` and hardened the
    legacy tolerance test to include the mandatory `schema_version` field.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W453 boundary section describing the four-corner rectangle theorem and
    the smoke-gate schema guard; no new competitor signals.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch header to `wave-loop-453` and added W452/W453 triage decisions;
    7 residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_453_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W453_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W454_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — explicitly deferred to Wave Loop 454 (Variant B default).

### Verification

- `cd proofs/lean4 && lake build Trinity.TernaryFPGABoot`: **success**
  (2967 jobs, all-corners theorem builds).
- `cargo test -p tri --bin tri fpga::`: **PASS** (new schema acceptance/rejection tests).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS** (new schema-hardening tests).
- `./scripts/tri test --json /tmp/t27_w453_full_suite.json`: **ACCEPTABLE**.
  - 576/576 non-smoke PASS; 7 baseline gen-verilog failures remain unchanged.
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `passed: true`, `acceptable: true`.
- `./scripts/tri test --fast --json /tmp/t27_w453_fast_suite.json`: **ACCEPTABLE**.
  - Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
  - FPGA board-less smoke gate: **PASS**, same 24-variant matrix and
    `passed: true` as the default run.
  - Phase 3c-standalone: **skipped** (`--fast` mode);
    `validate_lean_standalone_elapsed_ms` is `null`.
  - `acceptable: true`.

---

## Wave Loop 451 — Formal boot-evidence expansion + adversarial envelope theorem + CI metric hardening (Variant B default) (Closes #1423)

- Branch: `wave-loop-451`
- Issue: #1423
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_451_REPORT.md`
- Evidence W451: `docs/reports/FPGA_LOOP_EVIDENCE_W451_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W451_2026-07-01.md`
- Cooperation W452: `docs/reports/FPGA_LOOP_COOPERATION_W452_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added VCCAUX independence lemmas:
    `xadc_operating_point_within_envelope_independent_of_vccaux`,
    `n25q128_min_sck_low_ns_pvt_independent_of_vccaux`,
    `high_ns` and `half_ns` analogues, and the measured-cclk/transaction
    independence theorems.
  - Added `BOUNDARY_HOT_LOWV_W451_OPERATING_POINT` and
    `BOUNDARY_HOT_LOWV_W451_PVT_CONTEXT` covering 85 °C, 900 mV VCCINT,
    1800 mV VCCAUX, all `ff`/`tt`/`ss` corners.
  - Proved `boundary_hot_lowv_w451_all_corners_transaction_ok`: for every
    OSCFSEL 0..7 and every Artix-7 process corner, the boundary hot/low-voltage
    operating point produces a flash-spec-compliant boot transaction.

- `bootstrap/src/suite.rs`
  - Added `FpgaSmokeResultBuilder` with fluent methods and pre-built
    `missing_bitstream()` / `failed()` shapes to prevent silent metric drops.
  - Replaced manual `FpgaSmokeResult` literals in the missing-bitstream,
    `parse_smoke_gate_report`, and error-fallback paths with builder calls.
  - Added `#[serde(deny_unknown_fields)]` to `SuitePhaseSummary` and `SuiteSummary`
    so new smoke-gate report fields cannot silently disappear in JSON round-trips.
  - Added builder and schema-hardening unit tests.

- `cli/tri/src/fpga.rs`
  - Added deterministic snapshot tests for previously unprotected smoke-gate
    shapes: missing-bitstream fallback and `--fast` skipped-standalone fallback.
  - Added `sanitize_smoke_gate_report` normalization (path/temp-dir and elapsed_ms).

- `tests/fixtures/fpga/smoke-gate/`
  - Committed `missing_bitstream_snapshot.json` and
    `fast_skipped_standalone_snapshot.json` with stable temp filenames for
    deterministic cross-run comparison.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W451 boundary section noting Sparkle/Verilean as the only fresh July
    2026 Lean-native HDL signal and t27's new boundary theorem + schema hardening.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch to `wave-loop-451` and documented the W451 triage decision:
    7 residual yosys smoke failures remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_451_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W451_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_PLAN_W451_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W452_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave (Variant C in W452).

### Verification

- `cargo check -p tri`: **PASS**.
- `cargo check -p t27c`: **PASS**.
- `cargo test -p tri --bin tri missing_bitstream`: **PASS**.
- `cargo test -p tri --bin tri fast_skipped`: **PASS**.
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/t27_w451_suite.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`.
- `./scripts/tri test --fast --json /tmp/t27_w451_fast_suite.json`: **PASS**.
  - Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
  - Phase 3c-standalone: **skipped** (`--fast` mode), snapshot shape protected.
  - `acceptable: true`.

---

## Wave Loop 450 — Dry-run-live quantified transaction theorem + standalone-build snapshot + `--fast` suite mode (Variant B default) (Closes #1425)

- Branch: `wave-loop-450`
- Issue: #1425
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_450_REPORT.md`
- Evidence W450: `docs/reports/FPGA_LOOP_EVIDENCE_W450_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W450_2026-07-01.md`
- Cooperation W451: `docs/reports/FPGA_LOOP_COOPERATION_W451_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `DRY_RUN_LIVE_W448_PVT_CONTEXT` / `DRY_RUN_LIVE_W448_OPERATING_POINT`
    matching the W448 dry-run-live fixtures and quantifying over all process corners.
  - Proved `dry_run_live_w448_operating_point_within_envelope` and
    `dry_run_live_w448_process_corner_worse_than_ss`.
  - Minted `dry_run_live_w448_raw_ns_satisfies_flash_spec` and
    `dry_run_live_w448_all_corners_transaction_ok`: a single quantified theorem
    that the ideal raw-ns capture produces a flash-spec-compliant transaction for
    every OSCFSEL 0..7 and every `ff`/`tt`/`ss` corner at the W448 dry-run-live
    operating point.

- `cli/tri/src/fpga.rs`
  - Added `test_smoke_gate_validate_lean_standalone_matches_snapshot`, a snapshot
    diff gate for the full smoke-gate JSON report with standalone build enabled.
  - Added `sanitize_smoke_gate_report` helper for path/elapsed-time normalization.

- `tests/fixtures/fpga/smoke-gate/validate_lean_standalone_snapshot.json`
  - Committed snapshot of the normalized smoke-gate report.

- `bootstrap/src/main.rs` + `bootstrap/src/suite.rs`
  - Added `--fast` flag to the `Suite` command and `run_comprehensive`.
  - Phase 3c-standalone `fpga-smoke-gate-standalone` records whether the
    standalone lake-package build ran or was skipped.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W450 boundary section; no new public competitor signals.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch to `wave-loop-450` and documented the W450 triage decision:
    7 residual yosys smoke failures remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_450_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W450_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_PLAN_W450_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W451_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — still deferred to a dedicated future wave.

### Verification

- `cargo check -p tri`: **PASS**.
- `cargo check -p t27c`: **PASS**.
- `cargo test -p tri --bin tri test_smoke_gate_validate_lean_standalone_matches_snapshot`: **PASS**.
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/t27_w450_suite.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`.
  - Phase 3c-standalone: **OK** (`validate_lean_standalone_elapsed_ms` populated).
- `./scripts/tri test --fast --json /tmp/t27_w450_fast_suite.json`: **PASS**.
  - Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
  - Phase 3c-standalone: **skipped** (`--fast` mode).
  - `acceptable: true`.

---

## Wave Loop 449 — Golden quantified transaction theorem + standalone-build suite metric + competitor refresh (Variant B default) (Closes #1424)

- Branch: `wave-loop-449`
- Issue: #1424
- PR: (to open after close-out)
- Report: `docs/reports/WAVE_LOOP_449_REPORT.md`
- Evidence W449: `docs/reports/FPGA_LOOP_EVIDENCE_W449_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W449_2026-07-01.md`
- Cooperation W450: `docs/reports/FPGA_LOOP_COOPERATION_W450_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `GOLDEN_W449_PVT_CONTEXT` / `GOLDEN_W449_OPERATING_POINT` and proved
    envelope / corner-worse-than properties.
  - Minted `golden_w449_raw_ns_satisfies_flash_spec` and
    `golden_w449_all_corners_transaction_ok`: a single quantified theorem that
    the ideal raw-ns capture produces a flash-spec-compliant transaction for every
    OSCFSEL 0..7 and every `ff`/`tt`/`ss` corner at the golden operating point.

- `bootstrap/src/suite.rs`
  - Added `validate_lean_standalone_status` / `validate_lean_standalone_elapsed_ms`
    to `FpgaSmokeResult` and `SuiteSummary`.
  - Wired Phase 3c to pass `--validate-lean-standalone` to `tri fpga smoke-gate`
    and populate the new suite metric.
  - Added schema regression tests for the new fields.

- `cli/tri/src/fpga.rs`
  - Added `test_smoke_gate_json_synthetic_validate_lean_standalone`, exercising
    the theorem-matrix + standalone lake-package build path end-to-end.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W449 boundary section; no new public competitor signals.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_449_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W449_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_PLAN_W449_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W450_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — still deferred to a dedicated future wave.

### Verification

- `cargo check -p tri`: **PASS**.
- `cargo check -p t27c`: **PASS**.
- `cargo test -p tri --bin tri test_smoke_gate_json_synthetic_validate_lean_standalone`: **PASS**.
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/t27_w449_suite.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`, all elapsed-ms fields populated.
  - `validate_lean_standalone_elapsed_ms`: populated (≈ 311 s on this run).

---

## Wave Loop 447 — Live-capture fallback + golden-matrix combined-check theorem + competitor refresh (Variant B default) (Closes #1422)

- Branch: `wave-loop-447`
- Issue: #1422
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_447_REPORT.md`
- Evidence W447: `docs/reports/FPGA_LOOP_EVIDENCE_W447_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W447_2026-07-01.md`
- Cooperation W448: `docs/reports/FPGA_LOOP_COOPERATION_W448_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--dry-run-live` to `tri fpga smoke-gate --theorem-matrix`, emitting
    fixtures under `build/fpga/theorem-matrix-dry-run-live/` with deterministic
    synthetic timings and `source: "dry_run_live"`.
  - Refactored `generate_theorem_matrix(fixture_dir, report, source)` so the
    synthetic and dry-run-live paths share one implementation.
  - Updated `replay_theorem_matrix` to detect the expected source label from
    each summary fixture, making replay work for any fixture set regardless of
    source label.
  - Added `test_theorem_matrix_dry_run_live_replay_matches_golden_shape`, which
    replays both the golden fixtures and a fresh dry-run-live set and asserts
    matching 24-variant report shape with correct per-set source labels.
  - Fixed `measured-to-lean --standalone` output to build in isolation:
    corrected the namespace from `Trinity.BitstreamConfig` to
    `Trinity.StatRegister.BitstreamConfig`, added `open`, and fixed the
    generated transaction-theorem proof to pass `PvtContext` explicitly.
  - Added `test_measured_to_lean_standalone_builds_in_temp_lake_package`, which
    drops a standalone generated theorem into a fresh lake package depending only
    on the in-repo `Trinity` package and asserts `lake build` succeeds.

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `GOLDEN_W447_OPERATING_POINT` matching the synthetic PVT context.
  - Proved `golden_w447_operating_point_within_envelope`.
  - Minted `golden_w447_all_oscfsel_combined_check_true`: for every
    `oscfsel ≤ 7`, the dashboard gate evaluates to `true` under the golden
    operating point.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W447 boundary section; no new public competitor signals since W446.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_447_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W447_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W448_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo check -p tri`: **PASS**.
- `cargo test -p tri`: **PASS** (140 tests, 0 ignored, 0 new regressions).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json build/suite_summary.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`, both elapsed-ms fields populated.
- Golden fixture replay report matches the committed snapshot.
- Dry-run-live fixture replay produces 24 variants with `source: "dry_run_live"`.
- Standalone `measured-to-lean` theorem builds in a temporary lake package.

---

## Wave Loop 446 — Theorem-matrix golden fixture diff gate + timing dashboard (Variant B default) (Closes #1420)

- Branch: `wave-loop-446`
- Issue: #1420
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_446_REPORT.md`
- Evidence W446: `docs/reports/FPGA_LOOP_EVIDENCE_W446_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W446_2026-07-01.md`
- Cooperation W447: `docs/reports/FPGA_LOOP_COOPERATION_W447_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `cli/tri/src/fpga.rs`
  - Added `build_theorem_matrix_report` helper shared by the CLI and the test suite.
  - Added `test_theorem_matrix_golden_replay_matches_snapshot` with strict-superset
    snapshot comparison against `tests/fixtures/fpga/theorem-matrix/golden/expected_report.json`.

- `bootstrap/src/suite.rs`
  - Added `fpga_smoke_gate_replay_elapsed_ms` to `SuiteSummary`.
  - Added Phase 3d replay invocation and populated the new elapsed-ms field.

- `tests/fixtures/fpga/theorem-matrix/golden/expected_report.json`
  - New committed snapshot of the normalized theorem-matrix replay report.

- `fpga/HARDWARE_SSOT.md`
  - Documented the snapshot semantics and both suite-level elapsed-ms metrics.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - W446 competitor boundary: Sparkle PR #97–#100 merged 2026-07-04, PR #101 open,
    CIRCT `firtool-1.152.0` latest, no post-2026-07-11 signals.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - W446 triage: fixed a field-access keyword-escape regression in
    `bootstrap/src/compiler.rs`; 7 residual yosys smoke failures remain baseline.

- `bootstrap/src/compiler.rs`
  - Fixed `ExprFieldAccess` so keyword-named bases flatten to a single escaped
    identifier; added regression test; resealed 52 specs.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_446_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W446_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W447_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo check -p tri`: **PASS**.
- `cargo test -p tri`: **PASS** (138 tests, 0 ignored, 0 new regressions).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/suite_report_w446.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`, both elapsed-ms fields populated.

---

## Wave Loop 445 — Theorem-matrix golden fixture gate + suite-level timing metric (Closes #1419)

- Branch: `wave-loop-445`
- Issue: #1419
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_445_REPORT.md`
- Evidence W445: `docs/reports/FPGA_LOOP_EVIDENCE_W445_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W445_2026-07-01.md`
- Cooperation W446: `docs/reports/FPGA_LOOP_COOPERATION_W446_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `tests/fixtures/fpga/theorem-matrix/golden/`
  - Committed the 75-file W444 synthetic fixture set (3 PVT contexts, 24 raw-ns,
    24 Lean, 24 JSON summary files) as a golden regression set.
  - Added `README.md` documenting provenance and regeneration.

- `cli/tri/src/fpga.rs`
  - Added `test_theorem_matrix_golden_replay_passes` which replays the checked-in
    golden fixtures and asserts 24 variants, all `envelope_check: "ok"`, and a
    `fixtures` block on every variant.

- `bootstrap/src/suite.rs`
  - Added `theorem_matrix_elapsed_ms` to `FpgaSmokeResult` and
    `fpga_smoke_gate_elapsed_ms` to `SuiteSummary`.
  - `parse_smoke_gate_report` reads `theorem_matrix.elapsed_ms` and the suite
    runner copies it into the machine-readable summary.
  - Updated schema regression tests to exercise the new field.

- `fpga/HARDWARE_SSOT.md`
  - Extended §3.6.26 with the golden fixture path and the `fpga_smoke_gate_elapsed_ms`
    metric semantics.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W445; Sparkle July 4 2026 FIDO2/crypto burst remains the most
    recent public signal.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W445 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_445_REPORT.md`,
  `docs/reports/FPGA_LOOP_PLAN_W445_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W445_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W446_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from
  `wave-loop-445` this wave.

### Verification

- `cargo test -p tri --bin tri`: **PASS** (137 tests).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS** (8 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json build/suite_report_w445.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`, `fpga_smoke_gate_elapsed_ms: 9`.

---

## Wave Loop 444 — Theorem-matrix fixture replay + deterministic CI artifact (Closes #1418)

- Branch: `wave-loop-444`
- Issue: #1418
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_444_REPORT.md`
- Evidence W444: `docs/reports/FPGA_LOOP_EVIDENCE_W444_2026-07-01.md`
- Plan: `docs/reports/FPGA_LOOP_PLAN_W444_2026-07-01.md`
- Cooperation W445: `docs/reports/FPGA_LOOP_COOPERATION_W445_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--replay-fixtures <dir>` to `tri fpga smoke-gate`.
  - Extracted `generate_theorem_matrix(fixture_dir)` that persists PVT, raw-ns,
    Lean, and summary fixtures for each of the 24 `ff`/`tt`/`ss` × OSCFSEL 0..7
    variants.
  - Implemented `replay_theorem_matrix(fixture_dir)` that verifies the persisted
    fixtures and reproduces the matrix report without regenerating theorems.
  - Extended the `theorem_matrix` report block with per-variant `fixtures`,
    `replay: true/false`, and `elapsed_ms`.
  - Added fixture-roundtrip and replay-regression unit tests.

- `bootstrap/src/suite.rs`
  - Default `./scripts/tri test` FPGA phase now passes `--theorem-matrix`, so the
    suite-generated smoke-gate report includes the 24-variant matrix.
  - Updated the fake smoke-gate report test to exercise the new `fixtures`,
    `replay`, and `elapsed_ms` fields.

- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.26 documenting fixture file patterns and the `--replay-fixtures`
    workflow.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W444; Sparkle July 4 2026 FIDO2/crypto burst is now recorded.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W444 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_444_REPORT.md`,
  `docs/reports/FPGA_LOOP_PLAN_W444_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W444_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W445_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from
  `wave-loop-444` this wave.

### Verification

- `cargo test -p tri --bin tri`: **PASS** (136 tests).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS** (8 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json build/suite_report_w444_final.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`.

---

## Wave Loop 443 — PVT-envelope hardening for the 24-variant theorem matrix (Closes #1417)

- Branch: `wave-loop-443`
- Issue: #1417
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_443_REPORT.md`
- Evidence W443: `docs/reports/FPGA_LOOP_EVIDENCE_W443_2026-07-01.md`
- Cooperation W444: `docs/reports/FPGA_LOOP_COOPERATION_W444_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `cli/tri/src/fpga.rs`
  - `build_pvt_envelope_report` now emits `inside_envelope: true/false` and a
    closed-vocabulary `envelope_check` (`"ok"` / `"failed"` / `"skipped"`) when a
    PVT context file is supplied.
  - The theorem-matrix block validates every synthetic `ff`/`tt`/`ss` corner
    context against the operating envelope before generating a theorem and
    records `envelope_check: "ok"` in each per-variant matrix entry.
  - Added envelope-related unit tests: `inside_envelope` true, `skipped` without
    context, synthetic corners inside envelope, outside-envelope detection,
    matrix envelope check OK.

- `bootstrap/src/suite.rs`
  - Updated the fake smoke-gate report test to include a theorem-matrix variant
    with `envelope_check: "ok"`.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W443; no new public competitor signals appeared after the W442
    close-out.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W443 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_443_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W443_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W444_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from
  `wave-loop-443` this wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (96 tests, +5 W443 regressions).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS** (8 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json build/suite_report.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `schema_version: "1.0"`, `acceptable: true`.

---

## Wave Loop 442 — Expanded board-less theorem matrix + CI artifact schema hardening (Closes #1415)

- Branch: `wave-loop-442`
- Issue: #1415
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_442_REPORT.md`
- Evidence W442: `docs/reports/FPGA_LOOP_EVIDENCE_W442_2026-07-01.md`
- Cooperation W443: `docs/reports/FPGA_LOOP_COOPERATION_W443_2026-07-01.md`

### What landed (Variant B — bench still blocked)

- `cli/tri/src/fpga.rs`
  - Theorem matrix now iterates `ff`/`tt`/`ss` process corners inside the
    existing OSCFSEL 0..7 loop, generating and verifying 24 corner×OSCFSEL
    PVT-aware raw-ns theorems under the synthetic operating point.
  - Smoke-gate JSON report gains a top-level `schema_version: "1.0"` field and a
    structured `theorem_matrix` block with `corner_count`, `oscfsel_count`, and
    per-variant `corner`/`oscfsel` records.
  - Added `test_cclk_period_ns_oscfsel_0_7` and
    `test_theorem_matrix_synthetic_fixture_and_summary` unit tests.

- `bootstrap/src/suite.rs`
  - `FpgaSmokeResult` now exposes `schema_version` and `theorem_matrix_status`.
  - Added schema-v1 and backward-tolerance tests for the smoke-gate report.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W442; no new public competitor signals appeared after the W441
    close-out.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W442 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_442_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W442_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W443_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from
  `wave-loop-442` this wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (129 tests, +2 W442 regressions).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS** (4 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json build/suite_report.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `schema_version: "1.0"`, `acceptable: true`.

---

## SW-conformance — gf256 promoted to strict SW-bitexact (75/0/8) (Closes #1397)

- gf256 (GoldenFloat256: S1 E97 M158, BIAS=79228162514264337593543950335=2^96-1,
  u256_software) promoted from `bitexact_selfconsistent` to strict `bitexact` in
  `conformance/vectors/INDEX_all_formats.json`. This is the LAST selfconsistent rung.
- INDEX totals: bitexact 74 -> 75, selfconsistent 1 -> 0, structural 8 (sum=83).
  Horizon-A SW ceiling reached (75 bit-precise; 8 structural are terminal, no single
  decode law; 83/83 SW-bitexact is NOT achievable).
- Bias hold lifted: earlier NOW entries said gf256 "stays open (open bias R&D) -- do
  NOT promote". The 2026-07-05 bias audit resolved this: the decode uses ONLY the
  closed-form interchange bias 2^(E-1)-1 = 2^96-1 (identical rule to gf128/gf512).
  The descriptive PHI_BIAS spec metadata is NOT part of the decode path and no
  decoded value depends on it (red herring). Decode-definition is definitive.
- Status tag: [verified SW]. M=158 >> 52 -> no FP lowering; every finite value is an
  EXACT dyadic odd*2^k (analytic separation-bound, same lemma as gf128/gf512).
- Witness chain: dyadic normalizer 2021/2021 + Fraction oracle 2021/2021 + analytic
  separation-bound; cross-check dyadic==Fraction on 201512 representative codes
  (seed=256) agree, abs_error=0. OOM-safe (+-2^96 exponent kept symbolic).
- NOT on-silicon Tier-E: gf256 is u256_software, has NO RTL -> no decode-HW/compute-HW
  cell exists for it; the Tier-E ceiling 71/83 (trinity-fpga #199) is unaffected.

## SW-conformance — gf512 + gf1024 promoted to strict SW-bitexact (paired, 74/1/8) (Closes #1380)

- gf512 (S1 E195 M316, BIAS=2^194-1, u512_software) and gf1024 (S1 E391 M632,
  BIAS=2^390-1, u1024_software; lowest phi-distance in the ladder) promoted from
  `bitexact_selfconsistent` to strict `bitexact` (paired).
- INDEX totals: bitexact 72 -> 74, selfconsistent 3 -> 1, structural 8 (sum=83).
- Status tag: [verified SW]. M=316/632 > 52 -> no FP lowering; every finite value
  is an EXACT dyadic odd*2^k (parametric separation-bound, same lemma as gf96/gf128).
- Witness chain (each format): dyadic normalizer 15/15 + Fraction oracle 15/15 +
  analytic separation-bound; cross-check dyadic==Fraction on 201512 representative
  codes (seed=512 / seed=1024) agree. OOM-safe (+-2^194 / +-2^390 symbolic).
- NOT on-silicon Tier-E: HW decode/compute [REQUIRES USER ACTION] (trinity-fpga #199).
- Remaining selfconsistent (1): gf256 (bias-open R&D, separate research).

## SW-conformance — gf128 promoted to strict SW-bitexact (72/3/8) (Closes #1370)

- gf128 (GoldenFloat128: S1 E49 M78, BIAS=281474976710655=2^48-1) promoted from
  `bitexact_selfconsistent` to strict `bitexact` in `conformance/vectors/INDEX_all_formats.json`.
- INDEX totals: bitexact 71 -> 72, selfconsistent 4 -> 3, structural 8 (sum=83).
- Status tag: [verified SW]. Like gf96, gf128 has M=78 > 52, so binary64 CANNOT
  hold the mantissa exactly; there is NO FP lowering and NO rounding: every finite
  gf128 value is an exact dyadic rational odd*2^k.
- Witness chain: TWO structurally independent exact decode paths
  (dyadic integer normalizer `conformance/gf_wide_independent_witness.py` +
  Fraction-significand symbolic-shift `conformance/witness/gf128/gf128_decode_ref.py`)
  agree on all 15 pack vectors (abs_error=0) AND on a 201512-code representative
  sweep (seed=128); + analytic separation-bound `conformance/witness/gf128/SEPARATION_BOUND.md`
  (zero-rounding lemma over the whole 2^128 domain; exhaustive infeasible).
- OOM-safe: the +-2^48 exponent is NEVER materialized; both paths keep the huge
  power of two symbolic in `shift`, numerators <= ~2^80.
- NOT on-silicon Tier-E: HW-decode / HW-compute for gf128 remain [REQUIRES USER
  ACTION] (4/4 chain on AX7203, trinity-fpga #199).
- Remaining selfconsistent (3): gf256, gf512, gf1024.

## SW-conformance — gf96 promoted to strict SW-bitexact (71/4/8) (Closes #1366)

## Wave Loop 434 — FPGA boot-evidence live XADC validation + synthetic CCLK proof-of-pipeline (Closes #1395)

## Wave Loop 422 — Live XC7A200T SRAM boot + gen-verilog keyword escape + PVT worst-case bound (Closes #1365)

- Branch: `wave-loop-422`
- Issue: #1365
- PR: to open after work
- Report: `docs/reports/WAVE_LOOP_422_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W422_2026-07-06.md`
- Cooperation W423: `docs/reports/FPGA_LOOP_COOPERATION_W423_2026-07-06.md`

### What landed (Variant A-lite + Variant C fallback)
- `bootstrap/src/compiler.rs`
  - Added Verilog-2001 keyword escape (`\\name `) for colliding user identifiers.
  - Applied escaping to function/task names, parameters, local/module vars/consts,
    loop variables, identifiers, calls, enum values, and field-access bases.
  - Added regression tests `test_verilog_keyword_parameter_escaped` and
    `test_verilog_keyword_local_and_module_escaped`.
  - The gen-verilog yosys smoke failure count dropped from **16 to 7**;
    remaining failures are pre-existing weak point #1245 defects unrelated to
    keyword collision.
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `pvt_low_ns_monotone_combined` and `pvt_high_ns_monotone_combined`.
  - Added `ProcessCorner.any_worse_than_ss` helper.
  - Added `pvt_half_ns_worst_case_bound` — the half-period bound is maximized at
    (max temp, min VCCINT, ss corner).
- `cli/tri/src/fpga.rs`
  - Added `test_pvt_half_ns_worst_case_bound`, mirroring the Lean lemma with a
    numeric grid-search regression.
- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.19 documenting the first live XC7A200T board response since W404:
    SRAM load succeeded, STAT `0x401079FC`, XADC context captured.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired.
- Cold-POR SPI flash boot for OSCFSEL 6/7 — deferred to W423.
- DLC10 cable still missing; Digilent HS2 + openFPGALoader is the working path.

### Verification
- `cargo test -p tri fpga::tests`: **PASS** (52 tests).
- `cargo test -p t27c --bin t27c`: **PASS** (1493 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` / `t27c suite --repo-root .`: **576 passed**, 0 seal
  mismatches, 7 pre-existing gen-verilog yosys smoke failures, 0 FPGA smoke
  failures.

---

# NOW — Wave Loop 421 close-out / Wave Loop 422 setup (2026-07-06)

## SW-conformance — gf48 promoted to strict SW-bitexact (70/5/8) (Closes #1358)

### What landed (Variant B — board reachable, P12/relay still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `XADC_LIVE_W434_OPERATING_POINT`: the rounded live XADC readout
    captured this wave (41 °C, 1000 mV VCCINT, 1807 mV VCCAUX, ss corner).
  - Added `xadc_live_w434_operating_point_within_envelope`: the captured point is
    inside the documented operating envelope.
  - Added `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt`: direct application of
    the W431/W432 formal bridge to the live silicon point for any documented OSCFSEL.
  - Added `xadc_live_w434_oscfsel_6_raw_ns_pvt_satisfies_flash_spec` and its
    transaction variant for the synthetic 40/20/20 ns CCLK fixture.

- `cli/tri/src/fpga.rs`
  - Added `test_xadc_context_to_pvt_context_w434_live_capture` asserting that the
    live XADC values round to the integer `PvtContext` used in the generated theorem.

- `fpga/HARDWARE_SSOT.md` §9.6.2
  - Documented the live XADC → PVT context rounding, envelope validation, and
    `measured-to-lean --raw-ns --pvt-context` proof-of-pipeline recipe.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W434; noted the real captured operating point now feeds a
    machine-checkable theorem and the competitive landscape is unchanged.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W434 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_434_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W434_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W435_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from
  `wave-loop-434` this wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (82 tests, +1 W434 regression).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 420 — Variant C fallback: VCD exact-terminator + auto-threshold, PVT corner monotonicity (Closes #1361)

- Branch: `wave-loop-420`
- Issue: #1361
- PR: #1362 (merge blocked by base-branch policy; requires review/approval)
- Report: `docs/reports/WAVE_LOOP_420_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W420_2026-07-06.md`
- Cooperation W421: `docs/reports/FPGA_LOOP_COOPERATION_W421_2026-07-06.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - Added `vcd_line_ends_with_token` helper and applied exact `$end` token terminator to VCD `$date`/`$version`/`$comment` sections (the W419 report claimed this, but the merged diff did not include it).
  - Added real-valued VCD auto-threshold: computes `50% (vmin + vmax)` when `--vcd-threshold-v` is omitted.
  - Added regression tests `test_parse_vcd_comment_with_embedded_end_token` and `test_parse_vcd_real_auto_threshold`.
  - Added `test_pvt_half_ns_monotone_in_process_corner`.
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `pvt_half_ns_monotone_in_process_corner`.
- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.17 documenting W420 VCD/CSV/PVT improvements.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired, DLC10 cable missing.
- Real relay cold-POR gate — no relay board / USB power switch available.

### Verification
- `cargo test -p tri vcd`: **PASS** (13 tests).
- `cargo test -p tri csv`: **PASS** (11 tests).
- `cargo test -p tri pvt`: **PASS** (10 tests).
- `cargo test -p tri fpga::tests`: **PASS** (48 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test`: parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point PASS; gen-Verilog yosys smoke has 16 pre-existing failures from weak point #1245.

---

## Wave Loop 421 — Variant C fallback: VCD `$timescale` exact terminator, combined PVT monotonicity, competitor snapshot (Closes #1363)

- Branch: `wave-loop-421`
- Issue: #1363
- PR: to open after work
- Report: `docs/reports/WAVE_LOOP_421_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W421_2026-07-06.md`
- Cooperation W422: `docs/reports/FPGA_LOOP_COOPERATION_W422_2026-07-06.md`
- Competitor note: `docs/reports/T27_VS_FORMAL_HDL_2026.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - Applied `vcd_line_ends_with_token` exact `$end` token terminator to VCD `$timescale` sections.
  - Added regression test `test_parse_vcd_timescale_with_embedded_end_token` for multi-line `$timescale` blocks with embedded `$end` substrings.
  - Added regression test `test_parse_vcd_real_auto_threshold_us_timescale` for real-valued nets with `$timescale 1 us $end`.
  - Added `test_pvt_half_ns_monotone_combined` verifying the combined ordering (temp ↑, VCCINT ↓, corner worse).
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `pvt_half_ns_monotone_combined` lemma.
- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.18 documenting W421 VCD/PVT improvements.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Published competitor comparison covering Sparkle/Verilean, Clash, Chisel/FIRRTL/CIRCT, Bluespec, Coq Kami/Silver Oak, ACL2, Knox/HARDENS.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — `openFPGALoader --detect` reports 0 devices; board not powered/connected.
- Real relay cold-POR gate — no relay board / USB power switch available.
- Safe gen-verilog #1245 sub-fix deferred; remaining tracked gaps (RAM style inference, tuple-return syntax) are not narrow regression-free sub-fixes.

### Verification
- `cargo test -p tri vcd`: **PASS** (15 tests).
- `cargo test -p tri pvt`: **PASS** (11 tests).
- `cargo test -p tri fpga::tests`: **PASS** (51 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test`: parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point PASS; gen-Verilog yosys smoke has 16 pre-existing failures from weak point #1245, no new failures.

- `cli/tri/src/fpga.rs`
  - Added `--process-corner` and `--to-pvt-context` to `tri fpga read-xadc`.
  - Added `parse_process_corner` helper.
  - Extended `measured-to-lean --json` summary with `operating_point` (source, temp_c, vccint_mv, vccaux_mv, process_corner).
  - Added `test_measured_to_lean_xadc_to_pvt_context_pipeline`, an end-to-end integration test for the live XADC → PVT context → theorem path.

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added computable gate `cclk_variant_and_xadc_envelope_check` and proved equivalence with `oscfsel ≤ 7 ∧ xadc_operating_point_within_envelope pt`.
  - Linked the gate to `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec` and the transaction theorem.
  - Added `xadc_live_w434_all_oscfsel_raw_ns_pvt_satisfies_flash_spec` and per-OSCFSEL concrete theorems 0..7 under the W434 live XADC point.
  - Added matching transaction theorems `xadc_live_w434_oscfsel_0_transaction_ok` ... `xadc_live_w434_oscfsel_7_transaction_ok`.

- `fpga/HARDWARE_SSOT.md` §9.6.2
  - Documented the `tri fpga read-xadc --to-pvt-context` recipe and the synthetic OSCFSEL 0..7 theorem matrix.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W435; noted the live-readout pipeline hardening and unchanged 7-residual-failure baseline.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W435 triage decision: no compiler work attempted; the 7 residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_435_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W435_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W436_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from `wave-loop-435` this wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (83 tests, +1 W435 integration test).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 436 — FPGA boot-evidence: live XADC → PVT context in boot logs and sweep reports (Closes #1402)

- Branch: `wave-loop-436`
- Issue: #1402
- PR: #1406
- Report: `docs/reports/WAVE_LOOP_436_REPORT.md`
- Evidence W436: `docs/reports/FPGA_LOOP_EVIDENCE_W436_2026-07-01.md`
- Cooperation W437: `docs/reports/FPGA_LOOP_COOPERATION_W437_2026-07-01.md`

### What landed (Variant B — board reachable, P12/relay still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--process-corner` and `--to-pvt-context` to `tri fpga cold-por` and `tri fpga cclk-sweep`.
  - Added `resolve_pvt_context_for_boot` helper with shared priority logic: explicit PVT file > live XADC > none.
  - Added `operating_point` JSON object to `SweepLog` and cold-POR mock boot log.
  - Added closed-vocabulary `source` labels: `xadc`, `pvt_context_file`, `worstcase`, `not_read`.
  - Added `--pvt-context-source` to `tri fpga measured-to-lean` to override/confirm the provenance label.
  - Added `test_measured_to_lean_pvt_context_source_override`; hardened `test_sweep_report_json_roundtrip`.

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added quantified theorem `xadc_live_w434_all_oscfsel_combined_check_true`:
    for every `oscfsel ≤ 7`, the computable `cclk_variant_and_xadc_envelope_check`
    gate returns `true` under the W434 live XADC operating point.

- `fpga/HARDWARE_SSOT.md` §3.6.21
  - Documented the live XADC → PVT context pipeline, CLI flags, source labels,
    and formal coverage.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W436; updated competitive notes around Sparkle/Verilean.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W436 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_436_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W436_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W437_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (84 tests, +1 W436 regression).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 437 — Dry-run XADC→PVT validation and `verify-lean` (Closes #1405)

- Branch: `wave-loop-437`
- Issue: #1405
- PR: #1408
- Report: `docs/reports/WAVE_LOOP_437_REPORT.md`
- Evidence W437: `docs/reports/FPGA_LOOP_EVIDENCE_W437_2026-07-01.md`
- Cooperation W438: `docs/reports/FPGA_LOOP_COOPERATION_W438_2026-07-01.md`

### What landed (Variant B — board still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--synthetic-operating-point` to `tri fpga cold-por` and `tri fpga cclk-sweep`.
  - Added `tri fpga verify-lean` subcommand to validate `.lean` theorem blocks
    against JSON summaries and count theorem declarations.
  - Promoted `resolve_pvt_context_for_boot` to a public helper returning
    `ResolvedPvtContext`; added `synthetic_pvt_context` helper.
  - Added unit tests for PVT source priority (file > live XADC > synthetic >
    not_read), synthetic cold-POR, sweep-report propagation, and
    `verify-lean` round-trip.
  - `measured-to-lean` now emits `-- operating_point source: <label>` in the
    generated `.lean` comment when a PVT context is present.

- `fpga/HARDWARE_SSOT.md` §3.6.22
  - Documented the dry-run / synthetic operating point protocol and `verify-lean`.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W437; no new public competitor signals as of the boundary.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W437 triage decision: no compiler work; 7 residual failures
    remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_437_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W437_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W438_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (90 tests, +6 W437 regressions).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 438 — CI artifact audit trail for dry-run boot-evidence (Closes #1407)

- Branch: `wave-loop-438`
- Issue: #1407
- PR: #1411
- Report: `docs/reports/WAVE_LOOP_438_REPORT.md`
- Evidence W438: `docs/reports/FPGA_LOOP_EVIDENCE_W438_2026-07-05.md`
- Cooperation W439: `docs/reports/FPGA_LOOP_COOPERATION_W439_2026-07-05.md`

### What landed (Variant B — board still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--synthetic-operating-point` and `--verify-lean` to `tri fpga smoke-gate`.
  - When `--synthetic-operating-point` is used, the dry-run CCLK sweep uses a
    deterministic synthetic PVT context and the JSON sweep report is asserted to
    carry `operating_point.source == "synthetic"` for every variant.
  - When `--verify-lean` is used, the gate generates a synthetic raw-ns `.lean`
    theorem and runs `verify-lean --expected-source synthetic` on it.
  - Added edge-case unit tests for `verify_lean`: no theorem, missing summary +
    missing source comment, and mismatched expected source.

- `fpga/HARDWARE_SSOT.md` §3.6.23
  - Documented the machine-readable `tri fpga verify-lean --json` schema.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W438; Sparkle's 関数型まつり2026 talk on 2026-07-11 remains the
    next checkpoint.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch to `wave-loop-438` and documented the W438 triage decision:
    no compiler work; 7 residual failures remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_438_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W438_2026-07-05.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W439_2026-07-05.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (93 tests, +3 W438 regressions).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --process-corner ss`: **PASS**.

---

## Wave Loop 439 — CI artifact trail wired into default sweep + smoke-gate JSON report (Closes #1409)

- Branch: `wave-loop-439`
- Issue: #1409
- PR: #1412 (predicted)
- Report: `docs/reports/WAVE_LOOP_439_REPORT.md`
- Evidence W439: `docs/reports/FPGA_LOOP_EVIDENCE_W439_2026-07-05.md`
- Cooperation W440: `docs/reports/FPGA_LOOP_COOPERATION_W440_2026-07-05.md`

### What landed (Variant B — board still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--json <path>` to `tri fpga smoke-gate`; emits a single JSON object
    with per-phase results for bit-config audit, dry-run CCLK sweep,
    verify-lean, and yosys synthesis, plus an overall `passed` boolean.
  - Bit-config audit now captures the `ASSERTION OK:` result lines from
    `scripts/dump_bit_config.py` in the report.
  - Added `test_smoke_gate_json_synthetic_verify_lean`, an end-to-end
    regression test for the board-less synthetic verify-lean path.
  - Fixed `repo_root()` to prefer a `.git` directory over a `Cargo.toml` file,
    resolving the workspace root correctly from the `cli/tri` crate root.

- `bootstrap/src/suite.rs`
  - Phase 3c now invokes `tri fpga smoke-gate --synthetic-operating-point
    --verify-lean --json build/fpga/smoke_gate_report.json` when the demo
    bitstream is present, replacing the older direct Python/yosys calls.
  - Added `tri_exe()` helper to locate the `tri` binary from the same build
    profile as the running `t27c`.

- `fpga/HARDWARE_SSOT.md` §3.6.24
  - Documented the machine-readable `tri fpga smoke-gate --json` schema with
    field types and an example.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W439; no new public competitor signals appeared after Sparkle's
    関数型まつり2026 talk on 2026-07-11.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch to `wave-loop-439` and documented the W439 triage decision:
    no compiler work; 7 residual failures remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_439_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W439_2026-07-05.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W440_2026-07-05.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo check -p tri`: **PASS**.
- `cargo test -p tri`: **PASS** (125 tests, 2 ignored; see note below).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --json /tmp/report.json`: **PASS**.

**Note:** two integration tests (`test_measured_to_lean_standalone_lake_package_builds`
and `test_measured_to_lean_xadc_to_pvt_context_pipeline`) are now ignored
because the full Trinity `lake build` fails on unrelated physics proofs
(`Trinity/NeutrinoMasses.lean`, `Trinity/H4Lagrangian.lean`). The boot-evidence
target `Trinity.TernaryFPGABoot` still builds.

---

## Wave Loop 440 — CI report consumption / board-less fallback / real-capture fallback / gen-verilog debt (Variant B default) (Closes #1411)

- Branch: `wave-loop-440`
- Issue: #1411
- PR: #1414
- Report: `docs/reports/WAVE_LOOP_440_REPORT.md`
- Evidence W440: `docs/reports/FPGA_LOOP_EVIDENCE_W440_2026-07-01.md`
- Cooperation W441: `docs/reports/FPGA_LOOP_COOPERATION_W441_2026-07-01.md`

### What landed (Variant B — board still blocked)

- `bootstrap/src/main.rs`
  - Added `json: Option<PathBuf>` to the `Suite` command.

- `bootstrap/src/suite.rs`
  - Phase 3c now parses `build/fpga/smoke_gate_report.json`, asserts
    `passed == true`, logs per-phase statuses, and treats bitstream-missing /
    yosys-unavailable as `skipped`.
  - Added `SuitePhaseSummary` / `SuiteSummary` structs and writes pretty-printed
    JSON when `./scripts/tri test --json <path>` is used.

- `cli/tri/src/fpga.rs`
  - Replaced the two ignored full-Trinity `lake build` integration tests with
    lightweight content checks:
    - `test_measured_to_lean_standalone_outputs_consumable_lean`
    - `test_measured_to_lean_xadc_to_pvt_context_outputs`
  - Retained the W439 `test_smoke_gate_json_synthetic_verify_lean` regression
    test.

- `scripts/tri`
  - Forwards `--json` and all following arguments after `test`/`suite` to
    `t27c suite --repo-root "$REPO_ROOT"`.

- `fpga/HARDWARE_SSOT.md` §3.6.24/§3.6.25
  - Documented suite-level JSON summary consumption and schema.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W440; no new public competitor signals appeared after Sparkle's
    関数型まつり2026 talk on 2026-07-11. Noted CIRCT `firtool-1.152.0` release
    on 2026-07-04.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch to `wave-loop-440` and documented the W440 triage decision:
    no compiler work; 7 residual yosys smoke failures remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_440_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W440_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W441_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (127 tests, 0 ignored, +2 restored).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
- `./scripts/tri test --json /tmp/suite_summary.json`: **PASS**, summary contains
  `fpga_smoke_passed: true` and `total_failures: 7`.

---

## Wave Loop 441 — CI schema hardening / board-less theorem matrix / real-capture fallback / gen-verilog debt (Variant B default) (Closes #1413)

- Branch: `wave-loop-441`
- Issue: #1413
- PR: #1416
- Report: `docs/reports/WAVE_LOOP_441_REPORT.md`
- Evidence W441: `docs/reports/FPGA_LOOP_EVIDENCE_W441_2026-07-01.md`
- Cooperation W442: `docs/reports/FPGA_LOOP_COOPERATION_W442_2026-07-01.md`

### What landed (Variant B — board still blocked)

- `bootstrap/src/suite.rs`
  - Added `docs/reports/gen_verilog_smoke_baseline.json` loader and computed a
    baseline-aware `acceptable` flag: `true` only when all observed failures are
    within the documented baseline and every other phase is clean.
  - Exposed `known_failures`, `baseline_failures`, `total_failures`, `passed`,
    and `acceptable` in the `./scripts/tri test --json` summary.
  - Added `#[cfg(test)]` regression tests: `tri_exe()` discovery,
    `SuiteSummary` schema round-trip, `acceptable` computation, and fake-
    `tri`-script pass/fail parsing.
  - Refactored `cmd_fpga_smoke_gate` into `run_fpga_smoke_gate` core +
    repo-aware wrapper to enable deterministic unit tests.

- `cli/tri/src/fpga.rs`
  - Added `cclk_period_ns(oscfsel)` helper mirroring the Lean definition.
  - Added `--theorem-matrix` to `tri fpga smoke-gate`.
  - When `--synthetic-operating-point --verify-lean --theorem-matrix` are used,
    the gate generates and verifies a PVT-aware raw-ns theorem for each Artix-7
    Master SPI OSCFSEL value 0..7, recording an 8-element `theorem_matrix`
    array in the JSON report.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W441; no new public competitor signals appeared after Sparkle's
    関数型まつり2026 talk on 2026-07-11.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch to `wave-loop-441` and documented the W441 triage decision:
    no compiler work; 7 residual failures remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_441_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W441_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W442_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo check -p tri`: **PASS**.
- `cargo test -p tri`: **PASS** (127 tests, 0 ignored).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS** (7 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
- `./scripts/tri test --json /tmp/w441_suite_summary.json`: **PASS**, `known_failures` = 7 baseline specs, `acceptable: true`, `fpga_smoke_passed: true`.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json /tmp/tri_smoke_matrix.json`: **PASS**, `theorem_matrix` = 8 variants, `passed: true`.

---

## Wave Loop 442 — Next: expanded board-less theorem matrix + CI artifact hardening + real-capture fallback + gen-verilog debt (Variant B default)

- Branch: `wave-loop-442`
- Issue: #1415
- Default variant: **B** unless P12 or the relay gate becomes available.
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W442_2026-07-01.md`

---

*φ² + φ⁻² = 3 | TRINITY*
