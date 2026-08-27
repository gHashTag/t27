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

## [0.2.0] - 2026-08-27

t27c only. Seven defects, five of them a green exit that was not a result.

### Fixed

- **`StmtForRange` was unreachable, so two backends could not lower `for`.**
  `parse_for_range` parses its start bound with the full expression grammar,
  and that grammar carries `..` as a binary operator (for slices), so
  `for i in 0..8` came back as one `ExprBinary` and every range loop was built
  as the collection form. Measured on the previous release: **0** `StmtForRange`
  nodes across 746 tracked specs against 383 `StmtFor`, which made
  `gen_c_for_range_stmt` and `gen_verilog_for_range_stmt` dead code.
  - `gen-c` emitted no loop header at all — a comment and a bare block, so the
    body lowered exactly once. 391 sites in 48 specs.
  - `gen-verilog` emitted the range as the bound: `for (i = 0; i < (0 .. 8); …)`,
    which iverilog rejects. 32 sites in 14 specs.
  - Fixed with a `no_range` suppression beside the existing `no_struct_literal`.
    Verified by running: C now prints 8 for a loop over `0..8`; iverilog accepts
    the Verilog; the seven `for_range` unit tests that already encoded this
    lowering and were red now pass.
- **`gen-rust` emitted an empty `match` for every `switch`.** The arm loop
  tested for a node kind the parser never builds, so `match a { }` shipped with
  exit code 0 while `gen-c` and `gen-verilog` lowered the same construct.
  Patterns are qualified from the scrutinee's declared type.
- **`gen-rust` dropped the body of every `for` loop** and renamed the induction
  variable to the literal string `body`, reading the capture and the body from
  the wrong children.
- **`health` was red, and the broken thing was the compiler's own embedded
  self-check spec** — its invariant used a `forall` form the parser rejects, so
  `t27c health` failed at parse and never reached typecheck or any backend.
  Rewritten to a form that parses *and lowers*; all six stages now report.
- **`ci` printed `CI: PASSED` and exited 0 over a repository root that did not
  exist.** Now refuses a root holding neither `specs/` nor `compiler/`, and
  reports `CI: NO INPUT` with exit 2 when the tree exists but holds no `.t27`.
- **`battery --dir` ignored its argument.** `repo_root.join(dir)` replaces the
  base when `dir` is absolute and the directory read swallowed the failure, so
  the fallback ran this repository's own gates and reported on a tree the caller
  had not named. Now refuses a non-directory, prints the oracle and gate counts
  separately, and refuses when the oracle count is zero.

### Added

- **The inclusive range `a..=b`.** The lexer emits `..` and a separate `=`;
  lowered to the exclusive range over `b + 1`, so no backend needs a new
  operator. This unblocked `specs/math/constants.t27`, which 259 of 746 specs
  import.
- **Paren-less `if` in expression position.** The statement parser had accepted
  it since W578; the expression parser still required parentheses and failed
  with the identical diagnostic that parser's own comment says was fixed.
- **The parenthesised range for, `for (i in a..b)`.** `if (…)` and `while (…)`
  already accepted the form. Checkpointed so Zig's `for (xs) |x|` is unaffected.
- **`--version` / `-V`.** The `version` subcommand existed; the flag did not.

### Measured

| | 0.1.0 | 0.2.0 |
|---|---|---|
| specs that parse (746 tracked) | 603 | **620** |
| suite `parse` failures | 110 | **92** |
| t27c unit tests | 1622 passed / 13 failed | **1629 passed / 6 failed** |
| `gen-c` specs with a dropped loop header | 48 | **37** |
| `StmtForRange` nodes in the corpus | 0 | reachable |

Zero parse regressions at every step. The 37 remaining dropped loops are the
collection form `for x in xs`, which `gen-c` does not lower either — a separate
defect. `parse-no-discard` and `seal-verify` both rose, mechanically: a spec
that could not parse at all now parses and reveals that it discards tokens, and
changing an emitter's output makes stored seals stop matching.

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
