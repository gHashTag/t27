# /tri Core Issues and Improvement Plan

## Problem Statement

The `/tri` CLI currently operates in "detection mode" instead of following constitutional protocol. It detects file changes and spec modifications but skips the critical coordination layer, creating a disconnect between agent actions and swarm state.

## Constitutional Violation

**SOUL.md Law #6 (Akashic Coordination First):**
> "Before any task, every agent must read `.trinity` Akashic Chronicle, inspect active claims, queue, and swarm state, then acquire an exclusive claim on its target spec_path or graph_node. No mutation without prior read + claim."

**What `/tri` currently does:**
```
1. Detects SOUL.md changes ✅
2. Detects gf4.t27 changes ✅
3. Says "No active claims" ❌ (не читал .trinity/claims!)
4. Asks what to do instead of following protocol
```

**Missing steps:**
1. ❌ Read `.trinity/events/akashic-log.jsonl`
2. ❌ Read `.trinity/claims/active/`
3. ❌ Check `.trinity/state/queen-health.json`
4. ❌ Check if target resource is claimed
5. ❌ Acquire claim (if available)
6. ❌ Record `task.intent` event

## Current Architecture Issues

### Single Responsibility Problem

`/tri` mixes detection logic with action prompts. This violates:
- **Single Responsibility Principle**: One component should do one thing well
- **Separation of Concerns**: Coordination (read .trinity, check claims) vs Action (run tri gen, commit)
- **Unclear Control Flow**: User can't see what happened and what's happening

### Coordination Disconnect

The coordination layer (`.trinity/`) is created but not integrated:
- Doctor agent exists and runs independently
- Event schemas are defined
- Scripts (`swarm-health.sh`, `replay-step.sh`) are ready
- BUT `/tri` doesn't use them before showing options

## Improvement Plan: Native Coordination for /tri

### Phase 1: Coordination Foundation

**Goal:** Embed `.trinity` read directly into `/tri` before any action.

**Changes needed:**

1. **Import `.trinity` package** — Create Go module for reading event log, claims, state
   ```go
   package trinity

   func ReadEvents() []Event
   func ReadClaims() map[string]*Claim
   func ReadState() State

   // Append-only reader
   func ReadAkashicLog() ([]Event, error) {
       file, _ := os.Open(".trinity/events/akashic-log.jsonl")
       defer file.Close()
       // Stream read, append to list
       scanner := bufio.NewScanner(file)
       for scanner.Scan() {
           var event Event
           if err := json.Unmarshal(scanner.Bytes(), &event); err == nil {
               events = append(events, event)
           }
       }
       return events, nil
   }
   ```

2. **Create coordination module** — `pkg/coordination/`
   ```go
   package coordination

   type Coordinator struct {
       events   []Event
       claims   map[string]*Claim
       state    State
       agentID  string
   }

   func NewCoordinator() *Coordinator {
       // Load on init
       events, _ := ReadAkashicLog()
       claims, _ := ReadClaims()
       state, _ := ReadState()
       return &Coordinator{events, claims, state}
   }

   func CheckClaim(resource string) (bool, error) {
       claim, exists := claims[resource]
       if !exists {
           return true, nil  // Available
       }
       // Check if expired
       if time.Now().After(time.Unix(expires_at, 0)) {
           return true, nil  // Stale, can reclaim
       }
       return false, fmt.Errorf("claimed by %s", claim.AgentID)
   }
   ```

3. **Initialize coordinator** in `/tri` main
   ```go
   var coord = coordination.NewCoordinator()
   ```

### Phase 2: Claim Protocol

**Goal:** Implement claim acquisition and release inline with PHI LOOP actions.

**Changes needed:**

1. **Before any `tri <subcommand>` operation:**
   ```go
   func preCommandCheck(cmd []string) error {
       // Check if target is claimed
       resource := getTargetResource(cmd)
       if available, claimed, _ := coord.CheckClaim(resource)
       if !claimed {
           return fmt.Errorf("resource not available: %s (claimed by %s until %s)",
               resource, claim.AgentID)
       }
       return nil
   }
   ```

