# Wave Loop 651 — 98 constants were silently wrong

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_650_REPORT.md`](WAVE_LOOP_650_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
Set out to fix the 23 `::` leakages. Found something that outranks them.

  pub const A : u8 = constants::COMPLEXITY_HIGH;

  gen (Zig)    pub const A: u8 = constants;
  gen-rust     pub const A: u8 = constants;
  gen-c        static const uint8_t A = constants;
  gen-verilog  parameter [7:0] A = constants;

Four backends, four silently wrong VALUES, no error, no warning.
98 such initialisers across 29 specs.

T67  and the `::` forecast, pre-registered: 0 of 24. Measured by
     simulating the fix. `::` is the outermost of 4-6 stacked defects.

T65  reviewing W640's 22 deferred baselines: they froze the T57 bug.
     45 of 265 COMMITTED baselines did too.
```

---

## 1. T66 — the defect underneath the defect

The same path **inside a function body** keeps both segments — Zig emits
`return constants.COMPLEXITY_HIGH;` correctly. **Only the module-level const
initialiser truncates.**

`parse_const_decl`:

```rust
let name = self.current.lexeme.clone();
if self.peek.kind == TokenKind::LBrace || self.peek.kind == TokenKind::LParen {
    let lit = self.parse_expr()?;      // handles `::` correctly
} else {
    val_node.name = name;              // FIRST SEGMENT ONLY
    self.advance();                    // `::COMPLEXITY_HIGH` is then skipped
}
```

**`constants::make(5)` already worked** — the `(` selected the `parse_expr`
branch. Only the bare-path spelling took the truncating branch. **T60's shape a
fourth time:** met on the path that happens to carry a delimiter, missed on the
one that does not.

> **A defect that produces a *wrong value* is invisible to every check that asks
> whether the artefact is *well formed*** — and `A = constants` is well formed in
> all four target languages. **Nine gates were built this session and not one
> could see it.** The only signal was a compile defect being investigated for an
> unrelated reason sitting one layer above it.

**The repair makes the naive metric worse.** C and Verilog now emit
`constants::COMPLEXITY_HIGH`, which they cannot compile — **a visible error
replacing a silent falsehood** — while Zig and Rust now emit the correct
reference. Any metric counting compile failures scores this as a regression.

---

## 2. T67 — the forecast, pre-registered at 0

Following T44, the `::` yield was forecast **before** any fix and committed to a
number: **0 of 24.** Not a range.

**Method: simulate the most generous plausible fix** — regenerate all 24 and
rewrite every `::` to `_` — then compile: `total=24 pass=0 still_syntax=10`.

- 14 trade their syntax error for an **elaboration** error.
- 10 keep a syntax error **on a line that never contained `::`** — `++` string
  concat, `@as(...)`, `reg [31:0] ;`, `.len(1'b0)`, two-arg `assert`.

**The tell is the smallest residual.** `jones_topology_decision_gate` drops to a
*single* error — and it is not `::`, it is `Unable to bind parameter
'jones_topology_filter'`: **T66's truncation.** Neutralise that too and the file
jumps to 12+ errors.

> **`::` is the outermost of four to six stacked defects**, and iverilog aborts
> at the first failing stage, so every residual count is a **floor**.

**Root cause, one line of wiring:** `run_gen_verilog_for_simulation` **never
calls `use_resolve::resolve`**, while Zig (`main.rs:3669`), C (`4530`) and Rust
(`4547`) all do.

**The cross-backend oracle gave its most useful answer: no backend is correct.**
Zig *looks* clean — `zig_ident` joins segments with `.`, so `grep '::'` finds
zero hits — but `constants::PHI` becomes `constants.PHI`, **the same dangling
reference**, and `zig ast-check` fails on 23 of 24, 17 naming the module
qualifiers. **A grep for the symptom in one backend's spelling is not a
measurement of the defect.**

---

## 3. T65 — the deferred review paid off

W640 acquired 22 Icarus baselines and I left them **uncommitted and unreviewed**.
Reviewing them now: one records

```
[BENCH] matrix_local_bench : %0d cycles          3
```

