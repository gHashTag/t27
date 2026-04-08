# Dissertation Workflow — AGENT EXECUTION

## Overview

The Trinity S³AI doctoral dissertation is formalized as an agent-executable workflow using:
- `.trinity/dissertation_program.tri` — Canonical task graph for all strands
- `.trinity/strand_i_workflow.tri` — Strand I specific workflow tasks
- `.trinity/experience/dissertation/` — Per-strand artifacts and outputs
- `tri` CLI commands — Workflow orchestration
- `specs/dissertation/verification/` — Reproducible verification specs

## Execution Phases

### Phase 1: Structure Audit

Agent runs structure verification on each section:

```bash
tri verdict --dissertation-structure strand-i/chapter-03.md
```

**Output**: Validates academic structure, theorem flow, RQ mapping.

**Checks**:
- Abstract present and within word limit
- All RQs introduced in introduction
- RQ-theorem mapping consistent
- Theorem flow logically ordered
- Limitations section comprehensive
- Conclusion addresses all RQs

**Artifact**: `.trinity/experience/dissertation/strand-i/structure/structure_report.json`

### Phase 2: Proof Verification

Agent verifies each formal proof:

```bash
tri verdict --proof-check strand-i/chapter-03.md
```

**Output**: Step-by-step proof validation, missing justification flags.

**Proof Types**:
- `trinity_identity` — Theorem 3.1
- `fixed_point_convergence` — Theorem 4.1
- `golden_float_allocation` — Proposition 4.2
- `vsa_binding_properties` — Theorem 5.1
- `vsa_permutation_invariants` — Theorem 5.2

**Artifact**: `.trinity/experience/dissertation/strand-i/proofs/proof_validation_report.json`

### Phase 3: Citation Audit

Agent checks reference support:

```bash
tri verdict --citation-audit strand-i/bibliography.bib
```

**Output**: Claims without citations, weak support flags, novelty validation.

**Checks**:
- Every novelty claim has citation
- VSA operations cited to original sources
- Sacred constants reference CODATA
- GoldenFloat formats compare to IEEE 754
- No orphan citations

**Artifact**: `.trinity/experience/dissertation/strand-i/citations/citation_audit_report.json`

### Phase 4: Verification Workflow

Generate and run reproducible checks:

```bash
tri gen specs/dissertation/verification/trinity_identity.t27
tri test specs/dissertation/verification/trinity_identity.t27
```

**Output**: Pass/fail results captured in `.trinity/experience/`.

**Verification Specs**:
- `specs/dissertation/verification/trinity_identity.t27` — Theorem 3.1
- `specs/dissertation/verification/fixed_point_convergence.t27` — Theorem 4.1
- `specs/vsa/vsa_core.t27` — Theorem 5.1 (existing)
- `specs/numeric/goldenfloat_family.t27` — Proposition 4.2 (existing)

**Artifact**: `.trinity/experience/dissertation/strand-i/verification/verification_results.json`

### Phase 5: Terminology Normalization

Ensure consistent terminology across all strands:

```bash
tri verdict --terminology strand-i/draft.md
```

**Output**: Term normalization tables.

**Canonical Terms**:
- "Trinity S³AI" (not "Trinity S3AI" or "Trinity-AI")
- "Vector Symbolic Architecture" (not "VSA alone")
- "Hyperdimensional Computing" (not "HDC alone")
- "GoldenFloat" (not "golden-float" or "Golden Float")
- "TRI-27" (not "tri-27" or "Trinity 27")
- "Ternary" (not "3-valued" or "three-valued")
- "Trit" (not "ternary digit" or "trit-digit")

**Artifact**: `.trinity/experience/dissertation/strand-i/terminology/terminology_normalization.json`

### Phase 6: Cross-Strand Continuity

Create Strand I → II/III dependency maps:

```bash
tri verdict --continuity strand-i/program.md
```

**Output**: Cross-reference tables showing theorem reuse.

**Strand I → II Mappings**:
- Theorem 3.1 (Trinity Identity) → Cognitive structural invariants (II.3)
- Theorem 4.1 (Fixed-Point) → Learning as attractor dynamics (II.4)
- Proposition 4.2 (GoldenFloat) → Quantized cognitive states (II.3.2)
- Theorem 5.1 (VSA Binding) → Symbolic reasoning primitives (II.5)
- Theorem 5.2 (VSA Similarity) → Cognitive matching metrics (II.5.2)
- Proposition 5.1 (Trit Encoding) → Ternary cognitive representations (II.5.3)

**Strand I → III Mappings**:
- Proposition 4.2 (GoldenFloat GF16) → FPGA GF16/TF3 arithmetic (III.2)
- Proposition 4.2 (GoldenFloat Family) → Quantized neural networks (III.3)
- Proposition 5.1 (Trit Encoding) → FPGA trit storage (III.2.1)
- Theorem 5.1 (VSA Binding) → Parallel binding/unbinding (III.4)
- Theorem 5.2 (VSA Similarity) → FPGA similarity search (III.4.1)

**Artifact**: `.trinity/experience/dissertation/strand-i/continuity/cross_strand_dependency_map.json`

## Deliverables

### Per-Strand Directories

