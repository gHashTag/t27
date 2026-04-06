# CLARA-Bridge Tests

Test suite for CLARA-Bridge deliverables to verify they are functional.

## Purpose

The CLARA-Bridge MVP implements a high-assurance pipeline pattern. These tests verify:
- JSON schema validation for all deliverables
- run_scenario.py functionality
- Consistency between scenario files and schema

## Running Tests

```bash
# Run all CLARA-Bridge tests
pytest clara-bridge/tests/

# Run specific test file
pytest clara-bridge/tests/test_vetted_blocks.py
pytest clara-bridge/tests/test_scenarios.py

# Run with verbose output
pytest clara-bridge/tests/ -v

# Run with coverage
pytest clara-bridge/tests/ --cov=clara-bridge --cov-report=html
```

## Test Files

| File | Purpose | Tests |
|------|---------|--------|
| `test_vetted_blocks.py` | Validate vetted-blocks JSON schema | 14 tests |
| `test_scenarios.py` | Validate scenarios JSON schema | 14 tests |
| `test_experience_schema.py` | Validate experience-schema.json | 17 tests |
| `test_run_scenario.py` | Test run_scenario.py functionality | 14 tests |

## Coverage

The test suite covers:
- ✅ Schema validation (required fields, types, constraints)
- ✅ Logical consistency (step ordering, dependencies)
- ✅ Critical path verification (verdict steps marked)
- ✅ Dry-run functionality (commands printed, not executed)
- ✅ Error handling (invalid inputs, malformed JSON)
- ✅ Exit code behavior (0=success, 1=invalid)

## Integration with CI

These tests can be added to CI pipeline:
```yaml
# .github/workflows/clara-bridge-tests.yml
- name: CLARA-Bridge Tests
  run: pytest clara-bridge/tests/ -v
```

## Test Philosophy

These are **schema and functional tests**, not unit tests of mathematical formulas:
- Verify data structures are correct
- Verify runner behavior matches specification
- Verify edge cases are handled properly
- Do NOT test physical correctness (that's kepler_newton_tests.py's job)
