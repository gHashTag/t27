# Wave Loop 632 — the gate is in force, and it is blind in one direction

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_631_REPORT.md`](WAVE_LOOP_631_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T40  --corpus-only: 4057 s -> 314 s, verdict BIT-IDENTICAL.
     12.9x, no trade-off -- the cost had been paid for results the
     verdict was discarding. A nightly becomes a per-PR gate.

T41  a ratchet is exactly as blind as its phases. `t27c parse` returns
     0 on trailing garbage, so appended junk is invisible to the gate.
     Found by trying to verify it and getting the wrong answer.

Wired: .github/workflows/corpus-ratchet.yml (blocking, per-PR).
Documented: docs/CORPUS-RATCHET.md, including what it does NOT cover.
```

---

## 1. T40 — the speedup that cost nothing

W631 left the gate working but 68 minutes long, and I flagged the risk myself:
*"a nightly that nobody reads is worse than nothing."*

The ratchet gates on **primary corpus** failures only. Walking `specs/scratch/`
therefore produces results the verdict discards.

| | full walk | `--corpus-only` |
|---|---:|---:|
| bytes walked | 612 924 235 | **6 810 547** (1.11%) |
| wall time | 4057 s | **314 s** |
| ledger / observed | 173 / 173 | **173 / 173** |
| unexpected failures · passes · expired | 0 · 0 · 0 | **0 · 0 · 0** |
| verdict | `CLEAN`, rc 0 | **`CLEAN`, rc 0** |

**Soundness is one line:** a scratch file can only ever block *itself*, so it
never enters a corpus file's attribution. That is a property of W627's per-file
attribution, not an assumption about the corpus.

> **T40 — restricting a walk to the sub-population the verdict reads is
> semantics-preserving, not an approximation.** The engineering content is not
> the 12.9× but that it required **no trade-off**: the cost had been paid for
> results that were being thrown away.

Now wired as [`corpus-ratchet.yml`](../../.github/workflows/corpus-ratchet.yml)
— blocking, on `pull_request` and `push` to master, with a failure step that
explains all four verdicts and how to bless. Procedure documented in
[`CORPUS-RATCHET.md`](../CORPUS-RATCHET.md).

---

## 2. T41 — and then it failed to catch a break

I appended `))) W632 deliberate break (((` to `specs/igla/race/ternary_mac.t27`
and ran the gate, expecting `UNEXPECTED FAILURE`.

```
RATCHET: CLEAN      rc = 0
```

**The gate was right.** `t27c parse` returns **0** on that file — the parser
stops at the last valid top-level construct and does not require EOF. Trailing
garbage is not a parse error; it is **silent truncation**, the class this
document has recorded since W559 and W577 (7 623 test bodies, then 16 792 lines,
discarded behind a stray brace).

A mid-file break *is* caught:

```
UNEXPECTED FAILURES: 1
  + specs/igla/race/ternary_mac.t27 [parse]
RATCHET: FAIL       rc = 1       315 s
```

> **T41 — a ratchet's sensitivity is bounded above by the union of its phases'
> sensitivities.** No property of the ledger, the cap, the expiry, or the
> unexpected-pass rule can raise that bound. **The amnesty mechanism is
> orthogonal to coverage.**

**And the compiler ships the detectors.** `t27c parse-complete` exists precisely
to report *"specs the parser accepts WITHOUT consuming the whole file"*;
`t27c lex-dropped` reports characters the lexer discards. The phases `suite`
runs are `parse`, `typecheck`, `gen-zig`, `gen-rust`, `gen-verilog`, `gen-c`,
`seal-verify`, `gen-verilog-yosys-smoke`, `fpga-smoke-gate-standalone`,
`fixed-point`. **Neither detector is among them.**

§4's standing rule is *a stage that cannot fail cannot be trusted*. `parse`
**can** fail, so it passed the smell test — but it cannot fail *on this input
class*, and a gate built on it inherits the hole exactly. **A good gate over an
incomplete predicate set produces confident green, which is worse than no gate,
because the confidence is now mechanised.**

