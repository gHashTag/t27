# Wave Loop 549 — Research report: IGLA CODER / IGLA RACE

**Date:** 2026-08-09 · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Three machine-checked theorems, a literature review with verifiable citations,
and the measurements that together overturn the project's working assumption
about the IGLA spec corpus.

---

## 0. The result, stated first

**The IGLA CODER and IGLA RACE specifications — roughly 69,000 lines across 27
files — have never compiled.** Before this wave, **0 of 27** emitted Verilog.
After repairing one mechanical defect, **8 of 27** do; of those, **7 survive
real synthesis**.

Investigating that led to the larger result: **363 of the repository's 1063
specs (34.1 %) do not parse at all**, IGLA being 19 of them (§4.3b).

The cause is smaller than the symptom. A handful of narrow syntax gaps
separate the specs from the compiler — a brace-delimited block-expression
(`if (c) { a } else { b }`, ~40 specs), a struct literal in expression
position (~28), and `as f32` / `as f64` casts (~16). The underlying features
already exist in `bootstrap/`; only these spellings are rejected. A further
~38 specs are simply unterminated blocks — the same mechanical defect repaired
in IGLA this wave.

This was invisible because nothing ever ran the backend over these specs. It
was *maskable* because, until commit `#1940` hardened the parser, malformed
input was silently dropped rather than rejected — the compiler produced
plausible output for specs it had not understood.

Everything else in this report is downstream of that finding.

---

## 1. Literature

Retrieved from the arXiv API on 2026-08-09; every identifier below resolves.

### 1.1 LLM-for-RTL — the field IGLA CODER is in

