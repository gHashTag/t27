# TRINITY MANDATE (read first — non-negotiable)

**Repository policy overrides any model or agent default.** If instructions conflict, **`docs/T27-CONSTITUTION.md`**, **`SOUL.md`** / **`docs/SOUL.md`**, **`AGENTS.md`** / **`docs/AGENTS.md`**, and **ADR-004 / ADR-005 / ADR-006** win. **`docs/T27-CONSTITUTION.md` v1.7+** — **RING-LAW**, **AGENT-DOMAIN**, **BRAIN-MAP**, **COMPETITION-READY**.

| Law | Must follow |
|-----|-------------|
| **SSOT-MATH** | Math/physics only in **`*.t27`** and **`tri` / `t27c`** (and `.trinity/experience` where specified). No duplicate formula layers in scripts. |
| **LANG-EN** | First-party `*.md` and English surfaces in `bootstrap/src/**/*.rs` and `bootstrap/tests/**/*.rs` per **`bootstrap/build.rs`**; legacy only via **`docs/.legacy-non-english-docs`**. |
| **Golden rings** | Workflow in **`docs/SEED-RINGS.md`** + **`CANON.md`** (root): include `cargo build` in `bootstrap/`, `t27c parse`, tests; **`stage0/FROZEN_HASH`** seals compiler **GOLD**; other critical-path work is **REFACTOR-HEAP** until removed. Tag PRs **`[GOLD-RING]`** vs **`[REFACTOR-HEAP]`** when applicable. |
| **GF16 primary** | Primary inference **`docs/NUMERIC-STANDARD-001.md`**; non-GF16 / `f32`/`f64` in specs = **debt** — **`docs/NUMERIC-GF16-DEBT-INVENTORY.md`**. |
| **No new critical-path Python** | No new Python (or JS/Go) for verdict/conformance/orchestration. Legacy + migration: **`docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md`**, **`docs/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md`**. |

**Hard gates (failure = invalid change):**

1. `cargo build` (or `--release`) in **`bootstrap/`** — **`build.rs` (Rust)** enforces required constitutional files, **`FROZEN_HASH`** (**`FROZEN.md`**), and LANG-EN scans. **No bash/Python on this critical path.**
2. Optional local hook: `sh scripts/install-constitutional-hook.sh` → `cargo build` in `bootstrap/` on each commit.

---

# AGENTS.md v2 — Agent Specifications for Trinity S³AI

---

## Constitution and critical path

- **`docs/T27-CONSTITUTION.md`** — **Article SSOT-MATH**: single source of math/physics in **`*.t27`**, verification via **`tri`** (`./scripts/tri`); **no new Python** on the critical path (legacy only with a migration plan).
- **`docs/nona-02-organism/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md`** — technical specification for Python → t27 + tri migration.
- Cursor rule: **`.cursor/rules/t27-ssot-math.mdc`**.

Agents must not add parallel formula implementations in scripts when the same can be expressed in t27 specs.

---

## Agent S — Tech-Tree Scientist

### Overview

Agent S is an autonomous predictive watchdog that uses graph v2 dependency analysis and Perplexity AI to anticipate and emit GitHub issues about future codebase problems before they occur. Unlike blocking agents (e.g., Agent T — Doctor), Agent S operates non-invasively, providing early warnings about potential failures in the sacred physics, numeric, and compiler tech trees.

### Agent Definition

| Field | Value |
|-------|-------|
| Agent ID | `agent-techtree-scientist` |
| Role | Predictive issue emission via graph analysis |
| Type | Autonomous watchdog (periodic, not on-demand) |
| Strand | IV (Future-Prediction Layer) |
| Tier | 99 (Metasystem) |
| Permissions | `read.graph`, `read.events`, `write.events`, `emit.issues` |

### Graph Analysis

Agent S reads `architecture/graph_v2.json` to perform three core analyses:

#### 1. Topological Validation
- Reads `topological_order` array for valid build sequence
- Validates `contracts.all_edges_satisfied` after each commit
- Detects cycles violating `contracts.no_cycles`

