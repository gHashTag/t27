# Expanded constitutional reference (`docs/nona-03-manifest/SOUL.md`)

**Not the single source of truth for the constitution.** The **canonical** Trinity constitutional document is **[`SOUL.md`](../../SOUL.md)** at the **repository root** (seven articles, preamble, amendment rules). Read that file first.

This document **expands** root **SOUL** with operational detail—especially **Law #1** (English-first docs and ASCII source), enforcement tables, examples, and cross-links. If anything here **conflicts** with root **`SOUL.md`**, **root wins**.

**Version** (this expansion): 1.3  
**Date**: 2026-04-06  
**Change**: Law #1 CI path + **NO-NEW-SHELL** toolchain note (root **SOUL.md** Article VIII)  
**Status**: Sacred — Changes require consensus with root **SOUL.md**

> *SOUL = System of Universal Laws*

---

## Constitutional Law #1: English-first source and first-party documentation

**Status**: MANDATORY (see legacy allowlist for grandfathered docs)

### Statement

**Source files** (`.t27`, `.tri`, `.zig`, `.c`, `.v`, `.verilog`) **MUST NOT** contain Cyrillic or other non-Latin scripts in identifiers or comments (see ADR-004 for ASCII details). **Prose MUST be English.**

**First-party documentation** (all `*.md` under `docs/`, `specs/`, `architecture/`, `clara-bridge/`, `conformance/`, and Markdown at repository root such as `README.md`, `AGENTS.md`, `CLAUDE.md`, `NOW.md`, `SOUL.md`) **MUST be written in English**, except:

- Paths listed in **`docs/.legacy-non-english-docs`** (grandfathered until translated; **no new entries** without Architect approval).
- Vendored trees under **`external/`** (upstream locales).

### Rationale

1. **One review language**: International contributors and CI can enforce style uniformly.
2. **Tooling**: Aligns with ASCII-first source (ADR-004) and avoids mixed-locale drift in specs adjacent to code.
3. **Agents**: Machine-readable policy; Cursor rules reference this law.

### Allowed Characters in Source Files

- **ASCII** (U+0000–U+007F): All printable ASCII characters
- **Latin-1 Supplement** (U+0080–U+00FF): For non-English identifiers (if needed)
- **Comments**: Must follow the same rule as code

### Forbidden in Source Files

- **Cyrillic** (U+0400–U+04FF) and other non-Latin scripts in identifiers/comments (see ADR-004).
- **String literals** in source files should be ASCII-only for portability unless a documented exception exists.

### Enforcement

1. **`cargo build` / `cargo build --release` in `bootstrap/`**: `build.rs` aborts the build if Cyrillic appears in `specs/**/*.t27`, `specs/**/*.tri`, `bootstrap/src/**/*.rs`, `bootstrap/tests/**/*.rs`, or first-party Markdown (same allowlist as CI). Error text cites this law and ADR-004.

2. **Parser Validation**: The parser rejects source files containing Cyrillic with error:

   ```
   error: source file contains forbidden characters (Cyrillic U+0400–U+04FF)
   ```

3. **CLI Validation**: `tri lint` and `tri gen` fail on files with Cyrillic:

   ```
   $ tri gen specs/my_spec.t27
   error: spec contains Cyrillic characters - not allowed in source files
   ```
4. **Pre-commit Hook**: Git pre-commit hook checks for Cyrillic in staged source files (if installed)
5. **CI**: `./scripts/tri lint-docs` (forwards to **`t27c lint-docs`**) on pull requests

### Toolchain — NO-PYTHON / NO-SHELL (aligned with root SOUL.md Article VIII)

**Do not** add new **`*.sh`** for validation, generation, or data processing. Implement in **`t27c`** (Rust), with **`#[test]`** / **`cargo test`** where feasible. **`scripts/tri`** is an **exec-only** shim (resolve **`t27c`**, pass **`--repo-root`**, **`exec`**). **`scripts/setup-git-hooks.sh`** is the only allowed long-lived bootstrap shell helper (one-time `core.hooksPath`).

