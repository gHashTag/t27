# WIP patch — lower keyword-form invariants

**Status:** implemented, verified, **reverted** on 1 regression. Diff in
`W567-invariant-lowering-WIP.diff`.

## Why it matters

`parse_invariant_block` discards the body via `skip_to_next_top_level()`, so
codegen emits `// invariant: X verified (no statements)` — a comment claiming
verification. **5,163 invariants** are in this form.

## What worked

Two spellings exist, and both lower:

```
invariant name: <expr>          ->  comptime { if (!(expr)) @panic(...) }

invariant name                  ->  same, via the shared clause parser
    assert BOARD_NAME != ""         (this is the COMMON form: 76 of 81 in the
                                     currently-compiling specs)
```

`forall`-quantified invariants (837) are correctly skipped — they are not
runtime-checkable.

Measured on `specs/boards/arty_a7.t27`: assertions went **16 → 23**, with only
the 4 `forall` ones left inert.

Note these lower into `comptime` blocks, so a **false** invariant becomes a
compile error rather than a test failure. That is arguably the correct
semantics for an invariant, and no false invariant was found.

## Why it was reverted

One spec regressed: `specs/ml/optimizer/race_config.t27`, ALL_PASS → COMPILE_FAIL.

```
error: use of undeclared identifier 'abs'
    const diff = abs(RACE_BETA1_PHI_CANONICAL - RACE_BETA1_PHI_DAMPED);
```

**Not a false invariant — a missing builtin mapping.** The invariant body calls
`abs()`, which Zig spells `@abs`. Corpus-wide the bare builtins are:

| call | occurrences |
|---|---:|
| `abs(` | 425 |
| `sqrt(` | 111 |
| `floor(` | 99 |
| `round(` | 92 |
| `max(` | 62 |
| `min(` | 50 |

The lowering contract for these changes is *may only ADD assertions, never
break a file*. One regression violates it, so it is reverted — the same
discipline as W558.

## Prerequisite for the next attempt

Map bare math builtins to Zig builtins (`abs` → `@abs`, …) **guarded by a set
of functions the spec declares itself**, so a user-written `fn max(...)` still
wins. The generator has no such set today (`mut_names` and
`exposed_output_vars` are the only name sets), so one must be threaded through
`gen_zig`. Mapping unconditionally would silently shadow user functions — the
exact class of defect this chain has been removing.

Order: land the guarded builtin mapping first, confirm `race_config.t27` still
passes, then re-apply this diff and re-run both the harness and the full census.

*phi^2 + phi^-2 = 3 | TRINITY*
