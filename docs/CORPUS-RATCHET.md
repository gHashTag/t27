# The corpus ratchet — how to read it, and how to bless

**Status:** in force as of W632, via
[`.github/workflows/corpus-ratchet.yml`](../.github/workflows/corpus-ratchet.yml).
Ledger: [`reports/suite_expectations.json`](reports/suite_expectations.json).

---

## Why this exists

`t27c suite` reports `TOTAL FAILURES` in the thousands. W626 measured that a
gate whose baseline is already non-zero **cannot detect a regression**: a new
break lands inside the total and moves the exit code not at all. Three
end-to-end runs across three different compiler builds all reported **2614**,
term for term — the suite could not tell *"nothing changed"* from *"you broke
the compiler."*

The ratchet gates on an **identity-keyed ledger** instead. The unit of amnesty
is a `(path, phase)` pair, and the verdict is observed-versus-expected per
identity.

---

## The four ways it fails

| verdict | meaning | what to do |
|---|---|---|
| **UNEXPECTED FAILURE** | a corpus spec failed that is not in the ledger | fix it — or, if the break is intended and tracked, add an entry **by hand** and raise `max_entries` in the same commit |
| **UNEXPECTED PASS** | a ledger entry passed | **you fixed something.** Remove the entry and lower `max_entries` |
| **EXPIRED** | an entry is past its `expires` date | fix the spec, or renew the date with a reason in the PR |
| **OVER CAP** | the ledger outgrew `max_entries` | the cap only moves down automatically; raising it is a hand edit |

**An unexpected pass is a failure, and that is deliberate.** Gating only on new
breaks makes the ledger *monotone*: entries get added when defects appear and
never removed when they are fixed, because nothing observes the removal.
Discriminating power decays to zero — the same terminal state as a
never-updated baseline, reached by a different route. Every system in the field
that stays exact does this: LLVM `lit` counts XPASS as a failure, DejaGnu
accounts for it separately, TypeScript's `@ts-expect-error` errors when the next
line has none, Rust's `#[expect(lint)]` fires `unfulfilled_lint_expectations`.
pytest's `xfail_strict` exists because its default does not.

---

## Blessing a new expected failure

**`--bless-expectations` is the only writer.** Acquisition is never a side
effect of verification — a mode that can create the oracle must not be the mode
that checks against it.

```bash
cargo build --release -p t27c
./target/release/t27c suite --repo-root . --bless-expectations
```

Then **review the diff before committing it.** Every entry needs:

| field | rule |
|---|---|
| `path`, `phase` | the identity; `phase` is the **first** phase that rejected the file, never a downstream gated one |
| `reason` | what is actually wrong. `--bless` writes a placeholder; replace it |
| `issue` | a real tracking issue |
| `expires` | `YYYY-MM-DD`. **A past-due entry fails the run**, even when the sets agree |

`max_entries` moves **monotone downward**. Blessing a larger population writes a
ledger that immediately fails its own cap — by design: raising it must be a hand
edit in the pull request, which is the reviewable event.

### Faster: ratchet without a full run

Verified equivalent in W631 — the tool independently observed 173 against a
hand-ratcheted ledger of 173, zero unexpected either way. So after a fix you may
drop the now-passing entries directly and lower the cap, and let CI confirm:

```bash
# keep only the entries that still fail
python3 - <<'PY'
import json, pathlib, subprocess
p = pathlib.Path("docs/reports/suite_expectations.json"); d = json.load(p.open())
still = [e for e in d["entries"]
         if subprocess.run(["./target/release/t27c","parse",e["path"]],
                           capture_output=True).returncode != 0]
d["entries"] = sorted(still, key=lambda e: (e["path"], e["phase"]))
d["max_entries"] = len(still)
p.write_text(json.dumps(d, indent=2) + "\n")
print(len(d["entries"]))
PY
```

---

## What the ratchet does **not** cover

Read this before trusting a green run.

- **Only the hand-written corpus.** `specs/scratch/` is excluded — 606,113,688
  of 612,924,235 bytes (98.89%), generator output the ledger does not gate on.
  That exclusion is what makes the check 314 s instead of 4057 s with a
  bit-identical verdict.
- **Only the phases `suite` runs**: `parse`, `typecheck`, `gen-zig`,
  `gen-rust`, `gen-verilog`, `gen-c`, `seal-verify`, plus the smoke gates.
  **`parse-complete` and `lex-dropped` are not among them**, and `t27c parse`
  returns success on a file it did not fully consume — so appended garbage after
  the last valid construct is invisible to this gate. Verified: appending
  `))) … (((` to a corpus spec leaves the ratchet CLEAN; a mid-file break is
  caught and named.
- **Seal staleness is reported, not gated.** 1056 of 1064 seals are stale and
  ~940 carry an unchanged `spec_hash`; a ledger over golden-file drift would be
  debt, not a defect list.
- **`cargo test` is not run by `suite`.** There are 5 standing unit-test
  failures that have never appeared in any suite total.

---

## Reproduction

```bash
./target/release/t27c suite --repo-root . --ratchet --corpus-only --json out.json > run.log 2>&1
```

Expect `RATCHET: CLEAN` and rc 0. **Redirect; never pipe through `tail`** — it
buffers to end-of-stream and makes a working tool look silent for the whole run.

Full background: `theory/IGLA-FORMAL-RESULTS.md`, T27 (why a total carries no
signal), T30 (attribution before amnesty), T31 (blessing on absence), T32–T33
(the mechanism and its dual), T40 (the corpus-only equivalence), T41 (what the
phases cannot see).

**φ² + φ⁻² = 3 | TRINITY**
