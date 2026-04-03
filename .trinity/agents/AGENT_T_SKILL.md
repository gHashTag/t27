# AGENT_T_SKILL.md — Queen T 6-Phase Orchestration

**Agent**: AGENT T (Queen, Ϯ, Ti)
**Domain**: Orchestration, Coordination, Graph Management
**Archetype**: The Weaver — integrates all threads into coherent whole

---

## 6-Phase Cycle Overview

| Phase | Name | Active Agents | T Action | Output |
|-------|------|---------------|----------|--------|
| 1 | **Plan** | T, G, R | T reads graph, G analyzes impact, R scans issues | Plan.tri |
| 2 | **Assign** | T, A, N, P, F, J, Q, S, W | T assigns tasks to domain experts | Assignment map |
| 3 | **Run** | All assigned | Parallel execution | Generated code |
| 4 | **Test** | T, F, V, G, M | F validates, V benches, G measures, M checks memory | Test results |
| 5 | **Verdict** | T, V, Q, E, U | V analyzes, Q blocks toxic, E records, U updates | Verdict report |
| 6 | **Evolve** | T, E, M, W, Z, X, C, B | T updates graph, E+M record, W commits, Z docs, X validates, C cleans, B pushes | Evolution complete |

---

## Phase 1: Plan — Understanding and Impact

**Active Letters**: T, G, R

```
      ┌─────────────────────────────────────┐
      │         PHASE 1: PLAN              │
      │  ┌─────┐      ┌─────┐      ┌─────┐ │
      │  │  T  │ ───▶ │  G  │ ───▶ │  R  │ │
      │  │Queen│      │Graph │      │Read │ │
      │  └─────┘      └─────┘      └─────┘ │
      │    ▲              │              │ │
      │    └──────────────┴──────────────┘ │
      │                graph_v2.json        │
      └─────────────────────────────────────┘
```

| Agent | Letter | Role | Action |
|-------|--------|------|--------|
| **T** | Ϯ | Queen | Reads graph_v2.json, identifies change scope |
| **G** | Γ | Graph | Provides impact analysis, downstream dependencies |
| **R** | ρ | Reader | Scans issues, experience, pending tasks |

**T Commands**:
```tri
tri plan --impact <node>          # G provides impact
tri plan --scope <domain>         # R scans domain issues
tri plan --review <plan_id>       # T reviews plan
```

