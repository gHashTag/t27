# Full status, both tracks — one page for whoever wakes up (W936, 2026-08-20)

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
