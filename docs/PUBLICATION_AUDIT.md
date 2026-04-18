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

**Legend — Ready?:** Yes / Partial / No (subjective until gates pass).

---

## How to update

1. Add a row for each new candidate artifact.  
2. When **Ready?** becomes **Yes**, set **Next action** to “Tag release → Zenodo”.  
3. After deposit, set **DOI exists?** to the version DOI and link from [`publications/README.md`](../publications/README.md).

---

*If it is not in the audit table, it is not on the publishing conveyor.*
