# Testing Taxonomy — T27 Test Classification

**Status:** Active (Ring 040)
**Date:** 2026-05-09
**Purpose:** Define taxonomy for all test types in T27 project

---

## 1. Overview

T27 uses multiple test types to ensure correctness, performance, and reliability. This document classifies all test types and defines their purpose, scope, and execution context.

---

## 2. Test Hierarchy

```
T27 Testing
├── L1: Unit Tests
│   ├── Type tests
│   ├── Function tests
│   └── Operation tests
├── L2: Conformance Tests
│   ├── Spec conformance
│   └── Backend conformance
├── L3: Integration Tests
│   ├── Compiler integration
│   └── Backend integration
├── L4: Property-Based Tests
│   ├── Roundtrip properties
│   └── Invariant properties
├── L5: Differential Tests
│   ├── Reference comparison
│   └── Cross-backend comparison
├── L6: Benchmark Tests
│   ├── Performance benchmarks
│   └── Memory benchmarks
└── L7: Fuzz Tests
    ├── Parser fuzzing
    └── Codegen fuzzing
```

---

## 3. Test Types

### 3.1 L1: Unit Tests

**Purpose:** Test individual functions and types in isolation.

**Scope:**
- Type definitions (`type` blocks)
- Function implementations (`fn` blocks)
- Operation semantics (`+`, `-`, `*`, `/`, etc.)

**Example:**
```t27
test "gf16_add_positive" {
    let a: GF16 = 1.0;
    let b: GF16 = 2.0;
    let result: GF16 = gf16_add(a, b);
    assert result == 3.0;
}
```

**Execution:** Run via `./scripts/tri test --unit`

---

### 3.2 L2: Conformance Tests

**Purpose:** Validate that generated backends conform to specification.

**Scope:**
- Spec conformance vectors (`conformance/*.json`)
- Backend conformance (Zig, C, Verilog, Rust, TypeScript)

**Example:**
```json
{
  "schema_version": "1.0",
  "spec": "specs/numeric/gf16.t27",
  "backend": "zig",
  "tests": [
    {
      "name": "add_positive",
      "input": {"a": 1.0, "b": 2.0},
      "expected": 3.0,
      "tolerance": 0.001
    }
  ]
}
```

**Execution:** Run via `./scripts/tri validate-conformance`

---

### 3.3 L3: Integration Tests

**Purpose:** Test integration between compiler components and backends.

**Scope:**
- Compiler pipeline (parse → codegen → compile)
- Backend integration (generated code + runtime)
- End-to-end workflows

**Example:**
```t27
test "compiler_pipeline_full" {
    # Parse spec
    let ast = parse("specs/numeric/gf16.t27");
    # Generate code
    let code = codegen_zig(ast);
    # Compile
    let binary = compile_zig(code);
    # Run and verify
    let output = run(binary);
    assert output == expected;
}
```

**Execution:** Run via `./scripts/tri test --integration`

---

### 3.4 L4: Property-Based Tests

**Purpose:** Verify properties that hold for all inputs.

**Scope:**
- Roundtrip properties (encode → decode → original)
- Invariant properties (L5 IDENTITY, etc.)
- Algebraic properties (associativity, commutativity)

**Example:**
```t27
invariant "gf16_roundtrip" {
    for x in random_values(1000) {
        let encoded = gf16_encode(x);
        let decoded = gf16_decode(encoded);
        assert abs(decoded - x) < 0.001;
    }
}
```

**Execution:** Run via `./scripts/tri test --property`

---

### 3.5 L5: Differential Tests

**Purpose:** Compare implementation against reference implementation.

**Scope:**
- Reference comparison (Python `decimal`, MPFR)
- Cross-backend comparison (Zig vs C vs Verilog)
- Conformance vector validation

**Example:**
```t27
test "gf16_vs_reference" {
    for x in test_values() {
        let gf16_result = gf16_sqrt(x);
        let ref_result = decimal_sqrt(x);
        assert abs(gf16_result - ref_result) < 0.001;
    }
}
```

**Execution:** Run via `./scripts/tri test --differential`

