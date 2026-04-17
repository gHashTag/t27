# 32-Agent Swarm Shared Experience Architecture
## Trinity 5th Unfair Advantage — Ring-015 (LOW)

**Branch:** `ring-015-swarm-shared-experience`

---

## § 1  Purpose

Enable 32 agents to share `.trinity/experience/` through git — collective intelligence substrate activated.

---

## § 2  Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                             │
│    .trinity/experience/episodes.jsonl                   │
│                                                             │
│    ←─────────────────────────────────────────────────────────     │
│                                                             │
│   32 Agents (github)         Git Push           │
│                                                             │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Principle:** Experience ≠ Context Window. Every agent contributes to a shared, immutable record that persists across sessions.

---

## § 3  Core Components

### 3.1 Experience Store

- **Location:** `.trinity/experience/`
- **Format:** `episodes.jsonl` — one JSON line per experience
- **Schema:** `[Episode]` where:
  - `episode_id`: unique identifier (timestamp#auto)
  - `skill_id`: what was learned (e.g., "ring-000-complete")
  - `session_id`: which session generated it
  - `issue_id`: which issue it relates to (optional)
  - `spec_paths`: `.tri` files involved
  - `spec_hash_before`: hash before changes
  - `spec_hash_after`: hash after changes (SHA-256 aggregate)
  - `gen_hash_after`: hash of generated code (optional)
  - `tests`: `{ status, failed_tests, duration_ms }`
  - `verdict`: `{ toxic, score, notes }`
  - `bench_delta`: `{ metric, value, unit }`
  - `commit`: `{ sha, message, timestamp }`
  - `actor`: "agent:claude-code", "agent:autonomous", etc.
  - `sealed_at`: when the seal was written
  - `completed_at`: when the episode was complete
  - `metadata`: `{ environment, tri_version, notes, origin }`

### 3.2 Git-Based Synchronization

- **Sync Strategy:** `git pull --rebase`
- **Sync Interval:** 300 seconds (5 minutes)
- **Conflict Resolution:** K3 consensus (see §4)
- **Experience Monotonicity:** Appends only, never deletes

### 3.3 Agent Registry

**Location:** `.trinity/swarm/agent-registry.json`

```json
{
  "swarm_size": 32,
  "specializations": [
    {"id": 0, "name": "alpha",   "spec": "vm_core"},
    {"id": 1, "name": "beta",    "spec": "gf_formats"},
    {"id": 2, "name": "gamma",   "spec": "kleene_k3"},
    {"id": 3, "name": "delta",   "spec": "pipeline"},
    {"id": 4, "name": "epsilon", "spec": "experience"},
    {"id": 5, "name": "zeta",    "spec": "scientific_proof"},
    {"id": 6, "name": "eta",     "spec": "codegen"},
    {"id": 7, "name": "theta",   "spec": "vm_executor"},
    {"id": 8, "name": "iota",    "spec": "tri_cli"}
    {"id": 9, "name": "kappa",   "spec": "bench_runner"},
    {"id": 10,"name": "lambda",  "spec": "trib_format"},
    {"id": 11,"name": "mu",     "spec": "test_runner"},
    {"id": 12,"name": "nu",     "spec": "bench_runner"},
    {"id": 13,"name": "xi",     "spec": "phi_arithmetic"},
    {"id": 14,"name": "omnicron","spec": "trit_logic"},
    {"id": 15,"name": "pi",     "spec": "vm_core"},
    {"id": 16,"name": "rho",    "spec": "physics"},
    {"id": 17,"name": "sigma",  "spec": "math"},
    {"id": 18,"name": "tau",    "spec": "compiler"},
    {"id": 19,"name": "upsilon","spec": "isa"},
    {"id": 20,"name": "phi",    "spec": "nn"},
    {"id": 21,"name": "chi",    "spec": "graph"},
    {"id": 22,"name": "psi",    "spec": "ar"}
    {"id": 23,"name": "omega",  "spec": "queen"},
    {"id": 24,"name": "zeta",   "spec": "lotus"},
    {"id": 25,"name": "eta",    "spec": "hslm"},
    {"id": 26,"name": "theta",  "spec": "attention"},
    {"id": 27,"name": "iota",   "spec": "vsa"},
    {"id": 28,"name": "kappa", "spec": "formal"}
    {"id": 29,"name": "lambda","spec": "logic"},
    {"id": 30,"name": "mu",     "spec": "proof"},
    {"id": 31,"name": "nu",     "spec": "verification"}
  ],
  "shared_experience_path": ".trinity/experience/",
  "sync_strategy": "git_pull_rebase",
  "consensus_logic": "kleene_k3"
}
```

---

## § 4  K3 Consensus for Experience Synchronization

### 4.1 Consensus Logic (Kleene)

Given 32 agents voting on a skill improvement, consensus is determined:

| Vote | K3 Interpretation |
|-------|-------------------|
| **POS** | `TF3.pos` — All 32 agents agree improvement |
| **NEG** | `TF3.neg` — All 32 agents agree degradation |
| **NEU** | `TF3.zero` — Split vote (no consensus) |

### 4.2 Consensus Algorithm

1. Each agent contributes: `experience save <skill> <payload>`
2. Skill improvements are compared across sessions
3. Votes are aggregated: `pos_count`, `neg_count`, `zero_count`
4. Consensus = majority rule:
   - If `pos_count > 15`: **POS** (majority)
   - If `neg_count > 15`: **NEG** (majority)
   - Otherwise: **NEU** (no majority)

### 4.3 Experience Merge Logic

```json
{
  "old_bench": 50000000.0,
  "new_bench": 55000000.0,
  "delta_pct": 10.0,
  "verdict": "IMPROVED"
}
```

- **IMPROVED**: `delta_pct > 0` AND `new_bench >= target`
- **DEGRADED**: `delta_pct > 10.0`
- **STABLE**: Otherwise

---

## § 5  Swarm Operations

### 5.1 Agent Contribution

Each agent contributes an episode:

```bash
tri experience save <skill> <payload>
```

**Example:**
```bash
tri experience save "ring-000-complete" "gf_family_foundation_created"
```

### 5.2 Experience Sync

Every 5 minutes (300 seconds), agents run:

```bash
git pull --rebase origin main
```

- **Conflict Detection:** Merge conflicts → trigger `experience diff`
- **Auto-Merge:** Git attempts to merge using `git pull --rebase`
- **Consensus:** K3 vote on whether to accept upstream changes

### 5.3 Experience Evolution (ASHA+PBT)

**ASHA (Asynchronous Successive Halving Algorithm):**
- Start with 32 agents
- Each agent: `tri experience diff <skill> <old> <new>`
- Keep **best** 20% agents, eliminate **worst** 20% agents
- Successive halving reduces agent count by 50% each generation
- Goal: Identify global optimum with < 8 agents

**PBT (Population-Based Training):**
- Each agent: `tri experience evolve` (compare all skills)
- Top performers are promoted to **higher priority** tasks
- Mutations: Learning rate = 0.1 (exploration)

### 5.4 Swarm Verdicts

| Verdict | Meaning |
|----------|---------|
| Consensus(TF3.pos) | All agents agree (improvement confirmed) |
| Consensus(TF3.neg) | All agents agree (degradation confirmed) |
| Consensus(TF3.zero) | Split vote (need manual review) |
| Diverged(u8) | Number of agents in disagreement (e.g., 10 vs 22) |
| Evolving | ASHA+PBT optimization in progress |

---

## § 6  Collective Intelligence Substrate

### 6.1 Knowledge Accumulation

- **Total Episodes:** N (all agents combined)
- **Shared Memory:** Immutable `.trinity/experience/episodes.jsonl`
- **Monotonic Growth:** Episodes only append, never overwrite
- **Cross-Session Learning:** Experiences persist across agent sessions

### 6.2 Skill-Level Evolution

Each skill evolves independently:

| Skill | Evolution Strategy |
|--------|------------------|
| `ring-000` | Initial: 1.0x baseline |
| `ring-013` | ASHA: 32→16→8 agents, 0.1 learning rate |
| `ring-014` | PBT: Top 20% promoted to higher priority |

### 6.3 Swarm Self-Organization

**Emergent Properties:**
- **Swarm Consciousness:** Agents vote on important skills (e.g., "trinity")
- **Swarm Memory:** Most-accessed skills are cached (priority queue)
- **Swarm Learning:** Degraded skills trigger ASHA hyperparameter tuning

---

## § 7  Implementation Notes

### 7.1 Git as Swarm Bus

- **Advantages:** No message broker needed
- **Disadvantages:** Merge conflicts must be resolved via K3 consensus
- **Branching:** Each agent can have its own feature branch
- **Tagging:** Use semantic versioning (e.g., `v0.1.0`)

### 7.2 Experience Storage Optimization

- **Chunking:** `.trinity/experience/` should be < 1GB per ring
- **Archival:** After 10 rings, move old experiences to `archive/`
- **Indexing:** Build simple index JSON for fast skill queries

### 7.3 Conflict Resolution Protocol

When 32 agents disagree on experience merge:

1. **Detect:** Git returns `CONFLICT (content): Merge conflict`
2. **Consensus Vote:** Each agent runs `experience diff <skill>`
3. **K3 Result:**
   - `pos_count > 15` → Accept upstream
   - `neg_count > 15` → Reject upstream
   - Otherwise → Split vote (`experience diff` with both old and new)
4. **Action:**
   - Accept: `git add <conflicted_files>`, `git rebase --continue`
   - Reject: `git rebase --skip` (keep local version)
   - Split: Manually review and resolve

---

## § 8  Swarm Workflows

### 8.1 Agent Onboarding

1. New agent clones repo
2. Reads `.trinity/swarm/agent-registry.json`
3. Registers its `AgentID` with Queen
4. Runs `tri experience save "agent-onboard" "<agent_name>_registered"`
5. Swarm adds agent to consensus

### 8.2 Skill Learning Loop

1. Agent works on a skill (e.g., `ring-013`)
2. Completes ring → `tri experience save "ring-013-done" "<summary>"`
3. Swarm syncs: all agents pull latest `.trinity/experience/`
4. ASHA+PBT: `tri experience evolve` analyzes all skill deltas
5. Degraded skills are flagged for review

### 8.3 Swarm Coordination

- **Queen Brain:** Orchestrates 32 agents via `.trinity/state/queen-health.json`
- **Priority Queue:** Skills are prioritized by business impact
- **Load Balancing:** Distributes skills across agents to prevent bottlenecks

---

## § 9  Trinity Constitution Compliance

### 9.1 L5 Identity Law

All experience data must maintain:

- **Trinity Formula:** φ² + 1/φ² = 3
- **GF32 Precision:** < 1e-13 error on Trinity identity
- **42 Params:** p > 0.95 Monte Carlo significance

### 9.2 L4 Testability Mandate

Each `tri experience save` includes:

- **Tests:** `{ status, failed_tests, duration_ms }`
- **Verdict:** `{ toxic, score, notes }`
- **Benchmarks:** Performance metrics for experience-driven optimization

### 9.3 L2 Generation Law

`.trinity/experience/episodes.jsonl` is **NOT** source code — it is generated from `.tri` specs.

**Invariance:** No agent manually edits `.trinity/experience/episodes.jsonl` — all edits must come through `tri experience save` CLI.

### 9.7 ASHA+PBT Alignment

- **Hyperparameter Tuning:** Learning rate, swarm size, top performer retention
- **Exploration:** 10% of skill evaluations try new approaches
- **Exploitation:** 90% focus on top-performing hyperparameters
- **Decay:** Learning rate decreases by 50% every 5 generations

---

## § 10  Performance Metrics

### 10.1 Swarm Throughput

| Metric | Target | Actual | Status |
|---------|--------|--------|--------|
| Sync Interval | 300 seconds | 300 seconds | ✅ |
| Consensus Throughput | 1M/s | TBD | ⏳ |
| Experience Merges | 10K/s | TBD | ⏳ |
| ASHA+PBT Evolution | 100K generations/s | TBD | ⏳ |

### 10.2 Scalability

- **Minimum Swarm:** 1 agent (baseline)
- **Production Swarm:** 32 agents (Ring-015)
- **Maximum Swarm:** 256 agents (practical limit)

---

## § 11  Security Considerations

### 11.1 Experience Integrity

- **Immutable:** Once saved, episodes cannot be modified
- **Append-Only:** Deletion prohibited by `L4 TESTABILITY`
- **Verification:** SHA-256 seals prevent tampering

### 11.2 Confidentiality

- **Public Skills:** All skill names are visible in experience
- **Agent Secrets:** Private tokens should NOT be logged
- **Audit Trail:** Every `tri experience save` records timestamp and actor

---

## § 12  Future Extensions

### 12.1 Federated Learning

Multiple swarms can share experiences via `experience pull` across repositories.

### 12.2 Neural Swarm Optimization

Use `.trinity/experience/` as training dataset for K3-consensus neural network.

---

**Conclusion:** 32-agent swarm with shared `.trinity/experience/` enables Trinity to learn from all agents simultaneously, accelerating evolution toward optimal performance.

---

φ² + 1/φ² = 3 | TRIB=0x54524942 | 32 Agents | K3 Consensus | ASHA+PBT
