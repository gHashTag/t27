# TNF audit — the complete ledger

One page holding everything this audit found, fixed, reported and withdrew, so a
review of PR #603 does not have to walk fourteen commit messages. Every claim
below is measured; the measurement lives in the named commit or record.

**G8 cannot be closed from this bench** (measured 2026-08-19): the CI flow pins
`docker regymm/openxc7` on `xc7a200tfbg484-2`; the local Docker daemon is not
running (GUI app, not startable autonomously) and 3.5 GB free disk cannot hold the
image; the local chipdb is the QMTech `fbg676-1` — a different package and speed
grade, measured earlier to be non-substitutable. Closing G8 takes either one
`tnf-cost-sweep` dispatch on CI or the author's own logs for the sixteen
`tab:untraced` frequencies.

**Status frame: the release verdict is NO-GO on gate G8** (post-route evidence
absent for the sixteen published frequencies in `tab:untraced`) — the other
track's finding, and nothing here closes it. Everything below is beneath that
gate.

---

## 1. Document defects — fixed and merged (PR #601, merged by owner 2026-08-18)

| # | where | was → is | confirmed by |
|---|---|---|---|
| 1 | twelve `Section~\ref` | resolved to FIGURE numbers → seven `\label{sec:}` moved to their headings | `check_ref_kinds` 19 → 7 |
| 2 | `tab:law`, TNF8 row | `1.11e-2 / 0.36 / 1.082` → `1.54e-2 / 0.49 / 1.883` | `recompute_law_table.py`, char-for-char |
| 3 | `tab:ladderacc`, TNF8 cell | `1.16e-2 (82/187)` → `2.02e-2 (85/187)` | `recompute_ladder_exact.py` (187−102=85) |

Plus three gate corrections: `check_withdrawn_live` (three false-positive classes,
baseline regenerated 15→25, negative test passing), `check_self_consistency`
(vocabulary 12→17 marks), `recompute_ladder_table.py` marked SUPERSEDED.

## 2. Document defects — fixed in PR #603 (open)

| # | where | was → is | how found |
|---|---|---|---|
| 4 | `tab:invariant` caption | "7000 samples per row" → 5000 (record `n=5000`, same seed) | first clean run of its new oracle |
| 5 | `tab:window`, c=4, binary16 | `1.69e-4` → `1.68e-4` (record 1.684663e-4) | same oracle; other five cells exact |
| 6 | reach column + prose, **9 sites** | `±40/±121/±364` → `±39/±120/±363` | the shipped oracle: 2^39 finite, 2^40 saturates; `prop:uncentred` proves it; `per_rung.tnf_reach` stores Δ, the offset |
| 7 | bibliography | `fibbinary` = `fiandaca2025fibbinary` (one arXiv id, two keys, both cited) → merged | `check_bibliography.py` (new) |
| 8 | bibliography | `wintersteiger2025` = `...formal` — **and they disagree on the page range at one DOI** (157–166 vs 157–160) → merged, fuller kept | same; the true range needs the proceedings — **author** |
| 9 | `tab:tailsweep`, mixture/0-outliers | `7.09e-4` → `7.08e-4` (record 7.084535e-4) | its new oracle; same one-digit class as #5 |

## 3. Reported, not patched — the author's judgement

| finding | the crux |
|---|---|
| `tab:window` suppresses one-sidedly | binary16 clipped 50.1 % at c=16 → cell hidden (record holds 1.677e-4, four× better than TNF there); TNF clipped 49.6 % at c=40 → value printed |
| `tab:window` caption states a rule the table doesn't follow | "no representation from c=20 upward, marked —" — but c=16 is dashed too; prose repeats the wrong reason |
| `tab:tailsweep` stops before the failure point | printed sweep ends at σ=6 (worst ratio 0.71); the record continues to σ=8 where two clips of 12,000 blow TNF's mean to **2.48e+35**. Both subsets are now count-disclosed; the *rule* is still unstated |
| `tab:cleandecode` control exceeds two entries | caption: bare wire = 112 LUT; GFTernary = 66, int8 = 76. A decoder cannot shrink an empty harness — either "same harness" isn't, or revisions differ. One caption sentence closes it |
| sampling prior undisclosed | four regenerators draw `_rng.integers(-38, 39)` (three live, one SUPERSEDED) — uniform over 77 binades, precisely the prior flat-precision wins under. "sampling prior" / "depends on the distribution": 0 occurrences |
| `measurements/README.md` misdescribes 3 entries | `inside_window` holds the GPT-2 intermediates the README attributes to `gpt2_window`; all 11 of its rows carry `inside:false`; "qualifying-pair count" belongs to `workloads_strict` |
| `per_rung.tnf_reach` stores the offset, not the reach | and **no generator exists to fix it in** — the oracle-side assertion is the only available defence |
| 10 of 14 records cannot be rebuilt | four have generators; the rest have readers only — **W935: the earlier clause "eight are mentioned by nothing in the tree" is withdrawn, every one of the 14 has a named consumer** |

