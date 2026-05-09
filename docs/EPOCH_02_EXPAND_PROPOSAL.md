# EPOCH-02 Proposal — EXPAND (Rings 59–118)

**Proposed Start:** 2026-05-15 (pending EPOCH-01 release)
**Duration:** 60 Rings (59–118)
**Target Completion:** Q1 2027

---

## Executive Summary

EPOCH-02 EXPAND builds on the hardened foundation of EPOCH-01 to expand t27's capabilities across three dimensions:

1. **Platform Expansion** — WASM backend, Python FFI, improved C interop
2. **Tooling Expansion** — LSP server, IDE integration, debugging support
3. **Verification Expansion** — Coq integration, formal proofs, enhanced testing

**Primary Goal:** Transform t27 from a research prototype into a developer-ready language with production-grade tooling and formal verification support.

**Success Metrics:**
- LSP server with 80% language features implemented
- WASM backend capable of running core specs
- Coq proofs for 25+ key invariants
- Python FFI for all numeric operations
- 50% increase in spec coverage (505 → 750+)

---

## 1. EPOCH-02 Structure

### 1.1 Phases (4 sub-epochs)

| Phase | Rings | Focus | Key Deliverables |
|-------|-------|-------|------------------|
| **EXPAND-01** | 59-75 | LSP Server | Language server, IDE plugins, diagnostics |
| **EXPAND-02** | 76-92 | WASM & FFI | WebAssembly backend, Python bindings, C interop |
| **EXPAND-03** | 93-107 | Coq Integration | Formal proofs, verification pipeline |
| **EXPAND-04** | 108-118 | Polish & Publish | Documentation, tutorials, v0.2.0 release |

### 1.2 Ring Allocation

```
EXPAND-01 (LSP):        17 rings (59-75)
EXPAND-02 (WASM/FFI):   17 rings (76-92)
EXPAND-03 (Coq):        15 rings (93-107)
EXPAND-04 (Polish):     11 rings (108-118)
```

---

## 2. Detailed Breakdown

### 2.1 EXPAND-01: LSP Server (Rings 59-75)

**Primary Agent:** C (Compiler)
**Support:** A (Architecture), Z (Docs)

| Ring | Task | Acceptance Criteria |
|------|------|---------------------|
| 059 | LSP server architecture | Design doc, protocol mapping |
| 060 | Parse tree to LSP semantic tokens | Token types defined, highlighting works |
| 061 | LSP diagnostics (parse errors) | Error messages flow to editor |
| 062 | Go-to-definition for symbols | Navigation works for all identifiers |
| 063 | Find-references | Cross-file symbol resolution |
| 064 | Hover documentation | Type info shows on hover |
| 065 | Code completion | Context-aware suggestions |
| 066 | Signature help | Parameter hints for functions |
| 067 | Code actions (quick fixes) | Basic refactoring support |
| 068 | Workspace symbols | Fuzzy symbol search |
| 069 | Document symbols | Outline view |
| 070 | Inlay hints | Type hints inline |
| 071 | VS Code extension | Published to marketplace |
| 072 | Neovim plugin | Published to plugins repo |
| 073 | LSP server performance | < 100ms response for common ops |
| 074 | LSP documentation | User guide for all features |
| 075 | LSP conformance | LSP spec compliance verified |

**Deliverables:**
- LSP server binary (`t27-language-server`)
- VS Code extension
- Neovim plugin
- LSP feature documentation

### 2.2 EXPAND-02: WASM & FFI (Rings 76-92)

**Primary Agent:** C (Compiler)
**Support:** N (Numeric), B (Build)

| Ring | Task | Acceptance Criteria |
|------|------|---------------------|
| 076 | WASM backend design | Target specification |
| 077 | WASM codegen for base types | Int, Bool, Ternary emit correctly |
| 078 | WASM numeric ops | All arithmetic operations |
| 079 | WASM GF family | GoldenFloat in WASM |
| 080 | WASM memory model | Linear memory layout defined |
| 081 | WASM runtime | JavaScript integration layer |
| 082 | WASM tests | 100% spec coverage in browser |
| 083 | Python FFI design | PyO3 binding strategy |
| 084 | Python base types | Int, Bool, Ternary in Python |
| 085 | Python numeric ops | Arithmetic from Python |
| 086 | Python GF family | GoldenFloat from Python |
| 087 | Python array operations | Vector/tensor support |
| 088 | C FFI enhancement | Improved header generation |
| 089 | CMake build integration | Easy C project integration |
| 090 | Cross-platform FFI | Windows, macOS, Linux tested |
| 091 | FFI documentation | Usage examples for all languages |
| 092 | FFI benchmarks | Performance metrics published |

