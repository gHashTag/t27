# The W578 brace-if question — refreshed under rung 0014 (W908)

The oldest open expression-grammar item: braced `if` used as a VALUE —
`let x = if (c) { 1 } else { 0 };`. W549 measured it at 29 specs / 4,465
assertion clauses; since then `parse_branch_value` (W578) taught the parser
the single-expression case, and the ladder's statement clauses read the
surrounding lines, so today only the BRACE INNARDS of complex forms drop
(chained `if {} else {} + if {} else {}`, multi-statement branches).

## Current price

Under rung 0014 the whole corpus discards 25,670 tokens. Of that,
~19,078 are forall bodies (measured by the option-2 rehearsal) and ~3,700 are
specs/ar-style dialect statement bodies (W905 map). **The brace-if family and
all other expression leftovers are bounded by the remaining ≈2,900 tokens**,
spread thin (jones_topology_filter keeps 16; most carriers keep <30).

## The options

1. **Status quo** — per-line drops, honestly counted; blocks READ except the
   brace groups themselves. Cost 0.
2. **Teach chained/multi-expr brace-if** — extend parse_branch_value to
   sequences and operators over branches. A real expression-grammar rung with
   the usual probe/panel/corpus cycle; yield ≤2,900 tokens corpus-wide.
3. **Migrate the spelling** — the corpus already prefers Zig-style
   `if (c) a else b` in new files; a mechanical rewrite is possible but
   touches test text (the thing the ladder existed to protect).

Recommendation: option 1 until forall and the dialect are decided — the
remaining yield is the smallest of the three open masses. (If option 2 is ever
taken, its probes exist: `break0/p9`, jones line 252.)

---

*φ² + φ⁻² = 3 | TRINITY*