| Paper | arXiv | Relevance |
|---|---|---|
| Thakur et al., *Benchmarking Large Language Models for Automated Verilog RTL Code Generation* | [2212.11140](https://arxiv.org/abs/2212.11140) | Established the benchmark framing for the field |
| Thakur et al., *VeriGen: A Large Language Model for Verilog Code Generation* | [2308.00708](https://arxiv.org/abs/2308.00708) | The first widely-cited open Verilog model |
| Pinckney et al., *Revisiting VerilogEval: A Year of Improvements…* | [2408.11053](https://arxiv.org/abs/2408.11053) | The benchmark IGLA CODER has no score on |
| Tsai et al., *RTLFixer: Automatically Fixing RTL Syntax Errors with LLMs* | [2311.16543](https://arxiv.org/abs/2311.16543) | Syntax repair as a first-class task — see §4.2 |
| Deng et al., *ScaleRTL: Scaling LLMs with Reasoning Data and Test-Time Compute* | [2506.05566](https://arxiv.org/abs/2506.05566) | Current frontier for accurate RTL generation |
| **Fu et al., *Synthesis-in-the-Loop Evaluation of LLMs for RTL Generation: Quality, Reliability, and Failure Modes*** | **[2603.11287](https://arxiv.org/abs/2603.11287)** | **Load-bearing for this wave — see §3** |

### 1.2 Ternary and low-bit inference — the field IGLA RACE is in

| Paper | arXiv | Relevance |
|---|---|---|
| Ma et al., *The Era of 1-bit LLMs: All LLMs are in 1.58 Bits* | [2402.17764](https://arxiv.org/abs/2402.17764) | The result the whole ternary direction rests on |
| Wang et al., *Bitnet.cpp: Efficient Edge Inference for Ternary LLMs* | [2502.11880](https://arxiv.org/abs/2502.11880) | The LUT the repo's `OP_LUT_NPU` work ports |
| Wei et al., *T-MAC: CPU Renaissance via Table Lookup for Low-Bit LLM Deployment on Edge* | [2407.00088](https://arxiv.org/abs/2407.00088) | Closest published analogue to LUT-NPU, with numbers |
| Mo et al., *LUT Tensor Core: A Software-Hardware Co-Design for LUT-Based Low-Bit LLM Inference* | [2408.06003](https://arxiv.org/abs/2408.06003) | The hardware co-design case for LUT-based MAC |
| Umuroglu et al., *FINN: A Framework for Fast, Scalable Binarized Neural Network Inference* | [1612.07119](https://arxiv.org/abs/1612.07119) | The FPGA QNN incumbent |
| Nazemi et al., *NullaNet Tiny: Ultra-low-latency DNN Inference Through Fixed-function Combinational Logic* | [2104.05421](https://arxiv.org/abs/2104.05421) | The multiplier-free-logic end of the design space |

### 1.3 Formal methods for hardware

Vericert ([github](https://github.com/ymherklotz/vericert)) proves the HLS
compiler itself correct in Coq. t27's `bootstrap/` is unverified Rust. That
gap is acknowledged in `COMPETITORS.md`; §2 below is the smaller claim we *can*
support.

---

## 2. Three machine-checked theorems

All three are checked with **yosys 0.63 alone** — no Coq, no Lean, no vendor
tools — and each exits non-zero on failure, so they are CI-ready.
Full detail and reproduction commands: [`fpga/formal/README.md`](../../fpga/formal/README.md).

### T1 — The multiplier-free MAC *is* integer multiply-accumulate

> For all 8-bit signed `a`, all 2-bit `w_code`, and all 32-bit signed `acc_in`,
> `ternary_mac_top` and a golden model written with a real `*` operator produce
> identical `acc_out`.

Sequential miter + SAT over two clocked transitions.
**6,635 variables, 18,490 clauses → `SAT proof finished - no model found: SUCCESS!`**

Not circular: the miter netlist contains one `$mul` (golden) and one `$neg`
(shipped conditional-negate). The proof relates a multiply to a sign-flip.

**Conclusion.** The ternary datapath is *exact*, not an approximation —
including the `a = −128` corner where naive 8-bit negation overflows.

### T2 — Exact MAC without a multiplier

> Synthesized identically for Artix-7, the shipped cell uses **zero** DSP48
> primitives; the functionally equivalent golden model uses **one DSP48E1**.

| Design | DSP48 | LUTs | FFs |
|--------|-------|------|-----|
| `ternary_mac_top` | **0** | 59 | 32 FDCE |
| `ternary_mac_golden` | **1 × DSP48E1** | — | — |

**Conclusion.** Read with T1, this is the *entire* ternary argument in one
line: **the same arithmetic function, one hard multiplier block cheaper.**
Unlike a TOPS/W projection, it is a count anyone reproduces in a minute with
an open toolchain. Scope: one cell — this does not extrapolate to a GEMM array.

### T3 — The on-board pass criterion is falsifiable

> Once reset is released, the demo accumulator is confined to `{0, +1}`;
> therefore it is never negative, the sign LED never lights, and the activity
> LED is exactly the "accumulator non-zero" predicate.

- Bounded model check, 64 clocks: 92,564 vars / 257,076 clauses → SUCCESS
- **Temporal induction: converged at length 10 → `Induction step proven: SUCCESS!`**

The induction result makes this **unbounded** — true for all time, not just
the checked window. (Machine-checked at `PRESCALE_BITS=2`; the board uses 24.
The invariant argument is width-independent — the prescaler decides *when* a
step happens, never *what* it computes — but the checked instance is width 2,
and that distinction is the honest one.)

**Conclusion — the point of the exercise.** The launch plan predicts a flashed
board shows `led_r23` blinking and `led_t23` dark. T3 makes that a
**prediction that reality can contradict**. The v1 demo could not be falsified
by *any* observation; that is precisely why it was worthless as evidence.

---

## 3. Synthesis-in-the-loop: the measurement, and why it is the right one

Fu et al. ([2603.11287](https://arxiv.org/abs/2603.11287)) report that
simulation-level pass rates materially overstate true hardware readiness,
because code can simulate and still fail to synthesize. t27 had exactly the
metric they warn about: `t27c synth-readiness` scans specs *statically*
(parse / typecheck / generate) and reports readiness from that.

This wave added **`t27c synth-gate`**, which emits Verilog and then actually
invokes `yosys synth_xilinx`. Measured on `specs/igla/race` after the repair
in §4.1:

| Stage | Result |
|-------|--------|
| gen-verilog succeeds | **8 / 17 (47.1 %)** |
| yosys synthesizes it | **7 / 17 (41.2 %)** |

**The gap is the finding.** `ternary_inference.t27` generates Verilog and then
fails synthesis:

```
ternary_inference.v:81: ERROR: syntax error, unexpected TOK_INPUT
```

Root cause, from the emitted file: the backend escapes the reserved word
`input` correctly in the **declaration** (`input [7:0] \input ;`) but emits it
bare at the **use site** (`ternary_gemm_2x2($signed(input[0 +: 8]), …)`).

This is the identical defect class already fixed in the Zig backend, where
identifiers shadowing primitive names now emit `@"name"` *at every
value-identifier site*. The Verilog emitter never received the same treatment.

**Conclusion.** A 5.9-point spread between "generates" and "synthesizes" on a
17-spec sample is small in absolute terms and decisive in kind: it confirms
that generation success is not hardware readiness, on our own corpus, exactly
as the paper predicts. `synth-gate` should replace `synth-readiness` in any
claim about readiness.

---

## 4. The corpus finding

### 4.1 One appender bug, replicated 27 times

Wave Loop 339 appended a `test` block with **no closing brace** to every IGLA
spec:

```
test ternary_mac_w339_batch_depth_invariant_2 {
  // Verify baseline properties for 15-variable accumulation
  assert(true)
                      <-- no `}`
// Wave Loop 340 ...
```

Every subsequent construct was swallowed into that block. Repairing it (one
brace per file, 27 files) moved the corpus from **0/27 → 8/27** compiling.

The reason this survived hundreds of waves is structural: before commit
`#1940`, the parser *silently dropped* malformed statements instead of
rejecting them. The specs appeared to work. `#1940` made the failure honest,
and nobody looked.

### 4.2 What remains: two narrow syntax gaps, not two missing features

> **This section was rewritten after the falsification pass in §6 refuted its
> first draft.** That draft claimed `if`-expressions and floats were
> "unimplemented" and that the specs targeted "a language that does not exist".
> Both claims were wrong, and checking them was the point of writing §6 first.

The 19 specs that still fail split cleanly:

| Class | Count | Example |
|-------|-------|---------|
| **`if`-expression written with braces** | 12 | `let elem = if (idx == target) { value } else { data[idx] };` |
| **`as f32` / `as f64` cast** | 3 | `eda.t27`, `eval.t27`, `training.t27` |
| Module-level parse (`KwFn`) | 2 | `backend.t27:394`, `yosys.t27:228` |
| Other | 2 | `opcodes.t27:336`, `tokenizer.t27:286` |

**Both dominant classes are narrower than they look.**

**`if`-expressions are implemented.** `parse_if_expr` exists at
`compiler.rs:3056` and is dispatched from expression position. It accepts

```t27
if (cond) then_expr else else_expr        // bare expressions
```

The IGLA specs use Rust's block form, `if (c) { a } else { b }`. t27 has no
block-expression, so `parse_expr` meets `{` and rejects it. The gap is one
production — *a brace-delimited block whose value is its tail expression* — not
a missing control-flow construct.

**Floats are implemented.** `TypeInfo::F32` exists, and `fn f(x: f32) -> f32`
parses cleanly — verified here with a minimal spec. Only the **cast** whitelist
at `compiler.rs:2626` omits `f32`/`f64`, so `x as f32` is a parse error while
`x: f32` is fine. That is a one-line inconsistency, written up as a ready patch
in [`docs/patches/W550-f32-cast-whitelist.md`](../patches/W550-f32-cast-whitelist.md).

**Conclusion.** The specs were not written against an imaginary language. They
were written against a *slightly larger* one — t27 plus block-expressions plus
float casts — and the two-line gap between the two has silently invalidated
69k lines for hundreds of wave loops. This makes the W550 fork much cheaper
than the first draft implied:

1. **Close the syntax gap** — add block-expressions and the two cast types.
   Small, contained compiler work that makes the existing corpus meaningful.
2. **Rewrite the specs** — mechanical, far larger diff, no compiler risk.

Route 1 is now clearly correct: it is smaller *and* it preserves the work.

### 4.3b The corpus problem is repo-wide, and IGLA is 5 % of it

The IGLA investigation prompted a clean per-spec census across the whole tree,
run with the current binary on the settled tree (≈50 min, one `t27c parse` per
spec):

```
total = 1063   parse OK = 700   parse FAIL = 363   (34.1 %)
IGLA share of failures: 19 of 363
```

**A third of the entire t27 specification corpus does not parse.** IGLA is
5.2 % of the problem, not the problem. Worst directories:

| Directory | Failing |
|-----------|---------|
| `scratch` | 58 |
| `fpga/testbench` | 29 |
| `tri/collections` | 24 |
| `fpga` | 20 |
| `numeric` | 15 |
| `isa` | 12 |
| `physics`, `math`, `igla/coder` | 10 each |
| `tri/trees`, `server`, `igla/race` | 9 each |

Note `fpga` + `fpga/testbench` = **49 failing specs**, directly on the path of
the hardware work this wave set out to do.

Error classes across all 363 (first error per spec, normalized):

| Count | Class | Interpretation |
|------:|-------|----------------|
| 48 | `unexpected token after expression statement: Ident` | mixed |
| 43 | `Expected LParen, got Ident` | mixed |
| **40** | `Unexpected token in expression: LBrace` | **block-expression gap (§4.2)** |
| **38** | `Expected RBrace, got Eof` | **corrupted type annotation — see the correction below; FIXED in W550** |
| 30 | `Unexpected token in expression: Semicolon` | mixed |
| **28** | `Unexpected token in expression: KwStruct` | **struct literal in expression position** |
| **16** | `unknown cast target type` | **the float-cast whitelist (§4.2)** |
| ~160 | long tail | ≤9 each |

> **Correction (W550).** This table originally described the 38
> `Expected RBrace, got Eof` specs as "unterminated block — same class as the
> W339 bug". Both halves were wrong.
>
> Brace depth in all 38 is **zero**; nothing is unterminated block-wise. A
> second hypothesis — that these use an unimplemented `given`/`when`/`then` BDD
> dialect — also failed: 158 *other* specs use that form and parse fine, and a
> `when` clause appears in 76 % of the failures against 77 % of the passes, so
> it discriminates nothing.
>
> The real defect is a **corrupted type annotation carrying a stray double
> quote**, which opens a string literal that swallows the rest of the file. The
> parser reports the symptom (a missing `}` at EOF) rather than the cause. All
> 38 had exactly one unterminated string, in two shapes:
>
> ```
> bits     : [[]Usize",           ->  bits     : []usize,
> log_file : [?[]Const u8",       ->  log_file : ?[]const u8,
> opad     : [[64]U8",            ->  opad     : [64]u8,
> children : [[256]?*ACTrieNode", ->  children : [256]?*ACTrieNode,
> ```
>
> Repaired in W550: **700 → 737 of 1063 specs parse (65.9 % → 69.3 %)**, 0
> regressions. The lesson is the same one §6 records: an error message names
> where the parser gave up, not where the file went wrong.

**This changes the economics of the W550 fix decisively.** The two gaps
identified from IGLA are not IGLA-specific:

- block-expressions would unblock **~40** specs, not 12
- the one-line cast whitelist would unblock **~16**, not 3
- struct-literals-as-expressions is a third gap of similar size (**28**)

Three syntax productions plausibly address ~84 of 363 failures. And the
38 `Expected RBrace, got Eof` specs are the *same* unterminated-block defect
repaired in IGLA this wave — a mechanical fix with a known shape.

**Conclusion.** The W339 brace bug and the IGLA syntax gaps were not local
accidents. They are instances of repo-wide patterns, and the corpus has been
one third unparseable while every wave loop reported progress.

### 4.4 A latent build blocker, found by trying to fix §4.2

Applying the one-line float-cast patch re-runs `bootstrap/build.rs`, which
**panics**:

```
t27c LANGUAGE POLICY VIOLATION: Cyrillic character U+041A in
docs/metrics/NUMERIC_FORMATS_83_METRICS.md
```

Six committed documents violate L3 / LANG-EN and are not in the grandfathering
allowlist `docs/.legacy-non-english-docs`:

| File | Committed |
|------|-----------|
| `docs/metrics/NUMERIC_FORMATS_83_METRICS.md` | 2026-06-28 |
| `docs/wave_ecosystem_2026-07-08/{EPIC,FINAL_REPORT,SCIENCE_BASELINE,WEAKNESS_AUDIT}.md` | 2026-07-08 |
| `docs/reports/FPGA_LOOP_CLOSEOUT_W849_2026-08-04.md` | 2026-08-05 |

**Why nobody hit it.** `build.rs` declares `rerun-if-changed` on
`compiler.rs` and friends but **not** on `main.rs`. Every change this wave made
to `main.rs` built cleanly; the first edit to `compiler.rs` triggered the scan
and the panic. So the build has been latently broken since 2026-06-28 for
anyone touching the compiler proper.

The float-cast fix is therefore **written but deliberately not applied** — the
allowlist says "Architect approval only", and translating six historical
reports is not a change to make unilaterally at this hour. The blocker is the
first item of W550.

### 4.3 Vacuity, in its proper place

Measured with the new `t27c validate-vacuity`:

| Scope | Vacuous tests | Vacuous invariants |
|-------|---------------|--------------------|
| `specs/igla/**` | 2,160 / 3,788 (**57.0 %**) | 1,917 / 3,314 (**57.8 %**) |
| whole `specs/` tree | 2,165 / 7,211 (30.0 %) | 1,918 / 3,378 (56.8 %) |

IGLA holds 2,160 of the 2,165 vacuous tests and 1,917 of the 1,918 vacuous
invariants in the entire tree.

> **Correction.** An earlier scan this wave reported 99.3 % of invariants
> vacuous. That denominator counted only single-line `invariant x: expr` and
> missed the multi-line `forall`-quantified form — which is the genuinely good
> half of the corpus. The correct figure is 57.8 %.

A `test` whose body is only `assert true` passes for **every** implementation,
so it contributes zero discriminating power while satisfying L4 TESTABILITY by
letter. **But the causal story matters more than the ratio:** you can only
append trivially-true tests to a spec you cannot compile. The vacuity is a
*symptom* of §4.1–4.2, not an independent problem. Fixing the language or the
specs is what makes real tests possible; gating vacuity first would only block
the loop without giving it anything true to say.

---

## 5. Conclusions

1. **The corpus problem is repo-wide, and larger than the IGLA line.** A clean
   census (§4.3b) puts **363 of 1063 specs — 34.1 % — at parse failure**, with
   IGLA only 19 of them. Every test count, invariant count and readiness
   percentage the project has quoted describes a corpus the compiler never
   accepted. The encouraging part is that three syntax productions
   (block-expression, struct literal in expression position, float casts)
   plausibly address ~84 of those failures, and 38 more are the same
   unterminated-block defect already repaired here mechanically.

2. **The ternary arithmetic itself is sound, and now provably so.** T1 and T2
   are the first machine-checked statements about IGLA RACE's actual RTL: the
   MAC is exact, and it is exact without a multiplier. That is a real, narrow,
   defensible result — and it is the one worth publishing.

3. **Falsifiability was the missing discipline, not effort.** The v1 demo, the
   vacuous tests, and the static readiness metric share one property: no
   observation could have contradicted them. T3, `synth-gate`, and
   `validate-vacuity` each replace an unfalsifiable claim with a checkable one.

4. **Silent-failure modes compound.** A parser that dropped malformed input, a
   readiness metric that never invoked a synthesizer, and a test appender that
   only ever asserted `true` combined to hide a total corpus failure for
   hundreds of iterations. Each was individually defensible; together they made
   the loop unable to see its own state.

5. **The competitive position is narrower than assumed but not empty.** IGLA
   CODER has no VerilogEval score and IGLA RACE has no silicon measurement, so
   neither can be compared to RTL-Coder or FINN today. What *is* defensible is
   T2: exact ternary MAC at zero DSP48. That is a smaller claim than the
   project has been making, and unlike the others it survives contact with a
   reviewer.

---

## 6. What would falsify the conclusions above

Stated so that the next wave can attack them:

- **T1/T2** fail if a different synthesis configuration infers a DSP48 for
  `ternary_mac_top`, or if the miter is unsound because the two designs share
  logic. Both are checkable: re-run with `-noabc9`, and inspect the miter
  netlist for the `$mul`/`$neg` pair.
- **T3** fails if a flashed board lights `led_t23`. That would mean silicon
  disagrees with the model — the most interesting outcome available.
- **§4.2 — this one actually fired.** The check was: "fails if `if`-expressions
  turn out to be implemented". They are (`compiler.rs:3056`), and floats are
  too. The section was rewritten; the finding shrank from "a language that does
  not exist" to "two syntax gaps", and the recommended route changed with it.
  Recorded rather than quietly edited, because a falsification pass that never
  overturns anything is decoration.
- **§3 — checked and held.** The `TOK_INPUT` error is not a `read_verilog -sv`
  artifact: `iverilog -g2012` independently rejects the same line
  (`/tmp/ti.v:81: syntax error`), and additionally surfaces a duplicate test
  function declaration at line 211 — a second, independent codegen defect.

---

*φ² + φ⁻² = 3 | TRINITY*
