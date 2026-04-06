# t27 Mathematical & Physics Test Framework — Full Specification

**Status:** Normative charter (planning)  
**Language:** English (repository **LANG-EN**)  
**Date:** 2026-04-06  

**Tracking:** Open a GitHub issue **“Ring 050: test framework core”** (or next free ring after 047–049 in your milestone plan). **Milestone:** `EPOCH-01-HARDEN` → `EPOCH-02-SCIENCE`. **Merge policy:** Issue Gate (`.github/workflows/issue-gate.yml`); every PR **Closes #N**.  

**Law:** Root **`SOUL.md`** (no Cyrillic in source), **`docs/nona-03-manifest/TDD-CONTRACT.md`** (each `.t27` spec must contain **`test`** or **`invariant`**), **`docs/nona-03-manifest/ISSUE-GATE-001.md`**.  

**Companions:** **`docs/nona-03-manifest/GOLDEN-CHAIN-TESTING-ATLAS.md`** (oracles), **`docs/nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md`** (rings / Rust vs `.t27` execution), **`docs/nona-03-manifest/RESEARCH_CLAIMS.md`** (claim tiers), **`docs/nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md`** + **`conformance/FORMAT-SPEC-001.json`** + **`conformance/axiom_system.json`** (axiom / format catalog).

---

## 1. Goals and context

t27’s strength is **evidence-grade** practice inside a **ring** architecture. This specification defines a **test framework** that, once implemented, applies immediately to:

- **Math:** GoldenFloat (GF4–GF32), φ-ratio arithmetic, algebraic laws.  
- **Physics:** sacred physics, CODATA references, dimensional consistency.  
- **Brain / CLARA:** metamorphic consistency, recovery, conflict handling (later sprints).

The framework is **authored in `.t27`**, **exercised by the official toolchain** (`tri` / `t27c`), and produces **reproducible artifacts** suitable for publications—not verbal claims alone.

---

## 2. Competitor analysis — strengths and gaps

### 2.1 Property-based testing (PBT)

