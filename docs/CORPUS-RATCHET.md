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

## The seven ways it fails

| verdict | meaning | what to do |
|---|---|---|
| **UNEXPECTED FAILURE** | a corpus spec failed that is not in the ledger | fix it — or, if the break is intended and tracked, add an entry **by hand** and raise `max_entries` in the same commit |
| **UNEXPECTED PASS** | a ledger entry passed | **you fixed something.** Remove the entry and lower `max_entries` |
| **EXPIRED** | an entry is past its `expires` date | fix the spec, or renew the date with a reason in the PR |
| **OVER CAP** | the ledger outgrew `max_entries` | the cap only moves down automatically; raising it is a hand edit |
| **DISCARD WORSENED** | a `parse-no-discard` entry threw away MORE tokens than it is pinned at — or this run took no reading at all | fix it, or bless and justify the rise in the PR |
| **DISCARD IMPROVED** | it threw away FEWER | **you fixed something.** Re-bless so the new, lower number is what the next run is held to |
| **DISCARD UNPINNED** | a `parse-no-discard` entry carries no `discard_tokens` | bless once; an amnesty with no bound is what that field exists to end |

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

- **Only the hand-written corpus.** `specs/scratch/` is excluded — and since
  `2255e4c32` (*"chore(specs): untrack specs/scratch — 455 files, 578 MB, 64.5%
  of the tree"*, #2283) **that directory is not in the tree at all**: 0 files in
  the index, absent from disk. The exclusion excludes nothing today, and the
  saving it was written to justify — "314 s instead of 4057 s" — was banked by
  the untracking, not by the filter. The corpus the ratchet walks is **650 files
  / 7,030,194 bytes** as of this note; the 606,113,688-of-612,924,235 figure
  described a tree that no longer exists. Note also that the untracking commit
  puts the same exclusion at **64.5%** where this line put it at **98.89%** —
  two numbers for one subject, and neither is re-takable now.
  The filter itself is kept: it costs nothing and it is what makes the
  exclusion explicit if the directory ever comes back.
- **Only the phases `suite` runs**: `parse`, `parse-no-discard`, `typecheck`,
  `gen-zig`, `gen-rust`, `gen-verilog`, `gen-c`, `seal-verify`, plus the smoke
  gates. `lex-dropped` is not among them.

  **This bullet used to say `parse-complete` was not among them either, and that
  appending `))) … (((` to a corpus spec left the ratchet CLEAN.** Both are now
  false, and the correction is dated 2026-08-29: `parse-no-discard` is a phase and
  runs the same accounting `parse-complete` reports. Re-verified by doing it —
  appending `))) foo bar (((` to `specs/account/auth.t27` produces

  ```
  UNEXPECTED FAILURES: 1
    + specs/account/auth.t27 [parse]
  ```

  A document that lists what a green run does not cover is the last place a
  stale claim should sit: it is read exactly when someone is deciding how far to
  trust a pass.
- **Seal staleness is reported, not gated.** 1056 of 1064 seals are stale and
  ~940 carry an unchanged `spec_hash`; a ledger over golden-file drift would be
  debt, not a defect list.
- **`cargo test` is not run by `suite`.** It still is not — but the five
  standing unit-test failures this bullet used to name are gone: measured
  2026-08-29, `cargo test --no-fail-fast` is 2429 passed, 0 failed. The flag
  matters: without it `cargo test` stops at the first failing binary, and every
  total taken without it was partial.

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

---

## The amnesty carries a number (W699, 2026-08-29)

An entry is an identity — `(path, phase)` — and for six phases that is the whole
truth: a spec either parses or it does not. For `parse-no-discard` it is not.
That phase's failure message has always carried a magnitude:

```
parser reached EOF but DISCARDED 208 top-level token(s); they never reach codegen
```

and the ledger threw the number away. A spec could go from discarding one token
to discarding six hundred and eighty-two without moving a gate. The blindness
runs both ways: two parser fixes on 2026-08-29 recovered **1 292 tokens** across
the corpus and nothing could price it, because the population was 87 either way.

So `parse-no-discard` entries now carry `discard_tokens`, and the ratchet
compares it:

```json
{
  "path": "specs/isa/ternary_deque.t27",
  "phase": "parse-no-discard",
  "discard_tokens": 1873
}
```

Three rules, matching the ones already here rather than inventing new ones:

- **more is a failure** — the regression signal that did not exist
- **less is also a failure** — same reason an unexpected PASS is one. Unclaimed
  slack is where the next regression hides. Re-bless to pin the lower number.
- **no reading is treated as WORSE, never as an improvement.** A spec that
  stopped being measured and a spec that discards nothing look identical from
  the ratchet's side, and defaulting the map to zero would have reported every
  unreadable spec as a triumph.

`t27c suite --bless-expectations` is still the only writer, and it writes what
the run measured — so lowering is automatic on a re-bless, and raising is a diff
a human reads.

### How this compares to the field

Notion's eslint ratchet records per-file how many exceptions are allowed and
**decreases the counts automatically** as issues are fixed. That is the same
shape one decision apart: there, an improvement silently tightens the bound;
here it fails the run until someone blesses it.

The difference is deliberate and it is the same choice `xfail_strict` makes. An
automatic tightening is invisible in review — nobody sees the improvement, and
nobody notices when the tool tightens the wrong thing. A failing run that says
*"you fixed something, pin it"* costs one command and produces a diff.

### Finding the next one

```bash
tri discard top --n 15
```

Ranked by tokens thrown away, with the pinned bound beside each. The count of 87
does not say where to start; `specs/isa/ternary_deque.t27` at 1 873 tokens does.
It reads `t27c parse-complete` rather than re-implementing the accounting: a
second implementation of a measurement is a second number to disagree with the
first.
