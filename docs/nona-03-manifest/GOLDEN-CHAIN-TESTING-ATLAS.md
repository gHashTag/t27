# Golden Chain Testing Atlas

**Status:** Charter (2026-04-06)  
**Language:** English (repository policy)  
**Companion:** **`docs/nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md`** (rings, issue spine, “when tests live in `.t27`”)

This atlas names **oracles**, **metamorphic** and **differential** strategies, and how they map onto t27’s **Rust seed → Ring 1 fixtures → Ring 2 self-eval** path. It is governance for *what* to test, not a duplicate of the ring schedule.

---

## 1) The test oracle problem

Every automated check answers: *how do we know the output is right?*

| Oracle | Meaning | Typical cost |
|--------|---------|--------------|
| **Reference** | Compare to a trusted implementation (another library, CAS, physics tables). | High setup; drift if reference changes. |
| **Golden / snapshot** | Freeze expected output; fail on diff. | Fast; can hide semantic regressions if goldens are wrong. |
| **Metamorphic relation** | Same program under a transformation; output must relate predictably (e.g. commutativity, scale invariance). | No single “right” number; catches large classes of bugs. |
| **Formal** | Proof or exhaustive enumeration over a small domain. | Expensive; strongest when domain is finite or axiomatized. |
| **Seal** | Cryptographic / manifest binding of artifact to policy (t27 `seal` family). | Integrity and policy, not full functional correctness alone. |

**Rule of thumb:** combine at least two families (e.g. golden + metamorphic, or reference + seal) before treating a subsystem as “trusted.”

---

## 2) Framework stack (conceptual ladder)

Ordered from cheapest, most local checks to heaviest integration. Not every ring needs every layer; **Ring 0** emphasizes the bottom; **Ring 3+** grows the top.

1. **Unit** — single function / module, mocked boundaries.  
2. **Snapshot** — AST, diagnostics, codegen text (stable fixtures).  
3. **Integration** — parser + lower + gen for a small program.  
4. **Property-based (PBT)** — generators + invariants (Rust `proptest` / custom).  
5. **Metamorphic** — relations over inputs/outputs (often pairs with PBT).  
6. **Differential** — two engines (e.g. seed eval vs high-precision reference) on same input.  
7. **Formal / exhaustive** — small finite domains, or future proof assistants (policy TBD).  
8. **E2E / ring suite** — `t27c suite`, conformance vectors, full corpus smoke.  
9. **Experience / artifact log** — `.trinity/experience/` (or successor): limits, failures, reproduction — *not* a substitute for (1–8).

**Bottleneck table (typical):**

| Bottleneck | Symptom | Mitigation |
|------------|---------|------------|
| Weak oracle | Green tests, wrong semantics | Add metamorphic or reference layer |
| Flaky goldens | Noise in codegen/diagnostics | Narrow snapshot surface; stable flags |
| Slow E2E | Developers skip full suite | Keep ring suites fast; shard CI |
| Brain / LLM paths | Non-determinism | Contract tests on *interfaces*; metamorphic paraphrase where defined |

---

## 3) Differential testing (numeric / physics)

Where t27 expresses **numeric or physical** claims:

- **High-precision reference:** arbitrary-precision or interval arithmetic in Rust (or external CAS) for a *subset* of expressions — differential check against the seed evaluator when Ring 1+ exposes `run` / eval paths.  
- **CODATA / published constants:** compare documented φ-related or physical ratios to tabulated values *with explicit uncertainty* where the spec demands it — see **`docs/nona-02-organism/NUMERIC-STANDARD-001.md`** and domain specs under `specs/`.

Differential tests are **reference-oracle** tests; they should be **narrow** (vectors, not whole language) to stay maintainable.

---

## 4) Metamorphic testing and “brain” surfaces

For **Trinity brain** (`specs/brain/` SSOT, per **`docs/nona-01-foundation/TRINITY-BRAIN-NEUROANATOMY-TZ.md`**):

- **Metamorphic relations** are often the right oracle when there is no single golden “thought”: e.g. reordering independent retrieval clauses, paraphrase of prompts into equivalent constraints (where the spec defines equivalence), idempotence of consolidation steps.  
- **Coverage of metamorphic relations** can be tracked as a *policy target* (which relations are required for a release), not only line coverage — aligns with Ring 2 / Ring 3 milestones in the bootstrap plan.

Charter-level: “brain reasonableness” should be **falsifiable** (see **`docs/nona-03-manifest/MULTI-MODEL-TRUST-CHAIN-ANALYSIS.md`**).

---

## 5) Issue-first CI and traceability

- **Pull requests:** `.github/workflows/issue-gate.yml` enforces linked issues / `Closes #N` — treat this as the **default** gate for substantive merges.  
- **Optional per-commit hooks** (require issue id in every commit message) are a **separate** policy item; track as its own issue if adopted.

When opening issues, use the **Bootstrap testing** template (recommended fields mirror **`docs/nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md`** § Issue template).

---

## 6) Sprint spine (where to look)

Concrete **ring / issue numbering** and exit criteria: **`docs/nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md`**.  
Ring freeze mechanics and oak layers: **`docs/nona-01-foundation/SEED-RINGS.md`**, **`docs/nona-01-foundation/GOLDEN-RINGS-CANON.md`**.

---

**φ² + 1/φ² = 3 | TRINITY**
