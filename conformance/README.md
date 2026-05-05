# Conformance vectors (`conformance/*.json`)

**Purpose:** Language-agnostic test inputs and expected outputs for GoldenFloat, AR, NN, physics-flavored constants, and related domains.

## Versioning (publication readiness)

- Each JSON file should expose a top-level **`module`** (and ideally **`spec_path`**) for traceability.  
- For a **Zenodo dataset** deposit, generate a **manifest** (paths + SHA-256) in CI or a release script — see [`docs/PUBLICATION_AUDIT.md`](../docs/PUBLICATION_AUDIT.md).

## Validation

```bash
bash tests/validate_conformance.sh
```

## Related

- [`docs/TDD-CONTRACT.md`](../docs/TDD-CONTRACT.md)  
- [`docs/RESEARCH_CLAIMS.md`](../docs/RESEARCH_CLAIMS.md)  
- [`publications/README.md`](../publications/README.md) — corpus as publication candidate