2. **Add `acquire` subcommand:**
   ```go
   // Automatically acquire claim before mutating
   tri gen --acquire specs/numeric/gf4.t27
   ```

3. **Add claim release to cleanup:**
   ```go
   func cleanupAfterSuccess(resource string) {
       // Release claim after successful tri skill commit
       releaseClaim(resource, "clean")
   }

   func cleanupAfterFailure(resource string) {
       // Release claim if toxic verdict
       releaseClaim(resource, "toxic")
   }
   ```

### Phase 3: Event Logging

**Goal:** All agent actions append to `.trinity/events/akashic-log.jsonl`.

**Changes needed:**

1. **Auto-logging wrapper:**
   ```go
   func LogEvent(event Event) error {
       // Auto-append to akashic-log.jsonl
       record := fmt.Sprintf(`{"ts":"%s","event":"%s",...}`,
           time.Now().Format(time.RFC3339Nano), event.Type, ...)
       appendToFile(".trinity/events/akashic-log.jsonl", record)
   }
   ```

2. **Events for PHI LOOP:**
   - `task.intent` — Before starting mutation
   - `task.started` — When mutation begins
   - `task.completed` — On clean verdict
   - `task.failed` — On toxic verdict

### Phase 4: Doctor Integration

**Goal:** Doctor agent reads `.trinity` events directly, not via CLI.

**Changes needed:**

1. **Doctor reads `.trinity/events/akashic-log.jsonl`** directly
2. **No CLI intermediary** — Don't go through `/tri` state check

### Phase 5: Improved Movement

**Goal:** Support iterative development with clear next steps.

**Output format:**

```
Current Status:
  Queen Health: 0.95 (GREEN)
  Swarm Health: 0.88 (GREEN)
  Last Handoff: loop-session-550e8400-1234-4b5a-9c6d-7e8f9a0b1c2

Active Claim: specs/numeric/gf4.t27
  Owner: agent-spec-1
  Expires: 2026-04-04T13:15:00Z

────────────────────────────────────
Recommended Next Steps:
  1. Commit existing changes (seal hash + verify + commit)
  2. Continue work on GF4 (tri gen + tri test)
  3. Start new spec (tri skill begin)
  4. Doctor health check only
```

## Implementation Priority

1. **HIGH** — Coordination foundation (Phase 1) - prevents coordination failures
2. **MEDIUM** — Claim protocol (Phase 2) - enables proper resource ownership
3. **MEDIUM** — Event logging (Phase 3) - ensures traceability
4. **LOW** — Doctor integration (Phase 4) - removes CLI layer dependency
5. **LOW** — Movement improvements (Phase 5) - better user experience

## Success Criteria

1. `/tri gen` automatically acquires claim before generating
2. `/tri` fails if resource already claimed
3. `/tri` releases claim on commit (clean) or verdict (toxic)
4. `/tri` shows claim status in all status messages
5. `/tri` Doctor reads `.trinity` directly for health monitoring

## Risks

- **Complexity:** Adding Go module and coordination layer increases complexity
- **Breaking Change:** Current `--acquire` behavior may break existing workflows
- **Performance:** Reading `.trinity` before every action adds overhead

## Mitigation

1. **Start with minimal change:** Only add pre-command check, don't change all output
2. **Phased rollout:** Implement coordination foundation first, then claim protocol
3. **Backward compatibility:** Keep existing behavior as option flag (`--legacy-mode`)

## Quick Win

Instead of full rewrite, apply minimal fix:

```go
// In preCommandCheck, add .trinity read
func preCommandCheck(cmd []string) error {
    resource := getTargetResource(cmd)

    // Quick check: is it claimed?
    claim, exists := coord.CheckClaim(resource)
    if exists {
        return fmt.Errorf("%s claimed by %s (expires %s)",
            resource, claim.AgentID)
    }

    // NOT blocking: just warning
    log.Warn("Resource claimed, consider: tri release %s", resource)
    return nil  // Allow operation with warning
}
```

This preserves existing functionality while improving coordination reliability.
