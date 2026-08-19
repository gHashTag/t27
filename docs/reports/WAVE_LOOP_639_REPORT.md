# Wave Loop 639 — T35's error, one wave after T35

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_638_REPORT.md`](WAVE_LOOP_638_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T49  W638's backend table was pooled over specs where the backend
     emitted NOTHING AT ALL. Conditioned properly:

       gen (zig)        97% tests  99% invariants   (printed 64/68)
       gen-c            97%        99%              (64/68)
       gen-verilog      97%        99%              (64/68)
       gen-rust          7%        37%              (5/25)
       gen-verilog-hir   7%        30%              (5/21)

     The correction makes the finding STRONGER. And it is exactly the
     error T35 names, committed ONE WAVE AFTER T35, in the table
     demonstrating T48.

The differential is now a gate: `backends-declare-omissions`.
It found 2 -- and they are exactly the two `spec X { }` dialect files
that parse, with 100% of their checks dropped by every backend.
```

---

## 1. The correction

W638 published 64%/68% against 5%/25%. **The denominator pooled specs for which
the backend emitted nothing at all** — an empty output is a *different* failure
from a silently-dropped construct, and those specs do not belong in the
denominator of *"did this backend lower the construct?"*

| backend | tests | invariants | first printed as |
|---|---:|---:|---|
| `gen` (Zig) | **97%** | **99%** | 64% / 68% |
| `gen-c` | 97% | 99% | 64% / 68% |
| `gen-verilog` | 97% | 99% | 64% / 68% |
| `gen-rust` | **7%** | 37% | 5% / 25% |
| `gen-verilog-hir` | 7% | 30% | 5% / 21% |

**The correction sharpens the result** — 97/99 against 7/30, not 64/68 against
5/25. Second time this session a correction has strengthened rather than
softened a finding (cf. T34 → T35).

Both the W638 report and the theory document are corrected in place.

---

## 2. T49 — why writing it down did not work

T35's own criterion:

> *"When some `Pᵢ` fail **by construction** — the measurement is undefined on
> them, not merely adverse — the remedy is not a better estimator; it is
> refusing to pool."*

A spec whose backend emitted nothing is a `Pᵢ` on which *"was this construct
lowered?"* is **undefined**. I wrote that criterion in W635 and violated it in
W638.

> **T49 — observing a fresh instance of a documented failure mode at
> `w(L) + 1`, produced by the author of `w(L)`, is evidence that documentation
> does not transfer to the author's own next artefact.** The mechanism is
> **availability, not ignorance**: `for spec: for backend: count` is the loop
> you naturally write, and conditioning needs a branch the lesson does not make
> salient at the moment you are writing the loop.

**Nine instances of syntactic-for-semantic selection are now recorded** — T16,
T20, T24, T29, T34, T35, T47's detector, T49, and W636's ledger scrape — and
**not one was prevented by having written the previous one down.** What has
actually caught them, every time, is **re-measurement by a different route.**

**So the remedy is mechanical, not mnemonic**, which is what the rest of this
wave builds.

---

## 3. The differential, as a gate

`backends-declare-omissions`: for every `test`/`invariant` a spec declares, each
backend must **either lower it or carry `NOT LOWERED BY THIS BACKEND`**. Silence
fails. The phase conditions correctly *by construction* — a backend that
produced no output is skipped, because the question is undefined there.

**Result: 2 primary failures**, and they are the right two:

```
FAIL backends-declare-omissions (specs/ar/coa_planning.t27):
  gen: 13 of 13 silently absent; gen-rust: 13 of 13; gen-verilog: 13 of 13
FAIL backends-declare-omissions (specs/ar/restraint.t27):
  gen: 19 of 19 silently absent; gen-rust: 19 of 19; gen-verilog: 19 of 19
```

**These are exactly the two `spec X { … }` dialect files that parse** — T35's
table records the dialect as "2 parse, 6 fail". They parse, all three backends
emit output, and **100% of their declared checks are dropped by every one**,
because the codegen does not recognise the dialect's blocks.

