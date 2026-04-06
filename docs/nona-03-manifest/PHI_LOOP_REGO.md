# PHI LOOP Policy-as-Code — Pseudo-Rego/OPA Rules

Version: 1.0
Purpose: Implementable guard rules for tri CLI

---

## 1. Rule Definitions

### 1.1. `allow_commit`

```rego
package phi_loop

default allow_commit = false

allow_commit {
    # Has active skill
    input.active_skill.status == "active"
    input.active_skill.skill_id != null

    # Has issue binding
    input.issue_binding.issue_id != null
    input.issue_binding.state != "closed"

    # Skill and issue are consistent
    input.active_skill.issue_id == input.issue_binding.issue_id
    input.active_skill.session_id == input.issue_binding.linked_session_id

    # Commit message has issue reference
    regex.match(input.issue_binding.required_commit_message_pattern, input.commit_message)
}
```

**Error messages:**
```rego
deny_commit[msg] {
    not input.active_skill.status == "active"
    msg := "ERROR: Cannot commit without active skill. Run: tri skill begin --issue <ID>"
}

deny_commit[msg] {
    input.active_skill.status == "active"
    not input.issue_binding.issue_id
    msg := "ERROR: Cannot commit without issue binding. Run: tri skill begin --issue <ID>"
}

deny_commit[msg] {
    input.active_skill.issue_id != input.issue_binding.issue_id
    msg := sprintf("ERROR: Skill issue %s != binding issue %s. State desync!", [
        input.active_skill.issue_id, input.issue_binding.issue_id
    ])
}

deny_commit[msg] {
    not regex.match(input.issue_binding.required_commit_message_pattern, input.commit_message)
    msg := sprintf("ERROR: Commit message must match pattern: %s", [input.issue_binding.required_commit_message_pattern])
}
```

### 1.2. `allow_mutation`

```rego
default allow_mutation = false

allow_mutation {
    # Has active skill
    input.active_skill.status == "active"
    input.active_skill.skill_id != null

    # Target path is allowed
    allowed_path(input.target_path)
}

allowed_path(path) {
    some i
    input.active_skill.allowed_paths[i]
    startswith(path, input.active_skill.allowed_paths[i])
}
```

**Error messages:**
```rego
deny_mutation[msg] {
    not input.active_skill.status == "active"
    msg := "ERROR: Cannot mutate without active skill. Run: tri skill begin --issue <ID>"
}

deny_mutation[msg] {
    input.active_skill.status == "active"
    not allowed_path(input.target_path)
    msg := sprintf("ERROR: Path '%s' not in allowed_paths for skill '%s'", [
        input.target_path, input.active_skill.skill_id
    ])
}
```

### 1.3. `allow_seal`

```rego
default allow_seal = false

allow_seal {
    # Has active skill
    input.active_skill.status == "active"
    input.active_skill.skill_id != null

    # Seal hash is provided
    input.seal_hash != ""

    # Seal hash matches current state
    input.seal_hash == calculate_seal_hash(input.spec_paths)
}
```

### 1.4. `get_status`

```rego
get_status[status] {
    # Check if active skill exists
    has_active_skill
    status := {
        "queen_health": input.queen_health,
        "coordination": {
            "active_skill": input.active_skill.skill_id,
            "active_episode": get_active_episode(),
            "issue": sprintf("#%s — %s", [input.issue_binding.issue_id, input.issue_binding.title]),
            "agent": input.active_skill.started_by,
            "verdict": get_verdict_status(),
            "experience": get_experience_status()
        },
        "guard": "GREEN",
        "policy_state": "COMPLIANT"
    }
}

get_status[status] {
    # No active skill but has changes
    not has_active_skill
    count(input.uncommitted_changes) > 0
    status := {
        "queen_health": input.queen_health,
        "coordination": {
            "active_skill": "none",
            "active_episode": "none",
            "issue": "none",
            "agent": "none",
            "verdict": "n/a",
            "experience": "n/a"
        },
        "guard": "RED",
        "policy_state": "VIOLATION",
        "reason": "Changes exist without tri skill begin"
    }
}

get_status[status] {
    # No active skill, no changes
    not has_active_skill
    count(input.uncommitted_changes) == 0
    status := {
        "queen_health": input.queen_health,
        "coordination": {
            "active_skill": "none",
            "active_episode": "none",
            "issue": "none"
        },
        "guard": "GREEN",
        "policy_state": "IDLE"
    }
}

has_active_skill {
    input.active_skill.status == "active"
    input.active_skill.skill_id != null
}
```

