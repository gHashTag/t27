<!-- Licensed under Apache License 2.0 — http://www.apache.org/licenses/LICENSE-2.0 -->

# DARPA CLARA Submission Checklist

**Solicitation:** PA-25-07-02
**Deadline:** April 17, 2026
**Submission:** via DARPA BAA Portal

## Volume 1: Technical & Management (≤10 pages)

- [x] docs/CLARA-PROPOSAL-TECHNICAL.md — primary document
- [x] Page count ≤ 10 (estimated ~8.6 pages at 250 words/page; 2,155 words, 337 lines)
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

- [ ] Detailed budget (see Section 7 of Technical)
- [ ] Personnel rates
- [ ] F&A rate documentation

## Supporting Evidence

- [x] docs/CLARA-EVIDENCE-PACKAGE.md — consolidated evidence
- [x] docs/CLARA-COMPOSITION-PATTERNS.md — 4 composition patterns
- [x] docs/CLARA-DEMO-PIPELINE.md — demo pipeline spec
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
- [ ] Git clean: no .bak files, no phantom changes
