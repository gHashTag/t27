# Publication audit — readiness for Zenodo / Trinity Publications

**Purpose:** Track **what can be deposited next** and **what is missing**. Update this file when an artifact moves toward a tagged release.

**Audit categories (gate):**

| Category | Ready for Zenodo when |
|----------|------------------------|
| Software release | Code, license, README, install/run, **Git tag**, `CITATION.cff` aligned |
| Research note | PDF or Markdown, methods, **limitations**, claim pointer (`RESEARCH_CLAIMS`) |
| Repro bundle | Pinned inputs, exact commands, output tables or hashes |
| Benchmark pack | CSV, methodology, hardware/software environment |
| Dataset / corpus | Vectors + schema + **version** + provenance |

---

## Audit register (t27-focused)

| Artifact | Repo | Series | Ready? | Missing | DOI exists? | Next action |
|----------|------|--------|--------|---------|-------------|-------------|
| t27 bootstrap + specs (language kernel) | t27 | Core language | Partial | Zenodo toggle for **t27**; first GitHub Release with notes | No (repo-level) | Enable Zenodo on `gHashTag/t27`; tag `v0.1.0` when ready |
| Conformance JSON corpus (`conformance/*.json`) | t27 | Core / dataset | Partial | Schema doc, checksum manifest for Zenodo | No | Add release manifest script; optional `version` field in JSON |
| `docs/nona-02-organism/LANGUAGE_SPEC.md` snapshot *(path corrected W845; `docs/LANGUAGE_SPEC.md` does not exist)* | t27 | Core language | No | Complete skeleton → stable v1 text | No | Finish §§ lexical–backend; export PDF/MD for Zenodo |
| GoldenFloat validation report | t27 | Numerics | No | Fill `NUMERICS_VALIDATION.md` tables + CSV | No | Run L4 differential oracle; attach CSV |
| Sacred formula + claim-status report | t27 | Physics / research | Partial | One-click export from `RESEARCH_CLAIMS` + spec excerpts | No | Generate static report on release |
| Repro smoke bundle | t27 | Audit / repro | Partial | `repro/Makefile` exists; pin Rust in doc | No | Add `rust-toolchain.toml` + Docker optional |
| Vasilev & Pellis phi-structures paper | Zenodo | Physics | Yes | — | Yes ([10.5281/zenodo.18950696](https://doi.org/10.5281/zenodo.18950696)) | Link in `publications/README.md` (done) |
| FPGA Autoregressive Ternary LLM | trinity | Hardware / AI | Yes | — | Yes | Listed in catalog |
| Self-Evolving Ouroboros | trinity | AI / agents | Partial | Formal criteria + logs for “self-evolving” | Yes | See `RESEARCH_CLAIMS` C-ternary-002 |
| VSA + SIMD / phi-RoPE / Sparse MatMul / VSA ops | trinity | Mixed | Yes | Independent replication where claimed | Yes | Listed in catalog |
| TRI CLI reference | trinity | AI / software | Partial | Versioned release + Zenodo for **trinities** | Partial | Align with trinity release train |
| Quarterly research audit | programme | Audit | No | Template + first issue | No | Create `docs/templates/audit-quarterly.md` (optional) |
| **TNF article** (`docs/theory/TNF_ARTICLE_RU.md`) | t27 | Numerics / hardware | **No** | Retractions section; seed-swept silicon table; toolchain re-declaration | No | See the three blockers below |
| TNF abstract (`docs/reports/TNF9-ABSTRACT-PROPOSAL.tex`) | t27 | Numerics | Partial | Must follow the article's re-scoping (T479b) | No | Rewrite abstract after the article settles |
| TNF post-route sweep (`docs/reports/tnf-sweep/`) | t27 | Benchmark pack | **No** | Placer seed not recorded; single placement per row | No | Re-run under `t27c verdict --seeds` (T619a) |

**Legend — Ready?:** Yes / Partial / No (subjective until gates pass).

---

## How to update

1. Add a row for each new candidate artifact.  
2. When **Ready?** becomes **Yes**, set **Next action** to “Tag release → Zenodo”.  
3. After deposit, set **DOI exists?** to the version DOI and link from [`publications/README.md`](../publications/README.md).

---

*If it is not in the audit table, it is not on the publishing conveyor.*

---

## W845 — the TNF article, and why it was not in this table until now

**It is the largest scientific artefact in the project and it was absent from the
register**, from `PUBLICATION_QUEUE.md`, and from `PUBLICATION_MAP.md`. By this
file's own closing rule that means it was never on the conveyor. Measured
2026-08-18: **2,890 lines, 95 sections, 70 tables, 101 status tags, 23 distinct
theorem ids** (T482 highest; the results file runs to T628a).

> **Two numbers in the first version of this banner were mine and wrong**, caught
> hours later by the full audit. I wrote "93 status tags" (a regex missing
> `[источник в тексте]`; there are 101) and "32 distinct theorem citations" (a raw
> `T<number>` match that swept up `GF-T8`, `GF-T16` and the pin names `T14`/`T15`;
> there are **23**). Counting carefully is not the same as counting the right
> thing.

### Blocker 1 — the silicon table is a single-placement measurement [high]

`TNF_ARTICLE_RU.md:354`  *(cited as :309; the file has moved -- the post-route rows are at :354 today)* publishes six post-route rows with $F_{\max}$ to two
decimals (161.11 / 153.23 / 131.73 / 83.27 / 53.26 / 33.23 MHz), plus 81.35 vs
147.32 MHz for the pipeline cut and 136.44 vs 131.73 for the $M{=}11$ choice.
The section says *"один стенд на всю линейку, чтобы строки были сопоставимы"* and
names no placer seed.

**W842 (T616) built one unchanged netlist at five seeds and measured $F_{\max}$
from 15.83 to 18.29 MHz — a 15% spread with the netlist held constant.** T619a
therefore requires agreement across at least three placements before a silicon
number is a result. The second decimal place cannot survive that, and neither can
the 136.44-vs-131.73 comparison, which is a 3.6% difference inside a 15% spread.

*Clears when:* the ladder is re-run under `t27c verdict --seeds 1,7,42` and the
table reports a range or a median with the seed set named.

### Blocker 2 — the declared toolchain is not the bench's toolchain [high]

`TNF_ARTICLE_RU.md:340`  *(cited as :305; the file has moved -- the Yosys 0.65 / nextpnr line is at :340 today)* declares **Yosys 0.65** and **nextpnr-xilinx 1743d0f**.
Measured on this bench 2026-08-18: **Yosys 0.63** (`70a11c6b`) and nextpnr
**c32135b0**. A reader following the article's own method on this repository gets
neither version.

*Clears when:* either the numbers are re-taken on the declared versions, or the
declaration is corrected to what produced them and the discrepancy is explained.

### Blocker 3 — six retracted-in-effect citations are still in the text [high]

T430, T431, T432, T439, T443 and T446 are each cited once. **T479a** found that
the two interventions this thread rests on (balanced coverage, depth `L=5`) are
significantly positive on Fashion and negative on UNSW — *"neither may be stated
as a property of the architecture."* **T479b** names the three claims that do
survive three tasks: normalisation (+17.85/+6.36/+20.48), fan-in 6
(+1.73/+0.91/+4.51), ternary activations as a cost (−2.37/−0.35/−1.83).

~~The article also carries no retractions section.~~ **Withdrawn, also mine.** The
article carries `# ЭРРАТУМ 2026-08-13` at `:2385` and a retraction-methodology
section at `:2023`. The real defect is worse and is B1 below: **four appended
errata retract claims the body still asserts verbatim**, and the abstract at `:65`
still prints three numbers its own erratum at `:2385` withdrew.

*Clears when:* the six citations are re-scoped to task observations and a
retractions section is added.

### Not a blocker, and worth saying

The 2026-08-17 commit `01ac4e4e0` (W804) already placed the three-task survivors
ahead of the alphabet sections, and the article now carries 93 status tags where
`TNF-ARTICLE-RECONCILIATION.md` reported zero. **That reconciliation document is
itself stale on that point** and should be read against W801-W804, not as current.

### The full register — 24 findings, each adversarially verified

`docs/reports/TNF-ARTICLE-AUDIT-W845.md`. Five readers returned 318 raw findings;
each load-bearing one was handed to an agent told to REFUSE it, defaulting to
refuted when uncertain. **24 of 24 survived.** Ranked, in that file:

| # | Blocker | Sharpest fact |
|---|---|---|
| B1 | The abstract asserts three retracted numbers | `:65` prints +10.2% / 6.1× / 8-of-20; its own erratum recomputes +10.85% / 5.245× / **12 of 24**. Two further sites (`:1270`, `:1779`) were never touched |
| B2 | The retraction count is stated four irreconcilable ways | five (`:69`), ten (`:89`), eleven (`:1552`), sixteen (`:2026`) |
| B3 | The Железо table contradicts §Ограничения | table prints a **TNF128** row; `:2274` says TNF128 did not route and is not claimed. TNF64 is 6,140 LUT in one and **7,479 / 48.20 MHz** in the other |
| B4 | Three LUT counts for one TNF16 multiplier | 212 (`:321`), 219 (`:367`), **372** (`:447`). The seed caveat I added today forecloses "different builds", and `:341` forecloses synth-vs-route |
| B5 | The zero-DSP stack table has no provenance | 78 МГц / 33 GOPS / 97% / XOR 4/4 / 24-25 эпох, untagged; **2 of 5 untraceable repo-wide** |
| B6 | Zero-DSP novelty is unavailable after 2026-07 | ELiTeFormer (arXiv:2607.03652); T490 already ruled the citation mandatory |
| B7 | Five silicon verdicts are single-placement claims | only `ternary_node` and `e8m0` re-audited under T619a; tnf17 / phi_weights / ternary_link had only timing checked |
| B8 | **Five named verification gates do not exist** | `check_paper_numbers.py`, `check_literal_widths.py`, `check_variant_declared.py`, `check_exponent_window.py`, `variant_map.json`, `gen_figures.py` — absent from the worktree AND from git history, each presented as a live red-teamed gate |
| B9 | Two mandatory publication gates point at dead paths | `RESEARCH_CLAIMS.md` and `LANGUAGE_SPEC.md` are both under `docs/nona-*/`; **no artefact can satisfy either as written** |

**The one that would have been hardest to find alone:** `cell_census` reported
**exactly 2× every cell count for 264 commits** (T500/T504/T505, `addf9a3df` ->
`2e2bea00f`). The article's five-row silicon table falls inside that window and
**all five LUT and CARRY4 values are even**, failing T505's parity test. Related:
the "66 LUT на MAC" figure the article uses at `:2627` to derive 2,039 parallel
MACs and 204 GMAC/s is withdrawn by T501 — **it is 33**.

### Needs the user, not a wave

- A venue decision. `PUBLICATION_MAP.md` routes *"FPGA / MAC / Verilog"* to FPL /
  DATE and *"GoldenFloat + validation"* to numerics; the article is a methods
  paper by its own reconciliation (§6) and fits neither cleanly.
- Zenodo/DOI and any GitHub Release remain human actions.
- `tnf-publication-readiness` (264 files, reported local-only) **is not present in
  this worktree** and cannot be inspected from here.