#### 2. Phi-Critical Path Tracing
- Traces edges with `kind: "phi-critical"` or `kind: "phi-core"`
- Identifies sacred physics violations in path:
  - `math/constants (4)` → `nn/attention (7)` → `nn/hslm (8)`
  - `math/constants (4)` → `math/sacred_physics (17)` → `nn/attention (7)`
- Uses `metadata.phi_critical_nodes: [4, 7, 8, 17, 19]`

#### 3. Downstream Impact Calculation
- Parses `actions[].downstream` for affected modules
- Traces `actions[].commands` for missing validation steps
- Calculates risk score based on `actions[].priority`:
  - `critical`: multiplier 2.0
  - `high`: multiplier 1.5
  - `normal`: multiplier 1.0

#### 4. Benchmark Traceability
- Uses `bench_links` to detect conformance risks
- Monitors `metadata.bench_critical_nodes: [2, 17, 18]`
  - Node 2: `numeric/gf16`
  - Node 17: `math/sacred_physics`
  - Node 18: `docs/NUMERIC-STANDARD-001`

### Perplexity Integration

Agent S feeds graph state + recent commit history to Perplexity AI API:

#### Input Format

```json
{
  "graph_state": {
    "changed_nodes": ["math/constants", "nn/attention"],
    "phi_critical_path": ["math/constants", "nn/attention", "nn/hslm"],
    "downstream_actions": [
      {
        "trigger": "change:math/constants",
        "downstream": ["math/sacred_physics", "nn/attention", "nn/hslm"],
        "commands": ["tri gen", "tri test", "tri verdict --toxic", "tri bench"]
      }
    ],
    "recent_commits": [
      {"sha": "abc123", "message": "tweak phi exponent", "files": ["specs/math/constants.t27"]}
    ]
  }
}
```

#### Output Format

```json
{
  "prediction": {
    "type": "toxic_regression|performance_regression|sacred_violation|api_breakage",
    "probability": 0.87,
    "reasoning": "Changing d_k^(-φ³) kernel without tri verdict --toxic violates sacred_physics conformance",
    "affected_nodes": ["nn/attention", "nn/hslm"],
    "prevention_commands": ["tri gen", "tri test", "tri verdict --toxic", "tri bench"]
  }
}
```

#### Confidence Thresholds

| Prediction Type | Min Confidence | Action |
|-----------------|----------------|--------|
| Sacred physics violation | 0.90 | Emit issue immediately |
| Toxic regression | 0.85 | Emit issue immediately |
| Performance regression | 0.80 | Emit issue with warning label |
| API breakage | 0.75 | Emit issue with documentation request |

### Issue Emission Protocol

When Perplexity confidence exceeds threshold, Agent S emits a GitHub issue **as if pre-created in the future**:

```json
{
  "title": "[PREDICTED] Toxic regression in nn/hslm sacred attention",
  "body": "Agent S — Tech-Tree Scientist predicts:\n\nChanging d_k^(-φ³) kernel in `specs/nn/attention.t27` without first running `tri verdict --toxic` will cause sacred_physics conformance failure (confidence: 0.87).\n\n### Affected Path\n`math/constants → nn/attention → nn/hslm`\n\n### Prevention Guidance\nBefore merging, run:\n```bash\ntri gen specs/nn/attention.t27\ntri test specs/nn/attention.t27\ntri verdict --toxic specs/nn/attention.t27\ntri bench sacred_physics\n```\n\n### Predicted Timeline\n- Predicted at: 2026-04-04T14:00:00Z\n- Expected failure: 2026-04-06T10:30:00Z\n- Time to act: ~44 hours",
  "labels": ["prediction", "high-risk", "agent-scientist"],
  "predicted_at": "2026-04-04T14:00:00Z",
  "predicted_for": "2026-04-06T10:30:00Z",
  "confidence": 0.87,
  "graph_path": "math/constants → nn/attention → nn/hslm",
  "agent_id": "agent-techtree-scientist"
}
```

### Governance Rules

