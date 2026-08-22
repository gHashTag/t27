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

---

## 30. W954 — the ladder's rung was never measured, and at matched range the lattice is at parity

### The name and the object had come apart

`tnf_ref.LADDER` defines the eighth rung as `TNFFormat(3, 4)`. **Every rig in this project
instantiated `TNFFormat(4, 3)`** and labelled it TNF8:

| | ladder TNF8 = (3,4) | measured as "TNF8" = (4,3) |
|---|---|---|
| physical width | **10 bits** | **11 bits** |
| distinct values | 961 | 2 017 |
| dynamic range | **30.95 binades** | **126.91 binades** |

The substitute carries TNF16's exponent field with a truncated mantissa — four times the
range and one extra bit. Every accuracy, activation, convolution and cell-census number
published here for "TNF8" describes that, not the rung the ladder specifies.

**And the module contradicts itself:** `LADDER` is bound to **v1-research** while
`DEFAULT_LADDER_VERSION` is **v2-spec**. Above rung 8 they disagree — TNF16 is **17 bits**
under one and **19** under the other. **This is the answer to issue #644**: 16 by name, 17
by the research ladder, 19 by the spec ladder. Three sources, three correct answers, three
different objects.

### At matched range, the lattice is at parity — and the +46 % priced range

The float-style lane put TNF4 at +46 % against `fp6 e2m3`. Those peers match in *width*
and not in *range*: 14.58 binades against 5.91. Build the range-matched peer and price it:

| format (6 bits) | binades | values | cells |
|---|---|---|---|
| `fp6 e2m3` | 5.91 | 31 | 74.00 |
| **TNF4** | **14.58** | **28** | **108.00** |
| **`fp6 e4m1`** | **15.58** | **31** | **106.00** |

**+1.9 %**, with the float carrying *more* values. Replicated at ten bits on the ladder's
true rung: **TNF8 380 vs `fp10 e5m4` 376 — +1.1 %**.

**So cost tracks dynamic range, not the lattice** — about **2.4 cells per binade per lane**,
for floats and for the φ-lattice alike. The earlier +46 % was a true measurement of a
comparison nobody should make.

**Consequence, and it is not favourable.** At equal width and equal range an ordinary float
matches TNF4 on area and beats it on value count. **There is no configuration measured in
this project where the lattice wins** — not accuracy, not stability, not area at matched
range. What it has is a particular point on a curve any float can be configured to reach.

### Reported, not hidden

The 11-bit arm — pricing `TNFFormat(4, 3)` itself, 126.91 binades and a ~270-bit aligned
bus — was killed without a traceback partway through and is **dropped, not silently
omitted**. Four completed synthesis runs went with it because lesson 1441 (write progress
incrementally) had been applied to the training rig and not to this one. Fixed.

Theorems **T805**, **T806**; lessons **1450**, **1451**. **101 derived checks.**

---

## 31. W955–W962 — six waves lost to a full disk, and the rate coefficient withdrawn

### The blockage, and what it taught

The volume reached zero mid-sweep and stayed there for **six waves**. The known trap is
that `Bash` dies because the harness opens a capture file before exec. What this
established is that **the `Write` tool dies for the same reason**: it stages through
`path.tmp.NNNN`, so truncating a large file to free its blocks — the obvious escape —
needs a *new* file first and fails identically. `Read` works and frees nothing;
`TaskStop` cannot reach a `nohup`'d process it does not own. **At true zero the session
has no lever at all.**

Two habits made the eventual recovery cost nothing: progress records written
**incrementally**, and finished work left **in the working tree**, where the first
unblocked wave simply committed it. Nothing was lost — `tri anomaly` and eight curve
points survived six waves of paralysis.

### The rate coefficient is withdrawn

T806 said cost tracks range at "about **2.4 cells per binade per lane**", from two format
pairs. The full split sweep refutes the functional form:

| format | binades | odd | shift | cells |
|---|---|---|---|---|
| `fp6 e1m4` | **4.95** | 5 | 4 | **80** |
| `fp6 e2m3` | **5.91** | 4 | 5 | **74** |
| `fp10 e3m6` | **12.99** | 7 | 12 | **230** |
| `fp10 e4m5` | **19.98** | 6 | 19 | **215** |

**Less range, more cost** — twice, at both widths. A binade-only regression gives **6.49
cells per binade at R² = 0.85**: neither the number quoted nor a defensible fit. Two terms
move oppositely as the split changes — the multiplier grows with the mantissa, the aligner
and accumulator grow with the shift span — so **no single rate exists**.

### What replaces it is exact

Characterise a lane by **(odd-mantissa bits, maximum shift)**, from factoring each grid
value as `|v| = odd · 2^s · u`. Formats sharing that pair share a bus and an accumulator:

| | odd | shift | bus | cells |
|---|---|---|---|---|
| **TNF4** | 2 | 15 | 34 | **108** |
| **`fp6 e4m1`** | 2 | 15 | 34 | **106** |
| **TNF8** | 5 | 34 | 78 | **380** |
| **`fp10 e5m4`** | 5 | 34 | 78 | **376** |

**Within 1.9 % and 1.1 %, with no fitting.** The φ-lattice contributes nothing beyond those
two integers. T806's parity conclusion stands and is now exact rather than empirical; its
rate is gone.

**Cross-validated:** every format in both the W954 and W955 records reproduces exactly
across independently launched runs — 106/106, 74/74, 82/82, 376/376.

New tool: **`tri anomaly`** encodes the four measurement-defect signatures this loop has
hit — zero-reading, flat-fit, twins, short-arm. Its own first run produced 255 false
positives by counting a per-epoch trace as runs; it now decides by record shape.

Theorem **T807**; lessons **1452**, **1453**. **118 derived checks.**

---

## 32. W963 — the published TNF8 penalty had the wrong sign

W954 found that every rig here bound `"tnf8"` to `TNFFormat(4, 3)` — 11 bits, 126.91
binades — while the ladder's eighth rung is `TNFFormat(3, 4)` — 10 bits, 30.95 binades.
The census rig carries the substitution in its own table:
`("tnf8", 11, lambda: (T, T.TNFFormat(4, 3)))`. The size of the error was unknown. It is
now measured, **in this project's original metric**, each format against a float of its
own physical width:

| format | bits | values | decoder | consumer |
|---|---|---|---|---|
| **TNF8 = (3,4)** — the ladder's rung | 10 | 962 | **12.00** | **212.57** |
| `fp10 e5m4` | 10 | 1023 | 14.00 | 214.57 |
| **"TNF8" = (4,3)** — as measured | 11 | 2018 | **29.00** | **270.57** |
| `fp11 e6m4` | 11 | 2047 | 16.00 | 257.57 |

R² ≥ 0.9996 on every fit.

**The true rung is 0.9 % CHEAPER than its width-matched float. The substitute is 5.0 %
DEARER.** The published penalty was not an overstatement — it had **the wrong sign**.

**The decoder is where it bit:** 12.00 cells against 29.00, a factor of **2.4**. The
substitute inherits TNF16's four-trit exponent, so its decode table spans 127 binades and
resists the synthesiser's factoring; the true rung's 31 binades do not.

**Two metrics, two signs, one conclusion.** In the MAC-lane metric this rung is **+1.1 %**
(380 vs 376); here it is **−0.9 %**. A constant multiplicand lets the synthesiser
specialise (T804), which flatters whichever format has the smaller decode table, so the two
metrics tilt oppositely by construction. **Two metrics straddling zero is a stronger
statement of parity than either number alone.**

**What is still open.** The accuracy, activation and convolution figures for "TNF8" remain
those of `TNFFormat(4, 3)`. Those rigs need datasets and have not been re-run. **The
ladder's eighth rung now has a measured area and no measured accuracy.**

Theorem **T808**; lessons **1454**, **1455**. **130 derived checks.**

---

## 33. W964 — the eighth rung has an accuracy at last, and it is parity

T808 left the ladder's true rung `TNFFormat(3, 4)` with a measured area and no measured
accuracy. MNIST, MLP 784-256-256-10, weights and activations quantised, five seeds, three
epochs, three recipes:

