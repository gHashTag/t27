# Decision register — RE-AUDITED W621

**Date:** 2026-08-11 · **Waves:** W568–W621 · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Read this first

This file was created in W612 by collecting sixteen wave reports that had each
ended with the phrase *"this is a specification decision"*. It was presented,
repeatedly and by me, as the highest-value artefact in the project — *"what sits
at the top of the pile is a small number of sentences from someone who owns the
spec."*

**W621 re-measured every entry independently. That framing was wrong.**

| Verdict | n |
|---|---:|
| **DISSOLVED** — never a decision | **12** |
| **NUMBERS_WRONG** — still a decision, every count wrong | **2** |
| **ALREADY_FIXED** — a later wave shipped it | **1** |
| **SURVIVES as recorded** | **0** |
| stalled mid-audit (entry 3) | 1 |

**Not one entry survived as written.** Entry 1 had already been dissolved in
W620 by the same procedure.

### How the errors happened

They were not random. Four distinct mechanisms, each reproducible:

1. **A number copied from the wrong column.** Entry 2 recorded "30 test points,
   24 consistent with `len`". There are **54** points; the "24" is the count of
   *invariant blocks*, which the tally had excluded and then reported as a
   consistency figure.
2. **A row with no evidence behind it.** Entry 2's table claims length 1 expects
   `{0, 1}`. **No assertion in the corpus pairs a non-empty input with an
   expected 0.** All four length-1 points expect 1.
3. **A premise that misread the code.** Entry 5 says `is_sacred_opcode` tests
   membership in eleven named opcodes. It is a **byte-range predicate**.
   Entry 6 describes a field mismatch in `PpaMetrics`, which has **zero
   declarations in the repository** and therefore no declared fields to mismatch.
4. **A dilemma whose second branch is empty.** Entries 7, 8, 10 and 17 each pose
   "either X or Y" where Y is refuted by the file's own contents — and entry 17's
   own source report (W617) says explicitly that it *"would not go to the
   decision register."* It was added anyway.

### The lesson, stated plainly

**A number written once and quoted thereafter becomes true by repetition.**
These counts were carried through dozens of wave reports and issue comments.
Re-measurement cost one wave and invalidated 15 of 16 entries.

---

## Entries, with W621 verdicts


### 2. bram_weights_depth — contradictory tests

**STILL A DECISION, numbers wrong**

The entry survives as a decision; every count in it is wrong except "neither = 0". RECORDED -> MEASURED Test points: 30 -> 54 (30 `test` blocks + 24 `invariant` blocks; the invariants are equally binding and 4 of them are emitted as comptime assertions in the generated Zig) Consistent with depth == len: 24 -> 51 Consistent with depth == len/2: 6 -> 8 Consistent with neither: 0 -> 0 (correct) Consistent with BOTH: not shown -> 5 (the five empty-input points) — so the two categories OVERLAP and do not partition the total Genuinely dissenting points: "6" -> 3 (w294 L1072, w295 L1086, w296 L1100) Corrected register table: | Assertion points | 54 | | Consistent with depth == len | 51 | | Consistent with depth == len/2 | 8 | | Satisfying both (empty input) | 5 | | Consistent with neither | 0 | | Genuinely dissenting | 3 | Contradictory lengths — TWO, not three: | 2 | {1, 2} | | 4 | {2, 4} | Le


### 3. throughput — tests do not describe a throughput

**Verdict: NOT RE-MEASURED** — the audit agent stalled mid-stream. Treat as unverified.


### 4. systolic_ternary_array — output length

**DISSOLVED — not a decision**

Entry 4 is not a decision: the invariant it cites is a GUARDED implication, and the test it cites falls outside the guard, so the two artefacts never make contact. L234 does not say "output length equals size". It says `activations.len() == size && weights.len() == size ==> out.len() == size`. The cited test (L195) supplies `weights = []TernaryWeight{}` with `size = 2`, so `weights.len() == size` is FALSE, the antecedent is FALSE, and the implication is vacuously true there. An implication invariant cannot be contradicted by inputs its antecedent excludes. W571's recording — carried verbatim into the register — quotes only the consequent; the `&& weights.len() == size` conjunct is dropped in the note (`.trinity/experience.md:20702`), and both the invariant and the test were committed together in a0828089db on 2026-07-04, so nothing changed underneath it. Once the guard is restored, one r