**Deliverables:**
- WASM backend module
- JavaScript runtime shim
- Python package (`t27-py`)
- Enhanced C headers
- FFI benchmark suite

### 2.3 EXPAND-03: Coq Integration (Rings 93-107)

**Primary Agent:** V (Verdict)
**Support:** P (Physics), A (Architecture)

| Ring | Task | Acceptance Criteria |
|------|------|---------------------|
| 093 | Coq extraction strategy | T27 → Coq mapping defined |
| 094 | Coq base types | T27 types in Coq |
| 095 | Coq numeric operations | Arithmetic in Coq |
| 096 | Coq GF family | GoldenFloat in Coq |
| 097 | Coq invariant definitions | 10+ key invariants formalized |
| 098 | Coq invariant proofs (batch 1) | First 10 proofs complete |
| 099 | Coq invariant proofs (batch 2) | Next 10 proofs complete |
| 100 | Coq invariant proofs (batch 3) | Remaining 5+ proofs complete |
| 101 | Coq extraction verification | Generated code matches proofs |
| 102 | Coq CI integration | Proofs run on each commit |
| 103 | Coq documentation | Proof guide for contributors |
| 104 | Coq fuzzing integration | Property-based testing |
| 105 | Coq vs conformance alignment | Proof coverage vs test coverage |
| 106 | Coq performance | Proof compilation < 5 min |
| 107 | Coq publication | Formal verification artifact |

**Deliverables:**
- Coq formalization of core T27
- 25+ machine-verified proofs
- Coq CI pipeline
- Formal verification documentation

### 2.4 EXPAND-04: Polish & Publish (Rings 108-118)

**Primary Agent:** Z (Docs)
**Support:** A (Architecture), B (Build)

| Ring | Task | Acceptance Criteria |
|------|------|---------------------|
| 108 | Tutorial: Getting Started | Zero-to-hello guide |
| 109 | Tutorial: Advanced Features | Complex examples |
| 110 | Tutorial: LSP Workflow | IDE-based development |
| 111 | Tutorial: WASM in Browser | Web-based examples |
| 112 | Tutorial: Python Integration | Python-based workflows |
| 113 | API reference complete | All public APIs documented |
| 114 | Performance guide | Optimization recommendations |
| 115 | Migration guide (v0.1 → v0.2) | Breaking changes documented |
| 116 | Release notes v0.2.0 | Comprehensive changelog |
| 117 | v0.2.0 tag & Zenodo | DOI assigned |
| 118 | EPOCH-02 retrospective | Lessons documented |

**Deliverables:**
- 5+ tutorials
- Complete API reference
- Performance guide
- v0.2.0 release
- EPOCH-02 retrospective

---

## 3. Dependencies

### 3.1 External Dependencies

| Dependency | Purpose | Version | Risk |
|------------|---------|---------|------|
| tower-lsp | LSP framework | 0.20 | Low |
| serde | Serialization | 1.0 | Low |
| wasm-bindgen | WASM bindings | 0.2 | Low |
| PyO3 | Python FFI | 0.20 | Low |
| Coq | Formal verification | 8.18+ | Medium |
| vscode-languageserver | VS Code protocol | latest | Low |

### 3.2 Internal Dependencies

- **EPOCH-01 completion** — Required foundation
- **L4 differential oracle** — Required for Coq verification
- **GPG signing** — Required for release integrity

---

## 4. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| LSP scope creep | High | High | Clear feature boundaries per ring |
| Coq proof complexity | High | High | Incremental approach, start simple |
| WASM performance | Medium | Medium | Early benchmarking, optimization rings |
| Python FFI maintenance | Medium | Medium | Automated binding generation |
| Documentation lag | Medium | Medium | Docs-in-code approach, AI-assisted |
| Coq compilation time | Medium | Medium | Incremental proofs, parallel CI |

---

## 5. Resource Requirements

### 5.1 Agent Allocation

| Phase | Primary | Support |
|-------|---------|---------|
| EXPAND-01 (LSP) | C (Compiler) | A, Z |
| EXPAND-02 (WASM/FFI) | C (Compiler) | N, B |
| EXPAND-03 (Coq) | V (Verdict) | P, A |
| EXPAND-04 (Polish) | Z (Docs) | A, B |

