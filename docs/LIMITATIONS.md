# Limitations — T27 Known Constraints

**Status:** Active (Ring 053)
**Date:** 2026-05-09
**Purpose:** Document known limitations and constraints of T27 for transparency

---

## 1. Language Limitations

### 1.1 Feature Gaps

| Feature | Status | Planned | Notes |
|---------|--------|---------|-------|
| Generics | Not implemented | Q3 2026 | Type parameters not supported |
| Traits/Interfaces | Not implemented | Q4 2026 | No trait system |
| Async/Await | Not implemented | Q1 2027 | Synchronous execution only |
| Macros | Not implemented | Q2 2027 | No metaprogramming |
| Pattern matching | Partial | Q3 2026 | Limited to `match` on enums |
| Closures | Not implemented | Q4 2026 | No lambda functions |
| Modules | Basic | Q2 2026 | Limited module system |

### 1.2 Type System Limitations

- No dependent types
- No type inference for complex expressions
- No generic constraints
- No higher-kinded types
- Limited enum variants (no associated data)

### 1.3 Expressiveness

- No operator overloading
- No custom literals
- Limited string manipulation
- No regex literals
- No first-class functions

---

## 2. Numeric Limitations

### 2.1 GoldenFloat Limitations

| Format | Min Value | Max Value | Precision | Use Case |
|--------|-----------|-----------|-----------|----------|
| GF4 | -3.0 | 3.0 | Very low | Binary masks |
| GF8 | -7.5 | 7.5 | Low | Weights |
| GF12 | -120.0 | 120.0 | Medium | Attention |
| **GF16** | **-65504.0** | **65504.0** | **High** | **Primary inference** |
| GF20 | -1.3e6 | 1.3e6 | Very high | Training |
| GF24 | -2.1e8 | 2.1e8 | Ultra high | Precision |
| GF32 | -3.4e38 | 3.4e38 | Maximum | Full precision |

**Known Issues:**
- No subnormal numbers (flush to zero)
- No NaN or Infinity (ternary domain)
- Rounding mode: nearest even (IEEE 754-like)
- Limited dynamic range vs IEEE fp32

### 2.2 Sacred Constants Accuracy

| Constant | GF16 Error | GF32 Error | Notes |
|----------|------------|------------|-------|
| PHI | 0.0526% | < 1e-15% | GF32 is exact |
| PHI_INV | 0.0326% | < 1e-15% | GF32 is exact |
| GAMMA_LQG | 0.0297% | < 1e-15% | GF32 is exact |
| G | N/A | < 1e-15% | Requires GF32 |
| OMEGA_LAMBDA | N/A | < 1e-15% | Requires GF32 |

**Gap:** Very small constants require GF32, not GF16.

---

## 3. Backend Limitations

### 3.1 Zig Backend

| Feature | Status | Limitation |
|---------|--------|------------|
| Standard library | Partial | Limited to generated code |
| FFI | Not supported | Cannot call external functions |
| Async | Not supported | No async/await |
| Concurrency | Not supported | No threads/goroutines |

### 3.2 C Backend

| Feature | Status | Limitation |
|---------|--------|------------|
| Standard library | Minimal | No stdlib integration |
| FFI | Planned | Not yet implemented |
| Memory management | Manual | No garbage collection |

### 3.3 Verilog Backend

| Feature | Status | Limitation |
|---------|--------|------------|
| Synthesis | XC7A100T only | Not tested on other FPGAs |
| Timing | 100MHz target | May not meet on all designs |
| Resources | Manual estimation | No automatic resource calculation |

### 3.4 Rust Backend

| Feature | Status | Limitation |
|---------|--------|------------|
| Ownership | Simplified | No borrow checking in generated code |
| Traits | Not supported | No trait implementations |
| Async | Not supported | No async/await |

### 3.5 TypeScript Backend

| Feature | Status | Limitation |
|---------|--------|------------|
| Types | Simplified | No advanced types |
| Async | Not supported | No async/await |
| DOM | Not supported | No browser APIs |