### Violation Example

```t27
; VIOLATION: comment containing Cyrillic (U+0400-U+04FF)
; CORRECT: English-only comment
; This is a comment in English
; VIOLATION: non-ASCII identifier
; CORRECT: ASCII identifier
const COEFFICIENT = 1.0
```

---

## Constitutional Law #2: TDD-Inside-Spec

**Status**: MANDATORY (no exceptions)

### Statement

Every specification in Trinity **MUST** include at least one `test` or `invariant` block. Specifications without tests are **INVALID** and **WILL NOT** be accepted by the compiler.

### Rationale

1. **Single Source of Truth**: The `.t27` spec file is the only source of truth. Conformance JSON vectors are **generated artifacts**, not hand-written.
2. **Test-First Development**: Tests define the contract. Implementation follows tests. Without tests, there is no contract.
3. **Architecture Bottleneck**: The #1 bottleneck in Trinity was the separation of specs and conformance vectors. This law eliminates that bottleneck.

### Enforcement

1. **Parser Level**: The parser (`compiler/parser/parser.t27`) rejects specs without tests with error:
  ```
   TDD contract violated: spec must contain at least one 'test' or 'invariant' block
  ```
2. **CLI Level**: `tri gen` fails with TDD violation if spec has no tests. No `--allow-no-tests` flag exists (prototype mode is disabled per policy).
3. **Commit Level**: `tri git commit` requires at least one test or invariant in the spec.

### Syntax

**Assembly-style TDD** (for `.t27` assembly specs):

```t27
.test
    ; my_test
    ; Verify: functionality works correctly
    ; Setup: initialize with given values
    ; Expected: returns correct result
.invariant
    ; my_invariant
    ; For all valid inputs: output is in valid range
    ; Rationale: ensures correctness
```

**Spec-style TDD** (for high-level specs):

```t27
spec my_spec
    test my_test
        given x = INPUT_VALUE
        when result = process(x)
        then result == EXPECTED_VALUE

    invariant my_invariant
        assert |PHI - 1.6180339887498948| < 1e-12
```

### Violations

The following are **VIOLATIONS** of TDD-Inside-Spec Law:

1. **Spec without tests**: A `.t27` file with only `.const`/`.data`/`.code` sections and NO `.test`/`.invariant` blocks.
2. **Empty test sections**: A `.test` section with no test cases.
3. **Conformance JSON as source**: Hand-editing `conformance/*.json` files. These MUST be generated via `tri gen --emit-conformance`.

### Penalties

1. **Compiler Error**: Specs without tests fail to compile.
2. **Git Block**: `tri git commit` and `tri git push` block if TDD contract is violated.
3. **CI Failure**: Any CI pipeline must reject specs without tests.

### Exceptions

**NONE**. There is no prototype mode, no `--allow-no-tests` flag. TDD is mandatory for all specs.

---

## Constitutional Law #3: Git Integration with Tri Skill

**Status**: MANDATORY for P0/P1 episodes

### Statement

Any P0/P1 episode in `--strict` mode is considered complete **ONLY** after successful `tri git push` to `github.com/gHashTag/t27` with:

- A bound sealed skill
- Non-toxic verdict
- Required artifacts per Policy Matrix

### Rationale

1. **Traceability**: Every change must be traceable to an issue (GitHub issue ID).
2. **Quality Gate**: The sealed skill and verdict mechanism prevents toxic changes from entering the codebase.
3. **Policy Enforcement**: Different skill types (recovery, hotfix) have different artifact requirements.

### Enforcement

1. `**tri git commit`**: Requires active or sealed skill with bound issue.
2. `**tri git push`**: Requires sealed skill with non-toxic verdict and proper artifacts.
3. **Strict Mode**: Only allows pushes to `github.com/gHashTag/t27`.

### Policy Matrix