---

### 3.6 L6: Benchmark Tests

**Purpose:** Measure performance and resource usage.

**Scope:**
- Performance benchmarks (operations per second)
- Memory benchmarks (bytes per value)
- Energy benchmarks (Joules per operation, FPGA only)

**Example:**
```t27
bench "gf16_add_performance" {
    let iterations = 1000000;
    let start = time_now();
    for i in 0..iterations {
        let _ = gf16_add(1.0, 2.0);
    }
    let end = time_now();
    let ops_per_sec = iterations / (end - start);
    report("ops_per_sec", ops_per_sec);
}
```

**Execution:** Run via `./scripts/tri test --bench`

---

### 3.7 L7: Fuzz Tests

**Purpose:** Find edge cases and bugs through random input generation.

**Scope:**
- Parser fuzzing (random .t27 syntax)
- Codegen fuzzing (random spec structures)
- Backend fuzzing (random operations)

**Example:**
```t27
test "parser_fuzz" {
    for i in 0..10000 {
        let random_input = generate_random_t27();
        let result = parse(random_input);
        # Verify result is either valid or expected error
        assert result.is_valid() || result.is_expected_error();
    }
}
```

**Execution:** Run via `./scripts/tri test --fuzz`

---

## 4. Test by Domain

### 4.1 Numeric Tests

| Test Type | Count | Location |
|-----------|-------|----------|
| GF4 roundtrip | 8 | `conformance/gf4_vectors.json` |
| GF8 roundtrip | 16 | `conformance/gf8_vectors.json` |
| GF12 roundtrip | 20 | `conformance/gf12_vectors.json` |
| GF16 roundtrip | 28 | `conformance/gf16_vectors.json` |
| GF20 roundtrip | 24 | `conformance/gf20_vectors.json` |
| GF24 roundtrip | 24 | `conformance/gf24_vectors.json` |
| GF32 roundtrip | 32 | `conformance/gf32_vectors.json` |
| Phi identity | 10 | `conformance/phi_identity_vectors.json` |
| Phi ratio | 8 | `conformance/phi_ratio_vectors.json` |

### 4.2 Sacred Physics Tests

| Test Type | Count | Location |
|-----------|-------|----------|
| Sacred constants | 15 | `conformance/sacred_physics.json` |
| Physics constants | 12 | `conformance/sacred_physics_constants.json` |
| Cosmology constants | 8 | `conformance/sacred_physics_cosmology.json` |
| Gravity constants | 10 | `conformance/sacred_physics_gravity.json` |

### 4.3 Neural Network Tests

| Test Type | Count | Location |
|-----------|-------|----------|
| Attention | 16 | `conformance/nn_attention_vectors.json` |
| HSLM | 14 | `conformance/nn_hslm_vectors.json` |

### 4.4 AR (Automated Reasoning) Tests

| Test Type | Count | Location |
|-----------|-------|----------|
| Ternary logic | 27 | `conformance/ar_ternary_logic.json` |
| Proof trace | 8 | `conformance/ar_proof_trace.json` |
| Datalog engine | 10 | `conformance/ar_datalog_engine.json` |
| Restraint | 6 | `conformance/ar_restraint.json` |
| Explainability | 8 | `conformance/ar_explainability.json` |
| ASP solver | 8 | `conformance/ar_asp_solver.json` |
| Composition | 12 | `conformance/ar_composition.json` |

### 4.5 FPGA Tests

| Test Type | Count | Location |
|-----------|-------|----------|
| MAC unit | 18 | `conformance/fpga_mac_vectors.json` |
| Bridge | 6 | `conformance/fpga_bridge.json` |
| SPI | 4 | `conformance/fpga_spi.json` |
| UART | 8 | `conformance/fpga_uart.json` |

---

## 5. Test Execution

### 5.1 Local Execution

```bash
# Run all tests
./scripts/tri test

# Run specific test type
./scripts/tri test --unit
./scripts/tri test --conformance
./scripts/tri test --integration
./scripts/tri test --property
./scripts/tri test --bench

# Run tests for specific spec
./scripts/tri test specs/numeric/gf16.t27

# Run tests with verbose output
./scripts/tri test --verbose

# Run tests with coverage
./scripts/tri test --coverage
```