| recipe | TNF8 = (3,4) | `fp10 e5m4` | paired | t |
|---|---|---|---|---|
| computed scale, per-tensor | 96.92 | 96.93 | **−0.010 pp** | −0.20 |
| computed scale, block 32 | 96.07 | 96.09 | **−0.018 pp** | −0.48 |
| learned scale, per-tensor | 97.08 | 97.02 | **+0.060 pp** | +0.89 |

**Nothing significant, and the sign changes between recipes.** Against its width-matched
float the eighth rung is indistinguishable.

**Zero failures in all 45 runs — including under the learned scale.** At six bits that
recipe destroyed `fp6 e2m3` in 28 of 40 runs. At ten bits it destroys nothing, for any
format. This is exactly what T801 predicts: failure requires the scale to collapse past the
format's headroom, and every ten-bit grid here spans 31 binades or more against
`fp6 e2m3`'s 5.91. **The instability this project studied for eight waves has a width above
which it simply does not exist.**

### The asymmetry

The same substitution **inverted the sign of the area result** (chapter 32: true rung 0.9 %
cheaper, substitute 5.0 % dearer) and is **harmless in accuracy** — true minus substitute is
+0.022, −0.108 and +0.052 pp, with only the block-32 arm significant (t = −2.71) and in the
substitute's favour by a tenth of a point.

Area depends on the decode table's structure, where 127 binades against 31 changed the
decoder by 2.4×; accuracy at this width depends on grid density, which the two formats
share. **So "the published numbers are approximately right" cannot be inferred from one
axis** — it was true for accuracy and false by a sign for area.

### Where the ladder stands

| rung | area | accuracy | stability |
|---|---|---|---|
| **4** (6 bits) | parity at matched range | no advantage | no advantage under the deployed recipe |
| **8** (10 bits) | parity (−0.9 % / +1.1 % in two metrics) | **parity, 3 recipes** | **no failures at all** |
| **16+** | unmeasured | unmeasured | unmeasured — and the width depends on the ladder version imported |

Theorem **T809**; lessons **1456**, **1457**. **147 derived checks.**

---

## 34. W965 — rung sixteen is strictly dominated, and the grids alone say so

The last row of the ladder marked "unmeasured" on all three axes is closed, and closing it
needed **no training and no synthesis**.

**Both versions, labelled by version.** `LADDER` (v1-research) is `TNFFormat(4, 9)` at 17
bits; `get_ladder(DEFAULT)` (v2-spec) is `TNFFormat(4, 11)` at 19. After W954 no rung here
is taken by its name again.

| format | bits | distinct values | binades | odd | max shift | aligned bus |
|---|---|---|---|---|---|---|
| **TNF16 v1-research** `(4,9)` | 17 | 129 025 | **127.00** | **10** | **135** | **290** |
| `fp17 e7m9` (range-matched) | 17 | **131 071** | **136.00** | **10** | **135** | **290** |
| **TNF16 v2-spec** `(4,11)` | 19 | 516 097 | **127.00** | **12** | **137** | **298** |
| `fp19 e7m11` (range-matched) | 19 | **524 287** | **138.00** | **12** | **137** | **298** |

**The structural pairs are identical** — 10/135 and 12/137 — so by T807 the lane cost, the
aligned bus and the accumulator are identical too. And the float **dominates on both
remaining axes**: more distinct values, and more range. The φ-lattice loses about **1.6 %
of its code space** to points that are unrepresentable or duplicated.

**This is not parity. It is strict domination**, and it is a property of the grids, not of
any experiment.

**The alternative framing is no kinder.** A width-matched float with a conventional
exponent (`fp17 e6m10`) spans 73 binades on a **166-bit** bus against the rung's 290. So at
these widths the choice is *same cost, more range, more values* or *far less cost, less
range*. **The rung is on neither frontier.**

**Scope, stated.** T807's law was validated by synthesis at six and ten bits; applying it at
seventeen and nineteen is a **prediction from a validated law**. The value counts and
structural parameters are exact. A synthesis check needs a structural decoder — a case
table over 2¹⁹ codes is not buildable.

**The ladder, complete as far as it has been measured:**

| rung | area | accuracy | stability |
|---|---|---|---|
| **4** (6 bits) | parity at matched range | no advantage | no advantage under the deployed recipe |
| **8** (10 bits) | parity (−0.9 % / +1.1 %) | parity, three recipes | no failures |
| **16** (17/19 bits) | **strictly dominated** | — | — |

Theorem **T810**; lessons **1458**, **1459**. **185 derived checks.**

---

## 35. W966 — same grid, two per cent dearer; and W965's "more values" was subnormals

W965 called rung sixteen strictly dominated: a range-matched float with the rung's
`(odd, shift)` pair, hence the same lane cost, but **more values and more range**. The cost
half was a prediction; the value counts were exact. **Both are now corrected by
measurement, and the result sharpens.**

### The discipline was not matched

TNF's own decoder maps offset 0 straight to zero — **the format has no subnormals**. The
floats W965 compared against did. Subnormals are what gave the float its extra values, and
they are not free: they need a leading-zero normaliser the rung never pays for. **W965
priced a design choice as a property of the lattice.**

### Rebuilt in TNF's own discipline, priced structurally, every code verified

| format | bits | distinct values | binades | decoder | consumer |
|---|---|---|---|---|---|
| **TNF16 v2-spec** `(4,11)` | 19 | **516 097** | **127.00** | **27.00** | **450.29** |
| `fp19 e7m11` FTZ | 19 | **516 096** | **126.00** | **18.00** | **441.29** |
| `fp19 e6m12` FTZ | 19 | 507 904 | 62.00 | 22.00 | 445.29 |
| **TNF8** `(3,4)` | 10 | **961** | **30.95** | **18.00** | **230.57** |
| `fp10 e5m4` FTZ | 10 | **960** | **29.95** | **13.00** | **225.57** |

**The grids are the same grid.** One value and one binade apart at nineteen bits; one value
and one binade apart at ten. **What differs is cost: +2.0 % and +2.2 %**, and it sits in the
**decoder** — 27.00 cells against 18.00, and 18.00 against 13.00 — where TNF's offset-max
sentinel and bias constant buy nothing the float lacks.

**This negative result is stronger than W965's.** "Dominated on three axes" invited the
reply that the axes were unmatched, and they were. "Same grid, two per cent dearer to
decode" does not.

**Cross-validated:** TNF16's structural cost reproduces the W942 record **exactly** —
450.29 against 450.286 — from an independently launched run over the same 524 288 codes.

**Limit stated.** The FTZ float's Verilog is verified against a Python reference of the same
semantics over every code, which validates the transliteration, not the definition. The
definition is ours, chosen to match TNF's zero and infinity handling. A reader who wants
subnormals in scope should read chapter 34's table — and then charge the float for the
normaliser.

Theorem **T811**; lessons **1460**, **1461**. **208 derived checks.**

---

## 36. W967 — the substitution is a class, and the tool that finds it had to be rewritten

`tri anomaly` checks the *shape* of records. It cannot see that a **name is bound to the
wrong object** — the defect that inverted the area result and left the eighth rung
unmeasured for eight waves. So that gets its own command.

**`tri rungs`** resolves every `TNFFormat(…)` call in every rig to its physical width and to
the rung it actually is, in **both** ladder versions, and distinguishes a bare substitution
from a labelled control by whether the same file also instantiates the true rung.

**Over the corpus: 34 instantiations, 5 standing alone as non-rungs.**

| rig | instantiates | width | status |
|---|---|---|---|
| `accuracy_coordinate.py` | `(4, 3)` | 11 | **stands alone** |
| `accuracy_seeds.py` | `(4, 3)` | 11 | **stands alone** |
| `activations.py` | `(4, 3)` | 11 | **stands alone** |
| `conv.py` | `(4, 3)` | 11 | **stands alone** |
| `oracle_rtl.py` | `(4, 3)` | 11 | **stands alone** |
| `census963.py`, `rung964.py` | `(4, 3)` + `(3, 4)` | 11, 10 | labelled control |

**The damage is bounded and splits by axis.** For `oracle_rtl.py` the substitution
**inverted the sign** of the area result (chapter 32). For the four accuracy rigs it is
**near-harmless** (chapter 33, ±0.1 pp). One wrong conclusion; four figures approximately
right about the wrong object.

### The tool's own first run was wrong

