# Full status, both tracks — one page for whoever wakes up (W940, 2026-08-20)

> **W940 — TWO SCALING AXES, AND A FRONTIER THAT CONTRADICTS THE RUNG NAMES.** On a
> 269 k-parameter MLP the 4-bit gap is **+37.88 pp (t = 12.4)** on MNIST and
> **+64.42 pp (t = 24.7)** on Fashion, 5/5 seeds — monotone along difficulty *and*
> capacity. The bigger baseline (97.26 %) now matches FINN's own fp32 MLP, and the
> 8-bit null survives both axes at 0.02–0.04 pp.
>
> Fourteen decoders priced behind an identical multiply give a monotone curve
> spanning **112×** (3.43 cells at 2 bits → 385.14 at 16), with the decoder at 2 %
> of the unit. Joined with accuracy: **at zero accuracy loss fp8 costs 138.57 cells
> and TNF8 costs 230.57 — 1.66× more, because TNF8 stores ten bits while named for
> eight.** Where TNF wins is four bits, where fp4 is not cheaper but unusable.
> Landed as **tf#641**. **Readiness 52 % → 57 %.** Theorems T784–T785; lessons
> 1415–1416; `tri frontier` added.

> **W939 — THE 4-BIT RESULT IS NOW SIGNIFICANT, AND THE ALPHABET IS THE ARGUMENT.**
> Five seeds, MNIST and Fashion-MNIST, paired: TNF4 − fp4 e2m1 = **+8.40 pp
> (t = 3.7)** and **+27.75 pp (t = 4.5)**, 5/5 seeds each, p < 0.05 — and the effect
> is **3.3× larger on the harder task**, which is how you know it is real. The
> losing formats are unstable, not merely worse (σ 13.95 pp against 0.51).
>
> And the fusion test: the decode gap **survives exactly** (TNF16 vs BNF16 is
> 8.000 cells bare and 8.000 fused), so the LUT-absorption objection does not erase
> it — **but it is 2 % of the unit**, and the consumer's own cost is set by the
> **alphabet**: 382.4 cells behind 16 input bits, 4.1 behind two. **93× on the
> consumer against 8 cells on the decoder.** Landed as **tf#640**. **Readiness
> 47 % → 52 %.** Theorems T782–T783; lessons 1413–1414; `tri last` added.

> **W938 — THE ACCURACY COORDINATE EXISTS.** `top-1`, `ImageNet`, `CIFAR`, `MNIST`
> were 0 hits in 7,858 lines. Now: MNIST 784-32-10 MLP, fp32 **93.39 %**, weights
> round-tripped through the shipped oracles with a per-tensor scale — at **16 bits
> six formats whose error spans 16× land within 0.02 pp**, at 8 bits within
> 0.19 pp, and at **4 bits TNF4 holds 93.38 % (−0.01 pp) while fp4 e2m1 and GF4
> lose 5.49**. Above four bits the format is invisible to the task. Landed as
> **tf#638**.
>
> **And a 70-point artefact of ours, caught and recorded:** the unscaled 4-bit run
> flushes 98.8 % of weights to zero, so it measures dynamic range, not the number
> system — the tell was two distinct formats agreeing to the digit. The empirical
> prior of a trained tensor is **8.1 binades**, against the **77** the regenerators
> draw from. **Readiness 41 % → 47 %.** Theorems T779–T780; lesson 1411;
> `tri cells` added.

> **W937 — THE BASELINE WAS DOWNLOADED AND THE PRIOR WAS MEASURED.** PACoGen, the
> field's reference posit hardware, cited zero times in the manuscript, is public
> Verilog: through our own rig it puts `data_extract_v1` at **92.000** cells
> against this tree's `posit16_decode` at **125.000** — **the reimplemented
> baseline is sound**, doing strictly more work for 1.36×. At operator level and
> matched 16-cell width, TNF's adder is **561.670** against `posit_add`'s
> **693.000**: **1.23×**, where the paper claims 6.1× from decoder models.
>
> And the accuracy prior: TNF16 leads under all five priors tested, but its
> advantage over posit16 is **14.63× under the paper's uniform-77-binade draw and
> 1.02× under a standard normal**. The claim that survives is **prior-invariance**
> — TNF16's error moves 1.046× where posit16's moves 14×. Landed as **tf#636**;
> optional honest-Fmax search as **tf#637**. **Readiness 36 % → 41 %.** Theorems
> T777–T778; lessons 1409–1410; `tri recall` added.

