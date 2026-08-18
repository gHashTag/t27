Follow-up to #601. Eleven commits, each independent — drop any without disturbing
the rest. Rebased on `ae868cfe`.

## Paper corrections

**1. `tab:invariant` has a regenerator** (`recompute_invariant_table.py`). The table
is a filtered view of a record already in the tree: `pair == "TNF(4,8)/takum16" AND
comparable == true` selects 30 rows for 30 printed rows. All 90 cells reproduce at
printed precision, plus the formatting — at the record's tolerance 0.02 the rows
split 12/4/14 and the table prints 12 bold, 4 daggered, 14 plain.

Two defects on its first clean run: the caption said `$7000$ samples per row` where
the record carries `n=5000` under the same seed (fixed to `$5000$`), and
`tab:window` at c=4 printed `1.69e-4` for a record value of `1.684663e-4` → `1.68e-4`
(the row's other five cells are exact).

**2. `tab:rungthr` has a regenerator**, and is the first table here that no single
record explains — which is why `per_rung_2026-08-13g.json` resisted placement:
it backs one *column*. `summary_tie_aware` is not taken on trust; every field is
recomputed from the 180 raw rows at the record's own tolerance.

**3. The reach column is off by one, in nine places.** The shipped oracle decides it:

    TNFFormat(4,8):  decode(encode(2^39)) → 5.498e+11 finite
                     decode(encode(2^40)) → the special value
    (5,23): 2^120 finite, 2^121 special.  (6,21): 2^363 finite, 2^364 special.

Your own `prop:uncentred` says the same and proves it: *"the representable binade
indices are −(Δ−1) … +(Δ−1)"* — the offset field takes 3^E values, top row reserved
for the special value, bottom for zero, leaving 3^E−2. `per_rung`'s `tnf_reach`
records **Δ, the offset constant**. Six sites printed it as the reach, and the 743
lines of `ba01de89` added three more.

*Why a reconstruction did not catch it:* the first version of
`recompute_rungthr_table.py` checked all 18 cells, confirmed reach against
`tnf_reach` **and** against the closed form `(3^E−1)/2`, and passed — because the
table and the record hold the same wrong quantity, and the two "independent" checks
are one number by construction. The script now takes the reach from the oracle and
separately asserts `per_rung` still holds Δ, so a later edit that "fixes" the record
to match the old table fails loudly instead of restoring the defect.

**4. Two works appear twice in the bibliography**, each cited once under each key,
five thousand lines apart: `fibbinary`/`fiandaca2025fibbinary` (arXiv:2511.01921) and
`wintersteiger2025`/`wintersteiger2025formal` (one DOI). **The Wintersteiger pair
disagrees with itself about its page range** — 157–166 against 157–160 at the same
DOI. Merged 92 → 90, 0 dangling, 0 uncited. `tools/check_bibliography.py` added,
matching on normalised title *and* on DOI/arXiv id, with a passing negative test.

## Measurements, offered as data rather than corrections

Different part (`xc7a200tfbg676-1`, QMTech Wukong V1), generated per-bit IOSTANDARD
xdc with no package pins, local yosys. **None of these figures is comparable to a
published one and none is offered as a replacement.**

**5. Seed dispersion is wider than a median suggests.** 21 designs on the
full-observation `w_*.v` harnesses, five placer seeds each: dispersion runs 1.6 % to
41.7 %. Every hardware caption states median-of-five, which is the right estimator;
none states the dispersion that median summarises.

**6. The placer/router pair moves Fmax up to 4.3×.**

|  | heap/router1 | heap/router2 | sa/router2 | best/worst |
|---|---|---|---|---|
| `w_baseline` | 997.0 | 874.9 | 873.4 | 1.14× |
| `w_gfternary` | 675.7 | 509.9 | 182.3 | **3.71×** |
| `w_tnf16` | 642.7 | 584.1 | 148.3 | **4.33×** |
| `w_posit16` | 134.8 | 121.1 | 38.8 | 3.48× |

Between-configuration spread reaches 96.7 % of the median; the widest
within-configuration seed spread here is 41.7 %. **`placer` occurs 0 times in
`tnf_paper.tex`, and so does `router`.** The captions name the tool and its commit,
the part, the DSP setting and the seed count — everything except the largest knob.

This is not pedantry: `tnf-cost-sweep.yml` tries the three configurations **in order**
and keeps the first that routes, so a published median is a median over seeds within
whichever configuration happened to succeed, and two designs that succeeded under
different configurations differ by a knob worth up to 4.3×. We are **not** claiming
any published figure came from `sa/router2` — if `heap/router1` routes, the fallback
never fires. The claim is that the caption does not let a reader tell.

---

# One question only you can answer

**`tab:cleandecode`'s control is larger than two of its entries.**

The caption states it plainly, which is why this is a question and not an accusation:
*"a bare wire in the same harness costs 112 LUT at 827.81 MHz."* The two cheapest
rows are GFTernary at 66 LUT and `int8` at 76.

Instantiating a decoder cannot make a harness smaller than the same harness
containing a wire. On the full-observation `w_*.v` family here, all 21 entries sit at
or above the 14-LUT empty control, which is the expected relation. The absolute
numbers differ by flow; **the relation does not**.

Two readings, and we cannot choose between them from outside:

- the "same harness" differs between the control and the entries — the bare wire
  carries 32 bits to the output where a narrow decoder carries fewer, so the control
  is measuring more than the entries are; or
- the control row and the entries come from different revisions of the harness.

A sentence in the caption naming what the bare wire carries would close it.

Incidentally, `w_int8` here equals the empty control **to the LUT**, with full
observation and nothing pruned — an int8 decode is a sign-extend the output register
absorbs. That is a property of int8, not of the measurement, and it makes `int8` a
poor floor to read other rows against.

## Verification

    recompute_invariant_table.py   OK: 30 rows, 90 cells, formatting, sample count
    recompute_rungthr_table.py     OK: 3 rows, 18 cells, summary re-derived from
                                       180 raw rows, reach from the oracle
    check_bibliography.py          OK: 90 entries, 0 dangling, 0 uncited, 0 duplicates
    pdflatex x3                    143 pages, rc=0, 0 undefined references

`check_ref_kinds`, `check_withdrawn_live` and `check_self_consistency` exit 1 on
`ae868cfe` itself, before any of these commits. `check_withdrawn_live` reports 16 —
the new text adds withdrawal passages, so `tools/withdrawn_live_baseline.txt` needs
regenerating against the current paper exactly as it did in #601.

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
