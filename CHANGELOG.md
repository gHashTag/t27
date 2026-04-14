# Changelog

All notable changes to t27 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