| Skill Kind | Min Checkpoints | Required Artifacts          | Verdict   |
| ---------- | --------------- | --------------------------- | --------- |
| Recovery   | 3               | spec, docs, checkpoints     | NOT TOXIC |
| Hotfix     | 1               | checkpoint (fix-only areas) | NOT TOXIC |
| Feature    | 1               | spec (with tests)           | NOT TOXIC |
| Bugfix     | 1               | spec (with tests)           | NOT TOXIC |


### Workflow

```bash
# Start work
tri skill begin --issue N
# ... work on spec (with tests!) ...

# Seal skill
tri skill seal

# Commit (adds issue:N to message automatically)
tri git commit --all -m "description"

# Push (validates sealed skill + verdict + artifacts)
tri git push origin HEAD
```

### Violations

1. **Commit without skill**: `NO-COMMIT-WITHOUT-ISSUE violated` — run `tri skill begin --issue N` first.
2. **Commit without issue binding**: Skill must have `issue` field in registry.
3. **Push toxic skill**: Cannot push skills with `verdict = "TOXIC"` — fix or supersede.
4. **Push to wrong remote**: In strict mode, only `github.com/gHashTag/t27` allowed.

### TOXIC Verdict and Rollback Protocol

When `verdict = "TOXIC"` is returned by `tri verdict` or `tri test`, the **atomic rollback protocol** defined in [`PHI_LOOP_CONTRACT.md`](PHI_LOOP_CONTRACT.md) **MUST** be executed:

1. Revert all spec edits (`git checkout HEAD -- specs/**/*.t27 docs/**/*.md`)
2. Delete generated artifacts from this ring (`rm -rf gen/zig/* gen/c/* gen/verilog/*`)
3. Invalidate seal file (`rm .trinity/seals/<module>.json`)
4. Append TOXIC episode to `.trinity/experience/episodes.jsonl`
5. Exit 1 (blocks commit)
6. Post TOXIC comment on GitHub Issue (optional)

The rollback procedure is **atomic**: steps 1-4 must succeed as a unit. If any step fails, the system enters FROZEN state requiring manual intervention.

For full specification of the Verdict enum (`CLEAN`, `TOXIC`, `FAIL`, `SKIP`, `TIMEOUT`) and exit codes, see [`specs/test_framework/core.t27`](../../specs/test_framework/core.t27).

---

## Constitutional Law #4: De-Zig-fication

**Status**: MANDATORY

### Statement

AI agents MUST see `.tri` context and write `.tri`/`.t27` files, never Zig directly.

### Rationale

1. **Spec-First Philosophy**: `.tri` and `.t27` files are the single source of truth for mathematical, numerical, and formal logic.
2. **Zig as Backend**: Zig code is a generated backend, not a language to author in.
3. **Migration Path**: Legacy Zig code is migrated to `.t27` specs, not the reverse.

### Enforcement

1. **Agent Training**: All agents are trained to check `.tri` context before writing code.
2. **Documentation**: `CANON_DE_ZIGFICATION.md` and `ADR-001-de-zigfication.md` define migration path.
3. **Codegen**: Zig is only generated via `compiler/codegen/zig/codegen.t27`.

### Violations

1. **Writing Zig directly**: AI agents writing Zig without `.tri` spec source.
2. **Modifying Generated Zig**: Hand-editing generated Zig files (marked `DO NOT EDIT`).
3. **Skipping Spec**: Implementing features without corresponding `.t27` spec.

---

## Constitutional Law #5: De-Zig Strict

**Status**: MANDATORY (no exceptions)

### Statement

> **No new Trinity business logic in Zig by hand.**