| Framework | Strengths | Limits |
|-----------|-----------|--------|
| **proptest** (Rust) | Strategies, shrinking, constraints ([proptest-rs](https://github.com/proptest-rs/proptest)) | Slower than some QuickCheck variants; complex AST strategies are hard. |
| **QuickCheck** (Haskell / Rust ports) | Simple, fast, type-directed generation | Weak value-level constraints; high rejection on small domains. |
| **Hypothesis** (Python) | DB of failures, strong shrinking | Python-only; not on t27’s Rust critical path. |

**Shared gap — oracle problem:** For **novel** arithmetic (φ-structured GoldenFloat), PBT alone does not define “ground truth.”

### 2.2 Formal verification (Rust ecosystem)

| Tool | Method | Strengths | Limits |
|------|--------|-----------|--------|
| **Kani** | Bounded model checking | Exhaustive on **small** state spaces ([AWS Kani](https://github.com/model-checking/kani)) | State-space explosion; not a full production certification stack alone. |
| **Creusot** | Deductive / Why3 | Research-grade Rust proofs ([POPL 2026 tutorials](https://popl26.sigplan.org/details/POPL-2026-tutorials/6/Creusot-Formal-verification-of-Rust-programs)) | Learning curve; evolving ecosystem. |

**t27 stance:** Use **Kani (or similar)** for **tiny** domains (e.g. GF4 exhaustively). For GF32-scale, combine **PBT + reference oracle** (e.g. high-precision differential), not unbounded formal search.

### 2.3 Physics / numerics testing

| Pattern | Idea | Fit for t27 |
|---------|------|-------------|
| **Sequential gate validation** (AeroTherm-style workflow) | `unit → physics → numerics → execution → audit` ([arXiv HTML 2410.01981v1](https://arxiv.org/html/2410.01981v1)) | **Yes** — canonical shape for **sacred physics pipeline** gates. |
| **mpmath** | Arbitrary precision / intervals | **Yes** — **reference oracle** for GoldenFloat differential checks (off critical path or hermetic harness per **SSOT-MATH**). |
| **NMODL / NEURON** | DSL → AST → codegen | Inspiration for **codegen testing** patterns only. |

**Gap in the wild:** Few stacks combine **ring-aware** policy, **seals**, and **language self-testing** the way t27 targets.

### 2.4 Distinctive t27 advantages (target state)

- **TDD-CONTRACT** — tests live **inside** `.t27` specs ([`TDD-CONTRACT.md`](TDD-CONTRACT.md)).  
- **Seal system** — tamper-evident artifacts per ring.  
- **Ring-aware oracles** — strategy **depends on ring maturity** (see §4).  
- **Self-hosted tests** — tests in the **same language** under test (per bootstrap plan Ring 2+).

---

## 3. Framework architecture (target module layout)

Normative **future** tree under `specs/test_framework/`:

```text
specs/test_framework/
├── core/
│   ├── assert.t27          # assert_eq, assert_ne, assert_lt, assert_approx
│   ├── generator.t27       # for_all, pick, range, sample
│   ├── runner.t27          # it(), describe(), report()
│   ├── oracle.t27          # oracle_reference, oracle_golden, oracle_metamorphic
│   └── reporter.t27        # TAP / JSON, CI exit codes
├── math/
│   ├── pbt_math.t27
│   ├── exhaustive.t27      # finite domains (GF4, GF8)
│   ├── approx.t27        # epsilon, ULP, NMSE
│   └── metamorphic.t27
├── physics/
│   ├── dimensional.t27
│   ├── codata_oracle.t27
│   ├── tolerance.t27
│   └── gate_pipeline.t27
└── brain/
    ├── consistency.t27
    ├── recovery.t27
    └── conflict.t27
```

**Note:** API below is a **normative target**. Concrete syntax must match the **live** t27 grammar and compiler; evolve this document when the parser and `tri test` land.

---

## 4. Core API (normative sketch)

Illustrative **intent** (not guaranteed to parse today):

```t27
-- Target: specs/test_framework/core/runner.t27
-- Refs: Closes #<RING_050_ISSUE>

spec TestRunner {
    fn assert_eq(label: Str, expected: Any, got: Any) -> TestResult
    fn assert_approx(label: Str, expected: F64, got: F64, tier: ToleranceTier) -> TestResult
    fn for_all(label: Str, gen: Generator, count: U32, property: Fn(Any) -> Bool) -> TestResult
    fn metamorphic(label: Str, input: Any, transform: Fn(Any) -> Any,
                   compute: Fn(Any) -> Any, relation: Fn(Any, Any) -> Bool) -> TestResult
    fn gate_pipeline(subject: Any, gates: List[Gate]) -> PipelineResult

    test "runner_self_test" {
        assert_eq("trivial", 1 + 1, 2)
    }

    invariant "runner_reports_failures" {
        -- failing assertion yields Fail
    }
}
```

Every shipped module **MUST** satisfy **TDD-CONTRACT**: at least one **`test`** or **`invariant`** block per `.t27` file.

---

## 5. Ring-aware oracle strategy

Central rule: **oracle richness scales with ring maturity.**

| Ring phase | Oracle | Typical mechanism |
|------------|--------|-------------------|
| **0** — Rust seed | Build / no panic | `cargo test`, `cargo build` in `bootstrap/` |
| **1** — `.t27` parsed by Rust | Golden / AST shape | `t27c parse`, snapshot fixtures |
| **2** — `.t27` evaluates `.t27` | Reference + golden | Differential vs high-precision reference (policy-compliant harness) |
| **3** — codegen Zig/C/Verilog | Backend equivalence | Same corpus → outputs equal within tolerance |
| **4** — GF4–GF8 | Exhaustive + formal slice | GF4: full small Cartesian products; optional Kani on extracted Rust |
| **5** — GF12–GF32 | PBT + reference | Differential vs mpmath / interval reference |
| **6** — sacred physics | CODATA + MR | Typed constants + dimensional metamorphic relations + **claim_tier** |
| **7+** — brain / CLARA | Metamorphic | Rephrase consistency, recovery, deterministic conflict resolution |

**t27 response to oracle gap:** **Double oracle** for novel math — **metamorphic relations** + **reference differential** (e.g. mpmath), per **`GOLDEN-CHAIN-TESTING-ATLAS.md`**.

---

## 6. Math suite requirements

### 6.1 GoldenFloat properties (mandatory targets)

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

**GF4 exhaustive:** 16 values → **256** pairs per binary op; **four** ops → **1024** checks is an explicit **Ring 051** acceptance target (before merge policy is tightened).

### 6.2 Phi-ratio claims and `claim_tier`

Every φ-related statement in tests **MUST** align with **`RESEARCH_CLAIMS.md`** vocabulary:

- **exact** — algebraic identities (e.g. φ² = φ + 1 in a defined real model).  
- **empirical** / **approximate** — fitted or numerical claims with stated tolerance.  
- **conjectural** — explicit falsification path required in the claims table.

Physics/math tests without **`claim_tier`** (where the claim is scientific) are **not merge-ready** once Ring 052 policy is active.

### 6.3 NMSE benchmark (Ring 034 / issue #129)

Target spec: compare **GF16** vs **bfloat16** vs **float16** on a fixed corpus; report **NMSE**; acceptance example: GF16 NMSE **≤** bfloat16 NMSE (exact inequality subject to charter). Implement as **`bench`** blocks per **SOUL** when the runner supports them.

---

## 7. Physics suite — sequential gate pipeline

**Pattern (AeroTherm-style):** strict ordering with **short-circuit**:

```text
parse → type_check → semantic → numeric_stability → physics_constraint → audit
```

If gate **N** fails, gates **N+1…** are **not** evaluated (undefined downstream). Encode as **invariants** on the pipeline spec once `gate_pipeline` exists.

### 7.1 CODATA 2022 reference oracle

Typed constants (illustrative values — **must** be verified against NIST CODATA tables at implementation time):

- Speed of light (exact definition in SI), Planck constant (defined), fine-structure inverse, mass ratios, etc.  
- Each constant carries **`ClaimTier`**: `Exact` | `Empirical_CODATA_2022` | …

### 7.2 Dimensional metamorphic relations

Examples: homogeneous scaling of kinetic energy (`v → 2v` ⇒ `KE → 4×`); dimensional homogeneity invariants. Implement as **`test` / `invariant`** with **`ToleranceTier`** matching physics meaning.

---

## 8. Brain / CLARA suite (Sprint D)

Target behaviors:

- **Consistency MR:** `query(Q)` vs `query(rephrase(Q))` — semantic equivalence predicate (may be statistical in early rings).  
- **Recovery MR:** `inject_noise` then `recover` restores declared invariants.  
- **Conflict MR:** same conflict → same resolution (deterministic oracle).

Acceptance example: **≥ 90%** consistency on **N = 100** controlled paraphrase pairs (empirical claim — must appear in **`RESEARCH_CLAIMS.md`**).

---

## 9. CI integration — phased alignment

**Today (repository fact):** `phi-loop-ci.yml` runs **NOW gate**, **`t27c suite`**, **validate-conformance**, **validate-gen-headers**, seal counts, **English-first** doc check.

**Target ladder (add steps only when commands exist — do not resurrect false-green shell harnesses):**

| Level | Intent | Target command / owner |
|-------|--------|----------------------|
| **L0** | NOW sync | `bash scripts/check-now-sync.sh` *(present)* |
| **L1** | Corpus suite | `./bootstrap/target/release/t27c suite --repo-root .` *(present)* |
| **L2** | GF4 exhaustive / math PBT | `./bootstrap/target/release/t27c test <spec>` *(requires `tri test` / `t27c test`)* |
| **L3** | Rust unit / nextest | `cargo nextest` or `cargo test` in `bootstrap/` *(optional gate)* |
| **L4** | Differential oracle | Hermetic Python or Rust harness vs mpmath *(off critical path per SSOT-MATH policy)* |
| **L5** | Conformance v2 | `t27c validate-conformance` extensions |
| **L6** | Seal integrity | Subprocess `seal --verify` with exit-code check *(already in suite philosophy)* |
| **L7** | Physics gate pipeline | `t27c test specs/physics/gate_pipeline.t27` *(future)* |

**Rule:** When adding YAML, **each step must have a tracked issue** and must **fail closed** (no `grep`-piped false greens).

---

## 10. Sprint decomposition (issues to file)

### Sprint A — Core (Ring 050) — blocker

| ID | Title | Acceptance (example) | Depends on |
|----|-------|----------------------|------------|
| A1 | `stdlib` / `specs/test_framework/core`: assert, for_all, metamorphic | `tri test specs/test_framework/core/runner.t27` → exit 0 | Parser + eval path for tests (see bootstrap plan **Ring 1 / 2**) |
| A2 | `ToleranceTier` + `ClaimTier` enums | Parses + codegen smoke | A1 |
| A3 | Sequential `gate_pipeline` with short-circuit | Invariant tests pass | A1 |
| A4 | TAP + JSON reporter | CI-parseable | A1 |
| A5 | **`tri test <spec.t27>`** wired to `t27c` | Documented in `scripts/tri` | A1 |

### Sprint B — Math (Ring 051)

| ID | Title | Acceptance | Depends on |
|----|-------|------------|------------|
| B1 | GF4 exhaustive (1024 checks) | 0 failures | A |
| B2 | GF8 PBT 10k | 0 property violations | B1 |
| B3 | φ² = φ + 1 (algebraic) | Tier **exact** | B1 |
| B4 | NMSE benchmark (#129) | Report table | B2 |
| B5 | mpmath differential harness | JSON fixtures | B1 |
| B6 | Parser fuzz | No panic 1M mutants | A |

### Sprint C — Physics (Ring 052)

| ID | Title | Acceptance | Depends on |
|----|-------|------------|------------|
| C1 | CODATA constants + tiers | 6+ constants labeled | B |
| C2 | Dimensional MR set (≥ 5 formulas) | All pass | C1 |
| C3 | Sacred physics tolerances | Closes linked tolerance issue (e.g. #145) | C1 |
| C4 | **RESEARCH_CLAIMS.md** — no unlabeled scientific rows | Audit | C2 |
| C5 | Physics gate pipeline spec | Sequential short-circuit proven | C2 |

### Sprint D — Brain (Ring 053)

Consistency, recovery, conflict, CLARA TA1 metrics — as in §8.

### Sprint E — Publication (Ring 054)

Experience logs, claims table v2, evaluation LaTeX, arXiv draft (e.g. #136).

---

## 11. Required fields in every GitHub issue (this program)

Paste into the issue body (aligns with **Issue Gate** + **`bootstrap-testing`** template):

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

**No issue → no merge** (existing workflow). **No `claim_tier`** on scientific physics/math tests → merge blocked once Ring 052 policy is enabled.

---

## 12. Definition of Done — whole framework

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
- [ ] **`docs/NOW.md`** updated on each ring seal (NOW sync policy).  
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

---

**φ² + 1/φ² = 3 | TRINITY**