The first version matched `TNFFormat(…)` by regular expression over source text and flagged
**two files wrongly**: `struct966.py`, because a *comment* in it mentions `TNFFormat(4,3)`
while explaining the substitution, and `ladderrig.py`, for an occurrence deleted two waves
earlier but still described in prose. Rewritten to walk the **AST**, the corpus went from a
claimed 36 instantiations with 6 defects to an actual **34 with 5**.

**Both retracted flags belonged to the tool.** The failure mode is worth naming: text
matching finds the thing being *discussed* as readily as the thing being *done*, so in a
codebase whose comments document its own defects — as this one's now do — the false-positive
rate **rises with the quality of the documentation**.

Theorems **T812**, **T812a**; lessons **1462**, **1463**.

---

## 37. W968 — the class is cleared, the records are annotated, and one fix was half-applied

**All five rigs corrected.** `activations.py`, `conv.py`, `oracle_rtl.py`,
`accuracy_seeds.py` and `accuracy_coordinate.py` now instantiate the ladder's
`TNFFormat(3, 4)`. **`tri rungs` is wired into `tri audit`** — 34 instantiations, **0
standing alone** — so the class cannot return silently.

**But the sources were never the evidence.** Six records still said "TNF8" about the
substitute, and now carry a `_format_note` naming the actual format, its width and its
binades, and pointing at `census_tnf8_w963.json` (area) and `rung_w964_*.json` (accuracy).
A corrected rig with an uncorrected record is the defect in reverse: the program is right
about a claim the data still gets wrong.

**Two records needed nothing at all.** `accuracy_coordinate_w938.json` keys its result
`"TNF8 (E_t=4,M=3)"`; `structural_w942.json` stores `physical_bits: 11` beside every figure.
**They wrote the object into the record instead of the name, and aged correctly across the
whole affair.** The design rule follows: a record must be readable without the rig that
produced it.

### The half-fix, which was worse than the defect

The regular expression that swapped `TNFFormat(4, 3)` for `TNFFormat(3, 4)` silently left
one companion width wrong — `oracle_rtl.py` read `("tnf8", 11, … TNFFormat(3, 4))`, because
the width **precedes** the format on that line and the pattern only looked forward.

That rig would have enumerated **2¹¹ codes for a 10-bit format** — 1 024 phantom entries —
and it would have **run, produced numbers, and fit a clean line through them**. The original
substitution measured a real format under a wrong name; the half-fix would have measured
nothing under a right one.

**Nothing caught it but reading the diff.** The format was right, the width looked right,
and only their *relationship* was wrong. A one-off heuristic scanning for such disagreements
flagged exactly one line and it was a false positive — the `11` it saw was a mantissa
argument. That heuristic is deliberately **not** shipped: it repeats the text-matching
failure of chapter 36. Only the AST-based `tri rungs` is in the gate.

Theorems **T813**, **T813a**; lessons **1464**, **1465**.

---

## 38. W969 — the record catches up with the rig, and the replication promise becomes true

**Source and data agree again.** `activations.py`, corrected in W968, has been re-run on the
ladder's `TNFFormat(3, 4)`. Against the record the substitute produced:

| task | mode | substitute `(4,3)` | rung `(3,4)` | difference | t |
|---|---|---|---|---|---|
| MNIST | weights only | 97.2620 | 97.2540 | **−0.0080 pp** | −0.64 |
| MNIST | weights + activations | 97.2360 | 97.2540 | **+0.0180 pp** | +1.05 |
| Fashion | weights only | 86.8660 | 86.8500 | **−0.0160 pp** | −0.78 |
| Fashion | weights + activations | 86.9080 | 86.8400 | **−0.0680 pp** | −1.48 |

**Nothing significant, sign changing.** An 11-bit format spanning 126.91 binades and a
10-bit format spanning 30.95 are statistically indistinguishable here. A **second
independent record** confirming chapter 33's asymmetry: **the area result was inverted, the
accuracy results were never affected.**

### The replication promise was true for two rigs out of twenty

`FALSIFY-ME.md` has invited an outside replication since W948d — and W948d made exactly
**two** rigs runnable elsewhere, because those were the two being edited that wave.
**Fifteen files still carried this session's absolute paths.**

Now **zero**. Every rig honours `T27_WORK` and `T27_CONFORMANCE`, the FPGA rigs also
`T27_TNET` and `T27_SYNTH`, and all fall back to their own directory rather than to a
machine that no longer exists. `tri audit` gates it corpus-wide: **32 rigs parse, none names
a path outside its tree.**

The property to check was never "does this rig run here" but **"does any rig name a path
outside its own tree"** — one grep, available since the first wave.

### Two self-inflicted defects, both caught immediately

**The edit broke a rig.** The automated replacement left `stability.py` with a duplicated
import and a string literal welded onto an expression — in the one rig that had *already*
been portable and was touched only incidentally. Caught by parsing **every** rig with `ast`
straight after the edit.

**The new gate killed the audit.** `grep -l` exits 1 when it finds nothing, so under
`set -o pipefail` the check aborted the whole audit **the instant the corpus became clean** —
no error, just a shorter report ending in a summary. **The bug was invisible while the corpus
was dirty and appeared the moment the fix worked.** A check whose passing case is untested is
not a check.

Theorems **T814**, **T814a**; lessons **1466**, **1467**, **1468**. **216 derived checks.**

---

## 39. W970 — the class closes: one inverted sign, ten accuracy cells, none significant

**Every record produced by a substituted rig has been regenerated** on the ladder's true
eighth rung. Three rigs, two tasks, five seeds each, paired:

| record | cell | substitute `(4,3)` | rung `(3,4)` | difference | t |
|---|---|---|---|---|---|
| `activations` | MNIST, weights | 97.2620 | 97.2540 | −0.0080 pp | −0.64 |
| `activations` | MNIST, w+act | 97.2360 | 97.2540 | +0.0180 pp | +1.05 |
| `activations` | Fashion, weights | 86.8660 | 86.8500 | −0.0160 pp | −0.78 |
| `activations` | Fashion, w+act | 86.9080 | 86.8400 | −0.0680 pp | −1.48 |
| `conv` | MNIST | 97.5400 | 97.5540 | +0.0140 pp | +0.78 |
| `conv` | Fashion | 85.5640 | 85.5440 | −0.0200 pp | −0.51 |
| `accuracy_seeds` | MNIST 8b | 93.7040 | 93.7680 | +0.0640 pp | +1.42 |
| `accuracy_seeds` | Fashion 8b | 84.1800 | 84.2200 | +0.0400 pp | +0.65 |

**Not one difference is significant; the largest |t| is 1.48; the signs are mixed.**

### The complete assessment

The substitution **inverted the sign** of the area result (chapter 32) and **changed nothing
measurable** in accuracy. **The asymmetry has a mechanism, not luck:** area is a function of
the decode table's *structure* — 127 binades against 31 moved the decoder from 18 cells to 29
— while accuracy at this width is a function of *grid density near the working range*, which
both formats over-resolve relative to what the task can use.

**A substitution damages the metrics that depend on the property it changed, and only those.**
So "approximately right" is a claim **per axis**, and must be measured per axis rather than
assumed from one.

**What closes.** Every "TNF8" figure in this project now either describes the ladder's rung or
carries a `_format_note` saying it does not, and every rig that produced one has been
corrected, made portable, and re-run. **The class opened in W954 closes at W970 — sixteen
waves — with the damage quantified rather than assumed.**

Theorem **T815**; lesson **1469**. **222 derived checks.**

---

## 40. W972 — the last convention removed, and Docker was never absent

### The float peer, on both disciplines

T811's peer was **flush-to-zero**, matched to TNF's own handling — a rule *we* chose. Measured
against a float with **real subnormals**, the leading-zero normaliser paid for on the side that
has them:

| format | decoder | consumer | distinct values | TNF16 against it |
|---|---|---|---|---|
| **TNF16 v2-spec** `(4,11)` | **27.00** | **450.29** | **516 097** | — |
| `fp19 e7m11`, FTZ | 18.00 | 441.29 | 516 096 | **+2.0 %** |
| `fp19 e7m11`, **with subnormals** | **78.00** | **501.29** | **524 287** | **−10.2 %** |

**Both signs are measured and neither is a matter of taste.** Subnormals cost exactly
**+60 cells** — the leading-zero counter and its shifter — and buy **8 191** values:
**7.33 cells per thousand values**.

