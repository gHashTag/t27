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
| `docs/LANGUAGE_SPEC.md` snapshot | t27 | Core language | No | Complete skeleton → stable v1 text | No | Finish §§ lexical–backend; export PDF/MD for Zenodo |
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
2026-08-18: 2,863 lines, 95 sections, 70 tables, 93 status tags, 32 distinct
theorem citations.

### Blocker 1 — the silicon table is a single-placement measurement [high]

`TNF_ARTICLE_RU.md:309` publishes six post-route rows with $F_{\max}$ to two
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

`TNF_ARTICLE_RU.md:305` declares **Yosys 0.65** and **nextpnr-xilinx 1743d0f**.
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

The article also carries **no retractions section** while the programme has
withdrawn results repeatedly (T458 alone withdrew nine of eleven UNSW
measurements).

*Clears when:* the six citations are re-scoped to task observations and a
retractions section is added.

### Not a blocker, and worth saying

The 2026-08-17 commit `01ac4e4e0` (W804) already placed the three-task survivors
ahead of the alphabet sections, and the article now carries 93 status tags where
`TNF-ARTICLE-RECONCILIATION.md` reported zero. **That reconciliation document is
itself stale on that point** and should be read against W801-W804, not as current.

### Needs the user, not a wave

- A venue decision. `PUBLICATION_MAP.md` routes *"FPGA / MAC / Verilog"* to FPL /
  DATE and *"GoldenFloat + validation"* to numerics; the article is a methods
  paper by its own reconciliation (§6) and fits neither cleanly.
- Zenodo/DOI and any GitHub Release remain human actions.
- `tnf-publication-readiness` (264 files, reported local-only) **is not present in
  this worktree** and cannot be inspected from here.
