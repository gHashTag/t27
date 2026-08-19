# The residual map under 0007 — every suspect convicted by intervention (W901)

> **STALE AS OF W905.** This map predates rungs 0008–0012; the corpus residue
> it prices (34,175 tokens) is now 26,713 and several of its queue items are
> SHIPPED (0008 const poisoning, 0009 colon forms, 0010 invariant semicolons,
> 0011 in-membership, 0012 statement clauses). Kept for the conviction record;
> the current map is the W905 remeasure in the ladder README and theorems.

34,175 tokens still discarded after rung 0007 (125 files). This map differs
from its W899 predecessor in one decisive way: after lesson 1375 ("presence is
not causality"), every reader carried an INTERVENTION DUTY — copy the block to
scratch, delete the suspect, re-measure. A suspect below marked *causal* means
the drops vanished when it was removed and returned when it came back. Eight
readers, 368 tool calls, reconciliation 34,197 = 34,197 pre-checked (v1 dumps;
v2 differs by −22 tokens).

## The three masses

| cause | tokens | share | status |
|---|---|---|---|
| `forall` bodies | ~19,000 | ~56 % | deliberate — awaiting the FORALL-DECISION word |
| BDD block fallback | ~9,000 | ~26 % | causally mapped below — the 0008+ queue |
| imperative/dialect bodies | ~6,000 | ~18 % | see the coa_planning finding — partly NOT what it seemed |

## Convictions (each verified by removal)

1. **Semicolon-less top-level `const` poisons everything after it** — the
   headline find. `specs/ar/coa_planning.t27` (2,438 tokens, the corpus's
   largest discarder) was bucketed "Rust-form bodies" by two generations of
   readers. Intervention: add `;` to TWO const lines → all 2,438 discards
   vanish (replaced by an honest hard error at a later `for … in` — a separate
   construct). Minimal pair confirmed: two `const X: int = 10` lines without
   semicolons flip the parser into discard-fallback for every subsequent brace
   body; one such line, or semicolons present, is clean. The bodies were
   innocent the whole time. **0008 candidate, top yield.**
2. **`measure:` / `target:` colon-form clauses are ungrammatical as a FORM** —
   drops persist with trivial expressions (`measure: 5`), so the free prose
   and `ns` suffixes are not the cause; the clause form is. (The keyword form
   is fixed by 0007.)
3. **`bench name: expr` one-line headers drop** while the identical
   `invariant name: expr` parses — the bench-colon form is unsupported per se.
4. **`invariant name : EXPR;` — the trailing SEMICOLON is causal**; the colon
   form, the spacing, and `||` are all exonerated by variants.
5. **Bare assignment statements at block top** (`uart_state.rx_data = 0x99;`)
   fall back the whole block, taking valid sibling clauses with them.
6. **`forall` in clause position** (`and all_match = forall …`,
   `assert forall …`) poisons its block even with a trivial predicate.
7. **and-tuple destructure** (`and (a, b) = f(x)`) — convicted on the v1
   binary, **already cured by v2's paren rescue** (re-verified green).

## Exonerations

Struct-literal returns, `.push()`, `format()`, `if/else`, `let` inside fn
bodies (all parse clean in isolation — the const poisoning made them LOOK
broken); given-clause tuple destructuring; `then`-clause `or` operators;
`value in [-1, 0, 1]` list membership (drops only under the bench-colon form).

## The 0008+ queue, priced by verified yield

| rung | cause | est. tokens |
|---|---|---|
| 0008 | semicolon-less const state poisoning | ~3,600 (coa_planning + restraint share the shape) |
| 0009 | `measure:`/`target:` colon forms + bench-colon one-liners | ~700 across a dozen fpga specs |
| 0010 | trailing `;` on one-line invariants | ~100, trivial |
| — | block-top assignment statements | ~200, needs a statement-clause decision |
| — | `forall` (56 %) | the ring question — FORALL-DECISION.md |

---

*φ² + φ⁻² = 3 | TRINITY*