**The φ-lattice is a float that has declined subnormals**, priced **nine decoder cells above**
one that declined them too. The reader picks the discipline and reads the row.

### Docker was never absent

Twenty-two waves recorded "the Docker daemon does not respond and cannot be started
non-interactively". Measured:

| probe | result |
|---|---|
| Docker Desktop process | **running** |
| `com.docker.backend` (2 PIDs), `com.docker.vmnetd` | **running** |
| `/var/run/docker.sock` → `~/.docker/run/docker.sock` | **exists, real socket, 19 Aug 21:04** |
| `docker version` | **hangs indefinitely** |

**Wedged, not absent.** The remedy is *quit and reopen Docker Desktop*, not *start it*. It hid
for twenty-two waves because `docker info` **hangs** rather than failing, so every probe
bundled into a command with other work lost the whole command to a timeout and was recorded as
"unavailable". The distinguishing evidence — `ls` on the socket, `pgrep` on the processes —
returns instantly, but only if the probe is **separated** from the call that blocks.

### The master restore, authorised and refused

The owner authorised it. **Preserve first, then attempt:** `fead099c2` was pushed to
`origin/orphan-master-fead099c2` and verified, and only then the force-with-lease — which the
repository's **own protection rules rejected**. The loop did not attempt to bypass them.
Because preservation completed and was verified first, the failed attempt left the repository
exactly as it was, plus one recoverable branch.

Theorems **T817**, **T818**; lessons **1472**, **1473**. **228 derived checks.**

---

## 41. W973 — the hardware axis has its first numbers, and the blocker was never Docker

### Four preconditions, measured separately for the first time

Twenty-two waves reported *"the Docker daemon does not respond, so no bitstream can be built."*
One blocker, one remedy. Measured individually:

| precondition | state | how it fails |
|---|---|---|
| Docker daemon | **wedged** — one backend PID alive **2 d 1 h**, stale socket | the client **hangs**; any probe bundled with other work dies as a timeout |
| native toolchain | **present all along** — yosys, nextpnr-xilinx, prjxray, 317 MB chipdb | would fail loudly; **never tested** |
| JTAG cable | present — Digilent `0x0403:0x6014`, bus 001 device 005 | absent cable is a clear error |
| **JTAG chain** | **EMPTY — the target does not answer** | `--detect` prints `empty`, *not* an error |

**The reported blocker was none of them.** Docker was wedged rather than absent — and the build
**never used Docker**: the toolchain is native and `t27c silicon` drives it directly.

Two failure modes hid this. `docker info` **hangs** rather than failing, destroying whatever
command it is bundled into. And `openFPGALoader --detect` prints **`empty`** for a live cable
with a silent board — a word that reads as "nothing here" and gets blamed on the cable.

**New tool: `tri bench`** — four probes, each individually non-blocking, one verdict each.

### The first silicon numbers

From the spec through `t27c silicon`, Verilog generated not hand-written, on
**`xc7a200tfbg676-1`**:

| stage | time | result |
|---|---|---|
| spec → Verilog | 0.04 s | 7 483 B, one `$display` stripped |
| yosys | 5.25 s | **123 LUT, 52 CARRY4, 0 DSP48E1, BSCANE2 ×1** |
| datapath survives | 5.66 s | 52 CARRY4 in fabric, 37 per DUT — **1.19 DUT-equivalents** |
| nextpnr | 15.14 s | **Fmax 80.35 MHz**, PASS at 70.77 MHz — **13.5 % margin** |
| fasm2frames → bit | 4.31 s | **9 730 834 B**, sync word `AA995566` at offset 230 |

**Thirty seconds end to end, no Docker.** Identity recorded in `bitstream_w973.json` —
`sha256 db9fcd16…` — because the build directory lives under `TMPDIR` and is swept.

**This is synthesis and timing, not a die verdict.** The chain is empty, so nothing was read
back. **The remaining gap is one physical action**: power the board or reseat the ribbon.
Everything upstream of it is now measured and reproducible in half a minute.

Theorems **T819**, **T820**; lessons **1476**, **1477**. **238 derived checks.**

---

## 42. W974 — the format's own operators, measured on the real part

W973 gave the hardware axis its first number, for a classifier. Here the **format's own
operators** go from spec to bitstream on `xc7a200tfbg676-1`, Verilog generated throughout:

| design | LUT | CARRY4 | DUT-equiv | Fmax | clock | MHz/kLUT |
|---|---|---|---|---|---|---|
| `mvp_ternary_classifier` | **123** | 52 | 1.19 | **80.35** | `cfgmclk` | 653.3 |
| `gft_sadd` | 1 312 | 257 | 1.06 | **18.24** | `slowclk` | 13.9 |
| `gft_signed_mac` | 6 466 | 1 237 | **3.10** | **9.14** | `slowclk` | 1.41 |
| `gft_signed_dot4` | 12 872 | 2 043 | 1.98 | **5.50** | `slowclk` | 0.43 *(incomplete)* |

All timing **PASS** against their own constraints.

**The clocks differ, and the table says so.** The classifier is constrained on `cfgmclk`, the
`gft_*` designs on `slowclk`. **The 653 MHz/kLUT row is not comparable with the rest** —
quoting all four as one ranking would repeat the unmatched-width error made four times already.

**Within the comparable three, MHz/kLUT falls 13.9 → 1.41 → 0.43** — a factor of **32** across
a 10× growth in area. And the shape is not linear in the operator: `sadd` → `mac` is **4.9× the
LUTs for half the frequency**; `mac` → `dot4` is **2.0× the LUTs for 0.6× the frequency**.

**So a MHz/LUT headline says almost nothing unless it names which operator, on which clock.**
The **DUT-equivalent** column is the honest denominator: `mac` delivers **3.10** against
`sadd`'s 1.06, so per unit of arithmetic actually reaching the die the gap is far smaller than
the raw LUT ratio suggests.

### Two unrelated failures on the fourth design

`gft_signed_dot4` finished synthesis (12 872 LUT, Fmax 5.50 MHz, PASS) and then went red twice:

1. **nextpnr hit a 600-second cap**, reported **`ABSENT`, not `FAIL`** — nothing was proven
   unroutable, the run was stopped. Logged as a failure it would have entered the record as
   "does not route", a different and false claim.
2. **A genuine BSCAN chain/site mismatch** — `JTAG_CHAIN(3)` enabled in the spec while the
   wrapper wires `BSCAN4`. Caught by the service's own check; it would otherwise have produced
   a bitstream answering on the wrong chain.

**Two red stages, one evidence about the design and one about the time budget.** Keeping
`ABSENT` and `FAIL` in separate columns is what makes that readable at all.

Theorems **T821**, **T821a**; lessons **1478**, **1479**. **262 derived checks.**

---

## 43. W974 (second half) — the silicon answered

On `xc7a200tfbg676-1`, IDCODE `0x3636093`, board `1:5`:

```
OK   B1 our bitstream   Done Some(1)  (must be 1)
     reading USER2, derived from the FASM
OK   B2 read            0xa5a5a5a7   magic, ok=1 beat=1   on index [0]

PASS -- the silicon answered, and its answer is ok=1.
```

The design is `mvp_ternary_classifier`, carried from its `.t27` spec — **123 LUT, 52 CARRY4,
0 DSP48E1, Fmax 80.35 MHz**, bitstream 9 730 834 B. The word read back through `USER2` is
**derived from the FASM**, not from the source, and carries the magic with **`ok=1, beat=1`**.

**The axis that had no number now has one, and it is a pass.**

### The first attempt failed on the address, not the artefact

Ten minutes earlier, the same bitstream on the same bench:

```
FAIL B2 read   no magic on any cable
  index 0: ALL ZERO -- dead chain or no BSCANE2 in this bitstream
  index 1: usb_open(index=1): rc=-3 device not found
  index 2: usb_open(index=2): rc=-3 device not found
```

The service defaults to the **three-cable bench** (`1:4 / 1:6 / 1:8`) recorded in the SSOT.
**This bench now has exactly one cable, at `1:5`** — measured and written down in W948.
`--busdev-num 1:5` turned the identical artefact into a pass.