---

## 2. CLI Command Specifications

### 2.1. `tri skill begin`

```bash
tri skill begin --issue <ID> --description "<text>" [--skill <skill_id>]
```

**Implementation:**

```python
def cmd_skill_begin(issue_id: str, description: str, skill_id: str = None):
    # Load current state
    active_skill = load_json(".trinity/state/active-skill.json")
    issue_binding = load_json(".trinity/state/issue-binding.json")

    # Check if already active
    if active_skill.get("status") == "active":
        if active_skill.get("issue_id") != issue_id:
            raise Error(f"ERROR: Active skill '{active_skill['skill_id']}' already bound to issue {active_skill['issue_id']}. Close it first.")
        else:
            print(f"INFO: Already in skill '{active_skill['skill_id']}' for issue {issue_id}")
            return

    # Auto-detect skill from issue/description if not provided
    if skill_id is None:
        skill_id = detect_skill_from_context(issue_id, description)

    # Create session ID
    session_id = f"{datetime.utcnow().isoformat()}#{uuid4().hex[:4]}"

    # Determine allowed paths from skill
    allowed_paths = get_skill_allowed_paths(skill_id)

    # Write active-skill.json
    write_json(".trinity/state/active-skill.json", {
        "skill_id": skill_id,
        "session_id": session_id,
        "issue_id": issue_id,
        "description": description,
        "started_at": datetime.utcnow().isoformat(),
        "started_by": f"human:{getuser()}",
        "status": "active",
        "allowed_paths": allowed_paths,
        "metadata": {
            "priority": "normal",
            "tags": [],
            "origin": "cli"
        }
    })

    # Write issue-binding.json
    issue_data = fetch_issue_data(issue_id)  # From GitHub/Linear/etc
    write_json(".trinity/state/issue-binding.json", {
        "issue_id": issue_id,
        "source": issue_data["source"],
        "url": issue_data["url"],
        "title": issue_data["title"],
        "state": "in_progress",
        "linked_skill_id": skill_id,
        "linked_session_id": session_id,
        "last_synced_at": datetime.utcnow().isoformat(),
        "required_commit_message_pattern": f"\\[ref: {issue_id}\\]",
        "metadata": {
            "assignee": issue_data.get("assignee"),
            "labels": issue_data.get("labels", []),
            "repository": issue_data.get("repository")
        }
    })

    print(f"✓ Skill '{skill_id}' started for issue #{issue_id}")
    print(f"  Session: {session_id}")
    print(f"  Allowed paths: {', '.join(allowed_paths)}")
```

### 2.2. `tri skill end` / `tri skill close`

```bash
tri skill end [--force]
```

**Implementation:**

```python
def cmd_skill_end(force: bool = False):
    active_skill = load_json(".trinity/state/active-skill.json")

    if active_skill.get("status") != "active":
        print("INFO: No active skill to close.")
        return

    # Check if experience was saved
    episodes = load_jsonl(".trinity/experience/episodes.jsonl")
    last_episode = episodes[-1] if episodes else None

    if not force and (not last_episode or last_episode.get("session_id") != active_skill.get("session_id")):
        print("WARNING: Experience not saved for this session.")
        print("  Run: tri experience save")
        print("  Or force close with: tri skill end --force")
        return

    # Close skill
    write_json(".trinity/state/active-skill.json", {
        "skill_id": None,
        "session_id": None,
        "issue_id": None,
        "description": None,
        "started_at": None,
        "started_by": None,
        "status": "closed",
        "allowed_paths": [],
        "metadata": {}
    })

    # Clear issue-binding
    write_json(".trinity/state/issue-binding.json", {
        "issue_id": None,
        "source": None,
        "url": None,
        "title": None,
        "state": None,
        "linked_skill_id": None,
        "linked_session_id": None,
        "last_synced_at": None,
        "required_commit_message_pattern": "\\[ref: ISSUE_ID\\]",
        "metadata": {}
    })

    print("✓ Skill closed. PHI LOOP session ended.")
```

