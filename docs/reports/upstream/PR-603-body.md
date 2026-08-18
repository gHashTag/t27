Follow-up to #601, which you merged while this was being measured. Three commits,
each independent of the others; drop any one without disturbing the rest.

## 1. An oracle for `tab:invariant` (71 cells, previously none)

The table is a **filtered view** of a record already in the tree:
`pair == "TNF(4,8)/takum16" AND comparable == true` selects 30 rows for 30 printed
rows. `recompute_invariant_table.py` reproduces all 90 data cells at printed
precision plus the row formatting — at the record's tolerance 0.02 the rows split
12 wins / 4 ties / 14 losses and the table prints 12 bold, 4 daggered, 14 plain.

Two defects on its first clean run:

- **caption sample count** — said `$7000$ samples per row`; the record carries
  `n=5000` under the same seed `20260813`. Changed to `$5000$`, and the check now
  lives in the script.
- **`tab:window`, c=4, binary16** — printed `1.69e-4`; the record holds
  `1.684663e-4` → `1.68e-4`. The row's other five cells are exact.

## 2. An oracle for `tab:rungthr`, whose columns come from TWO records

The first table here that no single record explains — which is why
`per_rung_2026-08-13g.json` resisted placement for so long. It backs one *column*.

    column      source
    reach       (3^E_t-1)/2 - 1, checked against the oracle
    cells       rows[].tnf_phys, constant per pair        (strict_range)
    comp.       summary_tie_aware[i].comparable           (strict_range)
    threshold   midpoint of [max_D_loss, min_D_win]       (strict_range)
    separates?  separates + max_D_loss/min_D_win/wins     (strict_range)

`summary_tie_aware` is **not taken on trust** — it is derived inside the file it
summarises, so believing it would only check that a file agrees with itself. Every
field used is recomputed from the 180 raw rows at the record's stated tolerance
0.02 and only then compared to the table.

The threshold column is a **rule, not a field**: no record holds 9.5 or 21.9. Both
are the arithmetic midpoint of the bracketing pair. Two points would normally
underdetermine such a rule, but they discriminate — the geometric mean gives 21.8
where the table prints 21.9.

## 3. The reach column is off by one in all three rows

This is the one worth your attention.

    conformance/tnf_ref.py, TNFFormat(exp_trits=4, mant_bits=8)
        decode(encode(2^39))  ->  5.498e+11, finite
        decode(encode(2^40))  ->  the special value

    (5,23): 2^120 finite, 2^121 special.   (6,21): 2^363 finite, 2^364 special.

So the reach is **39 / 120 / 363**. Your own Proposition `prop:uncentred` says so
and proves it: *"the representable binade indices are −(Δ−1) … +(Δ−1)"* — the
offset field takes 3^E values, the top row is reserved for the special value and
the bottom for zero, leaving 3^E−2 rows.

`per_rung`'s `tnf_reach` field records 40/121/364, which is **Δ, the offset
constant**. Six sites print it as the reach:

    :941   "TNF16's exponent reaches $\pm40$"           -> $\pm39$
    :1394  "already clips nothing at $\pm 40$ binades"  -> $\pm 39$
    :1395  "would have bought $\pm 364$ binades"        -> $\pm 363$
    :6359-61  the three reach cells                     -> 39 / 120 / 363

**Why a reconstruction did not catch this.** The first version of
`recompute_rungthr_table.py` checked all 18 cells, confirmed reach against
`tnf_reach` *and* against the closed form `(3^E-1)/2`, and passed — because the
table and the record hold the same wrong quantity, and the two "independent"
checks are the same number by construction. The script now takes the reach from
the oracle, asserts `2^(Δ-1)` decodes finite and `2^Δ` saturates, and separately
asserts `per_rung` still holds Δ, so a later edit that "fixes" the record to match
the old table fails here rather than restoring the defect quietly.

### Provenance captions, in the two commits above

`tab:invariant` and `tab:rungthr` now name their records and regenerators, in the
form #601 established. `tab:rungthr` names two files and says which column each
covers — the first caption here that has to.

---

# Reported, not patched — these need your judgement

## `tab:window` applies its suppression standard in one direction

Measured over every row of `crossover2_2026-08-13e.json`:

| c | TNF clipped | binary16 clipped | binary16 cell |
|---|---|---|---|
| 16 | 0 / 9000 | **4509 / 9000 = 50.1 %** | **suppressed** (record holds 1.677e-4) |
| 40 | **4465 / 9000 = 49.6 %** | 9000 / 9000 | TNF's value **printed** |

The same clipping fraction hides the competitor's number at c=16 and publishes
TNF's at c=40. The suppressed cell is one where `binary16` is four times more
accurate on its unclipped half — its ratio is 0.2479, the same 0.25 the prose
calls "exactly the two mantissa bits" of advantage.

The standard itself is sound: a mean over the unclipped half of a sample is
biased. We are not proposing which way to resolve it — dash TNF at c=40, or state
in the caption that the c=40 figures are survivor means — because that is an
editorial call about your own comparison.

## `tab:window`'s caption states a rule the table does not follow

The caption says binary16 "has no representation at all from $c=20$ upward, marked
---". At c=16 the column is dashed and binary16 represented 4491 of 9000 samples.
The prose at :6167 repeats it: "the column only becomes empty because
`binary16` runs out of exponent."

Suggested wording: a dash marks a mean that is not a comparison — either the
format represents nothing (c ≥ 20) or it clipped half the bulk (c = 16).

## Two measured rows are absent with no mark

The record holds c = 0,4,8,12,16,20,**24**,28,**32**,36,40; the table prints nine,
jumping 20→28 and 28→36. Both missing rows are fully finite in the TNF and takum
columns with no clipping, and their ratios (2.01 and 3.49) sit inside the printed
envelope. No ellipsis, no stated selection rule.

## `measurements/README.md` misdescribes three of its entries

- `inside_window_2026-08-13f.json` holds the eleven GPT-2 block-0 forward-pass
  intermediates — the rows of `tab:landing`. The README attributes those to
  `gpt2_window`, which actually holds two weight tensors and backs
  `tab:gpt2window`.
- Every one of `inside_window`'s 11 rows carries `inside: false`, while the README
  describes the file as "rows inside the window". The paper's own claim is the
  opposite: *"Inside neural inference there are none."*
- "backs the qualifying-pair count" is attributed to `strict_range`, which has no
  `qualifies` and no `workload` field. `workloads_strict` has both, 7 of 22 rows
  qualifying.

## One methodological note

Six regenerators draw the exponent as `_rng.integers(-38, 39)` — uniform over 77
binades. The paper's central claim is flat precision across range, which is what
that prior rewards. The *range* is disclosed (bin labels, `|e| <= 38`); the
uniformity and its bearing on the result are not discussed anywhere —
"sampling prior", "the prior", "choice of workload", "depends on the
distribution" all have zero occurrences. A referee will ask.

## Verification

    recompute_invariant_table.py   OK: 30 rows, 90 cells, formatting, sample count
    recompute_rungthr_table.py     OK: 3 rows, 18 cells, summary re-derived from
                                       180 raw rows, reach from the oracle
    pdflatex                       136 pages, rc=0, 0 undefined references

Coverage: **10 of 59** numeric tables now have a shipped script that reads
`tnf_paper.tex`, compares cell by cell, and exits nonzero on disagreement —
against 4 before #601.

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
