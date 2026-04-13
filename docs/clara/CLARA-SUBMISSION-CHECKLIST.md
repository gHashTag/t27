<!-- Licensed under Apache License 2.0 — http://www.apache.org/licenses/LICENSE-2.0 -->

# DARPA CLARA Submission Checklist

**Solicitation:** PA-25-07-02
**Deadline:** April 17, 2026
**Submission:** via DARPA BAA Portal

## Volume 1: Technical & Management (≤10 pages)

- [x] docs/clara/CLARA-PROPOSAL-TECHNICAL.md — primary document
- [x] Page count ≤ 10 (1,702 words ≈ 6.8 pages at 250 words/page)
- [x] Abstract present
- [x] AR-based ML approach described (Section 1)
- [x] Application Task Domain + SOA (Section 2)
- [x] Polynomial-time tractability proofs (Section 3)
- [x] Basis for confidence (Section 4)
- [x] Metrics coverage (Section 5)
- [x] Schedule + milestones with deliverables (Section 6)
- [x] Budget summary (Section 7)
- [x] Bibliography

## Volume 2: Cost Proposal

- [x] docs/clara/CLARA-COST-PROPOSAL.md — detailed budget
- [x] Personnel rates (PI $180K, AR $150K, ML $150K, SE $120K)
- [x] F&A rate documentation (33% of direct costs)

## Supporting Evidence

- [x] docs/clara/CLARA-EVIDENCE-PACKAGE.md — consolidated evidence
- [x] docs/clara/CLARA-COMPOSITION-PATTERNS.md — 4 composition patterns
- [x] docs/clara/CLARA-DEMO-PIPELINE.md — demo pipeline spec
- [x] conformance/gf16_bench_results.json — GF16 benchmarks
- [x] conformance/clara_spec_coverage.json — spec parse coverage
- [x] scripts/clara/demo.sh — runnable demo (20/20 PASS)

## Compliance

- [x] Apache 2.0 headers on ALL documents
- [x] FAQ 38: Software focus (not hardware primary)
- [x] FAQ 21: AR in guts of ML (not just wrapper)
- [x] FAQ 53: Non-US entity eligible
- [x] ≤10 step proof traces (explainability)
- [x] Polynomial-time guarantees (tractability)

## Repository

- [x] All CLARA-relevant specs parse: t27c parse *.t27 (35/36 pass; 1 non-CLARA FPGA testbench)
- [x] Full test suite: `t27c suite` — 0 failures
- [x] Demo pipeline: scripts/clara/demo.sh — 20/20 PASS
- [x] Git clean: .bak files removed, Coq artifacts cleaned