## 4. Toolchain properties (measured here; different part, not comparable to published rows — the *properties* transfer)

| property | measurement |
|---|---|
| seed dispersion | 21 designs × 5 seeds: Fmax spread **1.6 %–41.7 %**, area identical across seeds. Captions state median-of-five (correct) but never the dispersion |
| configuration dominates seed | 3 placer/router pairs: Fmax moves up to **4.66×** (w_lns16; the 4.3× quoted earlier was the maximum of a 6-design subset, recomputed W935 over the full 21-design record); **32 pairwise ranking inversions**, incl. fp8-vs-TNF winners flipping on a router change alone. `placer`/`router`: **0 occurrences** in the paper; the CI's ordered fallback makes the choice silent |
| verdicts below seed noise | 25–37 of 210 pairwise winners alternate across the five seeds of a *single* configuration, at median margins up to ~38 %. The project's three-seed rule, needed for rankings too |

## 5. Oracle coverage

Before this audit: 4 of 59 numeric tables regenerable. Now: **20** —
`tab:invariant` (90 cells + formatting lock), `tab:rungthr` (two-record table,
summary re-derived from raw rows, reach from the oracle), `tab:tailsweep`
(selection disclosed, 10 unprinted rows listed on every run), plus five more in
flight (`tab:centring`, `tab:workloads`, `tab:landing`, `tab:gpt2window`,
`tab:convert`). New gates: `check_bibliography`, `check_selection_disclosure`
(each with a passing negative test).

## 6. Withdrawn by this audit — the misses, kept on the record

| claim | why it fell |
|---|---|
| "the record→table mapping is unreconstructible" (T676) | it was in the `\label` names; I searched caption text only |
| "strict_range→tab:invariant rejected" (T681) | size-corrected score punished a record for serving three tables; reconstruction settled it 30-for-30 |
| "the reach column is identified" (T688) | I confirmed the record against a formula — both held the same wrong quantity; only the oracle broke the tie |
| "seed count unstated" (T698a) | four captions state "median of five placement seeds"; I hadn't read them |
| "harnesses observe 16 of 32 bits" (T702) | already found, sharper, by `check_harness.py`, with all twelve files baselined by name |

Five withdrawals, one shared cause: **asserting before checking what the
repository, the caption, or the oracle already said.** `t27c known` now runs that
check in one command; it found the T702 answer — including the 50 % figure — in
one baseline line.

---