**T57's malformed format string, frozen as expected output.** They predate three
fixes (W640's `NOT CHECKED`, W646's format repair, W649's port guard).
**Discarded, not committed.**

**And 45 of the 265 *committed* baselines carry the same frozen bug.** W646's
one-character repair invalidated all 45, and nothing reported it — the checker is
opt-in (T51), so the invalidation is invisible twice over.

> **An oracle recorded from a generator is a memo of that generator's behaviour,
> not of the specification.** The set of oracles a change invalidates is not
> derivable from the change. Golden files need a **provenance stamp**.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| truncation, all four backends | both segments preserved |
| new tests | **2 passed** — the fix, and the call spelling that already worked |
| ratchet | **CLEAN**, 332/332, rc 0, 744 s — no regression |
| W640's 22 baselines | reviewed, **discarded** |
| committed baselines with the frozen bug | **45 of 265** |

---

## 5. What was NOT done

- **`use_resolve` is not wired into the Verilog entry points.** T67 forecasts
  yield 0 for it in isolation, so it was deliberately not spent this wave.
- **The 45 stale baselines were not re-blessed.** That needs the provenance
  question settled first, or it recreates T65.
- **The other 96 truncated initialisers were not audited for value correctness** —
  the parser now preserves them, but whether each resolves is a separate question.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W652)

### Option 1 — **Re-bless the 45 stale baselines, with a provenance stamp**

T65's finding is open: 45 committed oracles encode a fixed bug, and nothing
detects it. Add a `generator_sha` field, regenerate, and make a mismatch a
warning.

- **Cost:** low-medium. The blessing mode exists (W640); the stamp is a field.
- **Pays off in:** staleness becomes *decidable* rather than *discovered*, which
  is the only structural fix for a class that will recur on every generator
  repair.
- **Risk:** re-blessing 45 files is exactly the bulk acquisition T31 warns
  about. Each must be diffed, and the diff should show only the `%0d` line.
- **Confirming measurement:** all 45 differ from their predecessors in exactly
  the cycle line; the stamp is present; a synthetic generator change flags them.

### Option 2 — **Audit the 98 now-preserved initialisers for resolution**

T66 stopped the truncation; it did not check that `constants::COMPLEXITY_HIGH`
*resolves*. In Zig it becomes `constants.COMPLEXITY_HIGH`, which needs the
`@import` that W651's investigation found is **suppressed** by a
`module_referenced` gate matching only the dot spelling.

- **Cost:** medium; one gate condition plus verification per backend.
- **Pays off in:** turns 98 preserved-but-dangling references into 98 correct
  ones. **T66 made them visible; this makes them right.**
- **Risk:** the fix may be per-backend rather than shared — Zig's `@import`
  suppression, Rust's `use`, C's include. Measure each before assuming one fix.
- **Confirming measurement:** `zig ast-check` on the affected specs, and the
  `use of undeclared identifier` count naming module qualifiers falling from 17.

### Option 3 — **Wire `use_resolve` into the Verilog entry points**

The one-line asymmetry: three backends resolve, Verilog does not.

- **Cost:** low — thread the `Path`, call `resolve` at `main.rs:3681` and `3706`.
- **Pays off in:** removes a structural inconsistency between backends, and is a
  precondition for anything else in the `::` class.
- **Risk:** **T67 forecasts yield 0**, so this must be justified as
  *consistency*, not as *progress*. Committing to it on the expectation of
  compiling specs would repeat W650's disappointment.
- **Confirming measurement:** `::` occurrences in generated Verilog falling from
  521 tokens across 49 files — and the compile count explicitly *not* expected
  to move.

**Recommendation: Option 2.** T66 is this wave's result and it is only half
done: 98 constants stopped being silently wrong, and are now visibly dangling.
**Leaving them there converts a correctness defect into a compile defect and
calls it progress** — which is exactly the accounting T66 warns about. Option 1
is the tidiest and Option 3 is forecast to buy nothing.

---

## Appendix — reproduction

```bash
printf 'module M\n\npub const A : u8 = constants::COMPLEXITY_HIGH;\n' > /tmp/t.t27
for b in gen gen-rust gen-c gen-verilog; do ./target/release/t27c $b /tmp/t.t27 | grep 'A'; done
```

Before this wave all four printed `= constants`. **Forecast a yield before
fixing a class** — the method that worked here was to *simulate* the fix
(rewrite `::` to `_`) and compile, not to reason about it.

**φ² + φ⁻² = 3 | TRINITY**