> **W936 — THE DECODE COST IS MEASURED, AND THE FREQUENCY COLUMN HAS A NAMED
> DEFECT.** yosys runs locally, so the CI queue stopped being a blocker: each
> decoder was instantiated N times in a chain and `cells(N) = fixture + cost·N`
> fitted at N = 1,2,4,8 — eighteen of nineteen fits exact with integer slopes.
> **The ternary exponent field decodes 5× cheaper than the binary one** (2.000 vs
> 10.000 cells), TNF's cost is width-independent across 16/32/64, `int8` is
> exactly free, and the spread to the tapered formats is 62–152×. Landed upstream
> with its rig as **tf#634**.
>
> Reading nextpnr-xilinx's own source then corrected one of our theorems and
> named a defect in every frequency we have: `0.1 ns` setup/hold/clock-to-Q for
> every flip-flop, one speed grade in the chipdb, `--freq` consumed by router1
> and ignored by router2, and router2 emitting placer pre-route estimates in
> post-route-looking text (**tf#635**, T776, T771 erratum). **Readiness 33 % →
> 36 %.** Theorems T774–T776; lessons 1406–1408; `tri audit` added.

> **W935 — THE AUDIT TURNED ON US, AND THE PAPER GOT A REFEREE.** A hostile
> referee pass plus two prior-art sweeps, every load-bearing claim re-verified by
> hand: the manuscript's headline ranking (10.2 %) is **below the resolution the
> manuscript itself states** (11.4 %), `placer`/`router` occur **zero** times in
> 7 858 lines against a measured 4.66× effect, the empty control in
> `tab:cleandecode` is beaten in area and speed by two of its own rows, and
> thirteen defining references are missing. Filed as **tf#631**.
> **Publication readiness: 33 %** — the mathematics is submission-ready, the
> hardware section is not; re-centring on the 6.08× decode-cost separation is one
> decision worth ~55–60 %.
>
> **And three blocker findings were ours.** `G8-VERDICT.md`'s "LNS16 does not
> reproduce" is **WITHDRAWN** (tf#632): `MATRIX.md:35` lists LNS16 at 43.11 MHz,
> 0.16 % from the published value — we read a blank cell in our own reference
> table as absence of evidence, and applied the dispersion band with a
> denominator it was never defined with. tf#625 closed as not planned. The CAD
> configuration is now recorded per row and the report refuses to rank across
> configurations (tf#630). Theorems T764–T773; lessons 1402–1405; `tri` grew five
> local wave commands (t27#2244).

> **THE LANDING (W913–W916).** The user's standing order «сам все мержи всегда»
> converted every waiting row below into agent action: tf#603, tf#612, tf#615
> (audit → main), **t27#2217 (the whole ladder + master merge, ratchet
> 221/221 CLEAN) — MERGED 12:24:15**, and t27#2221 (nine dangling .lake
> gitlinks that broke every recursive checkout since #1304) — MERGED 12:44:54.
> G8's cost sweep runs in CI now (run 32249246232: generate green, ~100 yosys
> arms green, PnR ahead). Remaining human input: the two decision words
> (forall / dialect) and the G8 verdict when CI finishes.

Nineteen autonomous waves closed two arcs. Everything below is measured,
committed, and waiting on FIVE human actions; nothing else blocks.

## Track 1 — the TNF paper (`gHashTag/trinity-fpga`): NO-GO on one gate

Work done: 20 of 59 tables under runnable regenerators; document defects found,
fixed and confirmed (PR #601 merged by owner) — **the ledger enumerates twelve:
nine numbered paper defects plus three gate corrections; earlier pages said
"16", which no enumeration in the tree supports (W935 audit)**; 8 further
findings reported for the author's judgement; toolchain properties measured (seed dispersion
1.6–41.7 %, placer/router flips fp8-vs-TNF winners). Full ledger:
`docs/reports/upstream/TNF-FINDINGS-LEDGER.md`.

| # | action | who | cost |
|---|---|---|---|
| 1 | merge PR **tf#603** (19-commit paper audit) | owner | one review |
| 2 | merge PR **tf#612** (one-file workflow registration) — **the only thing between G8 and a green release checklist** | owner | one click |
| 3 | ~~after #612: one `tnf-cost-sweep` dispatch closes G8~~ — **superseded W920/W935**: the cost sweep measures (E_t, M) ladder arms, and `tab:untraced`'s sixteen are format-comparison tracts, so that dispatch could never have closed G8. It was closed by the tract sweep instead (19/19 routed) | done | — |

Also standing: three leaked credentials (trinity#601, trios-dwagent#1,
trios-railway#124) still need human rotation.

## Track 2 — the t27 grammar ladder (`gold-ring/0001-0002-…`, t27#2217)

Fourteen shipped rungs: 67,760 → **25,670** discarded tokens (−62.1 %),
BDD-line readability 45 % → **98.5 %**, zero undisclosed regressions, every
rung probed + adversarially panelled + corpus-certified. The residue is fully
priced, and BOTH remaining decisions are REHEARSED — built, measured on the
full corpus, reverted, patch filed:

| # | decision | page | effect of «2» |
|---|---|---|---|
| 4 | **forall bodies** (74 % of residue) | `docs/reports/gold-ring/FORALL-DECISION.md` | 25,670 → 6,592 |
| 5 | **dialect bodies** | `docs/reports/gold-ring/DIALECT-DECISION.md` | with #4: → **4,711 (−93 %)** |

Answer format: two words on t27#2217 (e.g. «forall: 2, dialect: 2»). Each
lands as a one-wave rung; W910 verified the patches REPRODUCE their quoted
numbers exactly (6,592 and 4,711) and that the measurement is
mutation-sensitive (a broken capture boundary shifts both the token count and
the parse-fail diff).

## Where everything lives

- Ladder branch: `gold-ring/0001-0002-compound-assign-nested-fn` (cumulative
  patch `LADDER-COMPLETE-0001-0014.CUMULATIVE.patch`, 1,273 lines, FROZEN_HASH
  verified at HEAD)
- Theorems T650–T753 in `docs/theory/IGLA-FORMAL-RESULTS.md`; method in
  `.claude/skills/oracle-method.md` (Parts I–III)
- Session narrative: `docs/reports/SESSION-SUMMARY-W846-W890.md` (through W906)
- Decision thread: t27#2217 (ten comments carry the whole ladder history)

---

*φ² + φ⁻² = 3 | TRINITY*
