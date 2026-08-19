# Wave Loop 558 Report — the gate that blocked five waves did not exist

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_557_REPORT.md`](WAVE_LOOP_557_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W557 concluded that four tracks were blocked behind a LANG-EN approval and that
the measuring was done. **That conclusion was wrong, and it was wrong for five
waves.** W558 read `build.rs` properly and found the gate is `FROZEN_HASH` — a
documented two-step ceremony, not an approval.

Consequences: one compiler fix landed, the missing ceremony tool was built, and
the largest outstanding fix was attempted, verified, and reverted on evidence.

---

## 1. The misdiagnosis

From W549 onward I recorded: *"`build.rs` panics on six Cyrillic documents;
`docs/.legacy-non-english-docs` is Architect-approval-only; therefore any
`compiler.rs` edit is blocked."*

`build.rs` treats Markdown language violations as **warnings**:

```rust
if let Err(msg) = scan_cyrillic(&path, &rel, &allow) {
    eprintln!("cargo:warning={msg}");     // <- warning, not panic
}
```

Only **spec** files panic. The actual panic, read properly this time:

```
thread 'main' panicked at bootstrap/build.rs:220:9:
t27c FROZEN HASH violation: bootstrap/src/compiler.rs has changed
without a seal update.
```

In W549 I saw a wall of `LANGUAGE POLICY VIOLATION` warnings above the failure
and attributed the panic to them **without reading the panic line**. Every wave
after inherited it. The six documents were never the blocker.

**This is the failure mode my own skill rule 15 warns about**, applied to
myself: I treated the loudest output as the cause instead of reading the exit.
Rule 21 has been added: *when a build fails, read the panic line, not the
warnings above it.*

---

## 2. The real gate, and the tool that was supposed to open it

`CANON.md:37` — *"Do not silently edit `FROZEN_HASH` — update only via freeze
ceremony (M5) … use `cargo run --release -- frozen-digest`."*

That command is referenced in `FROZEN.md:108/110/128`, `CANON.md:37` and
`build.rs:224` — **and did not exist.** The same documented-but-missing class as
`fpga-flash` before W549.

**Implemented `t27c frozen-digest`**, validated by reproducing the existing seal
byte-for-byte before any edit, then used to update the seal through the
ceremony rather than by hand.

---

## 3. Landed: `as f32` / `as f64` casts

`f32`/`f64` are first-class elsewhere in the compiler — `TypeInfo::F32` exists,
they are accepted as parameter and return types, the Zig/Rust/C emitters handle
them — but were missing from the `as`-cast whitelist.

**Measured: 9 of the 326 known-failing specs now parse** (326 → 317). Less than
the ~16 estimated from first-error classes, because fixing a spec's first error
reveals its next one: `eda`, `eval` and `training` move past the cast and stop
later.

This one-line change sat blocked for five waves on a gate that was not there.

---

## 4. Attempted, verified, reverted: BDD lowering

The largest outstanding fix — 7,623 test blocks and 5,163 invariants that
assert nothing.

**The lowering works.** Implemented as:

```
given x = expr  ->  StmtLocal x = expr
when  y = expr  ->  StmtLocal y = expr
then  expr      ->  StmtExpr( ExprCall "assert"( expr ) )
```

Proven end-to-end: the false-assertion spec generated
`if (!(x == 999)) @panic("assertion failed")` and `zig test` **aborted** — the
exact behaviour missing since the beginning.

**And it was reverted.** A full census gave `PARSE OK=726 FAIL=337` against a
317 baseline: **19 specs that parsed before stopped parsing.** I had committed
to reverting if verification failed, so it is reverted; all 19 parse again and
the f32 fix is preserved.

Two mechanisms found, one fixed:

1. **`and` continuation clauses** (fixed) —
   `given p35 = A` / `and p100 = B`. Not handling `and` stranded the parser on
   its `=`.
2. **`parse_expr` is greedy across newlines** (partially handled) — a binding
   value swallows the next clause's name as a binary `and` expression and stops
   on `=`. A whole-block checkpoint with fallback fixed `arty_a7.t27`.
3. **A third mechanism remains undiagnosed** — the 19.

The diff, the 19-spec regression set and the analysis are preserved in
[`docs/patches/W559-bdd-lowering.md`](../patches/W559-bdd-lowering.md) so the
next attempt starts with a fixture set. The likely correct approach is a
**line-bounded clause value** rather than checkpoint-based over-consumption
detection — greedy `parse_expr` across newlines is the root cause of both known
mechanisms.

---

## 5. State

| Track | Status |
|---|---|
| `as f32`/`as f64` casts | **landed**, 9 specs |
| `frozen-digest` ceremony tool | **landed** |
| BDD lowering (7,623 tests + 5,163 invariants) | attempted, reverted, **fixture set ready** |
| Hollow-synthesis / datapath root cause | **unblocked**, not started |
| Syntax gaps (~84 specs) | **unblocked**, not started |
| `.tri` migration (needs `pub type`) | **unblocked**, not started |
| G2/G3 flash | needs a board |

**Nothing is waiting on an approval.** The only external dependency left in the
whole project is a physical board for G2/G3.

---

## 6. Three cooperation variants for W559

### Variant A (recommended) — Finish the BDD lowering

The highest-value fix in the project, now with a proven lowering, two diagnosed
failure mechanisms and a 19-spec fixture set.

**Deliverables.**
1. Make the clause value **line-bounded** so `parse_expr` cannot cross a newline
   into the next clause.
2. All 19 regressions must parse; full census must show `FAIL <= 317` with an
   empty regression diff.
3. Then report how many of the 7,623 newly-executing tests **fail** — that
   number is the real, previously-hidden defect count and the most valuable
   figure this project could produce.
4. Freeze ceremony after the edit.

**What would falsify it.** If line-bounding the clause value breaks multi-line
`then` expressions that legitimately span lines, the grammar needs an explicit
terminator and the change is larger.

### Variant B — Datapath root cause

W554 established that generated modules have a fixed `clk/rst_n/en/ready`
interface, drive only `assign ready = 1'b1;`, and emit spec functions that
nothing instantiates — 0 logic cells across every spec sampled. Now unblocked.
This decides whether "spec to synthesisable RTL" is achievable at all.

### Variant C — Syntax gaps (~84 specs)

Block-expressions (~40 specs) and struct-literals-in-expression (~28) — the
classes measured in W549 §4.3b. Mechanical by comparison with A and B, and it
would move the parse rate materially.

---

## Recommendation

**Variant A.** It has the largest measured impact, the lowering is already
proven correct, and the remaining work is bounded by a concrete fixture set.
Attempt it with the census as a hard gate, exactly as W558 did — and revert
again if it does not hold.

---

*φ² + φ⁻² = 3 | TRINITY*