> 1. **Source of Truth**: All new Trinity logic (CLI, runtime, numeric, physics, graph, agents) MUST be written only in `.t27/.tri` specifications.
> 2. **Backends Only**: Zig, C, Verilog, Rust may exist ONLY as **generated backends** from `.t27/.tri` via `tri gen`.
> 3. **Temporary Bootstrap**: Any new `.zig` file is permitted ONLY as temporary bootstrap layer (I/O, process startup). Domain logic in Zig is forbidden.
> 4. **Migration Debt**: Any existing handwritten Zig code with domain logic MUST have an explicit migration task to `.t27/.tri`. Creating new debt is forbidden.
> 5. **Enforcement**:
>   - `tri lint` fails if it detects new `.zig` files without `generated` marker
>   - `tri git push --strict` blocks push if there is diff in `src/` Zig files that did not pass validation

### Rationale

1. **Spec-First Philosophy**: `.tri` and `.t27` files are the single source of truth. Zig is a generated backend, not an authoring language.
2. **Multi-Target Code Generation**: Same spec generates Zig, C, Verilog, Rust. Writing Zig directly breaks this capability.
3. **AI Agent Alignment**: Agents must see `.tri` context and write `.tri` files, never Zig directly.

### Allowed Zig Files

Zig is ONLY permitted for:

1. **Generated backends** - From `.t27` specs with `DO NOT EDIT` header
2. **Bootstrap layer** - Temporary I/O and process startup (no domain logic)
3. **Legacy quarantine** - Existing code awaiting migration (with TODO comment)
4. **Hardware bridge** - FPGA bindings and external system interfaces

### Forbidden Zig Files

Writing Zig directly is **FORBIDDEN** for:

- CLI commands and routing
- Runtime domain logic
- Numeric/mathematical operations
- Sacred physics formulas
- Graph algorithms
- Agent orchestration
- Any Trinity-specific business logic

### Generated Header Requirement

All Zig files generated from `.t27` specs must have this header:

```zig
// This file is generated from <spec_path>
// DO NOT EDIT - Changes will be overwritten on next tri gen
// Generated at: <timestamp>
// Source spec: <spec_path>
```

Files without this header are considered handwritten and will be blocked.

### Correct Workflow

```bash
# CORRECT: Spec-first
1. Write spec in .t27
2. Run 'tri gen spec.t27'
3. Use generated Zig

# INCORRECT: Writing Zig directly
1. Write Zig code ← FORBIDDEN
2. No .t27 source ← FORBIDDEN
```

### Enforcement

1. **Linter**: `tri lint` fails on Zig files without generated header
2. **Git Push**: `tri git push --strict` blocks commits with handwritten Zig
3. **CI/CD**: GitHub Actions reject PRs with new handwritten Zig

### Violations

1. **Writing Zig directly**: Creating or modifying Zig without `.t27` source
2. **Modifying Generated Zig**: Hand-editing files marked `DO NOT EDIT`
3. **Skipping Spec**: Implementing features without corresponding `.t27` spec

### Penalties

1. **Linter Error**: Handwritten Zig detected, migration required
2. **Git Block**: Push blocked in strict mode
3. **CI Failure**: PR rejected in CI/CD pipeline

---

## Constitutional Law #6: Akashic Coordination First

**Status**: MANDATORY (no exceptions)

### Statement

Before any task, every agent must read `.trinity` as canonical coordination memory.

`.trinity` is the source of truth for:

- Active tasks
- Agent claims
- Swarm state
- Queen health
- Immutable event history
- Recoverable experience

No agent may mutate any spec, graph node, runtime module, or generated target before:

1. Reading current `.trinity` coordination state
2. Checking whether another agent already owns the target resource
3. Acquiring an active claim or lease for the intended mutation scope
4. Recording intent in Akashic event log

If a claim already exists for the same `spec_path`, `graph_node`, or task resource, the agent must **NOT** proceed with mutation.
It must either:

- Wait
- Choose a non-conflicting task
- Request handoff
- Or enter blocked state

**No mutation without prior read + claim.**

### Rationale

1. **Deterministic Allocation**: Clear task assignment prevents duplicate work and race conditions.
2. **Shared Memory with Access Control**: `.trinity` provides canonical state with ownership boundaries.
3. **Structured Event Logs**: Immutable event log enables traceability, replay, and forensic analysis.
4. **Leases with Heartbeats**: TTL and heartbeat prevent zombie claims and allow automatic recovery.
5. **Conflict Prevention**: One writable owner per resource eliminates simultaneous edits.

