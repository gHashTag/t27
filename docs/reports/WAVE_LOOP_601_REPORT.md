# Wave Loop 601 — nine specs that named their own falsification and never ran it

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_600_REPORT.md`](WAVE_LOOP_600_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
44 invariants added across nine GF format specs
 1 real defect found IN THE SPECS THE INVARIANTS WERE ADDED TO
 2 compiler fixes the work forced, one of them a W599 regression
```

The recommended variant was **half falsified** before any file was edited, and
the surviving half turned out to be sharper than the whole.

---

## 1. The falsification, run first

W600's Variant A: *"38 specs compile and assert nothing — give them tests."*
Before writing one, the 38 were measured for what there is to test:

```
38 specs → 2 functions total
specs/sacred/gravity.t27 → 326 bytes, entire file
```

`specs/sacred/*.t27` (7 files) are **stubs** — a module header, two `use` lines,
and an empty banner reading `TDD: Tests (from .tri behaviors)`. W586 already
established that the `.tri` sources do not exist. **There is nothing in them to
test**, and writing tests would mean writing the specs. That is not this
variant; it is a different and much larger one, and it belongs to whoever owns
those files.

**But `specs/numeric/gf*.t27` (9 files) are not stubs — they are constant
tables**, and each ends with this:

```
; Claim-status: Conj
; Fpath: closed-form rule mis-applied (verify e = round((10-1)/phi^2) = 3, m = 6)
;        or RTL emission diverges from this constant set.
```

**Each of these specs states its own falsification path, in a comment, and
nothing has ever run it.**

## 2. Making the Fpath executable

The generating rule is `e = round((N−1)/φ²)`, `m = N−1−e`. Checked by hand
across all nine before writing anything — **zero mismatches**, so the constants
were right and the job was to make them stay right:

| Invariant | Form |
|---|---|
| widths partition the word | `SIGN_BITS + EXP_BITS + MANT_BITS == TOTAL_BITS` |
| the rule's second half | `MANT_BITS == TOTAL_BITS - 1 - EXP_BITS` |
| **the rounding, as bounds** | `(E − ½)·φ² ≤ N−1 ≤ (E + ½)·φ²` |
| layout | `SIGN_SHIFT == TOTAL_BITS-1`, `EXP_SHIFT == MANT_BITS`, `MANT_SHIFT == 0` |
| bias | `BIAS == 2^(E−1) − 1` |
| exponent ceiling | `EXP_MAX == 2^E − 1` |

The rounding is stated as **bounds** because `round` is not available at
comptime and the inequality is exactly equivalent: `e = round(x) ⟺ e−½ ≤ x ≤ e+½`.

**44 invariants** across `gf6`, `gf10`, `gf14`, `gf48`, `gf96`, `gf128`,
`gf256`, `gf512`, `gf1024`.

### They are enforced, not decorative — verified

```
gf10.EXP_BITS: 3 → 4   ⟹  BLOCKED: does not compile: error: assertion failed
                            note: called at comptime here   (the invariant's own line)
gf10.EXP_BITS: 4 → 3   ⟹  invariants  6   proved
```

## 3. The work found a defect in the specs it was added to

```t27
pub const EXP_BITS   : u8 = 391;      // specs/numeric/gf1024.t27
```

**`u8` cannot represent 391.** The *value* is correct — `round(1023/φ²) = 391` —
and every other rung's exponent width does fit (195, 97, 49, 36, 18), so this is
the one place the annotation was copied without checking. Changed to `u16`,
matching `TOTAL_BITS`, `MANT_BITS`, and all three shifts in the same file.

It was invisible for as long as nothing used the constant in arithmetic. **The
invariant did not merely document the rule; adding it is what made the compiler
look at the constant at all.**

## 4. Two compiler fixes the work forced

**A W599 regression, found one wave later.** The `__t27_assert_fail` helper was
emitted only when a file contained a `TestBlock`. A spec with **invariants and
no tests** therefore referenced a helper that was never emitted —
`specs/numeric/gf*.t27` are exactly that shape. The condition must match *"will
this file contain an assertion"*, not *"does it have tests"*.

