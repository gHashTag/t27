# Wave Loop 643 — the first thing that was simply wrong

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_642_REPORT.md`](WAVE_LOOP_642_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
16 iverilog rejections -> 6 are deliberate *_negative_* fixtures.
Ten are real. Grouped by the REJECTED CONSTRUCT (six say only
"syntax error", which is T37's warning):

   4  local array named `buf`                 syntax error
   2  function referenced but not emitted     No function named 'sum_param'
   2  undeclared `for` loop variable          register `c' unknown
   1  declaration with NO identifier          reg [31:0] ;
   1  array-returning call in an assignment   cannot be implicitly cast

T53  the escape EXISTS, is TESTED, and was applied at every expression
     site and both module-level array declarations -- and NOT at the two
     that emit a function-LOCAL array.

     real rejections 10 -> 6.  Ratchet CLEAN 332/332.
```

---

## 1. `buf` is a Verilog primitive gate

And the repository already knows it. There is a `verilog_keywords()` table
containing `buf`, a `verilog_safe_identifier()` emitting the `\name ` escape,
and three corpus specs — `w371_verilog_keyword`, `w372_local_keyword`,
`w374_module_keyword` — testing exactly this.

**The mechanism was correct and complete.** It was called at every expression
site and at both module-level array declarations. It was **not** called at the
two sites that emit a function-*local* array: its declaration and its
initialiser.

```verilog
reg [15:0] buf[0:3];     // before -- iverilog: syntax error
reg [15:0] \buf [0:3];   // after
```

> **T53 — correctness here is a *conjunctive* obligation over a set of emit
> sites that grows whenever one is added.** The presence of the escape, its test
> suite, and its correct application at `|S| − 2` sites is **no evidence at
> all** about the remaining two: a single unescaped site reproduces the full
> defect. **An escaping mechanism is only as good as its worst emit site, and
> nothing in the codebase makes `S` enumerable.**

---

## 2. Measured

| | before | after |
|---|---:|---:|
| real (non-fixture) iverilog rejections | 10 | **6** |

**Exactly the four identified as the keyword class.** And one further spec
moved rather than vanished:

```
w386_for_local_array_param
  before:  line 44: syntax error
  after:   line 48: register ``i'' unknown in …fill_n_body
```

**T19's unmasking, observed live** — the keyword defect was hiding an
undeclared-loop-variable defect in the same file.

---

## 3. Why this one is different

T43, T45, T48, T51 and T52 are all about what an artefact **claims**. **This is
the first defect since T18 where the output is simply wrong** — the backend
emits Verilog that its own simulator refuses, and no amount of honest reporting
improves it.

**It took running the phase those reports had been printing `0` for.** T51 found
that `Icarus simulation fails: 0` meant "not run" for nine waves; the very first
run of it surfaced a real correctness defect that twenty waves of auditing
reports had not. **A red gate nobody runs hides real defects, not just reporting
ones.**

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| new escaping test | passes — asserts decl **and** initialiser are escaped |
| `cargo test --bins` | 5 failures, **identical to the pre-W629 baseline** |
| probe: keyword-named local | `iverilog -g2012` **rc 0** (was: syntax error) |
| real iverilog rejections | **10 → 6** |
| corpus ratchet | **CLEAN**, 332/332, rc 0, 583 s |

The test asserts *both* sites, because escaping one and not the other is exactly
what produced the defect.

---

## 5. What was NOT done

- **Six rejections remain**: 2 missing-function, 2 undeclared loop variable,
  1 empty-identifier declaration, 1 array-return cast.
- **`S` is still not enumerable.** T53's falsification condition — a third
  unescaped emit site — is untested, and the absence of any enumeration
  mechanism makes it likely.
- **The other 15 Icarus failures** (9 module-level parse, 3 in-fn, 2 stale
  baseline, 1 genuine) are untouched.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W644)

### Option 1 — **Enumerate `S`: make the escape impossible to forget**

T53's real finding is not the two sites but that nobody can list them. Route
every Verilog identifier emission through one function, or add a debug assertion
that any emitted identifier matching `verilog_keywords()` is escaped, and let it
fail loudly on the next omission.

- **Cost:** medium; a refactor or a single assertion plus a corpus run.
- **Pays off in:** the class stops being findable-only-by-simulation. It is the
  mechanical remedy T49 argued for, applied to T53.
- **Risk:** a blanket assertion may fire on legitimate raw emissions (comments,
  string literals containing a keyword) and need narrowing — which is the work.
- **Confirming measurement:** the assertion is live and the corpus produces zero
  violations, or the violations it names are fixed.

### Option 2 — **The remaining six rejections**

2 × missing function, 2 × undeclared loop variable, 1 × empty-identifier
declaration, 1 × array-return cast. The empty-identifier one (`reg [31:0] ;`)
is the most alarming: a declaration with no name at all, from `let` destructuring.

- **Cost:** medium; four distinct defects.
- **Pays off in:** Icarus rejections to zero for the non-fixture population.
- **Risk:** T38 — expect further unmasking; budget for a class table rather than
  a count, and re-triage after each fix.
- **Confirming measurement:** real rejections 6 → n, with the residue's shape
  characterised as T50 requires.

### Option 3 — **T50's 754 tests that run and check nothing**

Still the only population where a repair increases what is *verified* rather
than what is honestly *reported*. The classifier is per-item, so forecast the
yield first (T44).

- **Cost:** medium; a lowering gap in the `then` clause.
- **Pays off in:** 754 simulations that drive the circuit and assert nothing
  become real checks.
- **Risk:** some `then` clauses may be inexpressible in Verilog, making the
  honest outcome a `NOT CHECKED` marker rather than a check — a reporting fix in
  a repair's clothes. Say which it turned out to be.
- **Confirming measurement:** vacuous blocks 754 → n.

**Recommendation: Option 1.** Option 2 fixes six files; Option 1 stops the class
recurring, and T53's own falsification condition says a third unescaped site is
the way to bet. This session's clearest lesson (T49) is that the remedy for a
recurring class is mechanical, not mnemonic — and here the mechanism is cheap.

---

## Appendix — reproduction

```bash
./target/release/t27c gen-verilog-for-simulation <spec> > out.v && iverilog -g2012 -o /dev/null out.v
```

Group failures by reading the rejected **line**, not iverilog's message — six of
ten say only `syntax error`. Split `*_negative_*` fixtures out of the population
before counting.

**φ² + φ⁻² = 3 | TRINITY**
