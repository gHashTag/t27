# OWNERS — conformance/

## Primary

**E-Evidence** — JSON and other conformance vectors; ring evidence artifacts.

## Dependencies

- `specs/` — semantic SSOT for expected behavior.
- `gen/` — generated code under test in CI.

## Outputs

Vectors consumed by `tests/` and PHI-loop validation scripts.

**Machine-readable standards (seed):** **`FORMAT-SPEC-001.json`** (GoldenFloat layout, aligned with **`docs/nona-02-organism/NUMERIC-STANDARD-001.md`**), **`axiom_system.json`** (axiom / theorem / physics claim catalog seed). Schema: **`schemas/numeric-format-v1.json`**.