---

## 3. The method note that matters

My first verification attempt was wrong, and the way it was wrong is the
session's recurring lesson: **I checked the gate with a perturbation I had never
confirmed the underlying predicate rejects.** The instrument produced the
observation (T26), again.

The rule, now in the skill: before using a deliberate break to verify a gate,
run the *phase predicate itself* on the corrupted file and confirm it fails.
`t27c parse <file>` → rc 1, **then** run the gate.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `cargo test --bins suite::` | 26 passed |
| `--ratchet --corpus-only`, clean tree | `CLEAN`, rc 0, **314 s** |
| same, full walk | `CLEAN`, rc 0, 4057 s — **identical verdict** |
| mid-file break | `FAIL`, rc 1, **names the file**, 315 s |
| trailing-garbage break | `CLEAN` — **and `t27c parse` agrees**, T41 |
| standing unit failures | 5, unchanged |

---

## 5. What was NOT done

- **`parse-complete` is not a suite phase.** That is T41's fix and Option 1.
- **The workflow has never run.** It is committed, not pushed; enabling it in
  the repository's CI is the maintainer's call.
- **The 173 remain**, a long tail of ~147 causes (T37) with yields below 1
  (T38).
- **The 26 non-`module` files still sit in `specs/`** (T35).
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a
  provider error for the entire session; everything named is described from
  general knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W633)

### Option 1 — **Close T41: add `parse-complete` as a gated phase**

One more phase over the same 609 files — marginal cost is one process spawn per
file on a check that already spends ~4 200. Then re-bless: every spec the parser
silently truncates becomes a named ledger entry instead of an invisible pass.

- **Cost:** low. The subcommand exists; it needs wiring and a bless pass.
- **Pays off in:** closes the exact hole this wave found, and the hole is a
  class that has cost this project 24 000+ silently discarded lines historically.
- **Risk:** the ledger may grow sharply — every silently-truncated spec is
  currently counted as passing. That growth trips `max_entries` by design, and
  the raise is the reviewable event. **Expect the corpus number to get worse,
  and that is the point.**
- **Confirming measurement:** append garbage to a corpus spec; the gate must go
  red and name it.

### Option 2 — **Audit the phase set against the subcommand set**

T41 found one missing detector by accident. `t27c --help` lists ~40 subcommands;
`suite` runs 10 phases. Enumerate the gap deliberately and decide, per
subcommand, whether it belongs in the suite — the same *"ask each stage to
account for its input"* rule applied to the harness itself.

- **Cost:** medium; ~30 judgements, each needing a read.
- **Pays off in:** the only systematic answer to "what else can the gate not
  see?", rather than waiting for the next accident.
- **Risk:** it will find more holes than there is appetite to fix, and a list of
  known-unchecked properties can become its own normalisation of deviance.
  Record them as ledger entries with expiries, not as prose.
- **Confirming measurement:** a written table, subcommand × in-suite, with a
  reason for every "no".

### Option 3 — **Work the 173 by file**

T37 and T38 say class-based planning does not work here. File-by-file, ledger
shrinking monotonically, `newly broken = 0` enforced by the gate that now exists.

- **Cost:** linear in files — measured progress rather than forecast progress.
- **Pays off in:** the strategy the evidence supports, and every step is now
  automatically checked.
- **Risk:** slow, and the temptation to re-derive a class shortcut returns.
- **Confirming measurement:** ledger falls monotonically, one commit per batch.

**Recommendation: Option 1.** T41 is a hole this wave opened by accident and left
open; leaving it while working the tail would mean the tail is being verified by
a gate that cannot see one of its largest historical failure modes. It is also
the cheapest of the three.

---

## Appendix — reproduction

```bash
./target/release/t27c suite --repo-root . --ratchet --corpus-only --json out.json > run.log 2>&1
```

To verify the gate catches a break: corrupt a spec **mid-file** (appending is
not enough — T41), confirm `t27c parse <file>` exits 1, then run the gate and
check it names that path.

**φ² + φ⁻² = 3 | TRINITY**
