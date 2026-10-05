# SOTA Research Review — t27/wave-audit

> State-of-the-art review for the areas relevant to the audit's findings.
> Tags: audit, research, plan

## Measurement context

- Command: `python3` urllib fetch of `http://export.arxiv.org/api/query?search_query=<query>&max_results=3&sortBy=submittedDate&sortOrder=descending` (arXiv API), three query rounds
- SHA at measurement: `53dcd80f4c53b6a5c82d804a41ac7d2787f99402` (`git rev-parse HEAD` in worktree `queen-5985`)
- Date: 2026-10-04

## Method

No browser is attached to this run and `curl` is not installed; live research was done through python3 `urllib` against the arXiv API (metadata only: titles, dates). The review therefore combines (a) fresh arXiv metadata per area, and (b) the repo's own positioning documents — `COMPETITORS.md` and `BENCHMARKS.md` — read at the SHA above. In keeping with `BENCHMARKS.md`'s restrained posture, no throughput parity is claimed against any external system.

## Area 1 — Ternary numerics and weight quantization (relevant to findings #5974, #5973)

Fresh arXiv hits (date, title):

- 2026-09-28 — *Fiona: Accelerating FHE Inference with Packing-Aware Ternary Weights*
- 2026-06-24 — *BitNet Text Embeddings*
- 2026-05-23 — *On the Stability and Realizability of Recurrent Polynomial Surrogate Ternary Logic Gate Networks*
- 2026-04-28 — *Hardware Generation and Exploration of Lookup Table-Based Accelerators for 1.58-bit LLM Inference*

Reading: ternary weights have moved from research curiosity toward deployment paths (FHE inference, embeddings, LUT-based silicon). The t27 repo's differentiator stays what `COMPETITORS.md` states: inspectable open silicon and a formal/assurance workflow, not throughput parity. The audit's finding that `gen-c`/`gen-zig` emit undefined-behaviour arithmetic (`%`, oversized shift) is directly material here: any ternary-quantization comparison built on generated code would inherit that UB.

## Area 2 — Scaled low-bit formats (MX / block floating point vs the GF16 family)

Fresh arXiv hits:

- 2026-09-23 — *MicroQonv: Reshaping Convolution Tensors for Efficient Microscaling in Training and Inference*
- 2026-09-09 — *MiX: Micro-Inverted-Scaling for End-to-End Low-Bit Vision-Language Model Acceleration*
- 2026-09-09 — *EFQ-Softmax: Exp-Free Quantization for Softmax*

Reading: the microscaling (MX) family is the active SOTA for scaled low-bit formats. The repo's GF16 family is registered in `conformance/FORMAT-SPEC-001.json` as the SSOT for the line; per `BENCHMARKS.md`, most rows of `gf_competitive_bench.json` are placeholders and stay omitted rather than projected. The GF16 line should keep defining itself by conformance vectors, not by chasing MX rows.

## Area 3 — LLM agent sandboxing and tool use (relevant to the repo's scheduler/worker design)

Fresh arXiv hits:

- 2026-10-01 — *KaliBench: A Fine-Grained Benchmark for Cybersecurity Tool Use on Kali Linux with Runtime-Free Verification*
- (no further on-point hits in the query window)

Reading: the trend is toward fine-grained, runtime-free verification of agent actions. The repo's own design — worker bees with explicit file-path boundaries, boundary checks before external actions, and review of anything written outside the declared boundary — matches this trend's shape. `docs/wave_ecosystem_2026-07-08/FINAL_REPORT.md` already documents that `.tri` compliance is by visual audit against the `specs/01-tri-lang-core.tri` template, with `t27c parse` verification pending on toolchain availability — a runtime-free verification posture for the spec corpus.

## Area 4 — Compiler self-hosting (relevant to epic #5980 / #5981)

Fresh arXiv hits: none on-point in the query window; the grounding is repo-internal.

Reading: the repo's self-host epic (#5980, #5981) targets an MVP compiler core in t27, byte-identical to `gen-c` output, with the corpus as its measured backlog. This is consistent with the broader inspectable-toolchain posture: self-hosting a sealed spec (`t27c`) with the corpus metric as acceptance. The measured blockers for that epic are exactly the t27b coverage gap (#5977: 1127 of 1174 specs rejected; `StructDecl` 393, string literal 288 as first-rejection classes) and the parser's silent misreads (#5968, #5978, #5949).

## Limitations

- arXiv API metadata only — no full-text reading, no non-arXiv sources (vendor pages, GitHub research issues) were fetched in this run.
- `COMPETITORS.md`/`BENCHMARKS.md` keep external links as primary sources; those pages were not re-verified in this run.
- The snapshot is dated 2026-10-04.
