# PHI LOOP Contract — Policy-as-Code Specification

Version: 1.0
Status: Active
Effective: 2026-04-04

---

## 1. PHI LOOP Contract (MUST/SHOULD)

### 1.1. General Principles

- PHI LOOP **MUST** be the ONLY legitimate method for modifying Trinity repository and `.trinity/` state for both agents and humans.
- All rules below **MUST** apply equally to humans and agents; bypassing via direct git/CLI is a protocol violation.

### 1.2. State Sources

PHI LOOP **MUST** use the following state sources for every command:

| File | Purpose |
|------|---------|
| `.trinity/state/active-skill.json` | Current active skill/session (exclusive mutation lease) |
| `.trinity/state/issue-binding.json` | Hard binding between active skill and issue/task |
| `.trinity/experience/episodes.jsonl` | Journal of completed PHI episodes for audit and analysis |

Any guard decision **MUST** depend only on these files and directly observable git state (status/diff), without hidden in-memory state.

### 1.3. Guard Conditions

#### 1.3.1. NO-COMMIT-WITHOUT-ISSUE

Any operation leading to git commit (including `tri skill commit`, `tri git commit`, internal auto-commits) **MUST** be blocked if:

- No active skill in `.trinity/state/active-skill.json`, OR
- No valid issue binding in `.trinity/state/issue-binding.json`, OR
- Commit message does not contain correct issue reference per accepted format (e.g., `[ref: 1234]` or `ISSUE-1234`)

On violation, PHI LOOP **MUST**:

- Return non-zero exit code
- Output diagnostic message explicitly indicating missing condition (e.g., "ERROR: Cannot commit without active skill + issue binding")

#### 1.3.2. NO-MUTATION-WITHOUT-SKILL

Any command that modifies `.trinity/` or specs `.tri/.t27` (including `tri spec edit`, `tri gen`, `tri verdict`, `tri skill seal`) **MUST** check for active skill.

If no active skill exists, command **MUST** fail with error before any file changes.

Writes to `.trinity/state`, `.trinity/events`, `.trinity/queue`, `.trinity/experience` **MUST** be prohibited outside active skill, except for strictly defined system operations and cold-start initialization.

#### 1.3.3. Immutable Audit

Every successful PHI LOOP completion (after seal/verdict/commit) **MUST** generate exactly one record in `.trinity/experience/episodes.jsonl`.

Already recorded episodes **MUST NOT** be modified; only appending new lines is allowed.

### 1.4. Mandatory PHI LOOP Workflow

State machine transitions:

1. `tri skill begin --issue <ID> --description "<text>"`
   - **MUST** create/update `.trinity/state/active-skill.json` and `.trinity/state/issue-binding.json` in consistent state
   - **MUST** refuse if there's already an active skill with different issue and lease wasn't properly closed

2. `tri spec edit ...`
   - **MUST** be allowed only with active skill + binding

3. `tri skill seal --hash ...`
   - **MUST** be prohibited without active skill
   - **MUST** verify spec_hash_before/after match current git state and expectations

4. `tri gen`, `tri test`, `tri verdict`
   - Each **MUST** check active skill + issue-binding before execution

5. `tri skill commit` / `tri git commit`
   - **MUST** be the only legal commit methods in PHI LOOP
   - **MUST** perform NO-COMMIT-WITHOUT-ISSUE check
   - **MUST** after successful commit: add record to `episodes.jsonl` and reset `active-skill`

### 1.5. Status Display (`tri status`)

Command `tri status` (or `tri status only`) **MUST**:

- Read `.trinity/state/active-skill.json` and display Active Skill (skill-id/name/description)
- Read `.trinity/state/issue-binding.json` and display linked issue (ID, title/summary)
- Display current git state (modified/untracked), highlighting:
  - Files inside `.trinity/`
  - Specs `.tri/.t27`
  - Other files (docs, code)
- Display current guard state (GREEN/RED) with all violated MUST conditions

---

## 2. State Source Specification

### 2.1. `.trinity/state/active-skill.json`

Purpose: Single source of truth for current active skill (exclusive mutation lease).

Format (JSON object):