### 2.3. `tri status`

```bash
tri status [--only]
```

**Implementation:**

```python
def cmd_status(only_mode: bool = False):
    # Load state
    active_skill = load_json(".trinity/state/active-skill.json")
    issue_binding = load_json(".trinity/state/issue-binding.json")
    queen_health = load_json(".trinity/state/queen-health.json")

    # Get git status
    git_status = get_git_status()

    # Determine status state
    has_active = active_skill.get("status") == "active"
    has_changes = len(git_status["modified"]) + len(git_status["untracked"]) > 0

    # Print header
    print("⏺ PHI LOOP Status ({})".format(datetime.now().strftime("%Y-%m-%d")))
    print()

    # Queen health
    health_val = queen_health.get("value", 1.0)
    health_color = "GREEN" if health_val >= 0.9 else "YELLOW" if health_val >= 0.7 else "RED"
    print(f"  Queen Health: {health_color} ({health_val})")
    print()

    # Coordination block
    print("  Trinity Coordination:")

    if has_active:
        skill_name = active_skill.get("skill_id", "unknown")
        episode = get_current_episode(active_skill.get("session_id"))
        issue_id = issue_binding.get("issue_id", "unknown")
        issue_title = issue_binding.get("title", "")
        agent = active_skill.get("started_by", "unknown")
        verdict = get_verdict_status()
        exp_status = get_experience_status()

        print(f"    Active Skill:   {skill_name}")
        print(f"    Active Episode: {episode or 'pending'}")
        print(f"    Issue:          #{issue_id} — {issue_title}")
        print(f"    Agent:          {agent}")
        print(f"    Verdict:        {verdict}")
        print(f"    Experience:     {exp_status}")
        guard_state = "GREEN"
        policy_state = "COMPLIANT"
    elif has_changes:
        print("    Active Skill:   none")
        print("    Active Episode: none")
        print("    Issue:          none")
        print("    Policy State:   VIOLATION")
        print("    Reason:         Changes exist without tri skill begin")
        guard_state = "RED"
        policy_state = "VIOLATION"
    else:
        print("    Active Skill:   none")
        print("    Active Episode: none")
        print("    Issue:          none")
        guard_state = "GREEN"
        policy_state = "IDLE"

    print()
    print("  Guard:")
    print(f"    NO-COMMIT-WITHOUT-ISSUE: {guard_state}")
    print(f"    NO-MUTATION-WITHOUT-SKILL: {guard_state}")

    if not has_active and has_changes:
        print()
        print("  ERROR: Cannot commit or seal without active skill + issue.")
        print()
        print("  Required Actions:")
        print("    1. tri skill begin --issue <N> --description \"<task>\"")
        print("    2. tri status only (current view)")
        print()
        print_available_skills()

    # Uncommitted changes
    if has_changes:
        print()
        print("  Uncommitted Changes:")
        for f in git_status["modified"]:
            print(f"  - M {f}")
        for f in git_status["untracked"]:
            print(f"  - ?? {f}")

    if only_mode:
        return

    # Available actions
    if has_active:
        print()
        print("  Available Actions:")
        print("    1. tri gen <spec>")
        print("    2. tri test <spec>")
        print("    3. tri verdict --toxic <spec>")
        print("    4. tri experience save")
        print("    5. tri skill commit")
```

---

## 3. Skill-to-Path Mapping