---

## 4. Performance Limitations

### 4.1 Compiler Performance

| Metric | Current | Target | Gap |
|--------|---------|--------|-----|
| Parse time (505 specs) | ~5s | < 1s | 5x slower |
| Codegen time (Zig) | ~10s | < 2s | 5x slower |
| Codegen time (C) | ~8s | < 2s | 4x slower |
| Full build | ~30s | < 10s | 3x slower |

### 4.2 Runtime Performance

| Operation | GF16 vs FP32 | Notes |
|-----------|---------------|-------|
| Addition | 1.3x faster | Simpler encoding |
| Multiplication | 1.2x faster | No denormals |
| Division | 1.1x faster | Simpler rounding |

---

## 5. Tooling Limitations

### 5.1 Compiler

- No incremental compilation
- No parallel compilation
- No caching of generated code
- No hot reloading

### 5.2 IDE Integration

- Limited LSP server
- No syntax highlighting for `.t27`
- No error highlighting in editors
- No auto-completion

### 5.3 Debugging

- No source-level debugging
- No step-through in generated code
- No breakpoints
- Limited error messages

---

## 6. Documentation Limitations

### 6.1 API Docs

- No auto-generated API documentation
- Limited examples
- No interactive tutorials

### 6.2 Learning Resources

- No video tutorials
- Limited blog posts
- No community guides

---

## 7. Ecosystem Limitations

### 7.1 Package Management

- No package registry
- No dependency management
- No version constraints

### 7.2 Community

- Small community
- Limited third-party packages
- No ecosystem of libraries

---

## 8. Security Limitations

### 8.1 Input Validation

- No input sanitization
- No validation of untrusted input
- No sandboxing

### 8.2 Memory Safety

- No memory safety in generated C code
- No bounds checking in generated code
- No overflow protection

---

## 9. Compliance Limitations

### 9.1 Standards

- Not IEEE 754 compliant (different format)
- Not POSIX compliant
- Not ANSI C compliant

### 9.2 Regulations

- Not SOC 2 compliant
- Not GDPR compliant (no PII handling)
- Not HIPAA compliant

---

## 10. Mitigation Strategies

### 10.1 Short-term (Next 3 months)

- Implement incremental compilation
- Add LSP server
- Improve error messages

### 10.2 Medium-term (Next 6 months)

- Add generics support
- Implement FFI
- Create package registry

### 10.3 Long-term (Next 12 months)

- Add async/await
- Implement traits
- Full IDE integration

---

## 11. Workarounds

### 11.1 For Numeric Limitations

Use GF32 for:
- Very small constants (< 1e-10)
- Very large constants (> 1e5)
- High-precision requirements

### 11.2 For Backend Limitations

Use multiple backends:
- Zig for system programming
- C for embedded systems
- Verilog for FPGA synthesis
- TypeScript for web

### 11.3 For Tooling Limitations

Use external tools:
- Standard text editors for editing
- Manual build scripts
- Custom debugging output

---

## 12. Future Work

See `docs/GOLDENFLOAT_VALIDATION_PLAN.md` for:
- L4 differential oracle implementation
- Sacred constants validation
- Cross-backend comparison

See `docs/EPOCH_01_HARDEN_PLAN.md` for:
- CI improvements
- Documentation enhancements
- Security hardening

---

## 13. References

- `docs/NUMERIC-STANDARD-001.md` — GoldenFloat specification
- `docs/BACKEND_CONTRACT.md` — Backend obligations
- `docs/TESTING_TAXONOMY.md` — Test limitations
- `docs/SECRETS_HYGIENE_AUDIT.md` — Security limitations

---

## 14. Reporting Limitations

To report a limitation or request a feature:

1. Search existing issues on GitHub
2. Create new issue with `limitation:` prefix
3. Describe the limitation clearly
4. Provide use case and workaround if known

---

**φ² + 1/φ² = 3 | TRINITY**
