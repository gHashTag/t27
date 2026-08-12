# Wave Loop 633 — the detector reported zero, and 130 specs were discarding 55,563 tokens

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_632_REPORT.md`](WAVE_LOOP_632_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
W632 said: add parse-complete, expect the ledger to grow sharply.
It ran in under a second and reported TRUNCATE 0.
The recommendation rested on a false premise.

T42  "reached EOF" is not "read the input". skip_to_next_top_level()
     is drop-recovery, so a parse can reach EOF by throwing tokens
     away -- and parse_ast_strict calls that "consume all".

     parse and consume all   436 -> 306
     parse but DISCARD    (unmeasured) -> 130 specs, 55,563 tokens

Worst offender among them: ternary_mac.t27, 1,368 tokens.
That is the spec T1 and T2 are theorems ABOUT.
```

---

## 1. The prediction was wrong, and that was the wave

W632 recommended adding `parse-complete` as a gated phase and predicted *"the
ledger may grow sharply — expect the corpus number to get worse, and that is the
point."*

```
  specs scanned            609
  parse and consume all    436
  parse but TRUNCATE       0
  do not parse             173
```

**Zero.** And the trailing garbage W632 had proved invisible to `parse` was
*also* invisible to `parse-complete`.

**The value of this wave came from asking why zero, not from the planned work.**

---

## 2. T42 — the wrong invariant

`parse_ast_strict` asks one question:

```rust
let ast = parser.parse()?;
if parser.current.kind != TokenKind::Eof {
    return Err("input not fully consumed: stopped at …");
}
```

*Did the parser reach EOF?* But `skip_to_next_top_level()` is **deliberate
drop-recovery** — on an unrecognised top-level item it advances past the tokens
and resynchronises to the next declaration. The repository documents this and has
tests characterising it. So a parse can reach EOF **by throwing tokens away on
the route**, and the check reports "consumed all".

> **Reaching the end of the input is not the same as reading it.** A
> completeness predicate of the form `position = EOF` certifies *termination of
> scanning*, not *coverage of input*. The sound predicate is
> `discard_count = 0`, and the two differ **exactly on the population
> error-recovery was designed to absorb** — the one most likely to contain
> unread specification.

**Instrumented** — a counter in `skip_to_next_top_level`, exposed through a new
`Compiler::parse_ast_accounted` — and re-measured over the same 609 specs:

| | before | corrected |
|---|---:|---:|
| parse and consume all | 436 | **306** |
| parse but TRUNCATE | 0 | 0 |
| **parse but DISCARD** | *not measured* | **130 specs, 55 563 tokens** |
| do not parse | 173 | 173 |

**The 436 was wrong by 130 files.** And the worst offenders are the specs this
project exists for:

| spec | tokens discarded |
|---|---:|
| `specs/igla/race/systolic_ternary.t27` | **5 358** |
| `specs/igla/race/cordic_top.t27` | 3 209 |
| `specs/vsa/ops.t27` | 3 146 |
| `specs/ml/optimizer/adamw.t27` | 2 098 |
| `specs/igla/race/cordic.t27` | 1 847 |
| **`specs/igla/race/ternary_mac.t27`** | **1 368** |

---

## 3. A detector is a stage

§4's table lists components that accepted input, produced less than they should,
and reported success — lexer, parser, C backend, `use_resolve`, and in W588 *"my
own measurement"*.

**`parse-complete` is a component built to catch exactly that, which accepted
input, checked the wrong invariant, and reported success.** Not a measurement
that was wrong — a *detector for wrongness* that was wrong in the way it was
built to detect.

**When a detector reports zero, ask what its predicate says, not what its name
promises.**

---

## 4. The gate did its job

`parse-no-discard` is now a suite phase, running **in-process** (no subprocess,
so the phase is nearly free). The ratchet immediately went red:

```
  ledger:              173 / 173 cap
  observed (primary):  303
  UNEXPECTED FAILURES: 130
    + specs/igla/race/ternary_mac.t27 [parse-no-discard]
    …
RATCHET: FAIL        rc = 1
```

