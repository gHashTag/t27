# BENCHMARKS.md -- Restrained Benchmark Posture

> **Policy:** publish only numbers we can reproduce from this repo,
> from a sealed spec or generated file. No "expected" or "projected"
> figures appear here. **When in doubt, omit the row.**

This document is a register of what is benchmarked (and what is **not**) in
this repository, kept conservative on purpose. It complements
[`COMPETITORS.md`](COMPETITORS.md), which states what we do not claim.

---

## 1. What exists in-repo today

### 1.1 Conformance vectors (correctness, not throughput)

| Vector file (under `conformance/`)        | Purpose                                          |
|--------------------------------------------|--------------------------------------------------|
| `FORMAT-SPEC-001.json`                     | GoldenFloat family registry (SSOT for the line). |
| `gf*_vectors.json`                         | Arithmetic conformance vectors for GF widths.    |
| `ar_*.json`                                | CLARA-style assurance reasoning vectors.         |
| `nn_*.json`                                | Neural architecture conformance vectors.         |
| `sacred_physics*.json`                     | phi / Trinity identity conformance.              |
| `gf_competitive_bench.json`                | Skeleton benchmark file. **Most rows are placeholders.** |

Validation entry point: `./scripts/tri validate-conformance`. These vectors
test **correctness against the spec**, not silicon throughput.

### 1.2 Benchmark specs (under `specs/benchmarks/`)

- `bench_main.t27`
- `bench_nn.t27`
- `ternary_vs_binary.t27`

These specs define **measurement procedures**. The numbers they would
produce belong with the chip repos or with a future
`bench/results_*.json` set. **No silicon-level numbers from these specs
appear in this document.**

### 1.3 FPGA / Vivado scripts

- `fpga/vivado/build.tcl`, `fpga/vivado/build_gf16.tcl`
- testbenches: `gf16_add_tb.v`, `gf16_mul_tb.v`, `gf16_dot4_tb.v`,
  `gf16_matmul4x4_tb.v`

These produce simulation and synthesis-level evidence (latch-free,
timing-reported), but **not** end-to-end accelerator throughput numbers.

### 1.4 Misc

- `bench/results_v02_real.json` -- legacy results file, not maintained as a
  benchmark target for this line. Treat as historical.
- `benchmarks/phi_attractor_convergence.py` -- a research convergence
  script, not a product benchmark.

---

## 2. What we deliberately do not publish here

Items in this list are **not** to be quoted from this document:

1. TOPS or TOPS/W for any TRI-NET chip until that chip reaches **SILICON**
   per [`STATUS.md`](STATUS.md).
2. Latency / throughput vs. Hailo-8, Coral Edge TPU, Axelera Metis,
   Qualcomm Cloud AI 100 Ultra, MediaTek Dimensity 9400+ -- see
   [`COMPETITORS.md`](COMPETITORS.md) for why.
3. Accuracy parity with FP16 / BF16 on ImageNet, LLM perplexity, or any
   other model-level benchmark, until a reproducible vector lands under
   `conformance/`.
4. Any "expected" / "projected" / "target" figure. If a number is not
   measured, it is not in this document.

---

## 3. How to add a benchmark (the only allowed way)

1. **Land the spec.** Add a `.t27` under `specs/benchmarks/` describing
   the measurement.
2. **Land conformance vectors.** Add a `*_vectors.json` under
   `conformance/`.
3. **Land a results file.** Add `bench/<name>_results.json` produced by
   `./scripts/tri test` or an equivalent reproducible run, including the
   commit hash of the run.
4. **Add a row to this document** pointing to the spec + vectors + results.

Rows that point at "expected" results MUST NOT be added.

---

## 4. External references (for context only)

For research context cited elsewhere in this docs package:

- BitNet b1.58, 1.58-bit ternary LLM weights:
  https://arxiv.org/abs/2402.17764
- Tiny Tapeout chip catalogue:
  https://tinytapeout.com/chips/

These links are **not** sources of benchmark numbers for TRI-NET; they are
sources of **direction** for the research line.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