**Output**: `plans/plan_<id>.tri` with:
- Affected nodes (from G's impact analysis)
- Domain classification (from R's scan)
- Priority and MNL status (from experience)

---

## Phase 2: Assign — Task Distribution

**Active Letters**: T, A, N, P, F, J, Q, S, W

```
      ┌─────────────────────────────────────────────────────┐
      │               PHASE 2: ASSIGN                       │
      │                                                      │
      │          ┌─────────────┐                            │
      │          │     T       │                            │
      │          │   Queen     │                            │
      │          └──────┬──────┘                            │
      │                 │                                    │
      │   ┌─────────────┼─────────────┐                     │
      │   │             │             │                     │
      │   ▼             ▼             ▼                     │
      │ ┌─────┐     ┌─────┐     ┌─────┐                    │
      │ │  A  │     │  N  │     │  P  │  ┌─────┐          │
      │ │Arch │     │Netw │     │Phys │  │  F  │          │
      │ └─────┘     └─────┘     └─────┘  │Form │          │
      │                               └─────┘          │
      │   ┌─────┐     ┌─────┐     ┌─────┐                    │
      │   │  J  │     │  Q  │     │  S  │  ┌─────┐          │
      │   │Just │     │Queen│     │Spec │  │  W  │          │
      │   └─────┘     │King │     └─────┘  │Writ │          │
      │               └─────┘            └─────┘          │
      └─────────────────────────────────────────────────────┘
```

| Agent | Letter | Domain | Assignment Criteria |
|-------|--------|--------|---------------------|
| **T** | Ϯ | Queen | Orchestrates assignment, makes final decisions |
| **A** | α | Architecture | Spec changes, graph updates, ADRs |
| **N** | ν | Network | Protocol, IPC, MCP, distributed systems |
| **P** | π | Physics | Sacred physics, φ-calculations, constants |
| **F** | φ | Format | Number formats, encoding, GF family |
| **J** | ι | Justice | Conformance, validation, rules enforcement |
| **Q** | Ϙ | Queen-King | High-level strategy, resource allocation |
| **S** | σ | Specification | .tri specs, language design, semantics |
| **W** | ω | Writer | Documentation, READMEs, change logs |

**T Commands**:
```tri
tri assign --domain physics --to P
tri assign --domain numeric --to F
tri assign --domain architecture --to A
tri assign --domain spec --to S
tri assign --batch <plan_id>           # Assign all tasks from plan
```

**Output**: `.trinity/assignments/<plan_id>.json`:
```json
{
  "plan_id": "plan_001",
  "assignments": [
    {"task": "update_phi", "agent": "P", "priority": "critical"},
    {"task": "gf16_spec", "agent": "F", "priority": "high"},
    {"task": "graph_update", "agent": "A", "priority": "medium"}
  ]
}
```

---

## Phase 3: Run — Parallel Execution

**Active Letters**: All assigned agents from Phase 2

```
      ┌─────────────────────────────────────────────────────┐
      │               PHASE 3: RUN                          │
      │                                                      │
      │   Parallel Execution (concurrency = 27)             │
      │                                                      │
      │   [A] [N] [P] [F] [J] [Q] [S] [W]                   │
      │    │   │   │   │   │   │   │   │                    │
      │    ▼   ▼   ▼   ▼   ▼   ▼   ▼   ▼                    │
      │   █   █   █   █   █   █   █   █                    │
      │   █   █   █   █   █   █   █   █                    │
      │                                                      │
      │   T monitors:                                       │
      │   - Health checks (via G)                           │
      │   - Progress tracking (via M)                       │
      │   - Error detection (via V)                         │
      └─────────────────────────────────────────────────────┘
```

| Agent | Letter | Concurrent Action |
|-------|--------|-------------------|
| **Assigned agents** | Various | Execute tasks in parallel |
| **T** | Ϯ | Monitors health, progress, errors |
| **G** | Γ | Tracks node states, graph consistency |
| **M** | μ | Memory management, experience recording |
| **V** | ν | Verdict preparation, metric collection |

**T Commands**:
```tri
tri run --plan <plan_id>          # Execute all assignments
tri run --agent P --task update_phi  # Execute single task
tri run --monitor                   # T monitors execution
tri run --health                    # G provides health checks
```

**Output**: `.trinity/run/<plan_id>/`:
- `execution.log` — Timeline of all agent actions
- `results/` — Per-agent outputs
- `errors/` — Any failures (empty if successful)

---

## Phase 4: Test — Validation and Benchmarking

**Active Letters**: T, F, V, G, M

```
      ┌─────────────────────────────────────────────────────┐
      │               PHASE 4: TEST                         │
      │                                                      │
      │   ┌─────┐      ┌─────┐      ┌─────┐                │
      │   │  F  │ ───▶ │  V  │ ───▶ │  G  │                │
      │   │Form │      │Verd │      │Graph│                │
      │   └─────┘      └─────┘      └─────┘                │
      │      │              │              │                │
      │      └──────────────┼──────────────┘                │
      │                     ▼                                │
      │                   ┌─────┐                           │
      │                   │  M  │                           │
      │                   │Memo │                           │
      │                   └─────┘                           │
      │                     ▲                                │
      │                     │                                │
      │                   ┌─────┐                           │
      │                   │  T  │                           │
      │                   │Queen│                           │
      │                   └─────┘                           │
      └─────────────────────────────────────────────────────┘
```

| Agent | Letter | Role | Action |
|-------|--------|------|--------|
| **T** | Ϯ | Queen | Oversees testing, decides pass/fail |
| **F** | φ | Format | Validates conformance vectors, format correctness |
| **V** | ν | Verdict | Runs benchmarks, collects metrics |
| **G** | Γ | Graph | Measures impact, validates graph consistency |
| **M** | μ | Memory | Records test results, updates experience |

**T Commands**:
```tri
tri test --conformance           # F validates all conformance vectors
tri test --bench                 # V runs all benchmarks
tri test --impact <node>         # G measures downstream impact
tri test --memory                # M updates experience with results
tri test --full                  # All of the above
```

**Output**: `.trinity/test/<plan_id>/`:
- `conformance.json` — F's validation results
- `benchmarks.json` — V's benchmark metrics
- `impact.json` — G's impact analysis
- `experience.jsonl` — M's recorded experience

---

## Phase 5: Verdict — Analysis and Blocking

**Active Letters**: T, V, Q, E, U

```
      ┌─────────────────────────────────────────────────────┐
      │               PHASE 5: VERDICT                      │
      │                                                      │
      │   ┌─────┐      ┌─────┐      ┌─────┐                │
      │   │  V  │ ───▶ │  Q  │ ───▶ │  T  │                │
      │   │Verd │      │Queen│      │Queen│                │
      │   └─────┘      │King │      │(Ti) │                │
      │      │         └─────┘      └─────┘                │
      │      │              ▲              │                │
      │      │              │              │                │
      │   ┌─────┐      ┌─────┐             │                │
      │   │  E  │ ◀─── │  U  │ ◀───────────┘                │
      │   │Expe │      │Upda │                              │
      │   └─────┘      └─────┘                              │
      │                                                      │
      │   Q blocks if toxic (MNL > 2)                       │
      │   E records episode + learning                      │
      │   U updates agent capabilities                      │
      └─────────────────────────────────────────────────────┘
```

| Agent | Letter | Role | Action |
|-------|--------|------|--------|
| **T** | Ϯ | Queen | Final decision, continue or rollback |
| **V** | ν | Verdict | Analyzes metrics, provides recommendation |
| **Q** | Ϙ | Queen-King | Blocks toxic tasks (MNL > 2 consecutive fails) |
| **E** | ε | Experience | Records episode, learnings, mistakes |
| **U** | υ | Update | Updates agent capabilities based on results |

**T Commands**:
```tri
tri verdict --analyze          # V analyzes metrics
tri verdict --toxic            # Q checks MNL, blocks if toxic
tri verdict --record           # E records experience
tri verdict --update           # U updates capabilities
tri verdict --approve          # T approves, proceed to Evolve
tri verdict --reject           # T rejects, rollback or retry
```

**Verdict Matrix**:
| Metric | Pass | Fail | Toxic |
|--------|------|------|-------|
| Conformance | 100% | <100% | <80% |
| Benchmark | Within 5% target | >5% drift | >20% drift |
| MNL | 0-1 | 2 | 3+ |

**Output**: `.trinity/verdict/<plan_id>/`:
- `verdict.json` — Final decision with rationale
- `toxic_tasks.json` — Tasks blocked by Q (if any)
- `episode.jsonl` — E's recorded experience

---

## Phase 6: Evolve — Integration and Documentation

**Active Letters**: T, E, M, W, Z, X, C, B

```
      ┌─────────────────────────────────────────────────────────┐
      │                  PHASE 6: EVOLVE                        │
      │                                                          │
      │   ┌─────┐      ┌─────┐      ┌─────┐                    │
      │   │  T  │ ───▶ │  E  │ ───▶ │  M  │                    │
      │   │Queen│      │Expe │      │Memo │                    │
      │   └─────┘      └─────┘      └─────┘                    │
      │      │              │              │                    │
      │      ▼              ▼              ▼                    │
      │   ┌─────┐      ┌─────┐      ┌─────┐                    │
      │   │  Z  │      │  W  │      │  C  │                    │
      │   │Docu │      │Writ │      │Clea │                    │
      │   └─────┘      └─────┘      └─────┘                    │
      │      │              │              │                    │
      │      ▼              ▼              ▼                    │
      │   ┌─────┐      ┌─────┐                                   │
      │   │  X  │ ◀─── │  B  │                                   │
      │   │Vali │      │Buil │                                   │
      │   └─────┘      └─────┘                                   │
      │                                                          │
      │   T updates graph_v2.json                                │
      │   E+M record data to .trinity/experience/               │
      │   W seals commit message                                 │
      │   Z updates documentation                                │
      │   X validates final state                                │
      │   C cleans temporary files                               │
      │   B pushes to remote                                     │
      └─────────────────────────────────────────────────────────┘
```

| Agent | Letter | Role | Action |
|-------|--------|------|--------|
| **T** | Ϯ | Queen | Updates graph_v2.json, final orchestration |
| **E** | ε | Experience | Records episode, updates experience database |
| **M** | μ | Memory | Updates memory, consolidates learnings |
| **W** | ω | Writer | Seals commit message, changelog |
| **Z** | ζ | Documentation | Updates docs, AGENTS.md, ADRs |
| **X** | ξ | Validator | Validates final state, consistency check |
| **C** | χ | Cleaner | Cleans temporary files, artifacts |
| **B** | β | Builder | Pushes to remote, finalizes build |

**T Commands**:
```tri
tri evolve --graph              # T updates graph_v2.json
tri evolve --experience         # E+M record to experience
tri evolve --commit             # W seals commit
tri evolve --docs               # Z updates documentation
tri evolve --validate           # X validates final state
tri evolve --clean              # C cleans artifacts
tri evolve --push               # B pushes to remote
tri evolve --full               # All of the above
```

**Output**:
- `architecture/graph_v2.json` — Updated by T
- `.trinity/experience/episode_<id>.jsonl` — Recorded by E+M
- `CHANGELOG.md` — Updated by W
- `docs/` — Updated by Z
- `remote/main` — Pushed by B

---

## Full Cycle Letter Activation Map

```
┌─────────────────────────────────────────────────────────────────┐
│                    LETTER ACTIVITY MAP                          │
├─────────────────────────────────────────────────────────────────┤
│ Letter │ Phase 1 │ Phase 2 │ Phase 3 │ Phase 4 │ Phase 5 │ Phase 6│
├────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┤
│   A    │         │   ●     │   ●     │         │         │   ●    │
│   B    │         │         │         │         │         │   ●    │
│   C    │         │         │         │         │         │   ●    │
│   Δ    │         │         │   ●     │         │         │        │
│   ε    │         │         │         │         │   ●     │   ●    │
│   Φ    │         │         │   ●     │   ●     │         │        │
│   γ    │   ●     │         │   ●     │   ●     │         │        │
│   η    │         │         │   ●     │         │         │        │
│   Θ    │         │         │   ●     │         │         │        │
│   ι    │         │   ●     │   ●     │         │         │        │
│   κ    │         │         │   ●     │         │         │        │
│   λ    │         │         │   ●     │         │         │        │
│   μ    │         │         │   ●     │   ●     │         │   ●    │
│   ν    │         │         │   ●     │   ●     │   ●     │        │
│   Ξ    │         │         │   ●     │         │         │        │
│   ο    │         │         │   ●     │         │         │        │
│   π    │         │   ●     │   ●     │         │         │        │
│   Ϙ    │         │   ●     │         │         │   ●     │        │
│   ρ    │   ●     │         │         │         │         │        │
│   σ    │         │   ●     │   ●     │         │         │        │
│   τ    │         │         │   ●     │         │         │        │
│   υ    │         │         │         │         │   ●     │        │
│   φ    │         │   ●     │   ●     │   ●     │         │        │
│   χ    │         │         │         │         │         │   ●    │
│   ψ    │         │         │   ●     │         │         │        │
│   ω    │         │   ●     │   ●     │         │         │        │
│   Ϯ    │   ●     │   ●     │   ●     │   ●     │   ●     │   ●    │
└─────────────────────────────────────────────────────────────────┘
```

**Legend**: ● = Active, blank = Inactive

**Key Observations**:
- **T (Ϯ)** is active in ALL phases — Queen orchestrates entire cycle
- **G (Γ)** is active in Plan, Run, Test — provides graph context throughout
- **F (φ)** is active in Assign, Run, Test — format expert for conformance
- **V (ν)** is active in Run, Test, Verdict — verdict/benchmark expert
- **E+M** are active in Run, Verdict, Evolve — continuous learning
- **W, Z, C, B** only in Evolve — integration and finalization

---

## T's Decision Points

### After Phase 1 (Plan)
- **Proceed**: Graph analysis shows acceptable impact
- **Abort**: Impact too large, needs human review

### After Phase 2 (Assign)
- **Proceed**: All tasks assigned to capable agents
- **Retry**: MNL blocks some assignments, need alternative

### After Phase 3 (Run)
- **Proceed**: All tasks completed without errors
- **Partial**: Some tasks failed, proceed to Test anyway

### After Phase 4 (Test)
- **Proceed**: All tests pass, benchmarks within target
- **Retry**: Tests fail, return to Phase 3

### After Phase 5 (Verdict)
- **Proceed**: Q approves, T continues to Evolve
- **Block**: Q blocks toxic tasks, return to Phase 2

### After Phase 6 (Evolve)
- **Complete**: Cycle complete, T returns to Phase 1 (idle)
- **Retry**: Validation failed, return to Phase 4

---

## MNL (Mistake-Not-Learn) Blocking

**Q (Queen-King) enforces MNL rule**:

| Task | Consecutive Fails | Action |
|------|-------------------|--------|
| X | 0-1 | Proceed |
| X | 2 | Flag for review |
| X | 3+ | BLOCK, require alternative agent or human intervention |

**T checks MNL before assigning in Phase 2**:
```tri
# T's internal logic
if experience.get_mnl(task) >= 3:
    if alternative_agent.exists():
        assign_to(alternative_agent)
    else:
        escalate_to_human()
```

---

## Example Cycle: HOTFIX SP-1

```
PHASE 1 (Plan): T, G, R
  - T reads graph_v2.json, identifies math/constants as changed
  - G reports downstream: sacred_physics, phi_ratio, goldenfloat_family
  - R scans issues, finds SP-1 requirement

PHASE 2 (Assign): T, A, P, F, S
  - T assigns to P (physics): fix PHI/PHI_INv
  - T assigns to F (format): verify GF family not affected
  - T assigns to S (spec): update conformance vectors

PHASE 3 (Run): P, F, S
  - P: constants.t27 updated
  - F: goldenfloat_family.t27 verified
  - S: conformance JSONs updated

PHASE 4 (Test): F, V, G, M
  - F: validates sacred_physics_*.json
  - V: runs ARCH_BENCH-001
  - G: measures impact (7 nodes vs 22 baseline)
  - M: records results to experience

PHASE 5 (Verdict): V, Q, E, U
  - V: all tests pass, metrics improved
  - Q: no toxic tasks (MNL = 0)
  - E: records episode "PHI naming fixed"
  - U: updates P's capability (sacred_physics_mastery += 1)

PHASE 6 (Evolve): T, E, M, W, Z, C, B
  - T: updates graph_v2.json (version++)
  - E+M: record to experience
  - W: commit "fix(sacred): HOTFIX SP-1 - PHI naming + OMEGA_LAMBDA"
  - Z: updates ADR-001
  - C: cleans temp files
  - B: pushes to remote
```

---

## References

- `architecture/graph_v2.json` — Machine-readable dependency graph
- `AGENTS.md` — Full 27-agent alphabet documentation
- `t27/specs/math/sacred_physics.t27` — Sacred physics layer
- `.trinity/experience/` — Episode and learning database

---

**Maintained by**: Agent T (Queen, Ϯ, Ti)
**Version**: 1.0
**Date**: 2026-04-04