```json
{
  "skill_id": "tri-constitution",          // string skill ID
  "session_id": "2026-04-04T06:45:12Z#1",  // unique session identifier
  "issue_id": "TTT-1234",                  // linked issue (duplicates issue-binding)
  "description": "Short human-readable task description",
  "started_at": "2026-04-04T06:45:12Z",    // ISO-8601
  "started_by": "agent:tri-doctor",        // or "human:<name/handle>"
  "status": "active",                      // one of: "active", "closing", "closed"
  "allowed_paths": [
    "docs/nona-03-manifest/SOUL.md",
    ".trinity/cells/registry.json",
    ".trinity/policy/",
    ".trinity/state/"
  ],
  "metadata": {
    "priority": "normal",
    "tags": ["phi-loop", "coordination"],
    "origin": "cli|telegram|api"
  }
}
```

Contract:

- When no active skill, file is either absent OR contains `"status": "closed"` with null key fields (skill_id/issue_id). Guard treats both as "no active skill".
- All commands requiring mutation lease **MUST**:
  - Read this file
  - Verify `status == "active"` and non-empty `skill_id`
  - Check that change targets are within `allowed_paths`

### 2.2. `.trinity/state/issue-binding.json`

Purpose: Hard binding between active skill and specific issue/task; used by commit guards.

Format (JSON object):

```json
{
  "issue_id": "TTT-1234",                  // ID in your task system (GitHub, Linear, custom)
  "source": "github",                      // "github" | "linear" | "jira" | "local" | ...
  "url": "https://github.com/org/repo/issues/1234",
  "title": "Short issue title",
  "state": "open",                         // "open" | "in_progress" | "blocked" | "closed"
  "linked_skill_id": "tri-constitution",
  "linked_session_id": "2026-04-04T06:45:12Z#1",
  "last_synced_at": "2026-04-04T06:45:30Z",
  "required_commit_message_pattern": "\\[ref: 1234\\]",   // regex or commit message template
  "metadata": {
    "assignee": "user:you",
    "labels": ["phi-loop", "coordination"],
    "repository": "org/repo"
  }
}
```

Contract:

- Commit guard **MUST** verify that:
  - `issue_id` exists and `state` is not `"closed"`
  - `linked_skill_id` and `linked_session_id` match those in `active-skill.json`
  - Commit message matches `required_commit_message_pattern` (or higher-level template like `[ref: ISSUE_ID]`)

- On PHI LOOP completion (successful commit), system **SHOULD**:
  - Update local `state` in this file (e.g., `"in_progress"` → `"needs_review"`)
  - Optionally sync issue status in external system (outside minimal contract)

### 2.3. `.trinity/experience/episodes.jsonl`

Purpose: Append-only journal of PHI LOOP episodes (each line = one completed loop).

Format: **JSON Lines**, one line per episode. Example:

```json
{
  "episode_id": "phi-2026-04-04T06:50:03Z#1",
  "skill_id": "tri-constitution",
  "session_id": "2026-04-04T06:45:12Z#1",
  "issue_id": "TTT-1234",
  "spec_paths": [
    "docs/nona-03-manifest/SOUL.md",
    ".trinity/cells/registry.json"
  ],
  "spec_hash_before": "sha256:abc...",
  "spec_hash_after": "sha256:def...",
  "gen_hash_after": "sha256:ghi...",
  "tests": {
    "status": "passed",           // "passed" | "failed" | "skipped"
    "failed_tests": [],
    "duration_ms": 12345
  },
  "verdict": {
    "toxicity": "clean",          // "clean" | "risky" | "toxic"
    "score": 0.02,
    "notes": "no obvious issues"
  },
  "bench_delta": {
    "metric": "none",
    "value": 0.0,
    "unit": "N/A"
  },
  "commit": {
    "sha": "ae12bc34...",
    "message": "feat: enforce PHI LOOP guards [ref: 1234]",
    "timestamp": "2026-04-04T06:50:03Z"
  },
  "actor": "agent:tri-doctor",
  "sealed_at": "2026-04-04T06:49:40Z",
  "completed_at": "2026-04-04T06:50:03Z",
  "metadata": {
    "environment": "local",
    "tri_version": "0.1.0",
    "notes": []
  }
}
```

