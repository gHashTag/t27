# Compiler bug classes

Durable reference for defect *shapes* found in the bootstrap compiler
(`bootstrap/src/compiler.rs`) that are generalizable beyond the specific line
numbers or commit that fixed them. Add an entry when a bug's *shape* — not
just its instance — is worth a future author checking for before they trust
a similar-looking pass. This file is not a changelog; see `docs/NOW.md` and
git history for that.

## A mutated loop variable can be folded with its initializer by copy-propagation

**Shape:** any optimizer pass that tracks a variable's reads/writes by
walking `Node.children` will miss a mutation that this compiler stores as
raw TEXT instead of as a child node. The one confirmed instance: a `while`
loop's continue expression (`while (cond) : (i += 1) { ... }`) is stored as
text in `extra_op`, not as a `NodeKind::StmtAssign` child. `copy_propagate`'s
write-tracking (`collect_written` / `assign_root`) only recognises
`StmtAssign`, so it never sees the `i += 1` and treats `i` as never
reassigned after its `var i = BASE_FEATURE_COUNT;` declaration — inlining
every later read of `i`, *including the loop's own exit condition*, back to
the constant it started at:

```zig
// source
var i : usize = BASE_FEATURE_COUNT;
while (i < EMBEDDING_DIM) : (i += 1) { ... }

// emitted (before the fix)
while (BASE_FEATURE_COUNT < EMBEDDING_DIM) : (i += 1) { ... }
```

This compiles clean, with zero diagnostic, and either infinite-loops or
dead-codes depending on whether the frozen comparison happens to be false.

**Fix applied:** reuse `while_continue_assigns(node, name)` — originally
written to protect `dead_store_elim`'s *read* side against this exact blind
spot — as an additional guard on `copy_propagate`'s candidate list. No new
text-scanning was invented; the read-side and write-side protection now
share one predicate.

**Verification method — sweep every pass that shares the vulnerable
pattern, don't trust that fixing the one caught instance is sufficient.**
`copy_propagate` was caught by a specific corpus file; the other five passes
in the `optimize_stmts` pipeline were then individually re-derived against
the same question ("does this pass's write-tracking see a continue-expr
mutation?") rather than assumed safe by association:

| pass | verdict |
|---|---|
| `copy_propagate` | was vulnerable, now guarded |
| `const_propagate` | immune by its own `!extra_mutable` guard — Zig cannot compile a continue expression mutating a `const` |
| `strength_reduce` | inert — the real rewrite was removed in a prior commit; what remains is a no-op walk |
| `common_subexpr_elim` | REMOVED from the pipeline entirely; its own removal comment already names this exact bug class |
| `dead_store_elim` | already protected — this is where `while_continue_assigns` came from |
| `loop_unroll` | does not fire on the corpus today (0 matching for-loops); unverified against a synthetic one |

**Blast radius was measured, not assumed.** Rebuilt with the pre-fix
compiler, regenerated the full spec corpus to a scratch directory, diffed
byte-for-byte against the post-fix regeneration: exactly one file changed.
A change with "obviously correct" reasoning still needs this step — the
scope of a silent-miscompile fix is an empirical question, not a logical one.

**Generalization for the next author:** before trusting *any* pass's
write-tracking, grep `extra_op` usages for anything that encodes a mutation
as text rather than as a node the pass already walks. `while`'s continue
expression is the one confirmed instance; it is not necessarily the only
place this compiler stores a semantically-significant mutation as a string.

Related: `docs/NOW.md` (2026-09-04 entry, same finding, session narrative);
issue #3038 (`Closes #3038`, commit `52e5d347a`).