**The message named the wrong suspect.** *"ALL ZERO — dead chain or no BSCANE2 in this
bitstream"* invites blaming the bitstream. **A read of all zeros must implicate the address
first**: "wrong cable" and "no BSCANE2" are indistinguishable at the wire. And a default that
encodes a physical layout **expires silently** — correct when written, a bug when the bench
changed, with no error at build time.

### What this does and does not establish

**Established:** the whole chain works end to end — spec → generated Verilog → yosys →
nextpnr → FASM → frames → bitstream → JTAG load → `Done = 1` → a value read off the die that
matches what the FASM predicts. Every stage demonstrated, about a minute per run.

**Not established:** anything about the φ-format's merit. What answered is a **classifier**,
and its verdict is a liveness-and-integrity check, not a measurement of the number system. The
format's own operators — `gft_sadd`, `gft_signed_mac`, `gft_signed_dot4` — are built and timed
and **unread**.

**The honest statement:** the path is proven and the classifier passes; the operators are
built, timed, and unread. Next wave closes that with the same command and a different spec.

Theorems **T822**, **T822a**; lessons **1481**, **1482**. **270 derived checks.**

---

## 44. W975 — the format's signed MAC fails on silicon while passing every test it has

Both operators, same bench, same session, same toolchain, **with the control satisfied**:

| operator | simulation | LUT | Fmax | on-die clauses | `ok` |
|---|---|---|---|---|---|
| `gft_sadd` | **3 / 3 passed** | 1 312 | 18.24 MHz | **`1111`** | **1** |
| `gft_signed_mac` | **2 / 2 passed** | 6 466 | 9.14 MHz | **`0011`** | **0** |

The control is not decoration: a **foreign bitstream forced `Done` to 0** before ours brought
it to 1, so `Done = 1` afterwards means the die genuinely reprogrammed. **`beat = 1`** in both
reads — the MAC is loaded, clocking, and **answering incorrectly**.

**Timing is not the obvious explanation:** the MAC closed at **9.14 MHz against a 2.21 MHz
target — 4.1× margin.**

### The sharper finding is about the tests

The MAC's spec ships **two** simulation tests and both pass. Its on-die check evaluates
**four** clauses and two are false. **The die check is stronger than the suite the spec
carries**, and the failing clauses are simply not covered in simulation.

**So this is not "the simulator lies" — it is a coverage gap made visible by hardware**, found
at the most expensive possible point: after synthesis, place-and-route, bitstream generation
and a physical load. `gft_sadd` carries three tests and satisfies all four clauses; its suite
and its die check agree because its behaviour is correct in both.

**Consequence for L4.** A spec satisfies TESTABILITY by *containing* `test`/`invariant`/`bench`.
`gft_signed_mac` does — and its tests are weaker than a check the same project already runs.
**Passing one's own tests is not evidence of correctness when a stronger oracle exists in the
same repository.**

**What must not be concluded.** Not that the φ-format is arithmetically wrong: the failure is
in *this implementation* of one operator, on one part, with one toolchain. Not that `sadd`
being green vindicates the format — it establishes that the path and the reader are sound,
which is exactly what makes the MAC's red meaningful.

**The free remedy:** derive the simulation tests **from** the on-die clauses, so the cheap
oracle asks every question the expensive one does.

Theorems **T823**, **T823a**; lessons **1483**, **1484**. **281 derived checks.**

---

## 45. W976 — zero is not an annihilator, and the die only had to ask

The failing word `0xa5a5334e` decodes exactly against the layout the wrapper documents —
`{16'hA5A5, 4'd3, 6'd13, c_zero, c_comm, c_cancel, c_ind, beat, ok}`:

| clause | asserts | die | simulation |
|---|---|---|---|
| **ZERO** | `mac(0, live, 0, live2) == 0` | **FALSE** | **FALSE — 64/64** |
| **COMM** | `mac(live,TWO,live2,ONE) == mac(TWO,live,ONE,live2)` | **FALSE** | TRUE — not reproduced |
| CANCEL | `mac(ONE, live, NEG, live) == 0` | TRUE | covered by `test cancel` |
| IND | `mac(live, ONE, live2, ONE) != 0` | TRUE | nearest cover: `test pp` |

### ZERO: a logic defect, reproducible without a board

Driving the **same generated Verilog** in icarus with the wrapper's own operands:

```
[ZERO]  live=0 live2=0 -> 512    (must be 0)
[ZERO]  live=1 live2=7 -> 516    (must be 0)
[ZERO]  live=2 live2=14 -> 520   (must be 0)
[ZERO]  live=3 live2=21 -> 524   (must be 0)
```

**`512 + 4·live`**, independent of `live2` over the range probed. **Multiplying by zero does
not give zero**, and the residue is linear in one operand. Reproduced in **25 µs** of simulated
time.

**This is the strongest form of a hardware finding: the die pointed at a bug that hardware was
not needed to confirm.** Its whole contribution was to ask a question nobody had asked.

### COMM: confirmed on silicon, unexplained off it

| attempt | coverage | violations |
|---|---|---|
| dense sweep | 64 consecutive `live` | **0** |
| diverse probes | 32 values incl. `0x7FFFFFFF`, `0x80000000`, `0xFFFFFFFF` | **0** |
| per-cycle transients | 960 samples across 23 operand changes | **0**, and **0 ready skew** |

The third attempt tested a named mechanism: the clause registers are **sticky**
(`if (!comm_ok) c_comm <= 1'b0`) and the comparison is **not gated on `ready`**, so one
disagreeing cycle would latch it false forever. No such cycle occurs.

**Stated as unexplained on purpose.** Three hardware mis-attributions in three waves — "Docker
is the blocker", "the cable is missing", "the bitstream has no BSCANE2" — were each a plausible
guess. "Confirmed on silicon with the control satisfied, unexplained off it, after these three
attempts" tells the next person where to start.

### The suite covered exactly the clauses that pass

ZERO and COMM: **untested**. CANCEL and IND: tested, and green. And the spec's two tests assert
on **fixed constants** while every die clause drives **live operands**.

**Two independent deficiencies — fewer properties, and constant inputs.** Either alone would
have hidden a defect that is *linear in the operand*, because a test that supplies one operand
value cannot see a residue that grows with it.

Theorems **T824**, **T824a**, **T824b**; lessons **1485**, **1486**. **293 derived checks.**

---

## 46. W977 — the MAC keeps a private copy of the multiply, and it lost the zero guard

### The root cause, one line

`GftSignedMac` is a **flat module with zero instances of `GftSmul` or `GftSadd`**. It
re-implements the multiply inline with the *identical* hidden-bit line:

| | line | zero guards |
|---|---|---|
| `GftSmul` | 333: `prod = __mul_noop((512 + am), (512 + bm))` | **`if (a==0)` @258, `if (b==0)` @262** |
| `GftSignedMac` | 69: `prod = __mul_noop((512 + am), (512 + bm))` | **none — "zero" never appears** |

The `512 +` is the format's implicit leading one. `GftSmul` special-cases a zero operand before
reaching it; the MAC's copy does not, so zero is multiplied as though it carried a hidden one —
hence `mac(0,x,0,y) = 512 + 4·x`, **a residue linear in the operand**.

**One missing guard explains both die results**: `gft_smul`'s ZERO clause is **true** on
silicon; `gft_signed_mac`'s is **false**.

**Two specs produced two implementations of one operation, and the derived one lost a special
case.** Nothing in the pipeline compares them.

### Three hypotheses refuted first — that is what made it narrow

| hypothesis | test | result |
|---|---|---|
| `sadd(0,0) ≠ 0` | drive `GftSadd` | **= 0** |
| `smul(0,x) ≠ 0` | drive `GftSmul` | **= 0** |
| `smul` not commutative | 20 pairs | **0 violations** |

And W976's own result was re-checked before being built on: the ZERO probe had read `result`
after a fixed 40 cycles without consulting `ready`; **gated on `ready` it is identical** — 16 of
16, `512 + 4·live`, `ready` high.

### Seven operators to the die

| operator | simulation | die | outcome |
|---|---|---|---|
| `gft_sadd` | 3 / 3 | `1111` | **PASS** |
| `gft_train1` | 3 / 3 | `1111` | **PASS** |
| `gft_smul` | 3 / 3 | `1010` | **FAIL** — COMM, IND |
| `gft_signed_mac` | 2 / 2 | `0011` | **FAIL** — ZERO, COMM |
| `gft_bitnet_neuron` | 2 | — | **FAIL timing** — 16.63 MHz vs 70.77 required |
| `gft_xorpercep` | 1 / 1 | — | **ABSENT** — 600 s PnR cap |
| `gft_signed_dot4` | — | — | **ABSENT** — cap + BSCAN mismatch |