Contract:

- File **MUST** use only "append" operations. Modifying or deleting already recorded lines **MUST NOT** occur in normal PHI LOOP operation.
- Each line **MUST** be valid JSON containing minimum:
  - `episode_id`, `skill_id`, `session_id`, `issue_id`
  - `spec_hash_before`, `spec_hash_after`, `gen_hash_after`
  - `tests.status`
  - `verdict.toxicity`
  - `commit.sha`, `commit.message`, `commit.timestamp`
  - `sealed_at`, `completed_at`

- Any analysis/diagnostics (tri doctor, tri analytics) **SHOULD** use this file as primary source of truth for past episodes.

---

## 3. Guard Pseudocode

### 3.1. Before Commit

```python
def guard_commit():
    active_skill = load_json(".trinity/state/active-skill.json")
    issue_binding = load_json(".trinity/state/issue-binding.json")

    # Check active skill
    if not active_skill or active_skill["status"] != "active":
        raise Error("ERROR: Cannot commit without active skill. Run: tri skill begin --issue <ID>")

    # Check issue binding
    if not issue_binding or not issue_binding["issue_id"]:
        raise Error("ERROR: Cannot commit without issue binding.")

    # Verify consistency
    if active_skill["issue_id"] != issue_binding["issue_id"]:
        raise Error("ERROR: Skill and issue binding mismatch.")

    # Check commit message
    commit_msg = get_git_commit_message()
    pattern = issue_binding["required_commit_message_pattern"]
    if not re.search(pattern, commit_msg):
        raise Error(f"ERROR: Commit message must match pattern: {pattern}")

    return GREEN
```

### 3.2. Before Mutation

```python
def guard_mutation(target_path):
    active_skill = load_json(".trinity/state/active-skill.json")

    if not active_skill or active_skill["status"] != "active":
        raise Error("ERROR: Cannot mutate without active skill. Run: tri skill begin --issue <ID>")

    # Check path is allowed
    if not any(target_path.startswith(p) for p in active_skill["allowed_paths"]):
        raise Error(f"ERROR: Path '{target_path}' not in allowed_paths for skill '{active_skill['skill_id']}'")

    return GREEN
```

---

## 4. Example: PHI LOOP Status Display

```
PHI LOOP Status (2026-04-04)

Queen Health: GREEN (1.0)

Trinity Coordination:
  Active Skill:   none
  Active Episode: none
  Issue:          none
  Policy State:   VIOLATION
  Reason:         Changes exist without tri skill begin

Uncommitted Changes:
  - docs/nona-03-manifest/SOUL.md — Laws #6, #7 (Constitution)
  - specs/numeric/gf4.t27 — GoldenFloat4 [S:1][E:1][M:2]

Guard:
  NO-COMMIT-WITHOUT-ISSUE: BLOCKED (no active skill)
  NO-MUTATION-WITHOUT-SKILL: BLOCKED (no active skill)

Available Actions:
  1. tri skill begin --issue <ID> --description "<task>"
  2. tri status only
```

With active skill:

```
PHI LOOP Status (2026-04-04)

Queen Health: GREEN (1.0)

Trinity Coordination:
  Active Skill:   tri-pipeline
  Active Episode: numeric-standard-001-gf4
  Issue:          #42 — NUMERIC-STANDARD-001 Recovery
  Agent:          S
  Verdict:        pending
  Experience:     unsaved

Uncommitted Changes:
  - docs/nona-03-manifest/SOUL.md — Laws #6, #7
  - specs/numeric/gf4.t27 — GoldenFloat4 [S:1][E:1][M:2]

Guard:
  NO-COMMIT-WITHOUT-ISSUE: GREEN (issue #42 linked)
  NO-MUTATION-WITHOUT-SKILL: GREEN (skill tri-pipeline active)

Available Actions:
  1. tri gen specs/numeric/gf4.t27
  2. tri test specs/numeric/gf4.t27
  3. tri verdict --toxic specs/numeric/gf4.t27
  4. tri experience save
  5. tri skill commit
```
