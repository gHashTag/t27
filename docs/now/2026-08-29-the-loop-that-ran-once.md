# NOW -- The loop that ran once (2026-08-29)

## The comment described a loop the code did not emit (Refs #2834)

- `gen_c_for_stmt` said "C doesn't have for-each natively; emit as a for loop with index" and emitted `{ body }` -- no induction variable, no bound, no increment
- `for (0..1000) |_| { ... }` ran its body ONCE, in C that cc accepts with no diagnostic
- 374 bare blocks in the corpus; 309 real loops after, 80 left for non-range iterables
- the range is an `ExprBinary` with `extra_op == ".."`, not the `ExprRange` variant -- that variant is declared in NodeKind and CONSTRUCTED NOWHERE, so my first condition matched nothing and the old path kept running
- acceptance is unchanged, 166 -> 166, and that is the point: the bare block was always valid C, it just meant something else
- the first acceptance reading said 163 -> 166 and was measured against a binary predating two merged changes; a control built from the same tree says 166 -> 166
- the remaining 80 keep a comment that now says what happens: "body emitted ONCE, not looped"