```python
SKILL_ALLOWED_PATHS = {
    "tri-pipeline": [
        "specs/numeric/",
        "docs/nona-02-organism/NUMERIC-STANDARD-001.md",
        "docs/GF_FAMILY_BENCH.md"
    ],
    "tri-sacred": [
        "specs/math/constants.t27",
        "specs/math/sacred_physics.t27",
        "docs/nona-02-organism/SACRED-PHYSICS-001.md"
    ],
    "tri-base": [
        "specs/base/types.t27",
        "specs/base/ops.t27"
    ],
    "tri-constitution": [
        "docs/nona-03-manifest/SOUL.md",
        "docs/ADR-*.md",
        ".trinity/cells/registry.json",
        ".trinity/policy/",
        ".trinity/state/",
        "docs/agents/AGENTS.md",
        "docs/nona-03-manifest/PHI_LOOP_CONTRACT.md"
    ]
}

def get_skill_allowed_paths(skill_id: str) -> List[str]:
    return SKILL_ALLOWED_PATHS.get(skill_id, [])

def detect_skill_from_context(issue_id: str, description: str) -> str:
    """Auto-detect skill from issue title/description"""
    desc_lower = (issue_id + " " + description).lower()

    if "numeric" in desc_lower or "gf" in desc_lower:
        return "tri-pipeline"
    elif "sacred" in desc_lower or "phi" in desc_lower:
        return "tri-sacred"
    elif "constitution" in desc_lower or "soul" in desc_lower:
        return "tri-constitution"
    elif "base" in desc_lower or "trit" in desc_lower:
        return "tri-base"
    else:
        return "tri-pipeline"  # Default
```

---

## 4. Example: Full PHI LOOP Session

```bash
# 1. Start skill
$ tri skill begin --issue 42 --description "GF4 spec for NUMERIC-STANDARD-001"
✓ Skill 'tri-pipeline' started for issue #42
  Session: 2026-04-04T14:30:00Z#a1b2
  Allowed paths: specs/numeric/, docs/nona-02-organism/NUMERIC-STANDARD-001.md

# 2. Check status
$ tri status
⏺ PHI LOOP Status (2026-04-04)
  Queen Health: GREEN (1.0)
  Trinity Coordination:
    Active Skill:   tri-pipeline
    Active Episode: pending
    Issue:          #42 — NUMERIC-STANDARD-001 Recovery
    Agent:          human:you
    Verdict:        pending
    Experience:     unsaved
  Guard: GREEN

# 3. Do work (allowed paths only)
$ tri spec create specs/numeric/gf4.t27
$ tri gen specs/numeric/gf4.t27
$ tri test specs/numeric/gf4.t27
$ tri verdict --toxic specs/numeric/gf4.t27

# 4. Save experience
$ tri experience save
✓ Episode saved: phi-2026-04-04T14:45:00Z#1

# 5. Commit (allowed now!)
$ tri skill commit
✓ Commit created: ae12bc34... feat: GF4 spec [ref: 42]
✓ Skill closed.
```

---

## 5. Git Hook Integration

### `.git/hooks/pre-commit`

```bash
#!/bin/bash
# Pre-commit hook for PHI LOOP enforcement

# Check if tri CLI is being used
if ! git config --get tri.session.active > /dev/null 2>&1; then
    # Not using tri CLI, block commit
    echo "ERROR: Commits must go through tri CLI for PHI LOOP enforcement."
    echo "Run: tri skill commit"
    exit 1
fi

# Check active skill
ACTIVE_SKILL=$(jq -r '.status' .trinity/state/active-skill.json)
if [ "$ACTIVE_SKILL" != "active" ]; then
    echo "ERROR: Cannot commit without active skill."
    echo "Run: tri skill begin --issue <ID>"
    exit 1
fi

# Check issue binding
ISSUE_ID=$(jq -r '.issue_id' .trinity/state/issue-binding.json)
if [ "$ISSUE_ID" = "null" ]; then
    echo "ERROR: Cannot commit without issue binding."
    exit 1
fi

# Check commit message format
COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")
if ! echo "$COMMIT_MSG" | grep -q "\[ref: $ISSUE_ID\]"; then
    echo "ERROR: Commit message must contain '[ref: $ISSUE_ID]'"
    exit 1
fi

exit 0
```

Install: `cp .githooks/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit`