```
.trinity/experience/dissertation/
├── strand-i/
│   ├── structure/        # Structure audit reports
│   ├── proofs/          # Proof verification results
│   ├── citations/       # Reference audit reports
│   ├── terminology/     # Term normalization tables
│   ├── verification/    # Appendix A verification artifacts
│   ├── continuity/      # Cross-strand maps
│   └── program.md       # Strand I program definition
├── strand-ii/
│   └── template.md      # Placeholder for Strand II
└── strand-iii/
    └── template.md      # Placeholder for Strand III
```

## Integration Points

### Existing Codebase Mappings

| Dissertation Concept | Codebase Path | Notes |
|------------------|---------------|-------|
| Trinity Identity | `specs/math/constants.t27` | TRINITY constant defined |
| Sacred Constants | `specs/math/constants.t27` | PHI, PI, E, etc. |
| Fixed-Point Theory | `specs/numeric/phi_ratio.t27` | φ-optimization logic |
| VSA Operations | `specs/vsa/vsa_core.t27` | bind, bundle, similarity |
| VSA Packed | `specs/vsa/packed_vsa.t27` | Trit encoding |
| GoldenFloat | `specs/numeric/goldenfloat_family.t27` | GF4-GF32 formats |
| GF16 | `specs/numeric/gf16.t27` | Primary ML format |
| Verification | `specs/physics/sacred_physics.t27` | Test framework |

### New Workflow Commands

| Command | Action |
|---------|--------|
| `tri verdict --dissertation-structure` | Structure audit |
| `tri verdict --proof-check` | Proof verification |
| `tri verdict --citation-audit` | Reference validation |
| `tri gen specs/dissertation/verification/*` | Create verification specs |
| `tri test specs/dissertation/verification/*` | Execute verification workflows |
| `tri verdict --terminology` | Normalize terms |
| `tri verdict --continuity` | Cross-strand mapping |

## Verification

After implementation, verify:

1. **Structure Audit**: Run `tri verdict --dissertation-structure` on Strand I → confirms RQ mapping, theorem flow
2. **Proof Verification**: Run `tri verdict --proof-check` → validates each proof step, flags gaps
3. **Verification Workflow**: Generate and run `specs/dissertation/verification/*.t27` → all pass
4. **Cross-Strand**: Templates for II/III created with Strand I references
5. **Experience Tracking**: All outputs captured in `.trinity/experience/` as JSONL
6. **Constitutional Compliance**: No new Python on critical path; all via tri/t27c

## Execution Order

1. Create `.trinity/experience/dissertation/` directory structure
2. Write `.trinity/experience/dissertation/strand-i/program.md`
3. Create `.trinity/dissertation_program.tri`
4. Create `.trinity/strand_i_workflow.tri`
5. Create `specs/dissertation/verification/trinity_identity.t27`
6. Create `specs/dissertation/verification/fixed_point_convergence.t27`
7. Create `docs/nona-03-manifest/DISSERTATION-WORKFLOW.md`
8. Create Strand II/III templates
9. (Optional) Extend `scripts/tri` or create `scripts/dissertation`

## Notes

- **Source of Truth**: All task specifications in `.trinity`, artifacts in `experience/`
- **Tri CLI Integration**: All workflows use existing `tri` infrastructure
- **Constitutional**: Follows TDD mandate (tests in specs), L1 (traceability via issues)
- **Existing Math**: `specs/math/constants.t27` already defines TRINITY constant
- **VSA Ready**: `specs/vsa/vsa_core.t27` provides bind/bundle/similarity

## Agent Execution Guidelines

When executing this workflow autonomously:

1. **Read First**: Always read `strand-i/program.md` before starting any task
2. **Verify Artifacts**: Check that output directories exist before writing
3. **Report Progress**: Write completion status to `.trinity/experience/dissertation/strand-i/` as JSON
4. **Handle Errors**: If a verification fails, log the error and continue with next task
5. **Constitutional Compliance**: Ensure all outputs follow L1-L8 laws
6. **Experience Tracking**: All actions should be traceable via `.trinity/experience/`

## Appendix: File Manifest

```
# Created Files
.trinity/experience/dissertation/strand-i/program.md
.trinity/experience/dissertation/strand-i/structure/README.md
.trinity/experience/dissertation/strand-i/proofs/README.md
.trinity/experience/dissertation/strand-i/citations/README.md
.trinity/experience/dissertation/strand-i/terminology/README.md
.trinity/experience/dissertation/strand-i/verification/README.md
.trinity/experience/dissertation/strand-i/continuity/README.md
.trinity/experience/dissertation/strand-ii/template.md
.trinity/experience/dissertation/strand-iii/template.md
.trinity/dissertation_program.tri
.trinity/strand_i_workflow.tri
specs/dissertation/verification/trinity_identity.t27
specs/dissertation/verification/fixed_point_convergence.t27
docs/nona-03-manifest/DISSERTATION-WORKFLOW.md
```

## References

- `AGENTS.md` — Agent execution guidelines
- `SOUL.md` — Canonical constitutional reference
- `docs/T27-CONSTITUTION.md` — L1-L8 laws
- `specs/math/constants.t27` — Sacred constants
- `specs/vsa/vsa_core.t27` — VSA operations
- `specs/numeric/goldenfloat_family.t27` — GoldenFloat formats