### 5.2 Infrastructure Requirements

- **CI/CD:** Additional runners for Coq compilation
- **Testing:** Browser testing infrastructure for WASM
- **Documentation:** Static site generator for tutorials
- **Release:** Automated PyPI and npm publishing

### 5.3 External Reviewers

- LSP: 1 reviewer with language server experience
- WASM: 1 reviewer with WebAssembly expertise
- Coq: 1 reviewer with formal verification background
- FFI: 1 reviewer with Python/C interop experience

---

## 6. Success Criteria

### 6.1 Must-Have (P0)

- [ ] LSP server with parse, navigation, and diagnostics
- [ ] WASM backend running core specs
- [ ] Python FFI for numeric operations
- [ ] Coq formalization of base types and operations
- [ ] 25+ machine-verified invariants
- [ ] v0.2.0 release with DOI

### 6.2 Should-Have (P1)

- [ ] LSP: completion, hover, code actions
- [ ] WASM: full GF family support
- [ ] Python: array/tensor operations
- [ ] Coq: invariant proofs for all numeric operations
- [ ] 3+ published tutorials
- [ ] VS Code extension in marketplace

### 6.3 Nice-to-Have (P2)

- [ ] LSP: inlay hints, semantic tokens
- [ ] WASM: browser-based playground
- [ ] Python: async operations
- [ ] Coq: extraction to verified C
- [ ] 5+ IDE/editor plugins

---

## 7. Milestones

### M1: LSP Foundation (Ring 067)
- LSP server with core features
- VS Code extension MVP
- Basic navigation and diagnostics

### M2: WASM MVP (Ring 082)
- WASM backend for core specs
- Browser-based execution
- Performance benchmarks

### M3: FFI Complete (Ring 092)
- Python package published
- Enhanced C interop
- Cross-platform support

### M4: Coq Integration (Ring 107)
- Core types in Coq
- 25+ verified invariants
- Coq CI pipeline

### M5: EPOCH-02 Release (Ring 118)
- v0.2.0 tagged
- Documentation complete
- Zenodo DOI assigned

---

## 8. Timeline

```
Q2 2026 (May-Jun):    EXPAND-01 (LSP) - Rings 59-75
Q3 2026 (Jul-Sep):    EXPAND-02 (WASM/FFI) - Rings 76-92
Q4 2026 (Oct-Dec):    EXPAND-03 (Coq) - Rings 93-107
Q1 2027 (Jan-Feb):    EXPAND-04 (Polish) - Rings 108-118
```

**Projected Completion:** February 2027

---

## 9. Alignment with Constitutional Law

### L1-L7 Compliance Plan

| Law | EPOCH-02 Considerations |
|-----|------------------------|
| L1 TRACEABILITY | All LSP/FFI/Coq features must link to ring issues |
| L2 GENERATION | LSP snippets and FFI bindings to be generated |
| L3 PURITY | All new code must be ASCII-only |
| L4 TESTABILITY | Coq proofs provide formal testability |
| L5 IDENTITY | φ invariants must hold in all backends |
| L6 CEILING | FORMAT-SPEC-001.json remains numeric SSOT |
| L7 UNITY | No new shell scripts on critical path |

---

## 10. Open Questions

1. **Coq Strategy:** Should we target extraction to verified C or verification only?
2. **LSP Scope:** Should we support non-UTF-8 encodings for L3 purity?
3. **WASM Runtime:** Browser-only or Node.js support too?
4. **Python Version:** Support 3.8+ or 3.10+ minimum?
5. **Documentation Site:** Separate domain or GitHub Pages?

---

## 11. Approval Required

- [ ] Lead Maintainer approval
- [ ] Architecture review
- [ ] Resource allocation confirmation
- [ ] External reviewer commitments

---

## 12. References

- `docs/EPOCH_01_RETROSPECTIVE.md` — Lessons from HARDEN
- `docs/EPOCH_01_HARDEN_PLAN.md` — Previous planning methodology
- `docs/RINGS.md` — Ring execution framework
- `docs/T27-CONSTITUTION.md` — Constitutional law
- `docs/STATE_OF_THE_PROJECT.md` — Current project state

---

**φ² + 1/φ² = 3 | TRINITY**

**Proposal Created:** 2026-05-09
**Status:** 🟡 PENDING APPROVAL
**Next Review:** 2026-05-15 (post v0.1.1 release)