*Everything in sections 1–2 is in `tnf-publication-readiness` (merged) or
`tnf-invariant-oracle` (PR #603). Section 3 needs the author. Section 4 is data
about the toolchain, shipped as three records in `measurements/` with their
generators.*


---

## 7. The instrument era (W913–W922, appended after the landing)

The user's standing merge mandate converted the audit's waiting rows into
action: #603/#612/#615 merged, the first G8 dispatch failed on a generator
that had never reached main (#615 fixed the topology), and two clean sweep
runs later the artefact trail showed the sweep targets the WRONG experiment —
`tab:untraced`'s sixteen are format-tract designs living in `fpga/tnet/`,
not (E_t, M) arms (docs/G8-INSTRUMENT-MAP.md, merged #617).

The missing instrument was built the same day (#618: tnf-format-throughput —
19 tnet tracts, the sweep's chipdb machinery, CI rows compared against BOTH
published series under the audited 1.6–41.7 % seed band) — and its first
verdict never ran, because run 3 of the cost sweep put **e2m1, a near-empty
adder, in routing-pending**: the catch-all status had been absorbing every
nextpnr failure with no reason line and no log (#620 fixed both sweeps; the
same hole was already copied into the day-old instrument, plus a missing
--xdc). Two diagnostic runs are in CI as this chapter is written.

Standing lesson for the ledger: **a green pipeline is evidence about the
pipeline, not about the experiment** — three consecutive "successes" here
carried, in order, a missing file, a wrong experiment, and a euphemism.


## 8. The gate, measured (W927)

Run 32263875250: all 19 tnet tracts ROUTED on xc7a200tfbg484-2 — G8's first
post-route rows. **14 of 15 instrumented untraced frequencies reproduce
within the audited seed band** (0.90×–1.32×; binary16 at 1.00× exactly).
Named exceptions **as first reported**: LNS16 and plastic-16bit.

> **W935 correction — the LNS16 exception is withdrawn.** MATRIX.md:35 lists
> LNS16 at 43.11 MHz, 0.16 % from the published 43.04, so the "no in-tree
> record" premise was false; it came from six `None` cells in our own reporting
> table (fixed, #632). The in/out call also used `(CI−published)/published`
> where the band is defined as `(max−min)/median`; by its own definition the
> pair is 37.1 %, inside 1.6–41.7 %. Issue #625 to the author is closed as not
> planned. What survives: the CI row disagrees with two agreeing in-tree numbers,
> and LNS16 is the most configuration-sensitive design measured (4.66×) — which
> is a job for the reference-configuration re-measurement enabled by #630, not
> for the author. **plastic-16bit** remains uninstrumented; that half stands. The W920 map's "no hex32 harness" claim was wrong —
s_ibmhfp.v existed and routed (1.11×); corrected in the verdict (#624).

G8 status: **unsourced → measured-with-two-exceptions.** The instrument path
took five iterations (missing generator → wrong experiment → euphemism
status → self-executing glob + anchored grep → measurement), each driven by
one artefact, none skippable.


## 9. W935 — the referee simulation, and the field the paper is landing in

A hostile-referee pass and two prior-art sweeps were run against the manuscript,
then **every load-bearing claim was re-verified by hand** before anything left
this tree. Filed to the author as **tf#631**; the IDCODE criterion question as
**tf#633**.

**Verified against the manuscript (greps and arithmetic, not report-reading):**

| finding | evidence |
|---|---|
| the headline ranking is unresolved by the paper's own instrument | abstract `:116-119` gives 0.1797 vs 0.1631 MHz/LUT = **10.2 %**; `:5638-5641` states the resolution as **11.4 %** |
| the largest knob is never named | `grep -ci placer` = **0**, `grep -ci router` = **0**, over 7 858 lines and 60 tables; measured effect **4.66×** (w_lns16), unequal across candidates |
| the empty control is not a control | `tab:cleandecode` caption: bare wire = 112 LUT / 827.81 MHz; rows 1–2 print **66 LUT / 974.66** and **76 LUT / 925.93** — smaller and faster than empty |
| "zero DSP" is a flag, not a property | `-nodsp` / "DSP inference off" at `:1195, :4276, :4704, :4966` — true for all 21 arms by construction |
| the inclusion rule carries the result | `int8` excluded for e = 0 (`:5092`) while the winner, also e = 0, is admitted by a redefinition at `:5101`; the excluded row measures 0.1736 — 3.5 % behind |
| thirteen defining references absent | Micikevicius 0, Jaiswal 0, Baalen 0, Umuroglu 0, Constantinides 0, Leong 0, Boutros 0, Betz 0, Fahmy 0, Tridgell 0, Alemdar 0, hls4ml 0, Ramachandran 0 — controls Etiemble 9, Hunhold 6 |
| no dispersion in any caption | "confidence interval" 0, "standard deviation" 0, while frequencies print to 0.01 MHz on a quantity with up to 41.7 % spread |

**The competing field, from the sweeps** — each an uncited work claiming on the
same axis: Ramachandran *Logarithmic Posits* (DAC 2024), ~2× performance **per
unit area**, the identical metric, against this paper's 10.2 %; Aggarwal
*Shedding the Bits* (FPL 2024), the same multi-format-on-one-FPGA method with the
integer baselines kept in; Mekhemer/Boutros/Betz *Jack of All Scales* (2026) and
Abdurakhmanov & Fahmy (TRETS 2025), MX formats measured for area and timing on
FPGA — the ground the block-scale claim lands on; Hunhold's takum codec, 38 %
latency and 50 % LUT against posit codecs, four to five times this paper's effect
on the same substrate; and the LUTNet/PolyLUT/SparseLUT line, without which the
28 LUT-per-weight figure has no baseline.

**Publication readiness: 33 %**, weighted over eight axes against evidence on
disk: mathematics ~90 % (the φ-uniqueness enumeration and the Z[φ] closure are
sound, and both sweeps found no prior FPGA/NN work using golden-ratio arithmetic
as a hardware number format), claim–evidence consistency ~15 %, methodology
~25 %, baseline fairness ~10 %, positioning ~35 %, artifact ~35 %, our own
document hygiene ~55 %, field-expected axes ~5 % (no power, no energy, no
on-hardware point). **Re-centring the paper on the decode-cost separation —
6.08× published, 8.93× on the corrected harness, the one effect that clears every
measured noise source — is a single editorial decision worth roughly 55–60 %
with no new measurement.**

Three of the six blocker-grade findings landed against `G8-VERDICT.md` were
**ours**, not the author's: see the W935 correction above, and T766/T767.


## 10. W936 — the decode cost measured, and the instrument read from its source

**A new measurement, not a re-reading.** Replication (`DECODER-COST-W936.md`,
`decoder_cost_w936.json`, rig `gen_replicated.py`, landed upstream as **tf#634**):
each decoder instantiated N times in a pipelined chain, `cells(N) = fixture +
cost·N` fitted over N = 1,2,4,8, eighteen of nineteen fits exact with integer
slopes.

| | (LUT+CARRY4)/decoder | | |
|---|---:|---|---:|
| `int8` | **0.000** | `GF14` | 36 |
| `GFTernary` | **2.000** | `VAX F` | 41 |
| `TNF16/32/64` | **2.000** | `binary16` | 54 |
| `binary32` | 5.000 | `GF+8` | 56.3 (R²=0.997) |
| `BNF16` | 10.000 | `posit16` | 125 |
| `fp8 e4m3/e5m2` | 12.000 | `IBM hex32` | 129 |
| `GF10` | 26 | `LNS16` | 159 |
| `minifloat` | 31 | `posit32` | 304 |

The **ternary exponent field decodes 5× cheaper than the binary one** (2.000 vs
10.000, and the two formats differ by their own source comment *exactly* in that
field: BNF pays 8 cells for a subnormal special case TNF does not need). TNF's
cost is **width-independent** across 16/32/64. `int8` is exactly free. Separations
against the field: 6× fp8, 62× posit16, 80× LNS16, 152× posit32.

**Self-correction inside the same run:** the first pass counted LUTs only and
reported the constant-add decoders at **0.000 with R² = 1.00000** — a perfect fit
around a wrong number, because the cost had gone to CARRY4 (lesson 1407).

**The instrument, read from its own source** (verified by hand at the URLs, not
by report): every flip-flop carries a hardcoded `0.1 ns` setup, hold and
clock-to-Q (`xilinx/arch.cc:2507-2509`) and the chipdb emits `# only one speed
grade currently` (`bbaexport.py:356`). **Speed grade is decorative on this
substrate**, and the constant endpoint delay favours shallow designs over deep
ones — a rotation of the ranking, not an offset (T776). Further: `--freq` reaches
PnR only through criticality and budgets, which **router1 consumes and router2
does not**, and router2 ran no timing analysis at all before 2026-08-11, so its
`Max frequency` lines were placer pre-route estimates in identically formatted
text. Rows now carry `fmax_source` (**tf#635**), and our own T771 carries an
erratum.

**The two tables are two campaigns.** All **19 common rows disagree in LUT**
(Σ|Δ| = 465, max 19.27 % on GF+8), the row sets differ by two members each, LUT
is seed-invariant, `tab:tnet` is a byte-identical projection of
`tab:fullthroughput`, only `tab:fullthroughput` has dated in-tree provenance, and
`DOCUMENT_TRACEABILITY_2026-08-10.md:52` already lists MATRIX.md's figures as
superseded. Reported on tf#631.

**Readiness moves 33 % → 36 %.** Up for a genuinely new exact measurement with a
committed, reproducible rig; not further, because the same wave gave the
frequency column a *named, sourced* defect where it previously had only an
unknown.


## 11. W937 — the baseline downloaded, and the prior measured

Two experiments the comparison never had. Both landed upstream (**tf#636**), both
with their rigs and records committed.

### The field's reference posit hardware, through our own flow

PACoGen (Jaiswal & So, IEEE Access 2019) is public Verilog and cited **zero**
times in the manuscript. Same replication rig, same metric, same synthesiser:

| unit | cells | R² |
|---|---:|---:|
| PACoGen `data_extract_v1` (N=16, es=2) | **92.000** | 1.00000 |
| this tree's `posit16_decode` | **125.000** | 1.00000 |
| this tree's `TNF16` decode | **2.000** | 1.00000 |
| PACoGen `posit_add` (N=16, es=2) | **693.000** | 1.00000 |
| `tnf_cost_e4m8_add_top` (16 physical cells) | **561.670** | 0.999999 |

Two results in opposite directions, which is why the exercise was worth running.
**The reimplemented baseline is vindicated:** our posit decode costs 1.36× the
reference's extraction while assembling a full fp32 the reference does not — the
least likely outcome a priori, and the most useful. **And the headline shrinks:**
at operator level, matched storage width, against a published implementation, the
advantage is **1.23×**, where the paper claims 6.1× from decoder models. The decode
ratio against the reference is 46× and is amortised the moment anything is computed
with the decoded value.

### How much of the accuracy result is the prior

| prior | TNF16 | posit16 | advantage |
|---|---:|---:|---:|
| **published**, uniform over 77 binades | 8.101e-05 | 1.185e-03 | **14.63×** |
| standard normal | 8.184e-05 | 8.346e-05 | **1.02×** |
| He init, fan-in 512 | 8.184e-05 | 1.349e-04 | 1.65× |
| Student-t, df = 3 | 8.471e-05 | 1.351e-04 | 1.59× |
| log-uniform, 17 binades | 8.101e-05 | 1.131e-04 | 1.40× |

TNF16 leads under **all five** — the ordering is safe. The multiple is not: 14.63×
becomes a statistical tie under a standard normal. The stronger claim the same data
support is **prior-invariance**: TNF16's error moves by 1.046× across priors whose
median magnitude spans six orders, while posit16's moves by 14× and takum16's by 8×.
Under the published prior, `binary16` cannot represent 1,786 of 6,000 values and is
tabulated anyway. (LNS16 is **not** measured by this path and its row must not be
quoted — the oracle refuses a rational for a log format.)

### Also this wave

`fmax_search` added to the throughput workflow (**tf#637**): an optional binary
search for the highest `--freq` that still routes cleanly, at the reference
configuration, reported beside the constrained figure. Off by default.

**Readiness 36 % → 41 %.** Baseline fairness was the weakest axis at ~10 % and now
has a reference-implementation comparison; methodology gains the prior-sensitivity
analysis. The headline number got smaller and the paper got more publishable —
those are the same event.


## 12. W938 — the accuracy coordinate, and the width where the format stops mattering

Landed upstream as **tf#638** with its script and record. MNIST, 784-32-10 MLP,
fp32 baseline **93.39 %**, trained weights round-tripped through the shipped
conformance oracles with a per-tensor scale, activations fp32.

| width | result |
|---|---|
| **16 bits** | six formats whose round-trip error spans **16×** land within **0.02 pp** |
| **8 bits** | five formats within **0.19 pp** |
| **4 bits** | **TNF4 93.38 % (−0.01 pp)** · GF4 87.90 % · fp4 e2m1 87.90 % (**+5.49 pp**) |

**Above four bits the number format is invisible to this task.** Four bits is the
width where the field fights and the only one here where a number-system argument
has anything to explain — and TNF4 is the format that survives it (T779).

**An artefact of ours, recorded so nobody quotes it.** The same 4-bit run without a
scale shows a **70-point** gap, because fp4 e2m1 and GF4 flush **98.8 % of weights
to zero** — the median trained weight is 0.056, below their smallest representable
magnitude. That measures dynamic range, not the number system (T780). The tell was
two distinct formats agreeing to the digit. It also bears on the int8 exclusion:
**at four bits every format needs a scale**, so excluding one for carrying a scale
excludes the deployable region.

**The empirical prior, measured rather than bounded.** The trained tensors span
**8.1 binades** between p01 and p99 (15.9 end to end), median |w| = 0.056 — against
the **77 binades** the accuracy regenerators draw from, ~9.5× wider than anything
the format will see.

Positioning, honestly: weights-only PTQ, max-scaling only, and MNIST at 25 k
parameters is the easiest task in this literature — a lower bound on difficulty,
not a competitive result.

**Readiness 41 % → 47 %.** The field-expected axis was the weakest at ~5 % and now
carries a real task number with a width sweep and a scaling ablation. Area and
accuracy exist in one document for the first time: TNF decodes in **2 cells**
against fp8's 12 and posit16's 125, and at four bits it is the format that holds
its accuracy.


## 13. W939 — the 4-bit result becomes significant, and the alphabet turns out to be the argument

Landed as **tf#640** with both records and both rigs.

### Five seeds, two tasks, paired

| task | TNF4 − fp4 e2m1, per seed | mean | SE | t (df=4) |
|---|---|---:|---:|---:|
| MNIST (base 93.76 ± 0.36) | +5.48, +5.81, +5.10, +8.55, +17.08 | **+8.40** | 2.25 | **3.7** |
| Fashion (base 84.20 ± 0.29) | +20.17, +49.23, +13.23, +23.46, +32.64 | **+27.75** | 6.21 | **4.5** |

Both p < 0.05, **5 of 5 seeds each**, and the effect is **3.3× larger on the harder
task** — the evidence that it is not an artefact (T782). Sixteen bits still
resolves nothing; the single 8-bit difference that passes a paired test (GF8 over
fp8 e5m2, +0.12 pp, t = 3.8) is real and irrelevant, and the report says both
halves. The losing formats are **unstable, not merely worse**: σ = 13.95 pp on
Fashion against TNF4's 0.51. GF4 and fp4 e2m1 agree to the digit on every seed of
both tasks — one lattice printed twice.

### What the consumer pays for

Each decoder measured bare and behind an identical 12×8 multiply:

| format | in bits | decoder | +multiply | the multiply alone |
|---|---:|---:|---:|---:|
| GFTernary | 2 | 2.000 | 6.104 | **4.104** |
| fp8 e4m3 | 8 | 12.000 | 141.739 | 129.739 |
| TNF16 | 16 | 2.000 | 384.417 | 382.417 |
| BNF16 | 16 | 10.000 | 392.417 | 382.417 |
| posit16 | 16 | 125.000 | 507.452 | 382.452 |

**The decode gap survives fusion exactly** — TNF16 vs BNF16 is 8.000 cells bare and
8.000 fused — so the surrounding logic does not absorb the format difference, and
the LUT-absorption objection does not erase it. **But it is 2 % of the unit.**

**And the consumer's own cost is set by the alphabet:** identical RTL costs 382.4
cells behind 16 input bits, 129.7 behind 8, and **4.1 behind a two-bit alphabet**.
That is **93× on the consumer against 8 cells on the decoder** (T783). The
strongest area argument available is not "our decoder is cheap" but "our alphabet
makes everything downstream cheap" — quoted, honestly, as a width effect against
the accuracy that width costs.

**Readiness 47 % → 52 %.** The accuracy axis now carries a paired, multi-seed,
two-task result at p < 0.05, and the area argument has both a denominator and a
larger effect to lead with.


## 14. W940 — two scaling axes, and the frontier priced in physical bits

Landed as **tf#641** with both records and both rigs.

### The 4-bit effect scales on a second, orthogonal axis

| network | task | base | TNF4 − fp4 e2m1 | t (df=4) |
|---|---|---:|---:|---:|
| 784-32-10 | MNIST | 93.76 | +8.40 | 3.7 |
| 784-32-10 | Fashion | 84.20 | +27.75 | 4.5 |
| **784-256-256-10** | MNIST | **97.26** | **+37.88** | **12.4** |
| **784-256-256-10** | Fashion | **86.85** | **+64.42** | **24.7** |

Monotone along difficulty **and** capacity, 5/5 seeds throughout (T784). The bigger
baseline also answers W938's objection: 97.26 % matches FINN's own fp32 MLP and
beats its binarised SFC. The **8-bit null survives both axes** at 0.02–0.04 pp, so
it is a property of the width rather than of an easy benchmark.

### The frontier, and it does not favour the naming convention

Fourteen decoders priced alone and behind an identical 12×8 multiply. The
multiply's own surviving cost is monotone in alphabet width and spans **112×**:
3.43 cells at 2 bits, 128.31 at 8, 212.57 at 10, 341.29 at 14, 385.14 at 16. The
decoder is **2 %** of that.

| format | physical bits | consumer cells | MNIST | Fashion |
|---|---:|---:|---:|---:|
| **fp8 e4m3** | 8 | **138.57** | +0.00 | −0.01 |
| fp8 e5m2 | 8 | 138.57 | −0.02 | −0.04 |
| **TNF8** | **10** | **230.57** | −0.00 | −0.02 |

**At zero accuracy loss fp8 is the cheapest measured and TNF8 costs 1.66× more**,
because TNF8 stores ten bits while named for eight (T785). The manuscript concedes
the premise in its own caption; this is the consequence — a name-matched table
flatters, a width-matched table decides.

**Where TNF wins is not on cost:** at four bits fp4 loses 38–65 points while TNF4
holds within 0.33 pp. Not cheaper versus dearer — usable versus not.

**Readiness 52 % → 57 %.** The project now has a measured cost–quality frontier on
one substrate, two scaling axes behind its headline effect, and a finding that runs
against its own naming convention — which is the kind of evidence referees weight
most.


## 15. W941 — the Pareto point, and a width error three instruments shared

Landed as **tf#643**.

### Every decoder generated from its own oracle

Enumerating all 2^n codes through each format's own conformance reference and
emitting a case statement removes both standing defects: no implementation-quality
difference between formats, and every width read from the format object rather
than a module name.

| format | phys bits | values | consumer cells | MNIST (W+A) | Fashion (W+A) |
|---|---:|---:|---:|---:|---:|
| fp4 e2m1 | 4 | 15 | **19.14** | −70.50 | −71.32 |
| GF4 | 4 | 15 | 21.14 | −70.50 | −71.32 |
| **TNF4** | **6** | 58 | **51.29** | **−0.33** | **−1.05** |
| GF8 | 8 | 255 | 145.57 | +0.02 | +0.04 |
| fp8 e5m2 | 8 | 247 | 151.57 | −0.01 | −0.01 |
| **fp8 e4m3** | 8 | 253 | **152.57** | −0.02 | −0.04 |
| posit8 | 8 | 255 | 165.14 | +0.01 | +0.02 |
| TNF8 | **11** | 2018 | 270.57 | +0.02 | −0.06 |

**TNF4 delivers fp8-class accuracy at 2.97× less datapath cost** — 51.29 cells
against 152.57 — and is the **only sub-8-bit format measured that works at all**
(T788). Both weights **and** activations are quantised: the 8-bit null survives
the experiment designed to break it.

### The width error, and how it hid

The oracle settles what three instruments each guessed differently: TNF16 is
physically **19 bits** (1 + 7 + 11), against 16 in the name, 17 in the manuscript's
caption, and 17 in the `tnf17_decode` module W940b switched to. TNF8 is **11**, not
the 10 of `tnf8s_decode` — which is also a different field layout.

It hid because **the positive half of a sign-magnitude format decodes perfectly
well alone**: a 10-bit enumeration of an 11-bit format yields 1,008 finite,
monotone, correctly spaced values and no negatives. Caught only by cross-checking
against the oracle's own round-trip — 98 of 200 samples disagreed, which was the
negative half (T787). Both rigs now assert that an enumerated value set contains a
negative number.

**Readiness 57 % → 63 %.** The project now has a Pareto point measured end to end
by one procedure, with weights and activations quantised, on two tasks and five
seeds — and its cost side is generated from specifications rather than written by
the party the outcome favours.


## 16. W942 — the frontier closed at nineteen bits, and the sign flips

Landed as **tf#645**; the width question filed as **tf#644**.

Structural decoders generated from each `TNFFormat`'s own fields and **verified
against the oracle over every code** — 64, 2,048 and **524,288**, zero mismatches.

| rung | phys bits | codes | mismatches | decoder | consumer |
|---|---:|---:|---:|---:|---:|
| TNF4 | 6 | 64 | 0 | 12.00 | **55.29** |
| TNF8 | 11 | 2,048 | 0 | 16.00 | 260.57 |
| **TNF16** | **19** | **524,288** | 0 | 27.00 | **450.29** |

**The advantage lives at exactly one rung.** Against the cheapest *working* option
per class: `binary16` 438.57 vs TNF16 450.29 (**TNF 2.7 % dearer**), `fp8 e4m3`
152.57 vs TNF8 260.57 (**1.71× dearer**), and at four bits `fp4` 19.14 loses 70 pp
while **TNF4 at 55.29 loses 0.33–1.05** — **2.76× cheaper than fp8 for fp8-class
accuracy, and the only sub-8-bit format measured that works** (T789).

**The chain, kept on purpose:** W940 priced TNF16 by module name (386.57), W940b
corrected to the caption's 17 bits (424.86) and had TNF16 beating `binary16`, W942
finds 19 bits (450.29) and **inverts** it. Three corrections, each forced by the
previous one's own principle, each moving the number against this project's
interest.

**Method bias measured rather than declared:** truth table vs structural differs by
under 8 % and not in one direction — the table flattered TNF4 by 7.2 % and
penalised TNF8 by 3.8 %. Neither headline ratio is inside that band.

Also landed: `docs/REFEREE-PAGE.md` — every claim, its record, its limitation, the
four withdrawals, and what no measurement here can settle.

**Readiness 63 % → 68 %.** The frontier is closed at every rung with verified
decoders, the one real advantage is isolated and quantified, and the project now
ships a single page a referee can audit against committed records.


## 17. W943 — the advantage is conditional on not retraining

Landed as **tf#646**. Two experiments aimed at this project's own best result.

| setting | TNF4 − fp4 e2m1 | SE | t | seeds |
|---|---:|---:|---:|---:|
| MLP, PTQ, MNIST | +37.88 | 3.07 | 12.4 | 5/5 |
| MLP, PTQ, Fashion | +64.42 | 2.61 | 24.7 | 5/5 |
| CNN, PTQ, MNIST | +12.98 | 6.11 | 2.1 | 5/5 |
| CNN, PTQ, Fashion | +24.90 | 5.10 | 4.9 | 5/5 |
| **MLP, QAT, MNIST** | **+0.19** | 0.03 | 6.5 | 5/5 |
| **MLP, QAT, Fashion** | **+0.89** | 0.20 | 4.6 | 5/5 |

**QAT closes the gap 44× on MNIST and 31× on Fashion** (T790). The advantage stays
positive, 5/5 and significant — the variances collapse with the means — but changes
category. The claim is now conditional and more useful:

> For a fixed model that cannot be retrained, the coarse grid costs **13–65 points**.
> Where retraining is available, it costs **under one**.

On convolutions the fp4 collapse is **smaller and far more variable** (−13.13 ±
13.66 and −25.21 ± 11.31), consistent with a per-tensor scale spanning fewer
magnitudes per filter. **The 8-bit null survives convolutions too** — three formats
within 0.03 pp — and has now held across MLP and CNN, weights-only and
weights+activations, two tasks, two sizes and five seeds.

**Branch C (hardware) is blocked locally:** the Docker daemon does not respond, so
no bitstream can be built on this machine; the three attached boards remain
unmeasured.

**Readiness 68 % → 71 %.** Up for two architectures, two training modes and a null
that has survived every attempt to break it; not further, because the headline
narrowed and the hardware axis is still empty.


## 18. W944 — the range is the result, and my own prediction was refuted

Landed as **tf#652**.

W943's limits section predicted that a stronger QAT recipe would close the residual
4-bit gap further. With a learned scale (LSQ gradient), three epochs, five seeds,
paired:

| configuration | TNF4 | fp4 e2m1 | gap | SE | t |
|---|---:|---:|---:|---:|---:|
| 4-bit weights, learned scale, MNIST | −0.03 | −1.61 | **+1.58** | 0.19 | 8.2 |
| 4-bit weights, learned scale, Fashion | −0.02 | −0.93 | **+0.91** | 0.33 | 2.7 |
| + 4-bit activations, MNIST | −0.26 | −83.09 | +82.83 | 2.90 | 28.5 |
| + 4-bit activations, Fashion | −0.55 | −1.47 | +0.92 | 0.20 | 4.5 |

**On MNIST the better recipe made the gap eight times larger** (T791). Epoch
budgets differ across waves (4 vs 3), so the cross-wave ratio is indicative and
the within-run paired tests are not.

**The activation row is an instability, not a defeat:** fp4 at 13.93 % ± 6.30 on
MNIST and 84.58 % on Fashion — some seeds diverge, some do not. The claim is
robustness: the competitor becomes seed-dependent, TNF4 does not (σ ≤ 0.58).

**The honest headline is a range:** 13–65 pp without retraining, 0.9–1.6 pp with a
trained scale, plus competitor instability once activations are quantised. Three
waves, three numbers for one comparison, each from a better experiment than the
last — the range is the result, and any single number inside it is a recipe.

Also landed: **`tri verify`** — every headline number recomputed from its committed
record, 23 checks, all passing. Its first run reported one drift that turned out to
be the checker's own wrong expectation (a small-net figure compared against the
big-net record), which is the failure mode the checker exists to prevent and now
carries a comment about.

**Readiness 71 % → 74 %.** The strongest remaining gap is the empty hardware axis
and the absence of an external replication of any of this.


## 19. W945 — most of the advantage was width, and the claim becomes stability

Landed as **tf#654**; the container digest as **tf#653**.

**Two errors, both ours.** W944 called the 4-bit-activation result an instability
with "some seeds diverging"; the per-seed record is 11.51, 10.28, 11.35, 25.17,
11.35 — all five collapsed. And the comparison was never width-matched: **TNF4 is
physically six bits** (57 grid values, smallest positive 0.125) against fp4 e2m1's
four (15 values, 0.5). MNIST is 80.7 % zeros, so the coarser grid zeroes 84.2 % of
activations and the signal disappears; Fashion at 50 % zeros survives.

Against real 6-bit floats from the same oracle:

| configuration | TNF4 − fp6 e3m2 | TNF4 − fp6 e2m3 |
|---|---:|---:|
| weights, MNIST | **+0.11** (t 2.2) | +0.82 (t 9.0) |
| weights, Fashion | **+0.17 (t 1.2 — n.s.)** | +0.84 (t 4.2) |
| weights+acts, Fashion | **−0.42 — fp6 e3m2 wins** | +0.12 (t 0.8) |

**What survives is stability**: TNF4's σ is 0.17–0.72 pp everywhere, against 46.09
and 32.33 for the fp6 formats on MNIST with quantised activations (T792).

**The chain, four links:** 37.9 → 0.19 → 1.58 → **0.11**, each step forced by the
previous step's own principle and each moving against this project's interest.

Also landed: the toolchain container is **pinned by digest**
(`sha256:eced1cd…c70c`, resolved 2026-08-20 through the registry API without a
daemon) at all 15 invocation sites — tool-version drift was the one reproducibility
hurdle still open.

**Readiness 74 % → 75 %.** The claim is smaller and the evidence is better: a
width-matched, significance-tested statement about trainability is worth more than
an unmatched one about accuracy.


## 20. W946 — parity at matched width, and the last advantage was our own omission

Landed as **tf#655**. This closes the measurement arc.

| axis | TNF4 (6 bits) | `fp6 e3m2` (6 bits) | verdict |
|---|---:|---:|---|
| consumer cells | 51.29 | **50.29** | TNF4 **2 % dearer** |
| accuracy, weights | — | — | **+0.11** (t 2.2) / **+0.17** (t 1.2, n.s.) |
| stability, proper LSQ | 96.70 ± 0.38 | 96.58 ± 0.56 | **parity**, 0/5 failures both |

**The stability axis was ours to lose.** Logging the learned scales showed that in
*every* failing run the layer-2 activation scale collapses monotonically —
0.81 → 0.29 → **0.0065** — and locks. That is the documented LSQ failure mode, and
our implementation omitted the gradient-scaling factor `1/√(N·Q_p)`. Restoring it
took `fp6 e3m2` from 2 failures in 5 to **0**, within 0.12 pp of TNF4 (T793).

**Eight corrections across seven waves**, every one against this project's
interest, every one forced by the previous step's own principle, **none found by an
outside reviewer**: four unmatched-width comparisons, a variance read as a mixture
when all five runs had failed, an enumeration missing a sign bit, a frontier priced
by module name, and an omitted term from a cited recipe.

**What stands:** the mathematics (φ-uniqueness enumeration, Z[φ] closure), the
8-bit null that survived every experiment designed to break it, the measurement
apparatus, and **parity itself** — a novel lattice matching a mature IEEE-style
encoding at equal width means the φ-structure costs nothing to adopt.

**Readiness 75 % → 76 %.** The evidence is now the strongest it has been and the
claim the smallest. The paper's current hardware claim is **not supported at
matched width**; a reframed paper on the mathematics, the parity result and the
methodology is.