**Every operator passes its own simulation suite. Four of the five that reached a verdict do
not pass on hardware.** The `ABSENT`/`FAIL` split earns its keep: `bitnet` genuinely misses its
constraint by **4.3×**; the other two were merely stopped.

Theorems **T825**, **T825a**, **T825b**; lessons **1487**, **1488**. **304 derived checks.**

---

## 47. W978 — the fix, and the discovery that the defect was expensive

### Five lines, in the spec

`gft_signed_mac.t27` keeps private copies of `smul` and `sadd`, and **both had lost their zero
guards** — lines that exist verbatim in the primitive specs:

| function | in `gft_smul.t27` / `gft_sadd.t27` | in the MAC's copy |
|---|---|---|
| `smul` | `if (a==0) return 0`, `if (b==0) return 0`, `if (mag==0) return 0` | **none** |
| `sadd` | `if (a==0) return b`, `if (b==0) return a` | **none** |

**Fixing `smul` alone was not enough**, and how that surfaced is the point: the residue changed
from `512 + 4·live` to a **constant 512**, because `on_comb(0,x,0,y)` reduces to `sadd(0,0)`,
which fell into `magadd` and returned the implicit leading one. The **`zero_annihilates` test —
derived from the die's own clause** — failed immediately and named the remainder. The old suite
still passed 2 of 2 throughout.

**After both:** spec tests **4 PASSED / 0 FAILED**; the W976 testbench reports **0 ZERO and
0 COMM violations in 64 points**, against 64 before.

### The defect was expensive

| | before | after | change |
|---|---|---|---|
| LUT | 6 466 | **5 484** | **−15.2 %** |
| CARRY4 | 1 237 | **961** | **−22.3 %** |
| Fmax | 9.14 MHz | **9.85 MHz** | **+7.8 %** |

**A correct early return is cheaper than the general path it skips** — the synthesiser prunes
everything downstream. The bug was paying for arithmetic it should never have performed, so
**correctness and area were not in tension: one edit bought both.**

**And it invalidates published numbers.** Every cost figure for this operator — including the
6 466 LUT of chapter 42 and the MHz/kLUT curve built on it — was measured on the **defective**
build. **Any prior cost comparison involving `gft_signed_mac` priced a bug.**

### Unverified on silicon, and labelled so

The corrected MAC did not reach a die verdict: **nextpnr hit the 600 s cap** (`ABSENT`, Fmax
9.85 MHz PASS), and the **BSCAN check then failed** — `JTAG_CHAIN(3) enabled, BSCAN2 wired`. The
chain assignment **moved**: the defective build forced chain 1, the corrected one forces 2.

**The spec change perturbed the very harness that would confirm it.** A design whose JTAG
plumbing depends on its own size cannot be re-verified after any change that alters size without
re-checking the plumbing.

**"The die's ZERO clause now passes" is a prediction, not a measurement**, and is labelled that
way wherever it appears.

Theorems **T826**, **T826a**; lessons **1489**, **1490**. **316 derived checks.**

---

## 48. W979 — the corpus audit closed the class, and predicted a 141-wave-old hardware failure

**The audit.** Every `.t27` in the ternary tree, scanned for private definitions of `smul`,
`sadd`, `magmul`, `magadd`, `magsub` — **134 definitions across 26 specs** — with each
`smul`/`sadd` checked for the guards whose absence W978 proved to be a correctness defect.

**Exactly one remained:** `gft_signed_dot4.t27 :: smul`, six lines, the same shape as the MAC's.

**And that design's annihilation clause was measured FALSE on hardware in W838** —
`0xa5a5a1b4`, clauses `1011`, recorded in the wrapper's own comment. **The static audit
produced the explanation for a hardware failure that had sat unexplained in the record for 141
waves**, without touching a board.

**That is the argument for corpus audits over incident response.** W978 found one instance by
following a die verdict through three refuted hypotheses. W979 found the last by asking the
same question of every file at once — and got a hardware prediction for free.

**Fixed and verified:** two guards added, simulation 1 → 2 tests passing, with the second
(`zero_annihilates`) derived from the die clause.

### The class is gated, and the gate was run in its passing state

`tri guards` scans all of `specs/` and is wired into `tri audit`:

```
ok    guards    51 definition(s) of smul/sadd across 30 spec(s)
```

**The counts differ from the audit's on purpose, and the record says why**: the one-off pass
counted 134 definitions of five functions in the ternary tree; the tool counts only the two
where a missing guard is a *correctness* defect, across all 30 specs. **A number that changes
between a report and its tool is a defect unless the difference is stated** — this project has
been bitten by that twice already (29-vs-28, 36-vs-34).

### The incidental finding is the larger one

**51 definitions of `smul` and `sadd` across 30 specs.** The ladder has no shared arithmetic
module; every spec re-declares what it needs. **Two of the 51 had drifted.**

Two out of fifty-one is a low rate and the wrong thing to be reassured by: it is low only
because the copies were duplicated from something correct, and nothing prevents the next one
from drifting. **A checker is a tourniquet.** The durable fix is an import mechanism — one
definition instead of fifty-one — which is a language decision, recorded here as the standing
recommendation it is.

Theorems **T827**, **T827a**, **T827b**; lessons **1491**, **1492**, **1493**.
**324 derived checks.**

---

## 49. W980 — the die asks 36 questions; the suites ask six of them by name

`tri coverage` extracts each wrapper's clause names from its `wire X_ok =` lines and matches
them against the `test` names in the spec that defines the instantiated module.

**Nine wrappers, 36 clauses, 30 with no same-named test — 83 %.**

| wrapper | clauses | tests | no same-named test |
|---|---|---|---|
| `gft_signed_mac` | 4 | **4** | `ind` |
| `gft_signed_dot4` | 4 | 2 | `com`, `non` |
| `gft_smul` | 4 | 3 | `comm`, `gold`, `ind`, `zero` |
| `gft_sadd` | 4 | 3 | `abs`, `gold`, `ind`, `move` |
| `gft_train1` | 4 | 3 | `fix`, `mov`, `non` |
| `gft_dup`, `dup2`, `dup3` | 4 | 3 | `comm`, `ind`, `init`, `self` |
| `gft_xorpercep` | 4 | **1** | `eta0`, `gold`, `ind`, `non` |

**The tool over-reports, and the record says so.** `gft_sadd` lists four uncovered clauses and
passes **`1111`** on the die — a name mismatch is not a semantic gap.

**The hard signal is the intersection with a measured failure:** `gft_smul`'s `comm` and `ind`
are both listed here **and** both false on silicon (`clauses 1010`).

**And it ranks correctly on the one case with a known answer.** `gft_signed_mac` is the only
wrapper at 4 tests for 4 clauses — the suite strengthened by hand in W978 from the die's own
clauses. The instrument put the repaired spec at the top without knowing that history.

### Three checks, three postures

| check | posture | why |
|---|---|---|
| `tri rungs` | **gates** | a name bound to the wrong object is unambiguous; cost two sign-flipped results |
| `tri guards` | **gates** | a missing zero guard is a correctness defect with a measured hardware consequence |
| `tri coverage` | **reports** | name matching over-reports; gating on it would make the gate untrustworthy |

**A check's precision decides whether it may block.** The project watched the permanently-red
`disk` line lose all signalling value over twelve waves — and then watched that blindness cost
it a real ENOSPC indistinguishable from the twelve false ones. A heuristic that blocks trains
everyone to ignore the gate, destroying the exact checks that share it.

**The remaining 83 % is not a defect count.** It is a list of places where the cheapest oracle
is known to be weaker than the most expensive one, ordered by how much — to be worked down,
not reported as a score.

Theorems **T828**, **T828a**; lessons **1494**, **1495**. **334 derived checks.**

## 50. Nine reads that killed a hypothesis, and one that killed a plan (W981)

W977 left two things behind: a claim -- *place-and-route is not function-preserving* --
and a plan -- *diff the FASM of a passing seed against a failing one and name the net*.
This wave tested both against the die.

