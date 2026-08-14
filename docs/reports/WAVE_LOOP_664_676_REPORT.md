# Wave Loops 664–676 — what a measurement is worth, and what it costs to find out

**Date:** 2026-08-14 · **Predecessor:** [`WAVE_LOOP_657_663_REPORT.md`](WAVE_LOOP_657_663_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Thirteen waves, T128–T141, lessons 411–454. **Six of the thirteen ended by
retracting a claim this project had been carrying** — four of them claims made
during this same session.

---

## Summary

```
THE BACKLOG, measured until the number stopped moving
  T125  124 defect specs, not 466 and not 289 -- populations sum to 617 exactly
  T133  108 of them are written in a dialect the compiler never implemented
  T129/T131  the oldest open question traced from symptom to a single predicate

WHAT A FIX IS WORTH
  T126  a fix moves the count iff it clears a spec's LAST class -- 3 confirmations
  T128  ...and then refuted, by a forecast written to be able to lose
  T136  a conditional estimate quoted unconditionally overstates its own case

THE READBACK, refuted four times, each more strongly
  T138  one control read      T139  a distribution
  T140  a 28-bit magic        T141  the root cause, upstream of us
```

---

## 1. The backlog is 124 specs, and the books close (W660–W662)

Three successive corrections of a number this project had been quoting:

| claim | reality |
|---|---|
| "466 specs fail" | 173 do not parse · 159 unwritten · 6 partial |
| "289 defect specs" | contaminated by unwritten specs — inflated 2.3× |
| **124 defect specs** | populations sum to **617** exactly, two code paths agree |

```
iverilog accepts 155 | does not generate 173 | UNWRITTEN 159 | PARTIAL 6 | DEFECT 124
```

**T125.** Both `impl-status` and the new `t27c spec-status` agree on every
population label. The figure had been inflated **3.8×** by counting
specifications nobody had written as broken ones.

**And 108 of the 124 are not compiler defects either.** T133 censused the field
types and found five spellings of *string*, capitalised names declared nowhere
(`Bool`, `Int`, `Float`), and types written as **string literals** (`"usize"`).
Verified end to end: `zig: error: use of undeclared identifier 'Bool'`. **They
are written against a language that does not exist**, and adopting the aliases is
a decision about what t27 *is*, not a repair.

---

## 2. What a fix is worth — a law, three confirmations, and a refutation

**T126**, from four fixes each correctly diagnosed and verified:

| wave | fix | specs repaired | depth | compiling count |
|---|---|---:|---|---:|
| W659 | escape-last | 13 | >1 | 151 → **151** |
| W660 | Verilog scaffold | 140 | 94% at 4+ | 151 → **151** |
| **W661** | **`#` is a comment** | **4** | **all depth 1** | **151 → 155** |
| W663 | Zig builtins | 17 | >1 | 155 → **155** |

> The three that moved nothing removed 170 specs' worth of real defects. The one
> that moved the number touched four.

**Then W664 refuted it.** The forecast said the count would not move and stated
that movement would refute T126. It moved: `specs/server/api.t27` showed **three**
distinct diagnostic classes and was repaired by **one** fix.

**T128.** Measured depth bounds nothing in either direction — one class name
merges unrelated causes (T127), one cause emits several class names (here).
T126's *mechanism* survives; the rule drawn from four data points was an artefact
of those four. **Yield is measurable only after the fact.**

---

## 3. The oldest open question, traced to one line (W665–W671)

The mission context carried *"489 `undeclared identifier`, NOT diagnosed"* as its
oldest open item. Traced end to end:

```
T129  symptom     declarations keyed by TYPE, uses keyed by VARIABLE
T131  cause       one all() in is_lowerable_scalar_struct admits only primitives
      population  242 of 444 generating specs carry the disabling marker
```

`BrainState { arousal: ArousalLevel, … }` — a single enum field drops the whole
struct into a fallback that declares `reg <TypeName>_<field>` while every use
emits `<varname>_<field>`. **They can never agree**, and the fallback is unsound
anyway: per-type registers are module globals.

**The repair was staged over three waves and each stage refused to ship silent
wrongness.**

| wave | accepted | refused, and why |
|---|---|---|
| W667 | `usize`/`isize` — 25 structs | floats: a packed slice of a `real` is silently wrong |
| W667 | — | nested structs: **the safety test caught 72 bits reported for a 56-bit struct** |
| W669 | — | unsized slices: `parse().unwrap_or(1)` sized `[]u8` as ONE element |
| W671 | nested structs | after `field_type_width` made the width right — `Cfg` = **56 bits** |

**T134** named the shape: `parse().unwrap_or(<plausible default>)` converts a
parse failure into a confident wrong answer. **T135** then audited every instance
of it and found the shape three times and the *reach* once — the other two sit
behind a guard or behind an input no spec writes. **A defect shape is not a
defect population.**

---

## 4. The readback, refuted four times (W672–W676)

Everything this project has on silicon rested on a lamp no machine reads.

| wave | claim | refuted by | strength |
|---|---|---|---|
| W673 | the verdict reads `0b0111` | one control read | a comparison |
| W674 | the alternation is `beat` | ten reads per bitstream | a distribution |
| W675 | the register is in the scan path | a 28-bit magic | one unforgeable read |
| **W676** | **why** | **per-line FASM bit count** | **root cause** |

**What was proven along the way**, each against a known answer: an FTDI MPSSE
transport (`IDCODE = 0x13636093`), `shift_ir` (IDCODE and BYPASS opcodes both
behave), `shift_dr_read` (low nibble of IDCODE), and the BSCANE2 primitive placed
at `BSCAN_X0Y0` with seven FASM lines.

**T141 — the root cause, and it is not ours.** Feeding `fasm2frames` one line at
a time:

```
BSCAN.JTAG_CHAIN_1                     ->  BITS SET
…BSCAN1_{SHIFT,CAPTURE,SEL,DRCK,TDI,TDO}  ->  NO BITS   (all six)
```

The chain-select bit is the only part the open flow can express. All six routing
entries produce **zero** configuration bits and `fasm2frames` returns `rc 0`
without a warning. **The primitive is selected and none of its signals is
connected** — so W674's `TDO`-edge hypothesis was never testable, because `TDO`
was never wired.

**T140a — and a claim of mine is retracted.** W672 recorded the BSCANE2 risk as
*resolved* because yosys instantiated the cell and nextpnr routed it with zero
errors. **A primitive that places is not a primitive that works.** P&R acceptance
is evidence about the placer, one layer below `Done 0x1` versus a computed
result.

Recorded as a warning in [`LOCAL-BITSTREAM-FLOW.md`](../fpga/LOCAL-BITSTREAM-FLOW.md)
so the next attempt starts from the answer.

---

## 5. Tools built, and why each exists

| command | the mistake it prevents |
|---|---|
| `tri corpus` | believing a diagnostic count (T119: 13,066 → 3,765 while 0 specs changed) |
| `tri backlog` | choosing a target by frequency instead of depth (T120) |
| `tri spec-status` | reimplementing an AST predicate with a regex — **four waves tried, four different answers** |
| `tools/jtag/mpsse_jtag.py` | the transport T137 named; proven by IDCODE |

---

## 6. Closing state of the corpus, measured today

```
617 specs
  generates Zig                444   72.0%
    ... and Zig accepts it     196   31.8%
  generates Verilog            444   72.0%
    ... and iverilog accepts   156   25.3%
  BOTH backends accept          64   10.4%
```

Against the population split that T125 established and `impl-status` still
confirms — 159 unwritten, 6 partial, 173 unparsed — **the 156 that compile are
measured against a real denominator for the first time**, and the gap between
*generates* and *accepts* is what remains.

**Thirteen waves moved that number from 151 to 156.** Five specs. The report
above is largely an account of why: three of the four repairs that removed real
defects moved it by zero, and the honest work of the period was establishing what
the number means rather than raising it.

---

## 7. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean, seal matches source |
| MVP, both backends | **31/31** Zig, **31 PASSED** iverilog |
| `prove_ternary_mac.ys` / `prove_mvp_classifier.ys` | `Induction step proven: SUCCESS!` |
| `tri prove --mutate` | fails on a perturbed golden, as it must |
| `impl-status` | 159 / 6 / 173, unchanged across thirteen waves |
| three boards | enumerate at 1:4, 1:6, 1:8; `Done 0x0 → 0x1` with a wrong-part transition |

---

## 8. What is NOT done

- **The verdict has never been machine-read.** Four refutations; the channel is
  closed upstream (T141). A UART needs a pin map for **this** board — the only
  one in the repository is for `CSG324`, not our `FGG676`.
- **The MVP does not implement `Z[φ]`.** `contrib` returns `±x`.
- **108 specs use unimplemented type spellings** — a language decision.
- **27 specs need generic types** (T122) — also a language decision.
- **159 specs are unwritten**, 667 declarations without bodies.
- **Depth is a proxy**, sound in ordering, unreliable in magnitude (T128).

---

## 9. Three ways to continue

### Option 1 — **A UART channel for the verdict**

The only remaining path to a machine-readable result on this toolchain.

- **Cost:** medium, and **blocked on a pin map for the FGG676 board** that the
  repository does not contain.
- **Risk:** the same class as BSCANE2 — a channel that looks alive and is not.
  The control is already designed: send a **magic constant**, not a status bit.
- **Confirming measurement:** a byte the host receives that the design chose.

### Option 2 — **Type aliases** (108 specs, T133)

The largest addressable population in the corpus.

- **Cost:** low to implement, but it is **a decision about what t27 is**:
  `Bool → bool`, `Int → i64`, `Float → f64`, one canonical string type absorbing
  five spellings.
- **Risk:** T128 — expected yield is unpredictable; state it as zero in advance.
- **This one needs the user, not the loop.**

### Option 3 — **Nested-struct arrays and the remaining `element_width` paths**

`Holder2 { items: [4]Good, k: u8 }` now lowers at 168 bits, but arrays of
structs still route through paths audited only by comment (T135).

- **Cost:** low; the safety battery from W671 already exists and can be extended.
- **Risk:** low — the guard is verified, this only widens what it guards.
- **Confirming measurement:** the four W671 checks still pass, plus a new case
  for arrays-of-nested-structs.

**Recommendation: Option 2, with the decision escalated rather than assumed.**
Thirteen waves have improved the corpus two specs at a time because everything
larger is a language question. **Naming that plainly is worth more than another
two-spec repair** — and T133 has already measured exactly what the decision buys.

**φ² + φ⁻² = 3 | TRINITY**
