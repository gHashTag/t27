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


## 21. W947 — recipe-insensitivity, and the mechanism is range

Landed as **tf#656**. This closes the thread W946 left open.

**Mechanism, measured at six physical bits:** TNF4 spans **14.6 binades** with 28
positive values; `fp6 e3m2` 8.8 with 31; `fp6 e2m3` **5.9** with 31. The φ-lattice
spends its width on range rather than precision, and under a max-rule scale that is
the difference between a grid where nothing underflows and one whose floor sits at
**1.7 %** of the peak.

**My mitigation hypothesis was refuted.** A percentile scale init made the narrow
grids worse and broke the one configuration that had worked:

| recipe | TNF4 | `fp6 e2m3` | `fp6 e3m2` |
|---|---:|---:|---:|
| max init, no gradient factor | 0/5 | 3/5 | 2/5 |
| max init, standard factor | 0/5 | 4/5 | **0/5** |
| p99.9 init, standard factor | 0/5 | 5/5 | **4/5** |
| KMNIST, max init, standard factor | 0/5 | 2/5 | **2/5** |
| **total failures** | **0/20** | **14/20** | **8/20** |

So W946's repair was **task-specific**. TNF4 has not failed once in twenty runs,
at 87.13 ± 0.15 on KMNIST — the tightest spread in this project.

**And the statistic was wrong before this wave.** `fp6 e3m2` on KMNIST reads 86.0,
86.8, 23.0, 87.3, 30.6; its mean of 62.73 ± 32.94 describes no run that happened.
The failure rate describes them all, and it makes the claim falsifiable in one run
(T794).

**Readiness 76 % → 78 %.** Three tasks, three recipes, a measured mechanism and a
statistic that matches the data — against which the remaining gaps are the empty
hardware axis and the absence of any external replication.


## 22. W948 — the range is free, the failures are dynamical, and the claims are now falsifiable

Landed as **tf#657**.

**The coarser step costs nothing at convergence.** Ten epochs, same seeds and
recipe: among the runs that trained, MNIST 97.68 (TNF4, n=5) against 97.54
(`fp6 e2m3`, n=1); Fashion 87.38 (n=5) against 87.10 (n=3) and **88.12**
(`fp6 e3m2`, n=1 — ahead of TNF4). Within 0.7 pp everywhere. **The 14.6 binades are
bought for free** at these depths.

**Longer training makes a collapse-prone grid worse.** `fp6 e3m2` moves from 0/5
failures at three epochs to **5/5 at ten** — same task, same recipe, only more
steps. Noise averages out with steps; positive feedback consumes them, and once the
activation scale is near zero the gradient that would raise it is gone with the
activations (T795).

| totals over six configurations | TNF4 | `fp6 e2m3` | `fp6 e3m2` |
|---|---:|---:|---:|
| failures | **0 / 30** | 20 / 30 | 17 / 30 |

**And the project is now refutable from outside.** `FALSIFY-ME.md` states five
claims — recipe-insensitivity, the range mechanism, parity on cost, parity on
accuracy, the 8-bit null — each with the experiment, the expected outcome and the
result that would kill it. The headline dies if any standard recipe brings either
fp6 grid to ≤ 2 failures in 20 across the same three tasks.

**Readiness 78 % → 80 %.** Six configurations, three tasks, two training lengths, a
measured mechanism, a statistic that matches the data, and a published falsification
protocol. What remains: the empty hardware axis, and the fact that all thirty runs
are still ours.


## 23. W948b — thirty epochs, the seven-wave summary, and the first bench number

Landed as **tf#659**; the summary filed as issue **tf#658**.

**Thirty epochs on Fashion completes both stories.** TNF4 climbs 85.50 → 87.38 →
**88.77 ± 0.41** across 3 → 10 → 30 epochs with **0/5 failures throughout**, so the
coarser step never starts costing within a tenfold range of budgets. Meanwhile
`fp6 e3m2` fails **2/5 → 4/5 → 5/5** and `fp6 e2m3` 2/5 → 2/5 → 4/5 — monotone in
the step count, which is a runaway and not a sampling process. At thirty epochs the
failures are no longer at chance (33, 41, 50, 52): a *partial* collapse (T796).