**Only 2, because W638's Rust header already closed the bulk.** A backend that
declares its omission passes the gate — which is the design working: the fix
made in W638 is what the W639 gate now enforces.

**The T46 check was applied before blessing:** the verdict said
`UNEXPECTED FAILURES: 2` and the list showed 2 — count and list agree, so the
list is complete and safe to consume. That comparison is exactly what T46 says
to do and what I failed to do in W636.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| backend coverage, conditioned | 97/99 · 97/99 · 97/99 · 7/37 · 7/30 |
| new phase | **2 primary**, 2 blocked |
| the 2 | both `spec X { }` dialect files, 13/13 and 19/19 dropped |
| ledger | 330 → **332**, cap hand-raised |
| W638 report + theory doc | corrected in place |
| re-verification | **CLEAN**, 332/332, rc 0, 368 s |

---

## 5. What was NOT done

- **`gen-verilog-hir` is not in the gate's backend list** — it is a second
  Verilog path whose reachability in the suite I have not established.
- **The `spec X { }` dialect is still unlowered.** The gate now names it; the
  language decision (migrate the 8 files, or teach the codegen the dialect)
  is untouched.
- **The Verilog false `PASSED` and the C inflated count remain** — both need
  their oracle re-blessed (108 Icarus baselines; `cc-gate`).
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a
  provider error for this entire session; everything named is described from
  general knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W640)

### Option 1 — **Repair the two dishonest emit sites and re-bless their oracles**

`gen-verilog`'s unconditional `PASSED` (3 429 blocks) and `gen-c`'s inflated
count. Both change generated output that a gate consumes, so both need the
oracle regenerated in the same change — and `save_icarus_baseline` must stop
recording on absence first (T31), or the re-bless is unaudited.

- **Cost:** low in code, medium in review.
- **Pays off in:** after seven waves of finding this class, it is the change
  that ends it in the artefacts that matter.
- **Risk:** the T31 self-blessing bug makes an unaudited re-bless possible;
  fixing it is a precondition, not a follow-up.
- **Confirming measurement:** vacuous Verilog blocks 3 429 → 0; 108 baselines
  **modified, none created**; `cc-gate` re-run clean.

### Option 2 — **Decide the `spec X { }` dialect**

Eight files, two of which parse and lower nothing. Either migrate them to the
`module` dialect or teach the codegen their blocks. T35 measured the dialect;
T49's gate now names the consequence.

- **Cost:** low if migrating 8 files; medium if teaching the codegen.
- **Pays off in:** removes an entire silent-drop population and one of the five
  file kinds that make every corpus aggregate a mixture.
- **Risk:** the 6 that do not parse may not be mechanically migratable, so
  expect a split answer and state the boundary.
- **Confirming measurement:** `backends-declare-omissions` primary → 0, and the
  kind table in T35 loses a row.

### Option 3 — **Make the gate suite-wide instead of per-construct**

The gate checks names appearing in output. A stronger form: assert that the
*set* of constructs each backend lowers is identical across backends, modulo
declared omissions — turning T48's oracle from "did anything vanish" into "do
the backends agree".

- **Cost:** medium; a set comparison rather than a membership test.
- **Pays off in:** catches divergence *between* backends, not just against the
  source — which is where T45 and T48 both actually lived.
- **Risk:** the three real backends already agree at 97/99, so this may find
  very little; that would itself be a bounding result worth having.
- **Confirming measurement:** a per-spec table of symmetric differences between
  backend construct sets, summing to a number the report states.

**Recommendation: Option 1.** T49 says the remedy for this class is mechanical
rather than mnemonic, and the mechanism is now in place — the gates exist.
What remains is that two artefacts still emit false and inflated claims, and
every wave that leaves them is a wave in which the repository's outputs
misrepresent themselves.

---

## Appendix — reproduction

```bash
./target/release/t27c suite --repo-root . --ratchet --corpus-only
```

Look for the `backends-declare-omissions` row. For the coverage table, count
declared `test`/`invariant` names appearing in each backend's output —
**conditioned on specs where that backend produced output at all**, which is the
whole point of T49.

**φ² + φ⁻² = 3 | TRINITY**
