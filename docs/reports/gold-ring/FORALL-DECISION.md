# The `forall` decision — one page for the Architect (W900)

After rungs 0001–0006 the corpus discards 37,786 tokens. **~18,750 of them —
now half of everything still lost — are `forall` quantifier bodies under
`invariant` headers, in 34 files**, and they are not a defect: the lowering
deliberately does not model quantifier bodies. Nothing below is urgent; this
page exists so the choice is made once, with numbers, instead of being
re-derived every audit.

## Where the mass sits

| file | ~tokens | share of file's drops |
|---|---|---|
| specs/igla/race/ternary_inference.t27 | 1,758 | 97 % |
| specs/igla/race/ternary_gemm.t27 | 1,503 | 75 % |
| specs/igla/race/systolic_ternary.t27 | 1,389 | 87 % |
| specs/igla/race/adder_tree.t27 | 1,028 | 89 % |
| specs/igla/race/systolic_array.t27 | 1,020 | 89 % |
| …29 more files | ~13,050 | — |

Concentration: the top eight are all `specs/igla/race/*` — the L4-mandated
invariants of the FPGA datapath specs.

## What a `forall` body actually holds

Measured shapes (readers over every span, W899): multi-binder heads
(`forall a : i8, w : TernaryWeight, psum : i16`), implication chains whose
`==>` lexes as two tokens (`== >`), local `let (s, c) = f(...)` bindings,
slice-typed binders whose brackets vanish (`forall a : []i8` → `forall a : i8`),
inline comma-form one-liners (`forall e : EDA, e.utilization >= 0.0`).

## The four options, priced

1. **Status quo** — bodies stay unlowered; certificates carry the mass in
   `discarded_top_level_tokens`. Cost: 0. Price: every seal on these 34 files
   certifies at most half the file; L4's own subject stays dark.
2. **Parse-only lowering** (recommended) — parse the body into the AST as a
   `forall` node (binders + predicate expression), execute nothing. The tokens
   become READ; seals become honest; no runtime semantics invented. Cost: one
   ladder rung (binder list + the `==>` lexer fix + predicate via `parse_expr`);
   the `==>`/`== >` lexer change is the only part that can touch non-forall
   code, so it rides behind the same probe-and-corpus certificate as every rung.
3. **Runtime property tests** — lower to sampled checks (draw binders, assert
   predicate). Invents semantics (sampling prior, counts, tolerances — the
   exact family of silent choices the TNF audit flagged upstream). Should not
   be bundled with parsing; possible later ON TOP of option 2.
4. **Reject and migrate** — force authors to rewrite 18,750 tokens of
   invariants in another form. The inventory says most BDD content did NOT
   need migration once the grammar was taught (23,033 → 1,589); the same
   argument applies here.

## The rehearsal (W908) — option 2 is one wave from shipped

A verbatim parse-only capture (58-line patch,
`FORALL-OPTION2-REHEARSAL.patch` in this directory) was built and measured on
the full corpus, then REVERTED pending the decision:

    0014 baseline   25,670 discarded tokens, 62 discarding files
    with option 2    6,592 discarded tokens (-90.3% from the original
                     67,760), 40 files, consume-all 389 -> 411,
                     parse-fails unchanged (173, zero new)

The rung ships in one wave on a "2" from the Architect.

## The ask

One word on option 2 (or a number 1–4) on t27#2217. Everything else in the
residue is either an expression-grammar rung already queued (bare-call given,
`measure/target` clause pair) or dialect content (`specs/ar` Rust bodies,
~1,900 tokens) with its own separate question.

---

*φ² + φ⁻² = 3 | TRINITY*