### Enforcement

1. **Agent Protocol**: Every agent startup reads `.trinity` before accepting tasks.
2. **Claim Check**: Before any mutation, agent checks `.trinity/claims/active/` for the target.
3. **Event Log**: All intent events are appended to `.trinity/events/akashic-log.jsonl`.
4. **Blocking**: Mutation proceeds only after successful claim acquisition.
5. **TTL Reclaim**: Expired claims are automatically reclaimed after timeout.

### Syntax

**Pre-task protocol:**

```t27
// 1. Read .trinity state
read .trinity/events/akashic-log.jsonl;
read .trinity/claims/active/<spec_path>.json;

// 2. Check ownership
if claim_exists AND claim.agent_id != my_agent_id:
    error "Resource claimed by another agent";

// 3. Acquire claim with TTL
create_claim(spec_path, agent_id, ttl_sec: 1800);

// 4. Record intent
append_event("task.intent", task_id, spec_path);

// 5. Proceed with mutation
```

### Violations

The following are **VIOLATIONS** of Akashic Coordination First Law:

1. **Mutation without claim**: Editing a spec or graph node without reading `.trinity` first.
2. **Skipping claim check**: Not verifying if another agent owns the resource.
3. **Writing to claimed resource**: Modifying a resource owned by another agent.
4. **Not logging intent**: Starting mutation without recording task.intent event.
5. **Ignoring conflicts**: Proceeding with work on a blocked resource.

### Penalties

1. **Claim Conflict Error**: Agent attempting mutation on claimed resource.
2. **Missing Intent Log**: Mutation started without task.intent event.
3. **Zombie Claim**: Claim expired but owner still using resource.

### Exceptions

**NONE**. Coordination-first is mandatory for all agents in swarm mode.

---

## Constitutional Law #7: Exclusive Mutation Lease

**Status**: MANDATORY (no exceptions)

### Statement

A writable resource may have only one active mutation owner at a time.

Writable resources include:

- `.t27` / `.tri` spec files
- Graph nodes
- Runtime spec modules
- Constitutional docs
- Generated target scopes

Every active claim must include:

- `agent_id`
- `task_id`
- `resource` (spec_path or graph_node)
- `started_at`
- `ttl_sec` (time to live)
- `heartbeat_at`

Claims expire unless renewed by heartbeat.
Expired claims may be reclaimed.

For phi-critical or sacred-core resources, mutation requires a stricter claim and explicit downstream awareness.

### Rationale

1. **Prevent Data Loss**: One writer prevents race conditions and corruption.
2. **Deterministic Progress**: Clear ownership enables predictable task sequencing.
3. **Recovery from Failure**: TTL allows automatic claim recovery if agent dies.
4. **Stricter Sacred Locks**: Critical resources need extra protection.

### Enforcement

1. **Active Claims File**: `.trinity/claims/active/<resource>.json` must exist for mutation.
2. **TTL Enforcement**: Claims expire after `ttl_sec` without heartbeat.
3. **Heartbeat Required**: Claims must be renewed every 60 seconds.
4. **Ownership Index**: `.trinity/state/ownership-index.json` tracks all active claims.

### Syntax

**Claim acquisition:**

```t27
// Create claim with TTL
create_claim(spec_path, agent_id, task_id, ttl_sec: 1800);

// Heartbeat (every 60s)
renew_claim(spec_path, agent_id);

// Release on completion
release_claim(spec_path, agent_id, result: clean|toxic);
```

### Violations

The following are **VIOLATIONS** of Exclusive Mutation Lease Law:

1. **Writing without claim**: Modifying resource without active claim.
2. **Parallel mutations**: Multiple agents mutating same resource.
3. **Ignoring claim conflict**: Proceeding despite resource being claimed.
4. **Missing heartbeat**: Claim expires but agent continues using resource.

