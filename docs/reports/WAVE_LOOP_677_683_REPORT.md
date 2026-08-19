# Wave Loops 677–683 — closing threads, and attacking the one claim that was left

**Date:** 2026-08-14 · **Predecessor:** [`WAVE_LOOP_664_676_REPORT.md`](WAVE_LOOP_664_676_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Seven waves, T142–T147, lessons 455–477. **Two long-running threads were closed
with numbers rather than repairs**, and the project's only surviving technical
claim was attacked on its own stated terms and held.

---

## Summary

```
THE CLAIM, attacked on its own terms
  T142  SMT tried: the refutation condition is not met, and the asymmetry is
        solver-independent
  T143  the algebraic method tried; T143a a diagnosis...
  T144  ...which W680 then refuted. SMT and both ABC engines were RUN at 64x8
        and all timed out; yosys sat was NOT run there (W689 correction).

THE STRUCT-LOWERING THREAD, ended with a measurement
  T145  two depth guards drifted; the gap was a silent wrong width
  T146  the shape audited by name: ZERO new instances, the search terminated
  T147  2.4% of the field population is reachable by any predicate work
```

---

## 1. T117 attacked on its own terms, three times (W678–W680)

T117 stated its own refutation condition: *"A SAT or SMT encoding, or an
algebraic method, that discharges 64 × 8 in minutes."* **None had been tried.**

| a × b | yosys `sat` | Z3 SMT | ABC `cec` | ABC `&polyn` |
|---|---:|---:|---:|---:|
| 8 × 8 | **0.23 s** ✅ | 0.45 s ✅ | 1.40 s ✅ | timeout |
| 12 × 12 | **191.71 s** ✅ | timeout | timeout | — |
| 64 × 4 | 4.41 s ✅ | 6.13 s ✅ | **2.67 s** ✅ | timeout |
| **64 × 8** | — | **timeout** | **timeout** | **timeout** |

**T142 — the asymmetry is solver-independent.** The claim rests not on the wall's
position but on its cause: it is set by the **weight** width. Two engines, two
encodings, the same wall between four and six bits.

**T143 → T143a → T144 — a chain of three corrections.** W679 concluded `&polyn`'s
uniform timeouts indicted the *setup* (`abc -g AND` flattening the arithmetic
structure). W680 tested the tool against a circuit that is *definitionally* a
multiplier — `assign y = a * b` — and it failed identically at 8 × 8. **The
diagnosis was refuted by the golden's own failure**, and with it the caveat it
had produced.

**W689 correction.** "Closed against all three named methods" overstates the
table printed above it: the `64 × 8` row shows `yosys sat` as `—`. **It was
never run there.** SMT and both ABC engines were run at 64 × 8 and all timed
out; the SAT term is closed by *inference* from the 64 × 6 result plus
monotonicity in weight width, not by a run.

**And the oldest tool won.** `yosys sat` proved 12 × 12 where Z3, `cec` and
`&polyn` all failed. A symbolic encoding is not automatically stronger than a
bit-blasted one.

---

## 2. The struct-lowering thread, ended (W681–W683)

**T145 — two guards on one invariant drifted, and the gap was silent.**
`field_type_width` and `packed_struct_width` recurse once *each* per nesting
level; the lowerability predicate counted one. A five-level chain reported
**2,728** bits where the arithmetic gives **10,920** — and the refusal that caused
it was a `return 0` that `sum()` swallowed.

```
nesting 1–4   correct        nesting 5   2,728 -> UNSUPPORTED_ICARUS
```

The repair is not a larger cap: both guards share `DEPTH_CAP` and the **accepting**
side is deliberately the stricter, so anything accepted can be sized.

**T146 — the shape audited by name, and the search terminated.** Three instances
had been found by accident over three waves. Enumerating every function whose
name carries `width`, `size`, `len`, `offset`, `count` or `bits` found **zero
new** — and showed the shape is not spread through the compiler: everywhere
outside the struct-packing path the code already returns `Option` or an empty
collection.

*The first reading of that wave was itself wrong*: it claimed a fourth instance
that W670 had already annotated, because the grep matched **the prose inside the
annotation warning about it**.

**T147 — and the thread ends with a number.**

| field classification | occurrences |
|---|---:|
| OK — primitive · nested struct | 1,545 · 132 |
| unresolved type name (T133) | **955** |
| fundamental — string · unsized slice · float | **700 · 602 · 212** |
| **fixable — enum · sized array** | **44 · 34** |

> **78 of 4,229 occurrences — 1.8% — are reachable by widening the predicate at
> all.** 1,519 are fundamentally unpackable. The capability ceiling has been
> reached; further predicate work cannot repay a wave.

That closes what W665 opened: the largest defect class traced to one predicate
(T129/T131), repaired in stages by W667/W669/W671/W681 — **each stage refusing to
ship a silent wrong width** — audited to closure by W682, ended by measurement in
W683.

---

## 3. A pattern in the loop's own output

**Two consecutive waves began by finding their own predecessor's recommendation
unwarranted.**

| wave | recommendation from | outcome | cost to check |
|---|---|---|---|
| W681 | W680: nested-struct arrays | **already done** in W671 | one command |
| W683 | W682: multi-dimensional arrays | **unwarranted** — all 106 occurrences are unsized | one measurement |

> **A recommendation written at the end of a wave is a hypothesis about the next
> one, and deserves the same check as any other.** Both cost one command; both
> would have cost a wave.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean, seal matches source |
| MVP, both backends | **31/31** Zig, **31 PASSED** iverilog |
| `prove_ternary_mac.ys` / `prove_mvp_classifier.ys` | `Induction step proven: SUCCESS!` |
| `tri prove --mutate` | fails on a perturbed golden, as it must |
| W671 safety battery | **4/4** |
| `impl-status` | 159 / 6 / 173 — unchanged across twenty waves |
| corpus | 444 generate · **156** `iverilog`-clean · 196 Zig-clean · 64 both |
| three boards | enumerate at 1:4, 1:6, 1:8 |

---

## 5. What is NOT done

- **The verdict has never been machine-read.** Closed upstream (T141): the open
  flow expresses only BSCAN's chain-select bit and drops all six routing PIPs.
- **T117 is untested against a purpose-built algebraic verifier.** `AMulet2`
  reads Verilog directly and preserves structure; three candidate repository URLs
  return 404 and it was not obtainable.
- **1,655 struct-field occurrences — 51%** — are the T133 dialect population.
  **A language decision, not a repair.**
- **159 specs unwritten**, 667 declarations without bodies.
- **The MVP does not implement `Z[φ]`**: `contrib` returns `±x`.

---

## 6. Three ways to continue

### Option 1 — **Type aliases** (T133, 51% of struct fields)

After T147 this is not the largest remaining item. **It is the only one.**
Everything else has been measured and closed.

- **Cost:** low to implement. `Bool → bool`, `Int → i64`, `Float → f64`, and one
  canonical string type absorbing five spellings.
- **Risk:** T128 — yield is unpredictable; state it as zero in advance.
- **This needs the user's decision, not the loop's.** It is a choice about what
  t27 *is*.

### Option 2 — **Write function bodies** (159 unwritten specs, 667 declarations)

The largest population in the corpus, and the one no compiler work reaches.

- **Cost:** very high, and it is specification work, not compiler work.
- **Risk:** low, but each body is a judgement about intent that the `.t27` file
  may not record.
- **Confirming measurement:** `impl-status` UNWRITTEN falls below 159.

### Option 3 — **Obtain a purpose-built algebraic verifier**

The one term of T117's condition tested with the wrong input format.

- **Cost:** medium; `AMulet2` was not obtainable through the paths tried.
- **Risk:** it may refute T117 — **which is the point of trying.**
- **Confirming measurement:** the tool proves 8 × 8, and only then does its
  verdict on 64 × 8 mean anything.

**Recommendation: Option 1, escalated rather than assumed.** Twenty waves have
improved the corpus five specs in total, because everything larger is a language
question. **T147 now proves that arithmetically** — 2.4% reachable by engineering,
51% waiting on a decision. Naming that is worth more than another repair.

**φ² + φ⁻² = 3 | TRINITY**