### 5. OP_ADD / OP_SUB vs the sacred opcode set

**DISSOLVED — not a decision**

Entry 5 is not a decision. Both branches of its question were already settled by artefacts that existed before the register was written, and the entry's premise is a misreading of the code. 1. The premise is wrong about what `is_sacred_opcode` is. It is not membership in a set of eleven names — it is the pure byte-range predicate `op >= 0xDE && op <= 0xE8` (specs/igla/race/opcodes.t27:51-53). Nothing about names enters it. So "neither is among them" cannot even be evaluated for OP_ADD/OP_SUB, because they are *undeclared identifiers* with no byte value at all: `zig test` on the compiler's own output reports `use of undeclared identifier 'OP_ADD'`. They are 3 of 10 undeclared opcode names in that file (OP_ADD, OP_SUB, OP_MUL, OP_NOP, OPCODE_NOP, OP_IDENTITY, OP_SACRED_BEGIN, OP_SACRED_END, OP_STORE_RESULTS, OP_HOLO_BEGIN) plus 4 undeclared functions. Singling out two of them as a decision


### 6. PpaMetrics — field mismatch

**DISSOLVED — not a decision**

Entry 6 as written is not a decision, and three separate measurements each remove one leg of it. (a) The recorded sentence describes a phenomenon that does not occur. `PpaMetrics` has ZERO declarations in the repo, so it has no "declared fields" to mismatch, and zero constructors — all 5 sites are `given` lines. The compiler reports `use of undeclared identifier 'PpaMetrics'` x3, never a field error. The one `no field named` error in the entire 76-error file is on a SynthesisMetrics literal (power_mw, eda.t27:1025) — a different name, and one the entry does not mention. (b) The "two field sets" the question asks us to choose between are not two candidates for one struct. Only one of them is ever compiled. The 3-field shape {area_um2, delay_ns, power_mw} lives entirely inside `bench ppa_delta_compute_latency`, and bootstrap/src/compiler.rs:5766-5768 emits bench bodies as `// TODO: impleme


### 7. The five false CORDIC assertions

**DISSOLVED — not a decision**

Entry 7's Question - "correct the assertions, or state a different intent?" - is a binary whose second branch is provably empty, so nothing is left to decide. (1) No implementation can make the five true. K(n) = prod_{i<n} 1/sqrt(1+4^-i) has every factor strictly < 1, so K decreases monotonically (measured: K(8)=0.60725910, K(12)=K(16)=0.60725296) - no restatement of intent makes K(12)>K(8). cordic_gain seeds cordic_sin_cos, so changing it breaks the 330 passing tests. The arctan table is atan(2^-i) by construction and ends at 15; a 17th entry is blocked by the parallel table's own passing end-marker cordic_pow2_neg_entry_boundary_16 (pow2_neg_entry(16) == 0.0). (2) The register already records the fact that answers it. W598, which entry 7 transcribes, states the rule - "changing a test to match it is only legitimate when the implementation is independently proved" - and then certifies "


### 8. gemm.t27 — widen the product matrix or narrow the multiply

**DISSOLVED — not a decision**

It is not a decision, for two separate reasons, and the entry's own framing collapses under both. 1. The dilemma has only one live horn. "Narrow the multiply" is refuted by gemm.t27's own text: :383 requires booth_mul_i16(32767,32767) == 1073676289 (31 bits) and :395 requires booth_mul_i16(-32768,2) < -1000000. And the product matrix is ALREADY declared wider than i16 by the spec itself -- invariant gemm_output_bounded at :273 states C.a11 in [-131072, 131072], i.e. +/-2^17, four bits past i16. Every statement in the file that mentions a value outside i16 range (exactly 3, measured) points the same way. Widening is not chosen, it is entailed. 2. Widening is a T10-shaped migration, not a trade. Mat2x2 i16 -> i32 keeps all 85 existing Mat2x2 literals (extremes -32768..22) and every `C.aij == k` assertion valid, and the call target the widened fields need -- booth_mul_i32 -- is already decl


### 9. The 25 stub specs

**STALE — already shipped**

Not dissolved — it is stale. W602 shipped the reclassification the entry is still asking for (commit 3d79e65a7, "the stub population"); `bootstrap/src/test_report.rs` carries `pub stub: bool` computed as `!declares(ast)`, and `t27c test-report --all` now prints STUBS as a population separate from "specs with NO TESTS <- L4 TESTABILITY", which today stands at 4. The three offered dissolution shapes were tested and all fail: T12 does not apply (all 25 module names are unique corpus-wide; the only 3 leaf-filename collisions are unrelated subjects), and T10/T11 have no analogue here since there is no field or argument-order question.


### 10. The 15 Markdown files named *.t27

**DISSOLVED — not a decision**

The entry poses "rename, or exclude from the parse census?" as a trade-off a maintainer must settle. It is not a trade-off, for three independently sufficient reasons. (1) THE TWO OPTIONS ARE NOT ALTERNATIVES — ONE IS THE OTHER. Every census site in the compiler (~20 in bootstrap/src) filters on `extension == "t27"` and nothing else; `grep -rniE "t27ignore|exclude|skiplist|ignore_list|EXCLUDED_SPECS" bootstrap/src/main.rs` returns nothing. So `git mv x.t27 x.md` removes a file from all ~20 censuses at once — renaming IS excluding, achieved by the mechanism that already exists. "Exclude" as a separate option means writing an exclusion mechanism into 20 call sites that currently have none. There is nothing "exclude" preserves that "rename" loses; rename strictly dominates and is free. A choice where one option implements the other for zero cost is not a choice. (2) THE ONLY STATED COST IS 


### 11. backend.t27 ↔ eval.t27 import cycle

**DISSOLVED — not a decision**

It is not a decision, for three independent reasons, any one of which is sufficient. (1) THE CYCLE IS ALREADY CLOSED. Tarjan SCC over all 1063 spec modules puts {igla::coder::eval, igla::race::backend, igla::race::rtl} in ONE strongly connected component today, without the proposed edge: backend.t27:12 imports rtl and rtl.t27:11 imports backend (a direct mutual pair), while rtl.t27:18 imports eval and eval.t27:18 imports backend. Adding backend -> eval adds an edge inside an SCC that already contains all three nodes. The entry's premise -- that the import "would close a cycle" -- is false; nothing is closed that was open. Both in-file comments asserting non-circularity (W608 in eval.t27, W614 in rtl.t27) are factually wrong about the very graph they cite. (2) THE COMPILER HAS NO CYCLIC-IMPORT FAILURE MODE. bootstrap/src/use_resolve.rs::resolve computes the transitive closure of imports g


### 12. tokenizer decode — contradictory tests

**DISSOLVED — not a decision**

This is not a decision. It is a variant spelling plus two false tests — the T12 shape. 1. `decode` is undeclared (zig: 22x "use of undeclared identifier 'decode'"). The similar DECLARED name is `detokenize(tokens: []u32) -> string` at L225, whose own docstring says it is both decoders under a discriminated input: ASCII for id < 256, `decode_keyword` for id >= 256. That is verbatim the option the register poses as the open question. The question was already answered in the same file, 564 lines earlier, before it was ever asked. 2. The T12 co-occurrence test passes cleanly: of 64 blocks that use a decoder, 46 use bare `decode` and 18 use `detokenize*`, and ZERO use both. The regions do not merely fail to co-occur inside a block — they are strictly disjoint in the file (detokenize: L385-L736; decode: L789-L1291). That is the signature of a later wave block adopting a different spelling, not


### 13. tokenizer encode — underdetermined

**STILL A DECISION, numbers wrong**

Not dissolved — but T12 gets closer than the entry admits, and I want to record exactly where it stalls, because one maintainer sentence on `decode` (entry 12) would finish it mechanically. T11 is inapplicable (`encode` is unary). T10 is inapplicable (no struct is involved). T12's PRECONDITION HOLDS, measured over all 406 test/invariant blocks: `encode` co-occurs with `tokenize` 0 times, `tokenize_verilog` 0, `tokenize_prompt_hybrid` 0, `detokenize` 0, `encode_keyword` 0, `encode_char` 0; `decode` co-occurs with `detokenize` 0 times. So `encode`/`decode` are variant spellings, not new functions. That already kills the register's third candidate: the "degenerate length-encoder" is not declared anywhere in the module, so under T12 it is not a candidate at all. The real candidate set is the three declared `(string) -> []u32` functions, and `encode("a").len() == 1` removes `tokenize_verilog`


### 14. sgd_update — scalar call sites vs a vector declaration

**DISSOLVED — not a decision**

The entry poses an exclusive choice — "is sgd_update the vector update (and 82 tests are wrong), or a scalar update (and the declaration plus 10 tests are wrong)?" — but the exclusivity is manufactured. This is the T10 shape: widening the declaration keeps every existing call site valid, so it is a migration, not a deletion choice. Nothing is in dispute except the container. All 104 sites are arity-3 in the declaration's own order (weights, grads, lr) — no order or arity disagreement exists, so there is nothing to arbitrate there either. And both regions assert the identical rule w <- w - lr*g: the scalar literals (1.0-0.1*(-5.0)=1.5, 8.0-0.25*2.0=7.5, 10.0-0.1*0.1=9.99) are exactly the arithmetic the vector test performs elementwise ([1,2,3] with [0.5,1,0] at lr 0.1 -> [0.95,1.9,3.0]). The scalar region is the rank-0 instance of the same rule the vector region states at rank-1; a length


### 15. bits_to_u64 — bool literals vs []u1

**DISSOLVED — not a decision**

There is nothing for a maintainer to settle: the `[]bool` branch is not a live option, and the bool literals do not mean anything different from the numeric ones. (a) The type is pinned by the function's own body, exactly as in entry 17. `bits_to_u64_inner` (rtl.t27:120-125) ends `return bits_to_u64_inner(bits, idx + 1, (acc << 1) + bits[idx])`, where `acc : u64`. The element is consumed as an addend of a u64, so it must be numeric. Declaring `bits: []bool` yields `error: incompatible types: 'u64' and 'bool'` pointing at that very line — the declaration would contradict itself two lines below, before any test is read. The same body with `[]u1` is also frozen in the checked-in companion specs/igla/race/rtl.zig:78-86. So the entry's question "are the bit vectors []u1 or []bool?" has exactly one satisfiable answer, and the file already gives it. (b) The bool spellings are a variant spelling


### 16. DataSample — declaration vs tests

**DISSOLVED — not a decision**

Entry 16 poses two questions and neither is a decision. Q1, "which field set is canonical for dataset.t27's DataSample": T10 already showed there is no choice to make -- widening the declaration with defaults keeps every existing literal valid, so both artefacts survive and nothing is discarded. That is not a preference between two field sets; it is the Protobuf/Avro compatibility rule applied mechanically. It was applied in W619 and the measurement confirms it landed: dataset.t27 now has **0** "no field named" errors (95 remaining errors, all of them undeclared-function and array-lowering classes; only 6 even mention DataSample and all 6 are `[]T` vs anonymous-tuple lowering). The register was never updated -- it still quotes a 3-field declaration that has not existed since commit 47f122f7f. Q2, "should the two same-named types in dataset.t27 and training.t27 be distinguished by name?":


### 17. TernaryWeight::plus/minus/zero

**DISSOLVED — not a decision**

It is not a decision, and the entry's own text half-admits it ("Unusually, this one needs no design decision"); its source wave report W617 says outright it "would not go to the decision register." The entry poses a FALSE DILEMMA — "support struct methods (a compiler change), or make these free functions and rewrite the 40 call sites?" Both horns are unnecessary, because a third spelling already exists, is already declared, already lowers, and already dominates the very same files. T12 applies. `TernaryWeight::plus/minus/zero` are undeclared names — no `plus`, `minus`, or `zero` is declared for TernaryWeight anywhere in specs/, as member or free function. They never CO-OCCUR with a declared counterpart. They are a variant spelling of `TernaryWeight { code: 1 | 2 | 0 }`, which appears 1576 times in the same three specs against 135 `::` occurrences. The mapping is not a judgement call: it 


---

## What remains

After the audit, the genuinely open items are:

- **entry 2** — `bram_weights_depth`: a real contradiction, but **51 points for
  `depth == len` against 3 dissenters** (not 24 vs 6), and **two** contradictory
  lengths (2 and 4), not three. The corrected split is 94% for identity.
- **entry 13** — `encode`: still underdetermined; the recorded counts are wrong
  (40 call sites, not 23).
- **entry 3** — `throughput`: **not re-measured**; the audit stalled.

Everything else is either mechanical, already fixed, or was never a question.

---

*φ² + φ⁻² = 3 | TRINITY*
