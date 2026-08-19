# Wave Loop 619 — T10 turns an "unsatisfiable" case into a migration

**Date:** 2026-08-11 · **Predecessor:** [`WAVE_LOOP_618_REPORT.md`](WAVE_LOOP_618_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
A  DataSample migrated       IGLA 1093 -> 1072 . no-field-named 51 -> 24
B  struct-method anomaly     narrowed to a CONTRADICTION, reverted
C  the board                 cable not found, verified

T10  widening with defaults is the constructive complement to T9
```

---

## 1. Variant A — the migration T9 did not predict

**T9** (W618) proved a literal carrying an undeclared field is *unsatisfiable*,
and concluded one of the two artefacts must go. **Its proof holds the field set
`F` fixed.** T10 supplies the case it did not consider — changing `F`.

### The corpus voted

| | |
|---|---:|
| `DataSample { … }` literals in `dataset.t27` | **187** |
| `rtl` / `template` / `prompt` — **declared** | 147 / 147 / **86** |
| `quality_score` — **not declared** | **61** |
| `bits` + four singletons — not declared | 8 |
| declared-but-unused | **none** |

**The declaration was right and incomplete.** All three declared fields are
heavily used and none is dead, so deleting `quality_score` from 61 literals
would discard data the tests assert on. A count decided it, not a preference.

### T10

> For `G ⊇ F` where every field of `G \ F` carries a default:
> **(1)** every literal valid under `F` stays valid — *backward* compatibility;
> **(2)** every literal naming fields in `G` is valid — *forward* compatibility.

**This is Protocol Buffers' and Avro's compatibility rule, derived for t27's
nominal structs** — and it is the rule this corpus had never written down.

### The sharper corollary the instance forced

Defaulting only the *added* fields was **not enough**: of the 187 literals,
**101 omit `prompt`** and 40 omit `rtl`/`template` — fields that were already
declared. So `F` itself had to be defaulted.

**With every field defaulted, any subset is a valid literal.**

| | before | after |
|---|---:|---:|
| `dataset.t27` errors | 116 | **95** |
| `no field named …` corpus-wide | 51 | **24** |
| IGLA total | 1 093 | **1 072** |

**21 errors, no test edited, no data discarded.**

## 2. Variant B — narrowed to a contradiction

Third wave on this anomaly. **One build, two probes:**

```
[loop] KwFn "fn"          <- the loop top sees KwFn
(no output)               <- `else if ... == TokenKind::KwFn` never fires
```

An `if / else if` chain over a single field **cannot** fail both `== Ident` and
`== KwFn` for a token whose kind prints as `KwFn`. So the `else if` is not in
the chain it appears to be in, or the `if Ident` arm consumes the token first.

A brace-depth calculation pointed the same way — **and that measurement is
itself unreliable**: it counts braces inside string literals and comments, and
the probe's own `eprintln!("{:?}")` inflates it. Recorded as a caution, not an
answer.

Reverted with `git checkout`; gates restored.

> **Three waves of edit-and-observe have reduced this to a contradiction between
> two printed facts.** It needs a debugger or a minimal standalone reproduction
> of the chain — not a fourth hypothesis. Recorded as **P34**.

## 3. Variant C — the board

```
dlc10 idcode  ->  DLC10 cable not found (VID=0x03FD)
```

## 4. Verification

| Gate | Result |
|---|---|
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | 402 / 608, 0 truncating |
| IGLA total | 1 093 → **1 072** |
| working tree | only `dataset.t27` changed |

---

## 5. Three cooperation variants for W620

### Variant A (recommended) — Apply T10 to the rest of the `no field named` class

24 errors remain, led by `SystolicState.a1` (11) and `BenchResult.pass` (3).
**T10 makes each one mechanical**: count the literals, confirm the declared
fields are live, widen with defaults. The judgement is now a *measurement*, and
this wave is the worked example.

### Variant B — `expected type 'i8', found 'TernaryWeight'` — 92 errors

The single largest class left, and **register entry 1** — `ternary_mac`'s
argument order. It is a decision, but T10 raises a question worth asking first:
**is it actually a decision, or is it another widening?** If `ternary_mac` can
accept both an `i8` and a `TernaryWeight` in that position, the 91-vs-80 split
stops being a fork.

That is a cheap check with a large payoff, and nobody has asked it.

### Variant C — Flash the board

Unchanged. Phase 0 complete; the FPGA family remains at 300/300.

---

## Recommendation

**Variant B**, for the reason T10 just demonstrated: this chain has twice
classified something as "a decision for an owner" that turned out to have a
constructive remedy nobody had looked for.

---

*φ² + φ⁻² = 3 | TRINITY*
