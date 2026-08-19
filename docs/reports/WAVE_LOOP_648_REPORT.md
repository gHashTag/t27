# Wave Loop 648 — the declaration was in the comment

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_647_REPORT.md`](WAVE_LOOP_647_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T60  the generated `for` never declared its loop variable -- and the
     comment at the emit site says
        // Emit: integer iter_var; for (iter_var = 0; ...)
     followed by only the `for`.

     Invisible because a CONSTANT bound unrolls and needs no variable.
     Third instance in six waves of "met on the common path, missed
     on the rare one".

     real iverilog rejections 6 -> 4.

T61  and my own T59 prediction crossed populations: the stratum stayed
     at 3/144, because all 16 rejections are in specs/scratch/ and the
     144 [BENCH] specs are corpus.
```

---

## 1. T60 — the declaration was in the comment and not in the code

```rust
// Emit: integer iter_var; for (iter_var = 0; iter_var < iterable; ...)
self.write(&format!("for ({} = 0; {} < ", iter_var, iter_var));
```

**Why it survived.** A loop with a *constant* bound is **unrolled** —
`buf[0] = …; buf[1] = …;` — and needs no variable. Only a loop over a
*parameter* emits a real `for`. `w386_for_local_array` passes;
`w386_for_local_array_param` does not, and **they differ in exactly the property
that decides whether the missing declaration matters.**

**Third instance of one shape in six waves:**

| wave | obligation | met on | missed on |
|---|---|---|---|
| T53 | escape a keyword identifier | expression sites, module arrays | function-local arrays |
| W644 | the same escape | everywhere else | `let`-binding declarations |
| **T60** | declare what you reference | the **unrolled** path | the real-`for` path |

> **The probability that a violation is observed is proportional to how often
> its path is taken, so violations concentrate on the rarest paths — which are
> exactly the ones a corpus under-samples and an author under-remembers.
> "It works in the common case" is not weak evidence about the rare case; it is
> the *reason* the rare case is broken.**

Fixed by hoisting loop variables into the function body's declaration block,
where the local `reg`s already go (Verilog forbids a declaration after a
procedural statement, so the existing hoist was the right home).

**Real (non-fixture) iverilog rejections: 6 → 4.**

---

## 2. T61 — the prediction crossed two populations

T59 concluded the output stratum's coverage grows as rejections are repaired.
Two were repaired. The measurement:

| | before | after |
|---|---:|---:|
| corpus specs emitting `[BENCH]` | 144 | 144 |
| of those, compiling under `iverilog` | **3** | **3** |

**No change** — all sixteen rejections are in `specs/scratch/`; the 144 are
corpus. **Disjoint populations.**

**Fifth population error of this session, and the first inside a *prediction***
— which is the variant that survives longest, because nobody checks a prediction
until they act on it.

**Corrected: what actually bounds the corpus build rate.**

| n | cause |
|---:|---|
| **62** | `syntax error` — unread, per T37 read the line not the message |
| **24** | ``'clk' has already been declared in this scope`` |
| 4 | unable to bind a wire/reg/memory |
| 2 | concatenation operand of indefinite width |
| 2 | method-name nesting unsupported by iverilog |
| 141 | total |

**The 24 are one cause:** `clk` is emitted as a module **port**
(`input wire clk,`) and again as a testbench **reg** (`reg clk;`) in the same
scope. **That is the repair that would actually widen the stratum**, and it is in
the corpus, where T59's argument applies.

---

## 3. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| real iverilog rejections | **6 → 4** |
| `w386_for_local_array_param`, `w387_2d_local_array_for` | now compile |
| output stratum coverage | **3/144, unchanged** — T59's prediction tested and disconfirmed |
| ratchet | **CLEAN**, 332/332, rc 0, 556 s |

---

## 4. What was NOT done

- **Four rejections remain**: 2 × missing function (`sum_param`), 1 ×
  empty-identifier declaration (`reg [31:0] ;`), 1 × array-return cast.
- **The 24 duplicate-`clk` specs are diagnosed, not fixed.**
- **The 62 corpus `syntax error`s are unread** — T37 says read the line; I have
  not.
- **Four gates remain unaudited** for their totality claims.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 5. Three ways to continue (pick one for W649)

### Option 1 — **The 24 duplicate-`clk` declarations**

One cause, 24 corpus specs, and **the only repair identified that would widen
the output stratum** — T61 measured that scratch repairs do not.

- **Cost:** low-medium. The testbench emitter declares a `reg` for a signal
  already declared as a port; the fix is to skip signals that are ports.
- **Pays off in:** correctness *and* T59's back-loaded coverage, in the corpus
  where it counts. 24 of the 141 blockers in one change.
- **Risk:** the testbench needs to *drive* `clk`, and a port cannot be driven by
  an `initial` block — so the fix may be to make the module's clk an input the
  testbench wraps, not simply to drop the `reg`. Determine which before editing.
- **Confirming measurement:** corpus `[BENCH]` specs compiling: 3 → up to 27,
  and the output stratum's coverage with it.

### Option 2 — **Read the 62 corpus `syntax error`s**

The largest single class blocking the corpus, and completely uncharacterised —
`syntax error` is iverilog's least informative message, which is precisely why
T37 says to read the offending line.

- **Cost:** low to characterise, unknown to fix.
- **Pays off in:** the 62 are 44% of the 141 blockers; nothing can be planned
  about them until they are grouped by construct.
- **Risk:** they may be many small causes rather than a few large ones — T37
  measured 147 source-shape classes over 178 parse failures, so expect a tail.
- **Confirming measurement:** a class histogram over the 62, grouped by the
  rejected source line, summing to 62.

### Option 3 — **Finish the gate audit: the remaining four**

`no-vacuous-invariant` (Zig only), `no-vacuous-verilog-test`,
`backends-declare-omissions` (3 of 5 backends), and the ratchet. T56 found the
first audited gate understating by 22%.

- **Cost:** medium; four enumerations.
- **Pays off in:** every figure those gates produce is a lower bound of unknown
  tightness, and this document quotes several as measurements.
- **Risk:** each widening reddens a gate and may need a bless.
- **Confirming measurement:** per gate, the T55 table.

**Recommendation: Option 1.** It is the only item that repairs correctness and
widens a gate at once, it is one cause covering 24 specs, and T61 just
established that the alternative repairs do not move the coverage. **Its risk —
that a testbench cannot drive a port — is the kind that must be settled by
reading the emitter before editing, which is a lesson this session has now
recorded three times.**

---

## Appendix — reproduction

```bash
./target/release/t27c gen-verilog-for-simulation <spec> > out.v && iverilog -g2012 -o /dev/null out.v
```

Split `*_negative_*` fixtures out before counting. For the stratum, measure over
the specs that actually emit `[BENCH]` — **the repaired population and the
measured population must be the same one** (T61).

**φ² + φ⁻² = 3 | TRINITY**
