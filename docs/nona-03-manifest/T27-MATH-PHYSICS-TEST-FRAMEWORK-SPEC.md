# t27 Mathematical & Physics Test Framework — Full Specification

**Status:** Normative charter (planning)
**Language:** English (repository **LANG-EN**)
**Date:** 2026-04-07

**Tracking:** Ring 47 P2 (issue #167) — inventory → test hooks
**Milestone:** `EPOCH-01-HARDEN` → `EPOCH-02-SCIENCE`
**Merge policy:** Issue Gate (`.github/workflows/issue-gate.yml`); every PR **Closes #N**.

**Law:** Root **`SOUL.md`** (no Cyrillic in source), **`docs/TDD-CONTRACT.md`** (each `.t27` spec must contain **`test`** or **`invariant`**), **`docs/ISSUE-GATE-001.md`**.

**Companions:** **`docs/nona-03-manifest/GOLDEN-CHAIN-TESTING-ATLAS.md`** (oracles), **`docs/nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md`** (rings / Rust vs `.t27` execution), **`docs/nona-03-manifest/RESEARCH_CLAIMS.md`** (claim tiers), **`docs/nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md`** + **`conformance/FORMAT-SPEC-001.json`** + **`conformance/axiom_system.json`** (axiom / format catalog).

---

## 1. Goals and context

t27's strength is **evidence-grade** practice inside a **ring** architecture. This specification defines a **test framework** that, once implemented, applies immediately to:

- **Math:** GoldenFloat (GF4–GF32), φ-ratio arithmetic, algebraic laws.
- **Physics:** sacred physics, CODATA references, dimensional consistency.
- **Brain / CLARA:** metamorphic consistency, recovery, conflict handling (later sprints).

The framework is **authored in `.t27`**, **exercised by the official toolchain** (`tri` / `t27c`), and produces **reproducible artifacts** suitable for publications—not verbal claims alone.

---

## 2. Ring-aware oracle strategy

Central rule: **oracle richness scales with ring maturity.**

| Ring phase | Oracle | Typical mechanism |
|------------|--------|-------------------|
| **0** — Rust seed | Build / no panic | `cargo test`, `cargo build` in `bootstrap/` |
| **1** — `.t27` parsed by Rust | Golden / AST shape | `t27c parse`, snapshot fixtures |
| **2** — `.t27` evaluates `.t27` | Reference + golden | Differential vs high-precision reference |
| **3** — codegen Zig/C/Verilog | Backend equivalence | Same corpus → outputs equal within tolerance |
| **4** — GF4–GF8 | Exhaustive + formal slice | GF4: full small Cartesian products; optional Kani |
| **5** — GF12–GF32 | PBT + reference | Differential vs mpmath / interval reference |
| **6** — sacred physics | CODATA + MR | Typed constants + dimensional metamorphic relations + **claim_tier** |
| **7+** — brain / CLARA | Metamorphic | Rephrase consistency, recovery, deterministic conflict resolution |

**t27 response to oracle gap:** **Double oracle** for novel math — **metamorphic relations** + **reference differential** (e.g. mpmath), per **`GOLDEN-CHAIN-TESTING-ATLAS.md`**.

---

## 3. Math suite requirements

### 3.1 GoldenFloat properties (mandatory targets)

For each GF width, verify (technique varies by domain size):

| Property | Metamorphic / algebraic idea | Technique | Oracle |
|----------|------------------------------|-----------|--------|
| Commutativity `+`, `*` | `a op b == b op a` | Exhaustive (GF4), PBT (GF8+) | Formal slice / reference |
| Associativity `+` | `(a+b)+c == a+(b+c)` | PBT | Reference |
| Additive identity | `a + 0 == a` | Exhaustive where cheap | Formal / golden |
| Distributivity | `a*(b+c)` vs `a*b + a*c` | PBT | Reference |
| Domain closure | result in representable set | Exhaustive / PBT | Kani slice / property |
| φ-metric sanity | non-negativity, axioms | PBT | Reference |
| Round-trip | `decode(encode(x)) == x` | Exhaustive small | Golden |
| Robustness | no panic on valid grammar | Fuzz parser | None (robustness) |

**GF4 exhaustive:** 16 values → **256** pairs per binary op.

### 3.2 Phi-ratio claims and `claim_tier`

Every φ-related statement in tests **MUST** align with **`RESEARCH_CLAIMS.md`** vocabulary:

- **exact** — algebraic identities (e.g. φ² = φ + 1 in a defined real model).
- **empirical** / **approximate** — fitted or numerical claims with stated tolerance.
- **conjectural** — explicit falsification path required in the claims table.

Physics/math tests without **`claim_tier`** (where the claim is scientific) are **not merge-ready**.

### 3.3 NMSE benchmark (Ring 034 / issue #129)

Target spec: compare **GF16** vs **bfloat16** vs **float16** on a fixed corpus; report **NMSE**; acceptance example: GF16 NMSE **≤** bfloat16 NMSE. Implement as **`bench`** blocks per **SOUL**.

---

## 4. Physics suite — sequential gate pipeline

**Pattern:** strict ordering with **short-circuit**:

```text
parse → type_check → semantic → numeric_stability → physics_constraint → audit
```

If gate **N** fails, gates **N+1…** are **not** evaluated. Encode as **invariants** on the pipeline spec.

### 4.1 CODATA 2022 reference oracle

Typed constants (illustrative values — **must** be verified against NIST CODATA tables at implementation time):

- Speed of light (exact definition in SI), Planck constant (defined), fine-structure inverse, mass ratios, etc.
- Each constant carries **`ClaimTier`**: `Exact` | `Empirical_CODATA_2022` | …

### 4.2 Dimensional metamorphic relations

Examples: homogeneous scaling of kinetic energy (`v → 2v` ⇒ `KE → 4×`); dimensional homogeneity invariants. Implement as **`test` / `invariant`** with **`ToleranceTier`** matching physics meaning.

---

## 5. CI integration — phased alignment

| Level | Intent | Target command / owner |
|-------|--------|----------------------|
<<<<<<< Updated upstream
| **L0** | NOW sync | `t27c check-now --repo-root .` |
| **L1** | Corpus suite | `./bootstrap/target/release/t27c suite --repo-root .` |
| **L2** | GF4 exhaustive / math PBT | `./bootstrap/target/release/t27c test <spec>` |
| **L3** | Rust unit / nextest | `cargo nextest` in `bootstrap/` |
| **L4** | Differential oracle | Hermetic harness vs mpmath |
=======
| **L0** | NOW sync | `t27c --repo-root . check-now` *(or `tri check-now`)* |
| **L1** | Corpus suite | `./bootstrap/target/release/t27c --repo-root . suite` *(present)* |
| **L2** | GF4 exhaustive / math PBT | `./bootstrap/target/release/t27c test <spec>` *(requires `tri test` / `t27c test`)* |
| **L3** | Rust unit / nextest | `cargo nextest` or `cargo test` in `bootstrap/` *(optional gate)* |
| **L4** | Differential oracle | Hermetic Python or Rust harness vs mpmath *(off critical path per SSOT-MATH policy)* |
>>>>>>> Stashed changes
| **L5** | Conformance v2 | `t27c validate-conformance` extensions |
| **L6** | Seal integrity | `seal --verify` with exit-code check |
| **L7** | Physics gate pipeline | `t27c test specs/physics/gate_pipeline.t27` |

**Rule:** When adding YAML, **each step must have a tracked issue** and must **fail closed** (no `grep`-piped false greens).

---

## 6. Sprint decomposition (issues to file)

### Sprint A — Core (Ring 050)
| ID | Title | Acceptance | Depends on |
|----|-------|------------|------------|
| A1 | `stdlib` / `specs/test_framework/core`: assert, for_all, metamorphic | `tri test specs/test_framework/core/runner.t27` → exit 0 | Parser + eval path |

### Sprint B — Math (Ring 051)
| ID | Title | Acceptance | Depends on |
|----|-------|------------|------------|
| B1 | GF4 exhaustive (1024 checks) | 0 failures | A |
| B2 | GF8 PBT 10k | 0 property violations | B1 |
| B3 | φ² = φ + 1 (algebraic) | Tier **exact** | B1 |
| B4 | NMSE benchmark (#129) | Report table | B2 |
| B5 | mpmath differential harness | JSON fixtures | B1 |

### Sprint C — Physics (Ring 052)
| ID | Title | Acceptance | Depends on |
|----|-------|------------|------------|
| C1 | CODATA constants + tiers | 6+ constants labeled | B |
| C2 | Dimensional MR set (≥ 5 formulas) | All pass | C1 |
| C3 | Sacred physics tolerances | Closes linked tolerance issue | C1 |
| C4 | **RESEARCH_CLAIMS.md** — no unlabeled scientific rows | Audit | C2 |
| C5 | Physics gate pipeline spec | Sequential short-circuit proven | C2 |

---

## 7. Required fields in every GitHub issue (this program)

```yaml
ring: [050 | 051 | 052 | 053 | 054]
language: [.t27 | rust | both]
test_type: [unit | snapshot | pbt | metamorphic | differential | exhaustive | formal | e2e]
oracle: [reference_mpmath | golden_snapshot | metamorphic_relation | kani_formal | codata_2022 | seal | mixed]
acceptance: |
  Exact command(s) + expected exit code / output.
claim_tier: [exact | empirical | approximate | conjectural | n/a]
closes: #NNN   # PR must use Closes #NNN per issue-gate.yml
```

---

## 8. Definition of Done — whole framework

The framework is **"Golden Chain complete"** only when:

- [ ] `tri test specs/test_framework/core/runner.t27` exits **0**.
- [ ] GF4: **1024** binary-op checks pass.
- [ ] GF8+: **10k** PBT trials, **0** property violations.
- [ ] mpmath differential: **0** divergences above tier tolerance.
- [ ] Physics gate pipeline: sequential short-circuit **verified**.
- [ ] CODATA block: constants loaded + **claim_tier** set.
- [ ] **`RESEARCH_CLAIMS.md`:** **0** unlabeled scientific claims.
- [ ] **`docs/NOW.md`** updated on each ring seal.
- [ ] arXiv / publication draft opened.

---

## 9. Cross-Links

<<<<<<< Updated upstream
- `docs/NUMERIC-STANDARD-001.md` — GoldenFloat family specification
- `docs/nona-02-organism/NUMERIC-GF16-DEBT-INVENTORY.md` — Numeric debt inventory (issue #167)
- `docs/TDD-CONTRACT.md` — TDD contract
- `docs/SOUL.md` — Constitution
- `conformance/FORMAT-SPEC-001.json` — Format SSOT
=======
The framework is **“Golden Chain complete”** only when:

- [ ] `tri test specs/test_framework/core/runner.t27` exits **0**.  
- [ ] GF4: **1024** binary-op checks (or agreed Cartesian product) pass.  
- [ ] GF8+: **10k** PBT trials, **0** property violations (per configured seed policy).  
- [ ] mpmath differential: **0** divergences above tier tolerance.  
- [ ] Physics gate pipeline: sequential short-circuit **verified** by tests.  
- [ ] CODATA block: constants loaded + **claim_tier** set.  
- [ ] Brain MR: consistency metric meets chartered threshold on fixed **N**.  
- [ ] Experience logs for rings **050–054** recorded under `.trinity/experience/` (or successor).  
- [ ] **`RESEARCH_CLAIMS.md`:** **0** unlabeled scientific claims.  
- [ ] **`NOW.md`** (repo root) updated on each ring seal (NOW sync policy).  
- [ ] arXiv / publication draft opened (e.g. #136).

---

## 13. Publication mapping (evaluation section)

| Evidence | Example paper row | Claim level |
|----------|-------------------|-------------|
| Kani / exhaustive GF4 | “All 1024 GF4 op pairs checked (bounded formal / exhaustive)” | Formal / exhaustive |
| PBT GF8 10k | “10⁴ random trials: no algebraic violation” | Empirical |
| mpmath differential | “Max deviation < ε for GF16 corpus” | Numerical |
| CODATA | “Constants within CODATA 2022 uncertainties” | Empirical |
| Brain consistency | “92% paraphrase consistency (N=100)” | Empirical |
| Parser fuzz | “0 panics / 10⁶ mutations” | Robustness |
| Backend diff | “34 conformance vectors: Zig ≡ C ≡ Verilog” | Equivalence |

---

## 14. References

1. [proptest (Rust)](https://github.com/proptest-rs/proptest)  
2. PBT survey context — use proptest / Hypothesis docs for methodology.  
3. [Kani model checker](https://github.com/model-checking/kani)  
4. [POPL 2026 — Creusot tutorial](https://popl26.sigplan.org/details/POPL-2026-tutorials/6/Creusot-Formal-verification-of-Rust-programs)  
5. [AeroTherm-style sequential validation (arXiv 2410.01981v1 HTML)](https://arxiv.org/html/2410.01981v1)  
6. [CODATA / NIST constants](https://physics.nist.gov/cuu/Constants/)  
7. [mpmath](https://mpmath.org/) — reference arithmetic (use only in allowed harnesses).  
>>>>>>> Stashed changes

---

**φ² + 1/φ² = 3 | TRINITY**
