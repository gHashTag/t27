# Wave Loop 111 Plan — Hierarchical Design + FPGA Validation + Industrial Benchmarks

**Date:** 2026-06-16
**Branch:** trinity-rust-rings
**Focus:** IGLA CODER / IGLA RACE only

---

## Situational Assessment

### New Competitors Discovered (June 2026 arXiv sweep)

| Competitor | arXiv | Pass@1 | Key Innovation | Threat |
|-----------|-------|--------|--------------|--------|
| **CHIPCRAFTBRAIN** | 2604.19856 | **98.7%** VE-Human | 6-agent RL, FPGA hardware validation, K-map solver | **EXTREME** |
| VeriGraphi | 2604.14550v2 | N/A | Spec-anchored Knowledge Graph, RISC-V 32I | HIGH |
| HDLFORGE | 2603.04646 | 91.2% VE-Human | Two-stage adaptive escalation | EXTREME (known) |
| StepPRM-RTL | 2606.04246 | 85.7% VE | Step-level PRM + MCTS | HIGH (known) |
| ACE-RTL | 2602.10218 | N/A | NVIDIA RTL-specialized + Claude 4 Sonnet | HIGH (known) |

**Critical gap:** CHIPCRAFTBRAIN achieves **98.7% Pass@1** — a **43.7 pp gap** to Trinity's estimated 0.55. Its FPGA hardware validation and symbolic K-map solver are capabilities Trinity lacks entirely.

### Weaknesses Identified

1. **No FPGA hardware validation** — CHIPCRAFTBRAIN validates on Intel Agilex 5 FPGA. Trinity has only conceptual stubs.
2. **No hierarchical decomposition** — VeriGraphi/CHIPCRAFTBRAIN generate RISC-V SoC (8/8 modules). Trinity only handles flat modules.
3. **No Knowledge Graph** — VeriGraphi uses spec-anchored KG for hierarchy, ports, wiring. Trinity has no structured IR.
4. **No industrial benchmark coverage** — CVDP, ChipBench not tracked. Only VerilogEval and RTLLM.
5. **No symbolic solver integration** — CHIPCRAFTBRAIN uses zero-cost K-map/truth-table solver. Trinity is neural-only.

---

## Decomposed Tracks

### Track A: FPGA Hardware Validation Layer (eval.t27)
Add conceptual primitives for real FPGA validation:
- `has_fpga_board(board_name) -> bool`
- `synthesize_to_bitstream(verilog_file, board_name) -> bool`
- `upload_to_fpga(bitstream_file, board_name) -> bool`
- `run_fpga_testbench(bitstream_file, vectors) -> bool`
- `fpga_validation_report(bitstream_file) -> YosysReport`
- **+5 tests**

### Track B: Hierarchical Design Knowledge Graph (pipeline.t27)
Add spec-anchored hierarchical decomposition primitives:
- `decompose_spec_to_modules(spec_text) -> []ModuleSpec`
- `build_knowledge_graph(modules) -> KnowledgeGraph`
- `wire_modules(kg, topology) -> string`
- `generate_hierarchical_verilog(kg) -> string`
- `validate_kg_connectivity(kg) -> bool`
- **+5 tests**

### Track C: Industrial Benchmark Support (benchmark.t27)
Add CVDP and ChipBench competitors + tracking:
- `chipcraftbrain_competitor() -> CompetitorScore` — NEW EXTREME
- `cvdp_benchmark_competitor() -> CompetitorScore`
- `chipbench_benchmark_competitor() -> CompetitorScore`
- `industrial_benchmark_supported() -> []string`
- `trinity_cvdp_estimate() -> f32`
- **+6 tests**

### Track D: Symbolic Solver Integration (eval.t27 or new file)
Add zero-cost symbolic solver primitives:
- `kmap_simplify(expression) -> string`
- `truth_table_generate(inputs, outputs) -> string`
- `boolean_minimize(expr) -> string`
- **+3 tests**

---

## Success Criteria
- 564/564 PASS maintained
- 0 clippy warnings
- 0 seal cascade mismatches
- CHIPCRAFTBRAIN documented as competitor #61 (EXTREME)
- Hierarchical design primitives in pipeline.t27
- FPGA validation primitives in eval.t27

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