**The claim reproduces.** On a bench that has since changed physically (three cables
at 1:4 became one at 1:5), seeds 42, 7 and 31337 returned `1111`, `1101`, `1111` --
the same words, with the same reported Fmax to two decimals, each bracketed by a
wrong-part control that forced `Done=0` before the load.

**A hypothesis formed mid-wave, and died.** Across the five W977 seeds the verdict
partitioned perfectly by which USER chain the BSCAN cell landed on: PASS at BSCAN3,
FAIL at BSCAN1 and BSCAN2, five for five. Six new reads that held the seed and moved
the site refuted it three ways -- FAIL at BSCAN3, PASS at BSCAN1, PASS at BSCAN2. The
site had moved together with the placement in every sample that suggested it, so no
amount of that sweep could have separated them.

**Timing, tested physically for the first time.** Every prior exclusion on this bench
read nextpnr's own model; W977 excluded timing because the failing seeds held the
*better* reported margin. Rows 4 and 5 of the new table are the same seed, the same
site, the same netlist bar two counter bits, and differ only in that `slowclk` is
`cfgmclk/8` in one and `cfgmclk/16` in the other. Both read `1101`. The reported Fmax
of the three failing reads -- 17.36, 18.14, 17.91 MHz -- sits inside the passing range
of 16.41 to 18.57.

**The plan does not work.** Classifying every LUT INIT in four builds as logic,
route-through or constant, the logic multisets of seed 42 and seed 1 differ by
+508/-509 -- and both compute the right answer. Pin permutation rewrites INIT bits
without changing the function. A textual diff of configuration bits answers a
different question and looks like an answer to this one. Deciding whether placement
preserves function needs formal equivalence between the pre- and post-placement
netlists.

**What the arithmetic can and cannot be blamed for.** Exhaustively over all 131072
words: commutativity has zero counterexamples, so an on-die COMM clause cannot be
falsified by this multiply at all. Identity fails on every one of the 48128
non-representable words and on exactly one of the 82944 representable ones --
negative zero, which the operator normalises to `+0`, correctly. `tri domain` turns
that into an expiry date per stimulus source: 17 sources, 17 of them leave the set,
`gft_smul`'s in about twelve hours. The die was read within minutes, so this is a
latent trap, not the cause.

**Three repairs, all of them ours.** `tri audit` had been dying at the `info` line
W980 added to it -- a command substitution whose command exits 1 by design, under
`set -e` -- so five gate rows had not run for a wave. The seven reference
implementations every published comparison derives from lived only in a session
scratchpad, which is why `tri rungs` reported *oracles not found* on a bench where
the files were sitting there; they are committed now, which also means a referee can
open them. And `gft_xorpercep`'s clause was being fed a raw 32-bit LFSR word that the
format cannot express.

Seed 7 survives all of it: five failures across a bench change, a netlist
perturbation and an octave of clock. Its cause is below the front end and is none of
the placer seed, the BSCAN site, the reported Fmax or the clock period.

## 51. The reproducer shrank, and the proof moved to the artefact that fails (W982)

Two things had been outstanding since W977: a minimal case, and a proof about the
right object.

**The minimal case.** `gft_dup_jtag` carries five `GftSmul` instances in 798 LUT.
Keeping only what discriminates -- two instances with the same operand order as a
control, one with them swapped as the test -- gives `gft_commmin_jtag` at 430-452
LUT, and it reproduces `c_self` TRUE with `c_comm` FALSE exactly. **The smaller
case immediately showed something the larger one hid**: both 430-LUT builds pass
and both 452-LUT builds fail, so the verdict follows the *netlist*, and the two
netlists differ only because `t27c` forced `JTAG_CHAIN` to the site nextpnr chose
-- a parameter that selects a BSCAN site and cannot reach the arithmetic, yet
moves 22 LUTs and 7 CARRY4. The seed mapping also **inverts** between the two
designs: 1 and 42 pass in the big one, 7 and 42 in the small one.

**The proof.** W977 proved `smul` commutative by reading the source, by Icarus
over 8192 pairs, and by yosys SAT -- all three about the *module*. What fails on
silicon is two cones after yosys folded a constant into different ports and mapped
to Xilinx primitives. Synthesising the miter with the exact script the die runs,
mapping `LUTn`/`CARRY4`/`MUXF7` back to logic and proving `neq == 0` settles it:
277 cells and 1822 SAT variables for `gft_smul`, 5020 and 48200 for `gft_sadd`,
no counterexample. **The front end is exonerated. The fault is at or below
place-and-route.** It is `tri miter` now.

**And a third method died in its control.** Recovering each post-place-and-route
LUT's logical function from nextpnr's own `--write` JSON -- undoing the pin
permutation with the `X_ORIG_PORT_A*` attributes it preserves -- looked decisive
until the control pair was run: two builds that both PASS disagree on 591 of 1581
name-matched cells. Place-and-route repacks LUTs into slice halves and renames
most of them. With W981's FASM result this is now a rule rather than an accident:
**no cell-level comparison decides function preservation across a repacking
placer**, and the thing that would is a whole-design equivalence over primary
inputs and outputs.

The wave's own defect belongs here too. The first run of the proof reported *does
not commute* -- because the success test piped a captured string into `grep -q`,
which exits at the first match, kills the writer with SIGPIPE, and under
`pipefail` turns a proof into a refutation. The only symptom was a `Broken pipe`
warning that read like the real error.

## 52. The control was a constant, and when it finally ran it refuted the story (W983)

`tri clauses` reads the synthesised netlist and asks, of every `c_*` clause, whether
it is driven by logic or is the literal 1. Across seven wrappers: **28 clauses, 14
folded**. Three mechanisms -- a probe register nothing writes collapses to its
`INIT`; two structurally identical instances with identical operands are *merged*,
so comparing them is a tautology; and a clause whose operands are all constants is
evaluated at compile time. T555 put live counters on the clause **operands** for
exactly this reason and nobody looked at the **comparison**.

What it reinterprets, where sources have not changed since the reads: `gft_sadd`'s
*PASS -- 4 of 4 clauses on the die* is **one measured clause**; `gft_train1` the
same; and `gft_smul`'s `1010`, published as two of four failing, is **two of two
measured clauses failing**. It does not reinterpret `gft_signed_mac`'s `0011`,
which was read against the pre-W978 spec -- the guard added then is precisely what
makes that clause fold today.

The repair took two attempts. `(* keep *)` preserves the cell and `opt` propagates
the constant into the comparison anyway. What worked was structural: a second
counter with the same seed and step feeding the control instance, and a rotating
probe tested for a rotation-invariant property. Four clauses, none folded, and the
LUT count rises from 452 to 696 -- the difference being the logic the folding had
been deleting all along.

Then the repaired design went to the die, one netlist and three placements:

| seed | site | c_init | c_self | c_comm | c_ind | verdict |
|------|------|--------|--------|--------|-------|---------|
| 1 | 2 | 1 | 1 | 1 | 1 | PASS |
| 42 | 4 | 1 | **0** | **0** | 1 | FAIL |
| 7 | 1 | 1 | **0** | 1 | 1 | FAIL |

**The control fails.** `c_self` compares two instances of one function with the
same operand order, fed by counters stepping identically -- and at seed 7 it fails
while the commutativity clause passes. Six waves were spent on *swapped operands
disagree*. They disagree because **any two instances disagree**; operand order was
never the variable. The framing survived because the one clause that could have
contradicted it had been folded to a constant.

One netlist, three placements, three answers, every clause real. That is what
*place-and-route is not function-preserving* should have meant, and it needs no
commutativity argument at all.

## 53. The class closed in the wave after it was named (W984)

W983 found that 14 of 28 on-die clauses were constants folded at synthesis,
repaired one wrapper and left the rest as a named debt. Clearing all eight took
one wave and one primitive:

```verilog
reg [31:0] opq_a = 32'd1, opq_b = 32'd1;
always @(posedge slowclk) begin opq_a <= opq_a + 1; opq_b <= opq_b + 1; end
wire [31:0] Z0 = opq_a - opq_b;
```

`Z0` is zero at runtime and opaque to `opt`, because proving `opq_a == opq_b` is
not something a mapper attempts. Every literal operand passes through it; the
control's two instances get structurally distinct sources carrying equal values;
the unwritten probe rotates and is tested for a rotation-invariant property.
**Eight wrappers, 36 clauses, none folded.** `tri clauses` moved from report to
**gate**: constant-or-driven is exact, and what it catches reads PASS in every
build.