| totals, seven configurations | TNF4 | `fp6 e2m3` | `fp6 e3m2` |
|---|---:|---:|---:|
| failures | **0 / 35** | 25 / 35 | 22 / 35 |

**The first hardware measurement of this arc.** `openFPGALoader` needs no Docker
daemon — a fact nobody checked for seven waves of reporting the hardware axis as
blocked. The read-only JTAG scan returns `idcode 0x3636093`, `family artix a7 200t`,
`model xc7a200`, confirming the t27 SSOT and confirming that trinity-fpga's
acceptance criterion `0x13636093` would reject this exact board (reported on #633).
It also corrected our own SSOT: the bench presents **one** cable at bus 001 device
005, not three at `--busdev-num 1:4/1:6/1:8`, which all fail (lesson 1429).

**And the author has one page instead of fourteen threads** (#658): what moved,
what survives, what it does to the manuscript, what we withdrew, and the single
human action — starting Docker — that unblocks the bitstream build.

**Readiness 80 % → 82 %.** Seven configurations, three tasks, three training
lengths, a measured mechanism, a falsification protocol, and now one number from
the bench. The open edge is convolutional models at convergence, named as the first
thing an outside replication should attack.


### 23a. W948c — the sweep completes, and our own statistic has a blind spot

Landed as **tf#660**. MNIST at thirty epochs, the last configuration:

```
TNF4       0/5 failures   98.0  97.6  97.8  97.9  97.8   (97.82 ± 0.15)
fp6 e2m3   4/5            19.2  81.0   9.6  12.7  11.3
fp6 e3m2   2/5            71.9  65.6  55.5  71.4  59.3
```

By threshold `fp6 e3m2` passes three of five — but its **best** run (71.9) sits
**25.7 points below TNF4's worst** (97.6), and the distributions do not overlap.
**A rate counts line-crossings and is silent about uniform degradation** (T796a).
The mean hides bimodality, the rate hides this; only the per-seed list hides
neither, which is now the house style for every table.

TNF4 converges rather than drifting: **96.76 → 97.68 → 97.82 ± 0.15** across
3 → 10 → 30 epochs.

**Totals over eight configurations: TNF4 0/40, `fp6 e2m3` 28/40, `fp6 e3m2` 24/40.**

---

## 24. W948d — the tally was transcribed, and it was wrong by one

The sweep is closed and the claim is unchanged. The **arithmetic reporting it was
not checked**, and it was wrong.

The report stated `fp6 e2m3` failed **29** of forty runs. Recomputing the tally
from the eight records gives **28**. Nothing in the rig or the data had changed —
the figure had been carried forward by hand while the record set grew underneath
it, and it reached three published documents that way.

What hid it was a second habit: the same measurement circulated in **two
polarities**. `FALSIFY-ME.md` stated successes ("20 of 20 ... 12/20 ... 6/20"),
while the ledger and status board stated failures ("0/40 ... 29/40 ... 24/40").
Two documents can carry one quantity in opposite senses and disagree by one
without looking like they are about the same thing.

**Both are now fixed at the root rather than in the text.** `verify_numbers.py`
enumerates every `stability*.json` record, applies the per-task threshold and
asserts the tallies — **27 checks, all passing**. Every count on the referee page
and the falsification page is that computation's output, stated as successes out
of forty with the failure-side equivalents named, so the polarities can never
again be mistaken for two measurements.

**Final tally, derived:** TNF4 **40/40** successes, `fp6 e3m2` **16/40**,
`fp6 e2m3` **12/40** — eight configurations, four recipes, three tasks, three
training lengths.

A second defect surfaced while checking. `stability.py` named its record from the
task and recipe but **not from `EPOCHS`**, so the ten- and thirty-epoch runs on the
same task wrote the same path and the second destroyed the first; a recount against
the scratch directory returned **thirty** runs where forty had been performed. The
lost configurations survived only because each record had been copied into the
repository under a wave-suffixed name. The rig now names records after the epoch
count as well, and the repository copy — not scratch — is the record.

Theorems **T797**, **T797a**; lessons **1433**, **1434**. This is the ninth and
tenth correction this project has made to itself, still with no outside reviewer.

---

## 25. W949 — the mechanism was computed in a convention the experiment never used

The stability result stood on an explanation: **range**. "Under a max-rule scale the
narrow grids zero everything below 1.67 % (`fp6 e2m3`) or 0.22 % (`fp6 e3m2`) of the
tensor peak, against TNF4's 0.0041 %." Those three numbers are `min(grid)/max(grid)` —
they assume the tensor peak lands on the format's **largest representable value**.

**No run in this project ever did that.** Every quantiser initialised `s = max|x|`,
putting the peak on grid value **1.0**. The grids peak at **3072**, **28** and **7.5**.
Under the convention actually used, the thresholds are **12.50 %, 6.25 % and 12.50 %** —
TNF4 zeroed **twice as much** as the competitor it beat and kept **7** usable levels
against that competitor's **12**, and won 40/40 anyway.

**The explanation is inverted by its own experiment, and is withdrawn.** What exposed it
was an anomaly, not a review: a block-scaling rig reported *identical* underflow for two
grids of very different span — impossible unless the scaling had erased the difference.

### The convention is a recipe axis, and it decides the competitor

One scalar per tensor, nothing else changed, MNIST, five seeds, three epochs:

| convention | TNF4 | `fp6 e2m3` | `fp6 e3m2` |
|---|---|---|---|
| `peak2one` — `s = max\|x\|` (all prior runs) | **0/5 fail**, 96.70 | 4/5, 30.19 | **0/5 fail**, 96.58 |
| `peak2max` — `s = max\|x\|/max(grid)` (standard) | **0/5 fail**, 96.80 | 5/5, 14.57 | **5/5 fail**, 31.61 |

`fp6 e3m2` goes from perfect to dead. TNF4 moves **0.10 pp**. The `peak2one` arm
reproduces the W946 record to the second decimal (96.70), which is what licenses reading
the difference as a change of convention rather than of rig.

**So the empirical claim widens while its explanation collapses:** over five recipe axes,
TNF4 **45/45**, `fp6 e3m2` **16/45**, `fp6 e2m3` **12/45** — and we no longer know why.

### The largest unmeasured threat, stated before a referee states it

Every comparison here scales **per tensor**. At four to six bits the field deploys
block-scaled MX formats — a shared exponent per 32 elements — so the element format need
only span the range *inside* a block. Measured on the enumerated grids, heavy-tailed
activations:

| block | TNF4 | `fp6 e3m2` | `fp6 e2m3` |
|---|---|---|---|
| 32 (OCP MX) | 0.01 % zeroed, **7.08 % RMS** | 0.35 %, 3.54 % | 2.51 %, **2.04 % RMS** |
| per-tensor | 0.12 %, 11.37 % | 6.56 %, 5.50 % | **44.41 %**, 22.12 % |

TNF4's underflow is negligible everywhere — the range claim is true. But its relative RMS
error is the **worst of the three six-bit grids at every block size**, by **3.46×** against
`fp6 e2m3` at block 32. Range and resolution are bought with the same 64 codes. **No
training run in this project has ever used a block scale.**

### And the published rig could not reproduce two of its own records

`FALSIFY-ME.md` instructs a replicator to set `EPOCHS ∈ {3,10,30}`. The **published**
copy of `stability.py` had `EPOCHS = 3` hard-coded; only the working copy read the
environment. Two copies of one program had drifted, and every patch since had been
applied to the published one — so it looked maintained while being unable to produce two
of the eight records it ships to support. Fixed, with the convention now selectable via
`SCALE_RULE` and named in the record file.

Theorems **T798**, **T798a**, **T798b**; lessons **1435–1438**. `verify_numbers.py` now
runs **38 derived checks**. Corrections eleven through thirteen, still no outside referee.

---

## 26. W950 — the mechanism recovered, and the last claim killed by it

W949 left this project with a result and no reason: the range explanation was
withdrawn, and nothing replaced it. **The replacement was already on disk.**

### The mechanism: saturation, not underflow

Every failing run logs its per-epoch scales, and in all of them the activation scale
**collapses** (0.81 → 0.29 → 0.0065). A shrinking `s` makes `x/s` **grow**, so the
quantity that matters is headroom **above** the operating point — `max(grid)`, which
is **3072 / 28 / 7.5** across the three formats. Prediction: a run fails when the
collapse exceeds the headroom, i.e. when the tensor **saturates**.

Over **all 120 recorded runs**, saturation and failure agree **90.8 %**, and
`fp6 e2m3`'s 28 failures saturate **28 out of 28**. The decisive number is inside the
table: **TNF4's scale collapses 32.4×** — twenty times harder than `fp6 e2m3`'s
*successful* runs (1.5×) — and TNF4 never fails. **Not stability. Room to fall into.**

### The prediction it makes, and the test that killed our claim

If failure is a collapsing **learned** scale, a scale that cannot collapse should end
it. The OCP microscaling formats specify exactly that: a shared **power-of-two** scale
per block, **computed** each forward pass. Same net, same seeds, same epochs:

| arm | TNF4 | `fp6 e2m3` | `fp6 e3m2` |
|---|---|---|---|
| block 32 | 0/5, 96.11 | 0/5, 96.10 | 0/5, 96.21 |
| **per-tensor** (control) | 0/5, **96.57** | 0/5, **96.94** | 0/5, 96.82 |

**All thirty runs succeed.** `fp6 e2m3` — 28/40 failures under our learned-scale recipe
— fails **nothing**, at the *same per-tensor granularity*. So the block size explained
nothing; **the learned scale was the entire effect.**

**And the ordering inverts.** Paired over five seeds, per-tensor: TNF4 **−0.376 pp**
against `fp6 e2m3` (t = −7.24, **0/5** seeds favour TNF4) and −0.250 against
`fp6 e3m2` (t = −5.15). At block 32 the three are indistinguishable (+0.010, t = 0.11).

### Where this leaves the result

The recipe-insensitivity claim is still literally true — TNF4 has survived every recipe
ever tried here, now 55/55. **Its implication is refuted.** Under the quantiser the
field actually deploys, a same-width float is not fragile, and it is significantly more
accurate at the granularity where this project made its comparisons.

**At six bits, on these tasks: no measured advantage on cost (2 % dearer), none on
accuracy, and under the standard recipe a measured deficit.** That is the honest state.

**The control arm carried the result.** Block scaling was the hypothesis and explained
nothing. Had the control been dropped as redundant, the published conclusion would have
been "block scaling rescues the float" — false, and unfalsifiable from that experiment.

Theorems **T799**, **T800**; lessons **1439–1442**. `verify_numbers.py` now runs **53
derived checks**, and `tri audit` refuses to pass while any of them disagrees with its
records. Corrections fourteen and fifteen, still with no outside referee.

---

## 27. W951 — saturation observed, the sweep redone, and a proxy retired

Two things W950 left open are now closed, and closing them cost one more claim.

### The mechanism, measured instead of inferred

T799 inferred saturation from an end-of-epoch scale ratio, because the records stored
scales but never tensor maxima. Agreement with failure: 90.8 % over 120 runs.

The rig now logs the actual quantity — `max|x| / s / max(grid)`, per layer, per epoch,
weights and activations, across every batch. Over **135 runs**:

| scale | outcome | n | overshoot median | range |
|---|---|---|---|---|
| learned | success | 36 | 36.1× | 2.75 – **1 510×** |
| learned | **failure** | 9 | **217 549×** | **84 775×** – 1 804 000× |
| computed | success | 90 | 2.00× | 1 – 2× |

**The binary criterion is refuted by its own measurement**: everything overshoots,
including all 90 successes, so "saturates ⟹ fails" agrees on **6.7 %**. What survives is
the magnitude — among the 45 learned-scale runs the two distributions **do not overlap**,
worst success 1 510× against best failure 84 775×, a gap of 56×. A little clipping is
harmless; five orders of magnitude of it is fatal.

**And the computed scale cannot get there.** Flooring the shared exponent leaves
`max|block|/s ∈ [max(grid), 2·max(grid))`, so the overshoot is bounded in **[1, 2)** by
construction. Measured maximum over 90 runs: **2.0000**.

### The sweep, redone under the recipe the field deploys

| task | learned, per-tensor | computed, per-tensor | computed, block 32 |
|---|---|---|---|
| MNIST | 0/5, 0/5, **4/5** | 0/5, 0/5, 0/5 | 0/5, 0/5, 0/5 |
| Fashion | 0/5, 0/5, **1/5** | 0/5, 0/5, 0/5 | 0/5, 0/5, 0/5 |
| KMNIST | 0/5, **2/5**, **2/5** | 0/5, 0/5, 0/5 | 0/5, 0/5, 0/5 |

*(TNF4, `fp6 e3m2`, `fp6 e2m3`)*

**Zero failures in 90 runs under the computed scale**, on every task. The instability is
the quantiser's, reproduced on all three tasks — the earlier forty-run sweep measured
that quantiser, not the number system.

### What is left standing

TNF4 has still never failed, anywhere, in any recipe — now 0 in 60 recorded runs plus
these 90. That is a true statement about tolerating a badly-behaved recipe. It is **not**
a statement about deployment, because under the deployed recipe the φ-lattice is at
parity (block 32) or measurably behind (per-tensor: −0.376 pp, t = −7.24).

New tool: **`tri sweep`** derives this whole table from every record on disk, so no
version of it is ever typed by hand again.

Theorems **T801**, **T802**; lessons **1443**, **1444**. **62 derived checks.**
Corrections sixteen and seventeen, still with no outside referee.

---

## 28. W952 — range is a bill presented at the accumulator

Every cell census this project ever ran priced **decode plus a multiply by a constant**
(8.0 cells of decode, 43.29 of multiply) — corrected in W953; the earlier wording here
said "a decoder", which was wrong by one component. On that basis TNF4 is 51.29 cells against 50.29 — **2 % dearer**. An
inference datapath also multiplies and accumulates, and those widths are set by the
dynamic range that this project spent eight waves calling its advantage.

### The widths are forced, and computed exactly

Every grid value must be representable, so the fixed-point width follows from the grid:

| format | binades | bits/value | bits/product | block-32 accumulator |
|---|---|---|---|---|
| TNF4 | 14.58 | **17** | **33** | **38** |
| `fp6 e3m2` | 8.81 | 10 | 19 | 24 |
| `fp6 e2m3` | 5.91 | **7** | **13** | **18** |

### Measured, two ways, on purpose

**A fixed-point MAC lane** — decode two codes, multiply, accumulate — replicated, cost as
the slope of `cells(N) = fixture + cost·N`, fixture exactly 0 and R² exactly 1:

| format | cells per lane | MUXF7/8 |
|---|---|---|
| TNF4 | **768.00** | 120 |
| `fp6 e3m2` | 308.00 | 71 |
| `fp6 e2m3` | **159.00** | 46 |

**TNF4 costs 4.83× an `fp6 e2m3` lane.** But that is one datapath style, and the one most
punishing to wide range.

**The accumulator alone** is forced by arithmetic, not design — 48 / 30 / 23 cells for 38
/ 24 / 18 bits. Amortised over the 32 elements sharing it: **+0.78 cells per element**,
about **+1.5 %** on a ~51-cell decoder.

### The honest form is a bracket

**The silicon cost of range spans +1.5 % to +383 %, depending on the datapath.** Both ends
are real designs. The published "2 % dearer" prices neither — it prices a component. The
missing third point is the **float-style lane** (mantissa multiply, exponent add, align),
which is what an MX engine most likely builds; until it is measured, quoting **4.83×**
alone would repeat exactly the error the 2 % figure made.

Taken with W949–W951: at block 32 the range buys **no accuracy** (3.46× worse RMS, T798b)
and **no stability** (the failures were the learned scale, T800), and here it is shown to
**cost width**. Range is not a free property of a lattice.

A rig defect worth recording: the first synthesis run passed `-q` to yosys, which
suppresses the `stat` block, and the parser read the silence as **zero cells** — four
zeros fitting a perfect line, R² = 1.00000, the same signature as lesson 1407. The rig now
refuses a reading of zero instead of fitting a line through it.

Theorems **T803**, **T803a**; lessons **1446**, **1447**. **80 derived checks.**

---

## 29. W953 — the bracket closes at +46 %

W952 left the silicon cost of range as **+1.5 % … +383 %**, because the datapath an MX
engine most likely builds had not been priced. It is now measured, and the answer sits
between the ends but much nearer the low one.

### Building the lane without assuming a field layout

The obvious form — `(1.mantissa, exponent)` — **fails on both fp6 grids**: their bottom
binade is truncated, so `M = mantissa·2^(e−e_min)` does not reconstruct the value. That was
tested and rejected *before* any RTL was generated. What holds for every grid: `|v| = M·u`
for integer `M`, and every integer factors as `M = odd·2^s`. The decode table emits
`(sign, odd, s)`; the product is `odd₁·odd₂` shifted by `s₁+s₂` — exact, no rounding, no
subnormal case.

| format | odd mantissa | max shift | aligned bus |
|---|---|---|---|
| TNF4 | **2 bits** | 15 | 34 |
| `fp6 e3m2` | 3 bits | 8 | 22 |
| `fp6 e2m3` | **4 bits** | 5 | 18 |

TNF4 has the **narrowest multiplier** and the **widest aligner**: 14.58 binades collapse
the mantissa to one explicit bit and stretch the shifter to 15 positions. The φ-lattice's
trade, in gates.

### Every datapath, one table

| datapath | TNF4 | `fp6 e3m2` | `fp6 e2m3` | TNF4 / `e2m3` |
|---|---|---|---|---|
| decode + **constant** multiply | 51.29 | 50.29 | 50.29 | **1.02×** |
| MAC lane, fixed point | 768.00 | 308.00 | 159.00 | **4.83×** |
| **MAC lane, float style** | **108.00** | 82.00 | **74.00** | **1.46×** |
| block-32 accumulator alone | 48.00 | 30.00 | 23.00 | 2.09× |

The float lane is cheaper **for every format**, so the fixed-point datapath is the wrong
design and **4.83× must not be quoted**. In the datapath a real engine builds, the
φ-lattice costs **+46 % per MAC lane**.

### And a correction to our own headline figure

The 1.02× row is **not a decoder**. It is decode plus a multiply **by a constant** — 8.0
cells of decode, 43.29 of multiply. A constant operand lets the synthesiser specialise the
multiplier and fold away exactly the width that range forces. With both operands varying
the same comparison is **1.46×**. The W952 chapter here described that figure as pricing
"a decoder"; that was wrong by one component and is corrected above.

**Six bits, complete:** range buys no accuracy at block 32 (3.46× worse RMS), no stability
once the quantiser is the deployed one, and costs **+46 %** of the multiply-accumulate
datapath. None of it was visible from the census this project ran for eight waves.

New tool: **`tri cost`** prints the table above from the records, all four datapaths.

Theorem **T804**; lessons **1448**, **1449**. **91 derived checks.**
