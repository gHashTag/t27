# Changelog

All notable changes to t27 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### FPGA — measured, W746-W761 (2026-08-14/15)

#### Added
- **A trained ternary network running across three XC7A200T dice** — 232 LUT
  total, zero DSP, zero SRL, **100/100 layer agreement** with the reference model
  on 100 real UNSW-NB15 rows. Weights come from the trainer, not from a seed.
- **Trainer-to-silicon export path** (`experiments/gfternary-line/gen_trained.py`).
  Before it, every silicon result in this repository ran `random.Random(seed)`
  weights and proved transport rather than computation.
- **`t27c yostat` refuses known-bad primitives** — exits 2 on `SRL16E`,
  `SRLC32E` or `DSP48E1`, naming the yosys flag to add.
- **33-bit BSCANE2 data register**, so a full 32-bit payload survives the
  Exit1-DR clock. The previous 31-bit limit silently truncated one ternary symbol
  and flipped 6 of 100 decisions.
- `docs/reports/OPENXC7-SRL16E-DEFECT.md` — reproduction, toolchain versions and
  diagnosis path for a new openXC7 defect.

#### Fixed
- **openXC7 emits a wrong bitstream for `SRL16E`** while the netlist is correct:
  0/6 rows agreed with the model, **24/24 with `synth_xilinx -nosrl`**, source
  byte-identical. Same class as the known DSP48E1 defect. Three waves of hardware
  debugging trace to this one cause.
- Every LUT figure published before W752 counted hidden layers only and **omitted
  the decision neuron** (87 LUT at fan-in 16). Corrected in `BENCHMARKS.md`.

#### Changed — claims narrowed
- **The golden alphabet is measured at +0.735 pp** (size) and +0.149 pp (shape,
  significant on 1 of 3 tasks), against inter-layer normalisation at **+29.15 pp**.
  The multiplier `phi` removes from weight application **returns in the pair
  resolve**, costing 8 DSP48E1 or ~2750 LUT. The algebra stands; the practical
  advantage does not.
- **No accuracy figure from before W749 is comparable** — the pre-activation
  scale was uncontrolled, inflating the alphabet-size effect by 39%.
- **No claim that this datapath suits any task.** The sparse penalty ranges over
  a factor of fifty across eleven measured tasks and no predictor survived a
  confirmation split.
- **No `LUT*ns` comparison** with any published system: the field's Fmax is not
  in this repository and the column stays empty rather than guessed.


### Added
- Repository best practices configuration (git hooks, CODEOWNERS, Dependabot, PR template)
- Pull request template with Issue Gate checklist
- GitHub CODEOWNERS file for reviewer routing
- Dependabot configuration for Rust and GitHub Actions dependencies

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

---

## [0.1.0] - 2026-04-07

### Added
- Initial release of t27 spec-first language
- 27 Coptic registers ternary ISA
- GoldenFloat family (GF4-GF32) with phi-structured formats
- Sacred physics constants derived from φ² + 1/φ² = 3
- Zig, C, and Verilog codegen backends
- Bootstrap compiler in Rust (`t27c`)
- `tri` CLI wrapper for common operations
- Conformance vectors under `conformance/`
- Git hooks for NOW.md date gate
- GitHub Actions CI/CD workflows
- Zenodo publication integration
- Coq formal verification support

### Spec Families
- **STRAND I** — Base: types, ops, constants (Rings 0-8)
- **STRAND II** — Numeric+VSA: GF4-GF32, TF3, phi, VSA ops (Rings 9-11)
- **STRAND III** — Compiler+FPGA: parser, MAC, ISA registers (Rings 12-14)
- **STRAND IV** — Queen+NN: Lotus orchestration, HSLM, attention (Rings 14-17)
- **STRAND V** — AR (CLARA): ternary logic, proof traces, Datalog, restraint (Rings 18-24)

---

## Version Policy

- **Major (X.0.0)**: Breaking changes to language syntax, semantics, or backward-incompatible spec format
- **Minor (0.X.0)**: New features, new spec families, new backends, backward-compatible additions
- **Patch (0.0.X)**: Bug fixes, performance improvements, documentation updates, conformance vector additions

---

**φ² + 1/φ² = 3 | TRINITY**