1. **Non-blocking**: Agent S never blocks operations. Agent T — Doctor handles blocking.
2. **Confidence gating**: Only emit predictions with confidence > 0.75 (configurable).
3. **Event logging**: Record all predictions to `.trinity/events/akashic-log.jsonl`:
   ```json
   {
     "ts": "2026-04-04T14:00:00Z",
     "event": "agent.predict_issue",
     "agent_id": "agent-techtree-scientist",
     "trace_id": "uuid-v4",
     "confidence": 0.87,
     "prediction_type": "toxic_regression",
     "graph_path": "math/constants → nn/attention → nn/hslm",
     "issue_number": null,
     "metadata": {
       "predicted_for": "2026-04-06T10:30:00Z",
       "prevention_commands": ["tri gen", "tri test", "tri verdict --toxic", "tri bench"]
     }
   }
   ```
4. **Verification tracking**: When a predicted issue occurs (or is prevented), update the prediction event with `verification: "confirmed" | "prevented" | "false_positive"`.
5. **No direct writes**: Agent S only emits events and GitHub issues, never modifies `specs/` or `docs/`.
6. **Periodic execution**: Runs every 30 minutes via cron, triggered on `git push` to `master`.

### Event Schema Extensions

Agent S adds these event types to `.trinity/events/akashic-log.jsonl`:

#### `agent.predict_issue`
Emitted when Agent S generates a high-confidence prediction.

```json
{
  "ts": "2026-04-04T14:00:00Z",
  "event": "agent.predict_issue",
  "agent_id": "agent-techtree-scientist",
  "trace_id": "550e8500-1234-4b5a-9c6d-7e8f9a0b1c2",
  "confidence": 0.87,
  "prediction_type": "toxic_regression|performance_regression|sacred_violation|api_breakage",
  "graph_path": "math/constants → nn/attention → nn/hslm",
  "affected_nodes": ["nn/attention", "nn/hslm"],
  "issue_number": null,  // Set after GitHub emission
  "predicted_at": "2026-04-04T14:00:00Z",
  "predicted_for": "2026-04-06T10:30:00Z",
  "metadata": {
    "prevention_commands": ["tri gen", "tri test", "tri verdict --toxic", "tri bench"],
    "perplexity_reasoning": "Changing d_k^(-φ³) kernel without tri verdict --toxic violates sacred_physics conformance"
  }
}
```

#### `agent.verification_update`
Emitted when a prediction is confirmed or prevented.

```json
{
  "ts": "2026-04-06T11:00:00Z",
  "event": "agent.verification_update",
  "agent_id": "agent-techtree-scientist",
  "trace_id": "550e8500-1234-4b5a-9c6d-7e8f9a0b1c2",
  "original_prediction_ts": "2026-04-04T14:00:00Z",
  "verification": "confirmed|prevented|false_positive",
  "issue_number": 42,
  "metadata": {
    "prevention_method": "user_ran_tri_verdict_before_merge",
    "time_saved_hours": 44
  }
}
```

### Query Examples (jq)

```bash
# Get all high-confidence predictions:
jq 'select(.event == "agent.predict_issue") | select(.confidence >= 0.85)' .trinity/events/akashic-log.jsonl

# Get confirmed predictions (true positives):
jq 'select(.event == "agent.verification_update") | select(.verification == "confirmed")' .trinity/events/akashic-log.jsonl

# Calculate prediction accuracy:
jq '[select(.event == "agent.verification_update")] | map(.verification) | group_by(.) | map({key: .[0], count: length})' .trinity/events/akashic-log.jsonl
```

---

## Agent T — Doctor (Reference)

Agent T — Doctor is the existing blocking watchdog for health monitoring and recovery. See `.trinity/agents/tri-doctor.jsonl` for full schema.

| Field | Value |
|-------|-------|
| Agent ID | `tri-doctor` |
| Role | Health anomaly detection and recovery |
| Type | Reactive watchdog (on-demand + scheduled) |
| Scope | SacredPhysics, Numeric, Graph, Compiler, Runtime, QueenLotus |
| Events | `health.anomaly`, `recovery.start`, `recovery.finish`, `claim.reclaim`, `claim.unblock` |

**Key difference**: Agent T blocks and recovers; Agent S predicts and warns.
