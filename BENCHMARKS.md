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

---

## 4. FPGA measurements, W746-W760 (2026-08-14/15)

Every row below was produced in this repository on **QMTech XC7A200T-FGG676**
hardware or by `yosys 0.63 -> nextpnr-xilinx (openXC7) -> prjxray`. Synthesis is
**`synth_xilinx -nodsp -nosrl`** — see §4.4 for why that is mandatory.

### 4.1 The single-die artefact

| system | LUT | accuracy | Fmax | latency | notes |
|---|---:|---:|---:|---:|---|
| ours, H16 L2, UNSW-NB15 | **123** | **81.37%** | — | 1 cycle | whole net, output stage included |
| ours, H16 L2, Fashion-bin | **123** | **86.91%** | — | 1 cycle | identical netlist, different task |
| ours, 16-16-1, UNSW | 126 | 78.45% | **99.46 MHz** | 1 cycle | timed variant |
| TreeLUT (II), UNSW-NB15 | 89 | 92.0% | *not in our record* | *not in our record* | published |

**Not claimed:** `LUT·ns` against the field. TreeLUT's Fmax and latency are not
in this repository and attempts to fetch them failed; the column stays empty
rather than guessed.

**Correction on record:** every LUT figure published before W752 counted hidden
layers only and omitted the decision neuron (87 LUT at fan-in 16). The rows above
include it.

### 4.2 The three-die ternary network

| die | layer | LUT | DSP | acceptance | agreement, 100 real rows |
|---|---|---:|---:|---|---|
| A | 593 features -> 16 ternary symbols | 78 | 0 | 0->1 | **100 / 100** |
| R | 16 -> 16 symbols | 67 | 0 | 0->1 | **100 / 100** |
| B | 16 symbols -> decision | 87 | 0 | 0->1 | **100 / 100** |

Weights come from the trainer, not a seed. The host shifts bits and performs no
arithmetic on any payload. Verified against a reference model generated from the
same seeds as the Verilog.

### 4.3 Cost rules, measured

| rule | measurement |
|---|---|
| **six bits per neuron** | <=6 input bits: **2.00 LUT/neuron**. 12 bits: **39-54 LUT/neuron**. |
| binary vs ternary inputs | a ternary symbol is two bits, so hidden layers take fan-in **3**, not 6 |
| area is not a lever | 6.5x the area buys **1.72 pp** (UNSW) / **1.38 pp** (Fashion) |
| the golden alphabet's resolve | `a + b*phi` against a threshold costs **8 DSP48E1**, or **~2750 LUT** without them |

### 4.4 Toolchain defects found here

| primitive | symptom | mitigation |
|---|---|---|
| **DSP48E1** (live operand) | netlist correct, bitstream wrong | `synth_xilinx -nodsp` |
| **SRL16E / SRLC32E** | netlist correct, bitstream wrong; 0/6 rows vs 24/24 with the flag | `synth_xilinx -nosrl` |

Both pass the wrong-part -> ours `0->1` acceptance criterion while computing the
wrong answer. `t27c yostat` now **exits 2** when either appears in a synthesis
log. Full reproduction: [`docs/reports/OPENXC7-SRL16E-DEFECT.md`](docs/reports/OPENXC7-SRL16E-DEFECT.md).

### 4.5 What is explicitly NOT claimed

- No `LUT·ns` comparison with any published system (§4.1).
- No claim that this datapath "suits" any task: the sparse penalty ranges over a
  factor of fifty across 11+ tasks and **no predictor for it survived
  confirmation**. Suitability must be measured per task, not argued.
- No accuracy figure from before W749 is comparable: the pre-activation scale was
  uncontrolled, which inflated the alphabet-size effect by 39%.
