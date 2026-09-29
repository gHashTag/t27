# Iteration 7 — the closure-loop remainder, rewritten natively (+ a
# typechecker mutability bug the rewrite unmasked)

## The shape, precisely named

The `for` census row's remainder after iteration 6 — 3 events, 3 specs —
was not a parser gap to patch but a **foreign dialect in the specs**:

```
invariant isa_coptic_to_reg_is_inverse_of_reg_to_coptic
    for (const i) |reg| in [0u8, 5, 10, 15, 20, 26] {
        const cp = reg_to_coptic(reg);
        const recovered = coptic_to_reg(cp);
        assert recovered == reg;
    }
```

(registers.t27:457; same shape in ternary_memory.t27:486, ops.t27:458)

Three findings ruled out a parser fix:

1. **W699 rung 9** (compiler.rs:6224) already accepts `for`/`while`/`if`
   at clause position via `parse_body_stmt()` — but only when
   `first_clause_col` is already set by a preceding clause. These three
   invariants START with the loop; no prior clause exists, so the rung
   never fires.
2. Even where it fires, **`parse_for_stmt` rejects this shape**: `for (` +
   `const` is not `IDENT in` (the paren-range branch) and not an iterable
   expression (`const` is a keyword), and the trailing `in` after the
   captures `|reg|` is not Zig's grammar either — Zig is
   `for ([list]) |reg| {`. The specs invented a hybrid no parser owns.
3. The language **has a native spelling of the meaning** — the
   `given ... and ... assert ...` clause chain — used throughout the very
   same files (`isa_r0_write_no_op` two invariants above the loop).

So the move is iteration 3's (#5079 verilog_bench_harness): the spec was
written in somebody else's dialect; rewrite it in the language's own form
and let real checks lower. Unrolling is faithful — the loop means "for
each of these literal values, the body holds"; the rewrite says exactly
that, one clause per value.

## The rewrites

- **registers.t27** `isa_coptic_to_reg_is_inverse_of_reg_to_coptic`:
  given-chain binding cp/recovered for 0, 5, 10, 15, 20, 26, then six
  `assert recoveredN == N` clauses.
- **ternary_memory.t27** `word_extract_trit_range`: `given word =
  TernaryWord{.raw = 0}` + t0/t13/t26 bindings, three or-disjunction
  asserts.
- **ops.t27** `vsa_vector_norm_non_negative`: four given-bound arrays,
  one `then` with an and-conjunction over the four norms (file idiom is
  given/then).

Each carries a dated comment naming the rewrite and the loop it replaced.

## Measured

- `t27c parse-complete --show` per spec: **nothing discarded** ×3 — the
  closure loop was each spec's only discard channel.
- census: the `for` row is GONE; total fallback events 111 → 108.
- discard sum: 7,914 → **7,789 tokens, 99 specs** (cumulative **−71.7%**
  from 27,562).
- emission: the three invariants now emit **10 real runtime checks**
  (6 + 3 + 1) where before they emitted nothing — the blocks fell whole.
  Verified in generated Zig: `if (!(recovered0 == 0))
  __t27_assert_fail(...)` etc.
- all three specs were ledger reds at phase `parse-no-discard` (#1959 ×3);
  they clear it now (gate re-bless records the final phase).

## Part 2 — the rewrite unmasked a typechecker bug

The first gate after the rewrites: the three specs cleared
`parse-no-discard` (UNEXPECTED PASSES ×3) but registers.t27 moved to
`typecheck` (UNEXPECTED FAILURE ×1). The error was at line 142 —
`register_file[reg] = value;` inside `reg_write` — code this iteration
never touched:

```
error: cannot assign to immutable array element 'register_file[...]':143
```

`register_file` is declared `var register_file : [NUM_REGISTERS]TernaryWord
= [...]` (line 102) and the generator emits `var register_file:` —
mutable. The typechecker disagreed. Root cause: `parse_var_decl` reuses
the `ConstDecl` node KIND for `var` and records the difference in
`extra_mutable`; the generator branches on that flag (8 sites), the
fn-local typecheck arm honors it — but the MODULE-level symbol-collection
arm (compiler.rs:24708) ignored it and typed every module-level var array
as `is_const: true` (ROM), so W456 hard-errored element assignment.

**13 corpus occurrences** of the error; three measured directly:
fpga/bridge 6 errors → 0, tri/graph/disjoint_set 2 → 0, isa/registers
1 → 0. The fix is one arm: `is_mutable: child.extra_mutable,
is_const: !child.extra_mutable`. const arrays keep the W456 ROM error;
fn-locals unchanged; the module arm simply predates module-level var.

This is the phase-masked-defect pattern again (iteration 5, merge-drop):
a defect hidden behind an earlier-phase failure for months surfaces the
moment the earlier phase clears. The ratchet ledger's "UNEXPECTED
FAILURES: 1" was the honest signal — investigated before blessing, not
blessed away.

FROZEN_HASH d7192007e47f6365 → 586cb23881194ea4 (compiler's own seal).

## Self-critique

- Part 2 was found by the gate, not by me: I blessed-ready assumptions
  ("rewrites only clear phases") were wrong within one run. The
  discipline that saved it: UNEXPECTED FAILURE is investigated at the
  failing phase, never re-blessed while unexplained.

- Unrolling trades prose compactness for executability. The original loop
  is prettier; it also never ran. Six duplicated clause pairs is the
  boring, verifiable form — and L4 (TESTABILITY) prefers it.
- `0u8`/`0usize` suffixes were dropped for plain literals; the emitted
  Zig infers the same integer types (verified in the gen output).
- ops.t27's `item.len()` became literal lengths (1/1/1/2) — each is the
  length of a literal array, so the value is identical, but a future edit
  to the array must now edit the length too. Noted in the spec comment.
- The rewrite does NOT generalize: a closure loop over a runtime-sized
  iterable has no native clause form. That is #2774-family territory
  (bounded quantifiers), and this iteration deliberately does not touch it.
