# Wave Loop 644 — 171 against 4

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_643_REPORT.md`](WAVE_LOOP_643_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T53 predicted "a third unescaped emit site is the way to bet".
T54's gate collected it on the first corpus run.

  Verilog keyword decls: 438 clean, 171 with a bare keyword
                         ^^^ against the 4 iverilog had surfaced

  site: the `let` binding decl -- `reg [63:0] input;`
        `input` is a Verilog keyword.

  one verilog_safe_identifier call later:
  Verilog keyword decls: 609 clean, 0 with a bare keyword
  Ratchet CLEAN 332/332 -- no bless needed.
```

---

## 1. T54 — check the artefact, not the producers

**T53's real finding was not the two unescaped sites but that nobody can list
them.** Correctness of an escape is a *conjunctive* obligation over a producer
set that grows whenever an emitter is added; care at the known members is no
evidence about the unknown ones.

**So change what is checked.** Declared names are extractable from the generated
Verilog by a total function. The emit sites are not.

| | site audit | artefact audit |
|---|---|---|
| completeness | depends on enumerating `S` | **total over the output** |
| survives a new emitter | no | **yes** |
| survives a refactor | no | **yes** |
| localises the defect | to a site | to a line, **which names the site** |

**Verified by reverting the repair.** With W643's fix in place the gate is clean;
with the declaration site reverted:

```
FAIL verilog-no-keyword-decl (specs/mini/kw.t27):
  generated Verilog declares 1 identifier(s) that are Verilog keywords:
  line 44: `buf` declared unescaped
```

**The gate names, in milliseconds, the defect that took a 100-minute Icarus run
to surface in W643.**

---

## 2. And it paid immediately

First corpus run:

```
Verilog keyword decls: 438 clean, 171 with a bare keyword
```

**171, against the 4 iverilog had surfaced.** The site is the `let` binding
declaration (`t27#1948`):

```verilog
reg [63:0] input; // t27#1948 let binding
```

`input` is a Verilog keyword — and far likelier to appear as a spec variable
name than `buf` was.

> **The 171-versus-4 gap is T21 and T54 in one number.** Simulation sees only the
> specs it *reaches* — in the Icarus set, actually run, actually simulated. The
> artefact check is **total over the corpus**. Same defect class, two orders of
> visibility.

One `verilog_safe_identifier` call later:

```
Verilog keyword decls: 609 clean, 0 with a bare keyword
Ratchet: CLEAN  332/332  rc 0
```

**No bless required, because the fix landed in the same wave as the detection.**
That is the intended shape — a gate that finds a whole class at once, and a
repair that empties it before the ledger ever grows.

---

## 3. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| gate on fixed code | 1 clean, **0** bare keywords |
| gate with W643's fix **reverted** | **FAILS**, names `buf` at line 44 |
| corpus, before the third fix | 438 clean, **171** bare keywords |
| corpus, after | **609 clean, 0** |
| ratchet | **CLEAN**, 332/332, rc 0, 502 s — no ledger growth |
| `cargo test --bins local_array_named` | passes |

---

## 4. What was NOT done

- **The gate models only `reg`/`wire`/`integer` declarations.** Its own
  falsification condition — an unescaped identifier in a declaration form the
  scanner does not parse — is untested.
- **Five iverilog rejections remain** (2 missing function, 2 undeclared loop
  variable, 1 empty-identifier declaration; the keyword class is now closed).
- **No new Icarus run.** The 171→0 fix should reduce the 31 further, and that
  has not been measured — it needs another ~100 minutes.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 5. Three ways to continue (pick one for W645)

### Option 1 — **Re-run Icarus and measure what the 171 fix bought**

The keyword class is closed corpus-wide, but the only measurement of Icarus
failures is 31, taken before both fixes. Re-run and re-triage.

- **Cost:** ~100 minutes of waiting, near-zero of work.
- **Pays off in:** the first number in this chain that measures a *repair's*
  effect on simulation rather than on a static scan.
- **Risk:** T19 — expect unmasking; some of the 31 will move rather than vanish,
  and the residue's shape is the finding (T50).
- **Confirming measurement:** Icarus failures 31 → n, retriaged by rejected
  construct, with movement distinguished from disappearance.

### Option 2 — **Widen the artefact gate to every declaration form**

The scanner parses `reg`/`wire`/`integer`. Verilog declares identifiers in
`function`, `task`, `parameter`, `localparam`, `genvar`, port lists and
`for` initialisers. Each is a hole in a gate whose whole premise is totality.

- **Cost:** low; the parsing is mechanical.
- **Pays off in:** T54's falsification condition stops being open. A gate that
  claims totality and covers three of ten forms is T43's shape applied to a
  checker.
- **Risk:** more forms means more false positives; each needs the T47 treatment
  — report precision, not just count.
- **Confirming measurement:** the gate covers every declaration form the backend
  emits, enumerated from the emitter rather than guessed.

### Option 3 — **T50's 754 tests that run and check nothing**

Still the only population where a repair increases what is *verified* rather
than what is honestly *reported*.

- **Cost:** medium; a lowering gap in the `then` clause.
- **Pays off in:** 754 simulations that drive the circuit and assert nothing
  become real checks.
- **Risk:** some `then` clauses may be inexpressible in Verilog, making the
  honest outcome a `NOT CHECKED` marker — a reporting fix in a repair's clothes.
- **Confirming measurement:** vacuous blocks 754 → n, residue characterised.

**Recommendation: Option 2.** T54's gate just justified itself 171 times over,
and its own stated weakness is coverage of declaration forms. **A checker that
claims totality and has holes is the exact defect this session has documented
eleven times** — closing them while the argument is fresh costs little and
removes the one thing that could make T54 wrong.

---

## Appendix — reproduction

```bash
./target/release/t27c suite --repo-root . --corpus-only 2>&1 | grep 'Verilog keyword decls'
```

To verify the gate rather than the code: revert `verilog_safe_identifier` at any
declaration emit site, rebuild, and confirm the gate names that line.

**φ² + φ⁻² = 3 | TRINITY**
