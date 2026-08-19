# Wave Loop 567 Report — the last inert population, landed

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_566_REPORT.md`](WAVE_LOOP_566_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W566 implemented the invariant lowering, verified it, reverted it on one
regression, and **wrote down the prerequisite**. W567 did the prerequisite
first, then re-applied the lowering. Both gates pass.

```
harness : ALL_PASS 16, COMPILE_FAIL 183, tests passing 209   -- 0 regressions
census  : PARSE OK=746  FAIL=317  (baseline 317)             -- 0 regressions
```

**65 invariants now emit a real compile-time check** in the 16 compiling specs,
where every one of them previously emitted
`// invariant: X verified (no statements)`.

---

## 1. The prerequisite

Specs call `abs(`, `sqrt(`, `floor(`, `round(`, `min(`, `max(` bare — **839
occurrences** — and Zig spells them as builtins, so the emitted name was
undeclared. That is exactly what regressed `race_config.t27` in W566.

Codegen now collects the spec's **own** function names into `declared_fns` at
the top of `gen_zig` and maps a bare call to `@abs`/`@sqrt`/… **only when the
name is absent from that set**. Verified in both directions:

```t27
fn f(x: i32) -> i32 { return abs(x); }          ->  return @abs(x);
fn max(a,b) …; fn g() { return max(2,5); }      ->  return max(2, 5);   // own fn wins
```

Mapping unconditionally would silently shadow user functions — the exact defect
class this chain has spent eighteen waves removing. That is why W566 reverted
instead of shipping it.

## 2. The lowering

```
invariant name: <expr>        ->  comptime { if (!(expr)) @panic(...) }

invariant name                ->  same, via the shared W559 clause parser
    assert BOARD_NAME != ""
```

The **clause form carries no colon** and is the common one — 76 of 81
invariants in the compiling specs. Requiring a colon lowered 3 of 81; routing it
to the clause parser is what made it work.

`forall`-quantified invariants (837) are correctly skipped as not
runtime-checkable, as is any shape the parser cannot model — which restores the
entry checkpoint and falls back to the original skip.

Invariants lower into `comptime` blocks, so a **false** invariant becomes a
compile error rather than a test failure. That is the right semantics for an
invariant, and **no false invariant was found**.

`specs/boards/arty_a7.t27`: assertions **16 → 23**, leaving only its 4 `forall`
invariants inert.

---

## 3. Where the corpus stands

| | |
|---|---:|
| Substantive assertion clauses written | 11,282 |
| Tests passing | 209 |
| Tests failing | 0 |
| **Invariants now compile-time checked** (16 specs) | **65** |
| Specs fully passing | 16 of 199 |
| Specs blocked by `default_input()` | 169 |

---

## 4. Three cooperation variants for W568

### Variant A (recommended) — Keep draining the compile queue

The remaining measured classes, each with a reproduction and a before/after
metric:

1. `duplicate test name` (5 specs) — a genuine spec defect; Zig rejects it
   outright, and it is a one-line rename each.
2. `operator <` on enums — emit `@intFromEnum` in comparisons.
3. Struct-literal syntax: `TernaryWeight{code:1}` should be `.{ .code = 1 }`.
4. `expected ; after statement` (5/80) — still undiagnosed.

### Variant B — Decide the fate of the 571 template tests

Unchanged since W562, now **six waves waiting**. 169 specs cannot compile
because of `default_input()`, proved unfixable mechanically (48 uniform types,
96 mixed, 25 calling functions that do not exist). Rewriting or deleting them
releases the largest remaining block. **Maintainer's call** — it changes test
intent.

### Variant C — Flash the board

Everything software-side has been ready since W553: bitstream at 150.63 MHz,
`fpga-flash` pre-flighting clean, T3 giving a falsifiable prediction. Needs the
QMTech Wukong V1 and its Digilent HS2 cable.

---

## Recommendation

**Variant A.** Each item is small, measured, and unblocked. **B** is the biggest
single lever and has needed a human for six waves; **C** needs hardware.

---

*φ² + φ⁻² = 3 | TRINITY*