**`test-report` was calling proved invariants unchecked.** Invariants lower to
comptime assertions and never reach `builtin.test_functions`, so a spec with six
invariants and no tests read as `tests 0` and was filed under **NO TESTS** — the
L4-violation bucket. That is wrong in the strongest way available: **if the spec
compiled, every one of its invariants held.** The command now counts them, and
the tree report has a fourth population:

```
specs MEASURED           compiles and has test functions -- the only ones with a rate
specs INVARIANTS ONLY    comptime -- compiling IS the check
specs with NO TESTS      asserts nothing at all -- L4
specs BLOCKED            never produced a binary
```

## 5. Verification

| Gate | Before | After |
|---|---|---|
| `specs/numeric` NO TESTS | 10 | **1** |
| `specs/numeric` invariants proved | 0 | **44** |
| `cordic.t27` | 330/336 | 330/336 |
| `adder_tree.t27` | 335/335 | 335/335 |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 | 29/29 · 13/13 |
| `cc-gate` | 101 | 101 |

**Corpus-wide, re-measured after the change:**

```
specs MEASURED           30      (29 at 100%)   unchanged
specs INVARIANTS ONLY     9      <- new population, all nine from this wave
specs with NO TESTS      29      was 38
specs BLOCKED           540      unchanged

1024 tests / 1018 pass / 6 fail / 99.4%          unchanged
invariants proved       445      <- never counted before
```

**445 invariants were already being proved and nothing reported it.** 44 are
this wave's; the other 401 sit in specs that also have tests, and were invisible
because invariants never reach `builtin.test_functions`.

---

## 6. Three cooperation variants for W602

### Variant A (recommended) — `formats_catalog.t27`: 83 getters, 32 KB, zero tests

**The variant I was about to write here was falsified while writing it.** It read
*"`specs/tri/` is 19 specs, the largest remaining group — measure them first."*
The measurement was one command: **17 of the 19 are 320–334 byte stubs**, the
same shape as `sacred/`. So are 7 `sacred/` and 1 `ml/`. **25 of the 38 are stubs**
— unwritten specs, not specs missing tests — and the real remaining L4 work is
**4 files, not 38**:

| Spec | Size | Declares |
|---|---:|---|
| `numeric/formats_catalog.t27` | **32.6 KB** | **83 fn getters** |
| `ternary/clocked_counter.t27` | 1.4 KB | 1 fn |
| `tri/agent/faculty_board.t27` | 1.3 KB | 6 const, 6 struct |
| `tri/utils/error.t27` | 0.6 KB | 1 const |

`formats_catalog.t27` is the target, and it is the corpus's largest untested
SSOT — it declares itself *"Single source of truth for every numeric format"* and
its output feeds six codegen targets. This wave already verified its 17 GF
entries by hand (P16, zero mismatches); the work is to put that check **in the
file**, and to extend it to the ~66 non-GF formats nothing has looked at.

### Variant B — Reclassify the 25 stubs, or delete them

They are not L4 violations and reporting them as such overstates the debt by
almost double. Either they are unwritten specs — in which case `impl-status`
should own them and `test-report` should say so — or they are dead files from a
generator that ran once, in which case they should go. **That is a decision, not
an implementation**, and it belongs to whoever owns `specs/tri/` and
`specs/sacred/`. The measurement is done and in
[`data/W600-specs-with-no-tests.txt`](data/W600-specs-with-no-tests.txt).

### Variant C — Flash the board

`verdict : BLOCKED -- no programmer on USB`. The FPGA spec family remains at
300/300.

---

## Recommendation

**Variant A.** This wave's value came almost entirely from refusing to write the
work before checking what the files contained. Two thirds of the target turned
out to be stubs; the remaining third already stated, in a comment, the test it
needed; and the one real defect was found by the compiler the moment something
finally used the constant. The same discipline applied to the next variant
falsified it before it was published — which is why Variant A above is a
different file from the one this section was drafted to recommend.

CORDIC argument reduction (T6) remains open and remains the only known code
defect in the measured corpus; it is a behaviour change and wants an owner.

---

*φ² + φ⁻² = 3 | TRINITY*