**130 unexpected failures on a population invisible to every gate this project
has ever run.** Blessing them took the ledger 173 → 303 — and `--bless`
deliberately writes `cap = min(prior, observed)`, so the new ledger *failed its
own cap* until `max_entries` was raised **by hand**. That refusal is the feature:
raising the cap is the reviewable event T33's design demands.

After the hand raise, green again at the new baseline:

```
  ledger:              303 / 303 cap
  observed (primary):  303
  UNEXPECTED FAILURES: 0    UNEXPECTED PASSES: 0    EXPIRED: 0
RATCHET: CLEAN       rc = 0       507 s
```

---

## 5. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `cargo test --bins suite::` | 26 passed |
| trailing garbage now detected | **yes** — 1368 → 1373 discarded tokens |
| ratchet before blessing | `FAIL`, rc 1, **130 named** |
| ratchet after blessing + hand cap raise | `CLEAN`, rc 0, 303/303, **507 s** |
| standing unit failures | 5, unchanged |

---

## 6. What was NOT done

- **The 55 563 discarded tokens were not read.** This wave made them *visible*
  and *gated*; nobody has yet looked at what is in them. That is Option 1.
- **`lex-dropped` is still not a suite phase** — 1 135 characters across 31
  specs, a second silent-discard channel one level below this one.
- **The ledger is now 303**, and 130 of those entries carry the same reason.
  They need triage like the parse failures did.
- **Still no web literature.** `WebSearch`/`WebFetch` failed with a provider
  error for the entire session; everything named is from general knowledge and
  **no citation was fabricated**.

---

## 7. Three ways to continue (pick one for W634)

### Option 1 — **Read what is being discarded, starting with `ternary_mac.t27`**

1 368 tokens are dropped from the spec **T1 and T2 are theorems about**. Print
the discarded spans, and answer one question: is any of it *semantic content the
theorems depend on*?

- **Cost:** low to start — print spans for one file; the answer may be large.
- **Pays off in:** it is the only work that can tell whether this project's
  headline formal results are stated about the spec that was actually compiled.
  **Nothing else in the backlog can invalidate a theorem.**
- **Risk:** the answer may be that T1/T2 are unaffected, in which case the
  finding is hygiene, not correctness. That is still worth knowing, and cheap.
- **Confirming measurement:** for each of the six worst specs, a printed list of
  discarded spans and a yes/no on whether any is a declaration the generated
  Verilog needed.

### Option 2 — **Add `lex-dropped` as a gated phase, one level below**

1 135 characters across 31 specs are discarded by the *lexer*, before the parser
sees them. Same shape as T42, one stage earlier: a documented, measured,
ungated silent-discard channel.

- **Cost:** low — the subcommand exists; wire it in-process like
  `parse-no-discard`.
- **Pays off in:** closes the last known silent-discard channel in the front end.
- **Risk:** it may add another large ledger class, and the cap raise will be
  another hand edit. Expect the number to get worse; that is still the point.
- **Confirming measurement:** the phase reports 31 specs, the ratchet names
  them, and the ledger cap moves by exactly that many.

### Option 3 — **Triage the 130 discard entries into causes**

The parse failures went from 206 paths to 48 named classes by re-running the
phase and normalising. Do the same here: group the 130 by *what construct* the
recovery skipped, using the discarded span rather than the message (T37).

- **Cost:** medium; a span-printing pass plus classification.
- **Pays off in:** turns 130 identical `reason` fields into a work queue, and
  likely finds two or three language features the parser lacks.
- **Risk:** T37 applies to itself — group by the discarded *source shape*, never
  by a message.
- **Confirming measurement:** a class histogram summing to 130, with the top
  classes' exemplar spans quoted.

**Recommendation: Option 1.** Every other item is hygiene. This one can tell
whether **T1 and T2 are theorems about the spec that was compiled or about a
spec with 1 368 tokens removed** — and that question outranks the rest of the
backlog combined.

---

## Appendix — reproduction

```bash
./target/release/t27c parse-complete
```

Look at the `parse but DISCARD` line. For the gate:
`t27c suite --repo-root . --ratchet --corpus-only`. To see the effect of a
change, append text to a spec and re-run `parse-complete` — the discarded-token
count for that file moves.

**φ² + φ⁻² = 3 | TRINITY**