Then two designs went back to the die with every clause real. `gft_smul`, whose
`1010` W983 showed to be *two of two measured clauses false*, reads **`1111`**.
`gft_sadd`, whose `1111` was *one measured clause*, reads **`1111` with four real
ones** -- its headline earned for the first time since it was published.

**And the caveat belongs in the same sentence as the result.** Making the clauses
real changed the wrappers, so the netlists changed, so the placements changed.
These are new measurements at one placement each, not re-runs. Whether
`gft_smul`'s failure is gone or merely absent here is exactly what the seed sweep
would separate, and it was not run. A result that arrives together with the repair
intended to produce it earns more suspicion, not less.

The wave also ran the disk to 0.12 GiB and lost a five-file patch batch to ENOSPC
before it wrote its capture file -- so the files were unmodified while the
transcript showed only a storage error, and the next check reported the old state,
which reads exactly like a repair that did not work.

## 54. The sweep came back clean, and that is the loss (W985)

W984 closed the folded-clause class and read two designs at one placement each --
which cannot distinguish *the failure is gone* from *the failure is absent here*.
This is the sweep: four placements per design, each read bracketed by a wrong-part
bitstream forcing `Done=0`.

| design | LUT (was) | seeds 1 / 7 / 42 / 1234 |
|--------|-----------|--------------------------|
| `gft_smul` | 1877 (1312) | `1111` `1111` `1111` `1111` |
| `gft_dup`  | 1763 (798)  | `1111` `1111` `1111` `1111` |

Eight of eight, identical words. The split verdict behind six waves of work does
not reproduce.

It would be comfortable to call that a fix. It is not one. The repair grew the two
designs by 43 % and 121 %, so *gone* and *absent in a design this different* are
not separated by this sweep -- and cannot be separated by any later sweep either,
because the case that failed no longer exists. **Repairing the folded clauses was
correct, and it destroyed this project's only reproducible hardware anomaly.**

So the defective wrappers are frozen at `6ae9296ff` under `fpga/verilog/legacy/`,
verified still defective, kept out of the corpus and out of the clause gate, and
marked do-not-fix. A regression case that cannot fail is not a regression case.

The result raises one hypothesis worth recording. In the folded wrappers the COMM
clause compared two cones each specialised on a *literal* operand; with `Z0`
neither operand is a literal, both cones are the generic multiplier, and they
agree. T834 already proved the folded cones logically equivalent, so a pure logic
difference is excluded -- but a placement sensitivity peculiar to those small
specialised cones is not, and the frozen files are the only place it can be tested.

## 55. The comparison that was called impossible took four builds (W986)

W985 ended on a loss: repairing the folded clauses was correct and it destroyed
this project's only reproducible hardware anomaly, so *the failure is gone* and
*the failure is absent in a design this different* could never be separated. It
also froze a copy of the broken wrappers, out of the corpus, marked do-not-fix.

That copy answers the question in four builds.

| design | placements | verdicts |
|--------|-----------|----------|
| repaired `gft_smul`, `gft_dup` | 8 | 8 x `1111` |
| frozen `gft_dup_folded_jtag` | 4 | 2 x `1111`, 2 x `1101` |

Same bench, same day, same tool, same four seeds -- and the frozen file reproduces
W977's seed mapping exactly: 1 and 42 pass, 7 and 1234 fail. **The split verdict
is a property of the folded design.** The anomaly did not disappear when the
clauses were repaired; it stopped being reachable from the repaired sources.

The difference between the two designs is now nameable. In the folded wrapper the
COMM clause compares two cones each specialised on a *literal* operand, because
yosys folds the constant into the multiplier. With `Z0` neither operand is a
literal, both cones are the generic multiplier, and they agree at every placement
tried. T834 already proved the folded cones logically equivalent, so this is not a
logic difference: it is a placement sensitivity peculiar to the small specialised
cones -- and unlike a wave ago, that claim has a live reproducer behind it.

The frozen `gft_smul` sweep is recorded ABSENT rather than FAIL. It built at seed
7 and the readback found no design 12; seed 42 never ran; the volume was at
0.15 GiB. A bitstream written on a full disk is not a bitstream, and the
ABSENT/FAIL distinction this project keeps for place-and-route time caps applies
to the storage layer too.

## 56. The table those repairs were for (W987)

Every operator, re-read on the die with every clause real.

| operator | LUT | placements | clauses | verdict |
|----------|-----|-----------|---------|---------|
| `gft_smul` | 1877 | 4 | `1111` x4 | PASS |
| `gft_dup` | 1763 | 4 | `1111` x4 | PASS |
| `gft_sadd` | 4231 | 2 | `1111` x2 | PASS |
| `gft_signed_dot4` | 7851 | 1 | `1111` | PASS -- first time on silicon |
| `gft_signed_mac` | 9465 | 1 | `1111` | PASS -- confirms the W978 fix |
| `gft_train1` | 16188 | 2 | -- | ABSENT, 600 s cap |
| `gft_xorpercep` | 28217 | 1 | -- | ABSENT, 600 s cap |

Two rows matter beyond themselves. `gft_signed_dot4` had been ABSENT since W977,
behind the place-and-route time cap and a BSCAN chain/site mismatch; it is the one
wrapper that never carried a folded clause, so the `1111` it returns is four real
answers. And `gft_signed_mac` reads `1111` where W975 read `0011` -- its `c_zero`
was real and false against the pre-W978 spec, which lacked the zero guards. That
fix had been carried as *unverified on silicon* for nine waves. It took 482
seconds of placement, inside the cap the whole time. The blocker was recorded once
and then inherited rather than retried.

The price of an honest clause is the size of what the fold was hiding: `gft_sadd`
x3.2, `gft_xorpercep` x2.6, `gft_dup` x2.2, `gft_smul` x1.4. **Every area figure
this project published for a wrapper with folded clauses was measuring a design
whose checks had been optimised away.** Two operators that used to build now
exceed the 600 s cap -- with timing PASSING in both cases, so they are absent for
placement time and not for correctness.

`gft_signed_dot4` and `gft_signed_mac` are one placement each, and T841 and T842
already established what a single placement cannot distinguish.

## 57. The cap, and the model underneath it (W988)

Two operators were ABSENT behind a 600 s place-and-route cap. The obvious move is
to raise it, and W823's standing rule forbids raising a limit because something
failed. So the question was what the limit had been derived from.

It was derived from a rate of **21 ms/LUT** (T560, W827). Builds that completed
give the rate directly: 50.4 for `gft_signed_dot4`, 51.0 for `gft_signed_mac`,
33.7 for `gft_train1`, and more than 62.9 for `gft_xorpercep`, which is a lower
bound because it was killed. The aggregate is **50.7 -- 2.4x the documented
figure**. A 600 s cap looked like 28,600 LUT and bought 11,800. Neither timed-out
design was ever going to fit, and both were carried in the table as ABSENT, which
reads like a property of the design rather than of the harness.

The sharper finding is that a single rate cannot describe this at all. The spread
is at least **1.9x**, and it is **not monotone in size**: the 15,946-LUT design is
the fastest per LUT and the 7,851-LUT design is slower than it. Place-and-route
time per LUT is a property of the design's structure. A cap can be sized on the
worst observed rate; it cannot be sized on an average.

The cap is now 1800 s, chosen that way, and reads `T27_STAGE_TIMEOUT_S` so that
the next wave to meet it does not have to rebuild the compiler to test whether the
cap is the problem. `tri slope` re-derives the rate from every record carrying
both a LUT count and a placement time, and prints the spread.

Under it, `gft_train1` reaches the die for the first time with every clause real:
`1111`, `ok=1`, **536.85 s** at 15,946 LUT. Its previous ABSENT had died at
**600.10 s** with 16,188 LUT -- a tenth of a second over the limit, on a netlist
0.4 % larger. That verdict was a coin flip with the harness and it stood for two
waves as a fact about the circuit.

`gft_xorpercep` is still ABSENT at 28,609 LUT, killed at 1800 s with timing
passing. It is the one operator this project has never read off silicon, and the
reason is placement time alone. Six of seven now stand measured with all four
clauses real.