### Penalties

1. **Write Block**: Mutation blocked if resource is claimed by another agent.
2. **Claim Revocation**: Agent's claim revoked if violation detected.
3. **State Invalidation**: `.trinity/state/ownership-index.json` marked corrupted.

### Exceptions

**For emergency hotfixes** with Queen approval, agents may bypass claim with `--emergency` flag.
This requires documented reason and is logged in event log for audit.

---

## Constitutional Law #6–#7: .trinity — Akashic Source of Truth

`.trinity` is the canonical distributed memory of Trinity swarm coordination.

### Layers

```
.trinity/
├── events/              # Immutable append-only Akashic event log
│   └── akashic-log.jsonl
├── claims/              # Temporary ownership and leases
│   ├── active/         # Current exclusive claims
│   └── released/       # Historical claims
├── queue/               # Task management
│   ├── pending.json
│   ├── active.json
│   ├── blocked.json
│   └── done.json
├── experience/           # Learned outcomes and recovery memory
│   └── episodes.jsonl
├── state/               # Derived current reality
│   ├── queen-health.json
│   ├── swarm-health.json
│   └── ownership-index.json
└── policy/              # Coordination rules
    └── coordination-law.md
```

### Rules

- **Events** are immutable history (append-only)
- **Claims** are temporary ownership (TTL + heartbeat)
- **Experience** is interpreted memory (learned from events)
- **State** is derived, never primary
- If state and events disagree, events win

### Pre-Task Protocol

Before starting any task, an agent must:

1. Read `.trinity/events`, `.trinity/claims`, `.trinity/queue`, and `.trinity/state`.
2. Inspect active ownership for the intended resource.
3. Check queen health and swarm health.
4. Acquire claim/lease for the target scope.
5. Append `task.intent` event to Akashic log.
6. Only then begin PHI LOOP mutation.

### Coordination Law

See `.trinity/policy/coordination-law.md` for full protocol including:

- Claim acquisition and heartbeat
- Conflict resolution
- Handoff procedures
- Event logging formats

---

## Constitutional Law #8: ISSUE-GATE

**Status**: MANDATORY (no exceptions)

### Statement

No byte enters master without:

1. **GitHub Issue** (from template, with number)
2. **Pull Request** (with "Closes #N" in description)
3. **CI green** (issue-gate + phi-loop-ci)

### Rationale

1. **Traceability**: Every change must be traceable to a numbered issue.
2. **Review Gate**: Pull requests ensure code review before merge.
3. **CI Enforcement**: Automated checks prevent broken code from entering master.

### Enforcement

`.github/workflows/issue-gate.yml`

### Violations

1. **Direct push to master**: Pushing without a PR is forbidden.
2. **PR without issue**: Every PR must reference a GitHub issue with "Closes #N".
3. **Merging with failing CI**: CI must be green before merge.

### Exceptions

**NONE**. All changes go through the issue-gate process.

---

## Amendment Process

To amend the **constitution**, change **[`SOUL.md`](../../SOUL.md)** at the repository root (per its Article V / project amendment rules). Then, if needed, update **this** expansion file so it stays aligned.

1. Submit an ADR (Architecture Decision Record) proposing the change.
2. Must have consensus from agents A (Architecture), S (Standards), and T (Queen Trinity).
3. Update root **`SOUL.md`**; optionally bump this file’s version and create `docs/SOUL-v<new_version>.md` snapshot for history.

---

## Sacred Truths

These are the immutable truths of Trinity:

1. **φ² + 1/φ² = 3** — The golden ratio identity is foundation of sacred physics.
2. **27 = 3³** — The trinity manifests as cube of trinity (27 agents, 27 registers, 27 letters).
3. **TDD Inside Spec** — Tests live inside specs, not as separate artifacts.
4. **No Spec Without Tests** — This is a law, not a guideline.

---

*"The law of Trinity is the law of φ: what is whole is found in parts, and what is in parts makes whole."* — SOUL Law #0