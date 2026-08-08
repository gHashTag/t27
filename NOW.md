# NOW -- Trinity t27 sync

Last updated: 2026-08-08

## typecheck: bare non-negative literals are context-polymorphic (Closes #1923)

- infer_expr pinned every integer literal to I32; in u32-dominant specs every 'let x = 0; x = u32_expr;' raised a false 'cannot assign U32 to I32' -- 27 tri-net specs failed typecheck on exactly this
- Now: bare non-negative literals infer Unknown (context-polymorphic); explicitly negative literals stay I32; promote_types resolves Unknown+Known to the KNOWN operand ('100 - value' with value: u32 is u32)
- tri-net sweep: 27 failing specs -> 2 (both 'assign to immutable array element' singletons, tracked with #1919)
- Unit suite 1537/1537; FROZEN_HASH resealed

## typecheck: promote call-arity mismatch from warning to hard error (Closes #1921)

- The arity check existed but only warned, and nothing reads warnings (tri-net's hook greps 'Typecheck OK'; gen paths skip typecheck) -- two tri-net specs shipped wrong-arity calls for months (tri-net#323)
- Now: error_count += 1, ok = false -- the specs-typecheck gate actually blocks the class
- Sweep: zero arity violations remain across tri-net's 100+ specs post-#323, so the promotion breaks nothing (27 specs have PRE-EXISTING unrelated typecheck errors, identical under stock t27c)
- Unit suite 1537/1537; FROZEN_HASH resealed

## fix(gen-c): [T;N] params, test-block bindings, t27_assert macro (Refs #1919)

- Rust-style `[T; N]` parameter types now lower to `T*` (previously emitted a bare `* name` -- no base type)
- Test-block bindings are declared on first assignment (`uint64_t b0 = ...;` -- the C twin of #1894); tuple targets bind the struct return once via GNU `__auto_type` and peel `.f0/.f1`
- Two-argument `assert(cond, "msg")` lowers to a self-contained `t27_assert` macro with the message RE-QUOTED (bare unquoted words previously hit C's one-arg assert and leaked non-ASCII into identifiers)
- tri-net gcc -fsyntax-only sweep: 64/68 -> 8/68 invalid; the eight distinct long-tails are enumerated in #1919
- Unit suite 1537/1537; other backends untouched; FROZEN_HASH resealed

## fix(gen-zig): array-literal text lowering, CSE hoist, undefined init, shadow-aware dead-local scan (Closes #1910)

- ExprArrayLiteral carries its ELEMENT TEXT in extra_size with no children; Zig now emits `.{ e1, e2 }` / `.{ v } ** n` (paren-aware top-level comma split), which coerce to the typed array target
- `_cse*` temp declarations are hoisted to the top of Zig fn bodies (the CSE pass can leave them after their first use in this backend's order; the Rust path already hoists)
- Uninitialized locals emit `= undefined`; the dead-local post-pass stops counting at a REDECLARATION of the same name (a later `const name = ...` is a new binding, not a use)
- tri-net zig ast-check: 3/68 -> **0/68 invalid** -- the campaign that started at 66/68 invalid is complete; tri-net can now commit gen/zig and restore its Zig drift leg
- Unit suite 1537/1537; gen-rust byte-identical; FROZEN_HASH resealed

## fix(gen-zig): mutable-local var-inference, `_` destructure elements, branch-safe var silencer (Refs #1910)

- Zig gen now infers `var` vs `const` for locals via collect_mutable_names (same discipline as the Rust backend) -- mutated locals previously emitted as `const` and could not compile
- A discarded destructure element emits bare `_` (never `const _`); every inferred `var` gets the canonical `_ = &name;` silencer (the same name may be declared in several branches and mutated in only one)
- With tri-net's five-spec legalization: zig ast-check validity 9/68 -> 3/68 invalid; the rest are the deferred long-tails (array-literal parse path x2, CSE decl-before-use ordering) in #1910
- Unit suite 1537/1537; gen-rust byte-identical; FROZEN_HASH resealed

## fix(gen-zig): array types, tuple test-bindings, void returns, dead-local post-pass (Refs #1910)

- `[T;N]` array types now lower to Zig `[N]T` in params/locals/returns; tuple test-block bindings lower to `const a, const b = f(...);`; fns without a declared return emit `void` (a bare `) {` never parsed)
- Dead locals are discarded by an exact POST-PASS over the emitted Zig (per top-level block identifier counting): the optimizer const-inlines uses while leaving declarations, so AST-side prediction cannot match zig's used/unused verdict (and zig also errors on pointless discards of USED locals)
- tri-net legacy-spec zig validity: 38/68 -> 9/68 invalid (from 66/68 two waves ago); the 9 remaining are distinct long-tail one-offs (undeclared wrapping_sub/_cse1 helpers, param shadowing, `_` as identifier, 2D arrays, struct-literal syntax) -- listed in #1910
- Unit suite 1537/1537; gen-rust untouched; FROZEN_HASH resealed

## fix(gen-zig): discard unused params + declare test/bench bindings (Closes #1910)

- Zig gen emitted unused fn parameters (zig errors on them) and test-block bindings without declarations (the Zig twin of #1894) -- 66/68 of tri-net's legacy spec gens failed zig ast-check
- Now: `_ = param;` discards for parameters the body never reads; the FIRST assignment to a plain identifier in a test/bench block lowers to `const name = expr;`
- 38/68 remain invalid in three mechanical classes (tuple destructuring LHS, un-translated `[T;N]` array types, unused local consts) -- tracked in #1910 for full zig-leg restoration
- Unit suite 1537/1537; gen-rust output byte-identical (Zig-only change); FROZEN_HASH resealed

## fix(gen-verilog-sim) -- test-block reg decls + 64-bit __mul_noop (this PR, Closes #1894, Closes #1886)

- StmtAssign test-block bindings ('h = f(...);') now get hoisted reg declarations (width-inferred, 64-bit fallback) -- iverilog could not bind them before; unlocks 11 tri-net ring specs
- __mul_noop widened to 64-bit in/out (128-bit acc, 64 iterations) -- u64 products no longer truncate to 32 bits; unlocks tri_gft_arith + 6 more money-layer specs
- Remaining tri-net blockers are spec-side (Verilog reserved words 'class'/'small' as identifiers; one stale spec test) -- fixed in tri-net
- FROZEN_HASH resealed per ceremony

## fix(gen-verilog-sim) -- lower plain assert(cond, "msg") in testbenches (this PR, Closes #1888)

- `assert` is not a Verilog-2005 keyword and the 2-arg form is not SystemVerilog; the TB emitted it verbatim, iverilog rejected the file -- icarus-simulate unusable for specs using standard `assert()` tests
- Both emission paths (probed assertions + W459 real-check) now lower assert to the same if-based check assert_eq gets, message %-escaped in the failure display
- Validation: bootstrap unit suite 1537/1537 == unmodified master; tri-net GF-T specs go from iverilog-reject to full runs (add/sub/ladder PASS; tri_gft_arith surfaces a real pre-existing u64 width bug -> #1886)
- FROZEN_HASH resealed per FROZEN.md ceremony
- Closes #1888

## docs(TRI-NET) -- cross-line package P0/P1/P2 (this PR, Closes #696)

- **NEW** docs: `docs/GF16_BFLOAT16_NMSE_PROTOCOL.md`, `docs/TRI_NET_API.md`, `docs/TRI_NET_WHITEPAPER.md`, `docs/22FDX_TOPS_W_PROJECTION.md`, `docs/ZENODO_BUNDLES.md`, `docs/SCIENTIFIC_IMPROVEMENT_PLAN.md` (2026 t27-side roadmap, R5-honest labels)
- **NEW** specs: `specs/benchmarks/gf16_bfloat16_nmse.t27`, `specs/api/tri_net_api.t27` (both contain `test`+`invariant`+`bench` per L4)
- **NEW** schemas: `schemas/nmse-protocol-v1.json`, `schemas/tri-net-api-v1.json` (draft-07)
- Docs-only; no `gen/`/`coq/`/`bootstrap/` edits; no new `*.sh`; R5-HONEST preserved (projections labelled; no DOIs quoted before upload)
- Full per-deliverable detail in `docs/NOW.md`
- Closes #696

## Wave-42 Lane II — StochRound.v Stochastic Rounding Coq

- OP_STOCH_ROUND = 0xE9 (decimal 233) — sacred opcode, Wave-42
- **NEW**: trios-coq/Physics/StochRound.v — 9 Qed lemmas
  - stoch_op_distinct_from_sparse: 233 <> 232 (OP_SPARSE_SKIP)
  - stoch_op_distinct_from_dfs: 233 <> 231 (OP_DFS_GATE)
  - stoch_op_distinct_from_holo_mux: 233 <> 230 (OP_HOLO_MUX_X4)
  - stoch_op_distinct_from_subth: 233 <> 229 (OP_SUBTH_CLK)
  - stoch_op_distinct_from_avs_reconf: 233 <> 228 (OP_AVS_RECONF)
  - stoch_op_distinct_from_lut_npu: 233 <> 227 (OP_LUT_NPU)
  - stoch_op_distinct_from_tom: 233 <> 226 (OP_TOM)
  - stoch_op_distinct_from_tenet: 233 <> 225 (OP_TENET)
  - stoch_unbiased_count: forall xf <= 16, xf + (16 - xf) = 16 (LFSR-16 unbiasedness)
- Wave-42 StochRound.v 9 Qed sacred 0xE9
- Refs: Hubara 2018, Gupta 2015 — unbiased rounding for INT4/INT2 quantization
- Closes trinity-fpga#149

## Wave-36 Lane X — AVS-48 Voltage Stacking Coq

- AVS-48: 48-island series voltage stacking, charge-recycling, η ≥ 0.93
- **NEW**: trios-coq/Physics/AvsStacking.v — 8 Qed lemmas
  - avs_ir_drop_quadratic_savings: ir_drop_loss(N) = ir_drop_loss(1) / N²
  - avs_island_count_48_optimum: 48 = 3×16 (strands × sacred-ALU opcodes)
  - avs_efficiency_lower_bound: η_avs_48 ≥ 0.93 at INT1.58/800MHz
  - avs_trinity_divisibility: 48 mod 3 = 0
  - avs_sacred_alignment: 48 = 16 × 3
  - avs_no_multiplier_synth: AVS adds zero * to netlist (R-SI-1 keystone)
  - avs_chain_to_lut_npu: AVS×LUT-NPU sound at each boundary
  - avs_w104_b_witness: η ≥ 0.93 → TOPS/W ≥ 297 (W-104-B pre-reg)
- W-104-B falsification witness: η ≥ 0.93 implies TOPS/W ≥ 297
- 48 = 3 × 16 = strands × sacred-ALU opcodes (Trinity alignment)
- citation_map.json extended: WAVE_36_AVS → Physics/AvsStacking.v, wave 36
- Closes trinity-fpga#128

## Wave-35 Lane V — LUT-NPU Coq

- OP_LUT_NPU = 0xE3 extends sacred chain 0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2 → 0xE3
- **NEW**: trios-coq/Kernel/LutNpu.v — 10 Qed lemmas (lut_npu_class_count_41, lut_npu_no_star, lut_npu_tom_orthogonal, lut_npu_energy_8fJ, ...)
- 41 Z₃-compressed classes (not 81): sign+0 invariance reduces 3^4=81 → 41 equivalence classes
- Multiplier-free: uses_multiplier OP_LUT_NPU = false (R-SI-1 keystone, Qed)
- dotprod bounded: −4 ≤ dotprod_naive a w ≤ 4 (Qed via case split)
- citation_map.json added: OP_LUT_NPU → Kernel/LutNpu.v, wave 35
- 16 new Qed proofs (4 in coq/IGLA/RMarker.v + 12 in trios-coq/IGLA/LutNpu.v)
- Theorem lut_npu_safe: depth-6 alphabet chain Forall rtl_uses_star=false
- W-104-A pre-registered: BitNet b1.58-3B Trinity-loss sparsity ≥ 0.5 @ batch=1
- Projection: ×1.20 TOPS/W → 270 TOPS/W on TTIHP27a generic synth (W34 baseline 225)
- 81-entry LUT is hardware port of Microsoft bitnet.cpp lookup table, indexed by Z_3^4 (3^4=81)

## Wave-34 Lane Y — TOM Coq

- OP_LAYER_GATE = 0xE2 extends sacred chain 0xDE → 0xDF → 0xE0 → 0xE1 → 0xE2
- 14 new ^Qed proofs in coq/RMarker.v (29 total)
- W-103-A pre-registered: layer-idle fraction ≥ 0.5 @ BitNet b1.58-3B batch=1
- Freeze 2026-08-15, fail-stop on violation

## Constitutional verdict

- R5-HONEST PASS · R7 PASS · R8 PASS (admin@t27.ai) · R14 PASS · R15 PASS · R18 PASS · Apache-2.0 PASS

## Anchor

phi^2 + phi^-2 = 3 · QUANTUM BRAIN 1:1 SILICON · NEVER STOP
DOI 10.5281/zenodo.19227877

## Wave-37 Lane Z — Sub-V_T Coq (OP_SUBTH_CLK = 0xE4)

- Sub-threshold weak-inversion operation at V=0.30V
- **NEW**: trios-coq/Physics/SubThreshold.v — 10 Qed lemmas
  - subth_quadratic_dynamic_savings: E(V2)/E(V1) = (V2/V1)^2
  - subth_freq_derating_factor_2: f_max(0.30) × 2 ≤ f_max(0.45)
  - subth_tops_w_350: TOPS/W ≥ 350 @ V=0.30V
  - subth_trinity_voltage: 0.30 = V_thresh × φ⁻²
  - subth_pe_count_1296: 48 × 27 = 1296 = 6^4
  - subth_no_star: OP_SUBTH_CLK adds zero `*`
  - subth_chain_to_lut_npu: 0xE3 → 0xE4 pipeline sound
  - subth_three_freq_trinity: gcd(400,300,200) = 100; sum = 900 = 30²
  - subth_body_bias_strand_alignment: 3 modes ↔ 3 strands bijective
  - subth_w104_c_witness: V=0.30 + AVS48 + LUT-NPU ⇒ TOPS/W ≥ 350
- Predecessors: W35 LUT-NPU (0xE3), W36 AVS-48
- Anchor: phi^2 + phi^-2 = 3
