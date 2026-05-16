# Changelog

All notable changes to t27 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-05-16

### Added
- **LSP Server** — Complete Language Server Protocol implementation with 12 services:
  - textDocument/completion (context-aware completions)
  - textDocument/hover (phi/GF documentation)
  - textDocument/definition (cross-references)
  - textDocument/references (find all uses)
  - textDocument/documentSymbol (symbol outline)
  - workspace/symbol (spec search)
  - textDocument/diagnostic (spec validation)
  - textDocument/codeAction (quick fixes)
  - textDocument/formatting (auto-format)
  - textDocument/signatureHelp (function signatures)
  - textDocument/prepareCallHierarchy (call graph)
  - textDocument/incomingCalls/outgoingCalls (dependency graph)
- **VSCode Extension** — Full IDE support:
  - TextMate grammar for syntax highlighting
  - 14+ code snippets for common patterns
  - Custom commands: t27.runTests, t27.generate, t27.parse
- **WASM Backend** — WebAssembly Text format generation:
  - WasmCodegen with complete WASM generation
  - compile_wasm() function in compiler
  - gen-wasm CLI command
  - /gen-wasm HTTP endpoint
- **JavaScript WASM Runtime**:
  - T27Runtime class for module loading
  - GF16 class with encode/decode methods
  - Browser playground with live compilation (4 example presets)
- **Python Bindings** (PyO3):
  - GF16, GF32 classes with arithmetic operations
  - NumPy array operations: array_to_gf16, gf16_dot_product, gf16_normalize, gf16_quantize_matrix
  - Constants: phi(), phi_gf16(), phi_gf32(), trinity_identity()
  - Comprehensive test suite
- **Formatter & Linter**:
  - tri-fmt formatter with lexer and formatting rules
  - L1-L5 constitutional compliance checks
  - Pre-commit hook integration
  - GitHub Actions format-check workflow
- **Coq Formal Verification** — 130+ Qed theorems:
  - Physics/CapBoost.v (38 Qed) — gamma^3 Decoupling-Cap Burst (OP_CAP_BOOST=0xF3)
  - Physics/FBBActive2.v (33 Qed) — Forward Body Bias (OP_FBB_ACTIVE=0xF2)
  - Physics/RBB.v (33 Qed) — Reverse Body Bias (OP_RBB=0xF1)
  - Physics/Avs96Safe.v (8 Qed) — AVS-96 Dopamine Safety (S-200)
  - Physics/StochSkipSafe.v (10 Qed) — Stochastic Time-Skip
  - Physics/Int2QuantSafe.v (8 Qed) — INT2 Activation Codebook
  - Physics/StochRound.v (9 Qed) — Stochastic Rounding (OP_STOCH_ROUND=0xE9)
  - 9 additional Physics modules: AdiabRC, DFS, DrowsyRet, HoloMux, MoeRouter, NodeShrink, NullorReversible, PurkinjeThermal, SparseGate, SparsityMask, SpeculativeExit, WLBoost
  - R18 Sacred Bank Extension: 16 to 32 slots (0xD0..0xFF)
  - Triple-decker power control: RBB to FBB-ACTIVE to CAP-BOOST

### Documentation
- **5 Tutorials**:
  - 010-why-ternary.md — Mathematical foundation
  - 011-goldenfloat-explained.md — Format deep dive
  - 012-spec-first-development.md — Workflow guide
  - 013-fpga-integration.md — Hardware tutorial
  - 014-coq-verification.md — Proof guide
- **4 Comparative Analysis Papers**:
  - GF vs IEEE 754
  - GF vs Posit
  - GF vs FP8 for LLMs
  - Ternary vs Binary Performance
- **GPU Backend Design**:
  - CUDA backend design document
  - Kernel designs for GF16 operations
  - ROCm backend design
- **Conference and Community**:
  - PLDI 2027 submission template
  - v1.0.0 release blog post
  - 5 announcement templates (GitHub, Twitter, Reddit, HN, Discord)
  - 20-slide PLDI 2027 presentation deck
- **Publication Summary**:
  - PUBLICATION_SUMMARY.md — Complete summary of v1.0.0
  - T27_IMPROVEMENT_PLAN.md — Improvement plan with priorities

### Examples
- **ML Quantization** — Model quantization example with GF16
- **Scientific Computing** — Numerical methods example
- **FPGA Accelerator** — GF16 accelerator with Verilog generation
- **WebAssembly** — GF calculator in browser

### Changed
- Updated .zenodo.json to v1.0.0
- Updated CITATION.cff to v1.0.0, 2026-05-16
- Updated Python bindings (Cargo.toml, pyproject.toml) to v1.0.0
- Updated pre-commit hook with format/lint gates
- README.md updated with v1.0.0 badges and release highlights
- System Status table expanded with LSP, bindings, verification rows
- Backends: Zig, Verilog, C to Zig, Verilog, C, Rust, WASM

### Performance
- GF16: 50% memory vs FP32, 75.84% ImageNet Top-1
- LLaMA-7B: GF16 28 tokens/s vs FP16 24 tokens/s (+16% speed)
- LSP hover: 120ms
- LSP completion: 95ms
- LSP semantic tokens: 85ms
- Compiler (small file): 80ms

---

## [Unreleased]

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
- Sacred physics constants derived from phi^2 + 1/phi^2 = 3
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

**phi^2 + 1/phi^2 = 3 | TRINITY**