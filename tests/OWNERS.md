# OWNERS — tests/

## Primary

**Q-QA** — integration coverage is defined by **`t27c suite`** (`bootstrap/src/suite.rs`), not shell scripts.

## Contents

- **`*.t27`** — spec-level tests and suite documentation (e.g. `comprehensive_suite.t27`, `ring0_trivial.t27`).
- **No `*.sh`** — orchestration lives in the Rust bootstrap (`suite`, `validate-conformance`, `validate-gen-headers`).

## Commands (from repo root)

```bash
./bootstrap/target/release/t27c suite --repo-root .
./bootstrap/target/release/t27c validate-conformance --repo-root .
./bootstrap/target/release/t27c validate-gen-headers --repo-root .
# or
./scripts/tri test
./scripts/tri validate-conformance
./scripts/tri validate-gen-headers
```

## Dependencies

- `specs/`, `compiler/`, `conformance/`, `gen/` (for header check), `.trinity/seals/`.