### 5.2 CI Execution

```bash
# Fast PR lane (subset of tests)
make ci-fast

# Full nightly lane (all tests)
make ci-full

# FPGA-specific tests
make ci-fpga
```

### 5.3 Reproducible Execution

```bash
# Create reproducibility bundle
make repro-bundle

# Verify bundle
tar -xzf repro/trinity-repo-*.tar.gz
cd repro/bundle
./scripts/tri test
```

---

## 6. Test Coverage

### 6.1 Current Coverage

| Domain | Specs | Tests | Coverage |
|--------|-------|-------|----------|
| Base | 2 | 36 | 100% |
| Numeric | 8 | 192 | 100% |
| Math & Physics | 2 | 55 | 100% |
| ISA | 1 | 27 | 100% |
| FPGA | 7 | 70 | 100% |
| VSA | 2 | 20 | 100% |
| Neural | 2 | 30 | 100% |
| Queen | 1 | 10 | 100% |
| AR | 7 | 79 | 100% |
| Compiler | 9 | 72 | 100% |
| **TOTAL** | **41** | **591** | **100%** |

### 6.2 Coverage Goals

- **Unit test coverage:** > 90%
- **Conformance coverage:** 100% (achieved)
- **Property test coverage:** > 80%
- **Fuzz test coverage:** > 70% of parser paths

---

## 7. Test Quality Metrics

### 7.1 Metrics Tracked

| Metric | Target | Current |
|--------|--------|---------|
| Test pass rate | 100% | 100% |
| Test execution time | < 60s (fast) | ~45s |
| Conformance coverage | 100% | 100% |
| Property test iterations | 1000 per test | 1000 |
| Fuzz test iterations | 10000 per test | 10000 |
| Benchmark stability | < 5% variance | < 3% |

### 7.2 Flaky Test Detection

Flaky tests are identified by:
- Failing on CI but passing locally
- Intermittent failures on same input
- Timing-dependent failures

**Action:** Flaky tests are disabled and tracked in `docs/FLAKY_TESTS.md`

---

## 8. Test Maintenance

### 8.1 Adding New Tests

1. Add test to appropriate `.t27` spec file
2. Add corresponding conformance vector to `conformance/`
3. Update test count in `docs/TESTING_TAXONOMY.md`
4. Run `./scripts/tri test` to verify
5. Commit with `test(spec): add <test_name>`

### 8.2 Updating Existing Tests

1. Update test in `.t27` spec file
2. Update corresponding conformance vector
3. Run `./scripts/tri test` to verify
4. Commit with `test(spec): update <test_name>`

### 8.3 Removing Tests

1. Remove test from `.t27` spec file
2. Remove corresponding conformance vector
3. Update test count in `docs/TESTING_TAXONOMY.md`
4. Run `./scripts/tri test` to verify
5. Commit with `test(spec): remove <test_name>`

---

## 9. Test Infrastructure

### 9.1 Test Runner

**Location:** `scripts/tri`

**Features:**
- Parallel test execution
- Test filtering by type, spec, or name
- Verbose output with test names and results
- Coverage reporting
- Benchmark timing

### 9.2 Test Generators

**Location:** `conformance/`

**Features:**
- JSON schema validation
- Test vector generation from specs
- Differential oracle support
- Cross-backend test comparison

### 9.3 Test Reports

**Location:** `reports/` (generated)

**Reports include:**
- Test execution summary
- Pass/fail counts
- Coverage report
- Benchmark results
- Flaky test detection

---

## 10. References

- `docs/CONFORMANCE_TRACEABILITY.md` — Conformance mapping
- `docs/NUMERICS_VALIDATION.md` — Numeric validation
- `docs/GOLDENFLOAT_VALIDATION_PLAN.md` — Validation plan
- `conformance/` — Test vectors

---

**φ² + 1/φ² = 3 | TRINITY**
