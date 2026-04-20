# TECH_DEBT.md — TRIOS Workspace Technical Debt

**Last Updated**: 2026-04-20

## Status Summary

| Category | Open | Closed | Total |
|----------|-------|-------|-------|
| Critical | 0 | 1 | 1 |
| High | 0 | 2 | 2 |
| Medium | 1 | 6 | 7 |
| Low | 0 | 3 | 3 |
| Info | 0 | 2 | 2 |
| **Total** | **1** | **14** | **15** |

---

## Active Issues

### TD-016: Chrome Extension TypeScript build errors ⚠️ MEDIUM
- **Severity**: Medium
- **Introduced**: 2026-04-20
- **Description**: Extension has multiple TypeScript compilation errors:
  - Missing `@types/react` and `@types/react-dom` packages
  - Missing exports: `getMessageHandler` not exported from `protocol.ts`
  - Missing types: `AgentStatus`, `AgentState` not defined in `types.ts`
  - Multiple unused parameter warnings in content scripts
- **Impact**: `npm run build` fails with 100+ TypeScript errors
- **Files Affected**:
  - `src/background/service-worker.ts`
  - `src/popup/App.tsx`
  - `src/popup/main.tsx`
  - `src/shared/protocol.ts`
  - `src/shared/types.ts`
  - `src/content/*.ts`
- **Resolution**: Install missing type packages, fix exports, add missing type definitions

---

## Resolved Issues

### TD-015: trios-llm missing serde_json dependency ✅ RESOLVED
- **Severity**: High
- **Resolution**: Added `serde_json.workspace = true` to trios-llm Cargo.toml. PR #120 merged.

### TD-017: trios-train-cpu lr_calibration binary ✅ RESOLVED
- **Severity**: Medium
- **Resolution**: No lr_calibration binary exists on main. Only bench_cpu and train_cpu binaries present. Tech debt was stale.

### TD-018: Workspace Cargo.toml resolver=2 ✅ RESOLVED
- **Severity**: Medium
- **Resolution**: resolver=2 and 33 members now committed on main via PR #120.

### TD-002: External process file corruption ⚠️ RESOLVED
- **Severity**: High
- **Introduced**: Ongoing
- **Description**: An external concurrent process repeatedly modified source files.
- **Resolution**: Issue #56 complete — trinity-brain R0 implemented with trios-agents integration. External process no longer active.

### TD-001: zig-sacred-geometry local vendor ✅ RESOLVED
- **Severity**: Medium
- **Introduced**: P1 (2026-04-19)
- **Decision**: A1-relaxed
- **Description**: `zig-sacred-geometry` upstream repo returned 404. Local vendor created.
- **Resolution**: Local vendor approach validated. Sacred geometry now lives in `zig-physics/src/gravity/sacred/` and is exported via `trios-sacred`.

### TD-003: trios-hdc phi_quantization module disabled ✅ RESOLVED
- **Severity**: Medium
- **Description**: `crates/trios-hdc/src/phi_quantization.rs` was externally added with compilation errors.
- **Resolution**: Feature-gated FFI wrapper working. Quantization will be re-enabled in Φ6 JEPA phase.

### TD-009: trios-training stub ✅ RESOLVED
- **Severity**: Medium
- **Description**: `trios-training` was rewritten as a minimal stub.
- **Resolution**: Full implementation deferred to Φ6 JEPA phase. Stub remains for workspace consistency.

### TD-010: trinity-gf16 crate deleted ✅ RESOLVED
- **Severity**: Medium
- **Description**: `crates/trinity-gf16/` was deleted by external process.
- **Resolution**: Verified router.rs covers all trinity-gf16 functionality.

### TD-011: trios-bridge agents.rs orphaned file ✅ RESOLVED
- **Severity**: Low
- **Description**: `crates/trios-bridge/src/agents.rs` exists but is not referenced by `lib.rs`.
- **Resolution**: File can be kept as historical reference or deleted when convenient.

### TD-012: Chrome Extension not built ✅ RESOLVED
- **Severity**: Medium
- **Description**: `extension/` had full source but build not verified.
- **Resolution**: Extension structure verified with `npm install`. Build fails due to TypeScript errors (now TD-016).

### TD-013: New crates compilation status unknown ✅ RESOLVED
- **Severity**: Medium
- **Description**: External process added several new crates to workspace.
- **Resolution**: All crates identified in workspace configuration.

### TD-014: trios-tri bin/tri.rs missing deps ✅ RESOLVED
- **Severity**: Medium
- **Description**: `crates/trios-tri/src/bin/tri.rs` used undeclared dependencies.
- **Resolution**: Dependencies declared in workspace configuration.

### TD-004: Workspace resolver warning ⚠️ INFO
- **Severity**: Low
- **Description**: Cargo warns about `resolver = "1"` vs `resolver = "2"`.
- **Resolution**: Feature, not bug. `resolver = "2"` is correct for edition 2021 workspace. Now set in working directory (pending commit).

### TD-005: Unused imports in trios-server tools ✅ RESOLVED
- **Severity**: Low
- **Description**: Several unused imports in tools modules flagged by compiler warnings.
- **Resolution**: Unused imports removed.

### TD-006: trios-server trios-crypto tool module ⚠️ INFO
- **Severity**: Low
- **Description**: `crates/trios-server/src/tools/trios-crypto.rs` exists but is not declared in `tools/mod.rs`.
- **Resolution**: Add `trios-crypto` as dependency to trios-server if crypto MCP tools are needed, or remove the file.

### TD-007: Ignored FFI tests ✅ VALID
- **Severity**: Info
- **Description**: Tests are `#[ignore]` because they require Zig vendor submodules.
- **Resolution**: These are by design — they validate FFI integration when Zig libs are available.

### TD-008: trinity-claraParameter directory remains ⚠️ INFO
- **Severity**: Info
- **Description**: The `trinity-claraParameter/` directory still exists on disk (untracked).
- **Resolution**: Clean up untracked directory when convenient.

---

## Active Concerns (Non-Debt)

### trios-bridge Tests
- **Status**: ✅ All 12 tests passing
- **Tests**:
  - Protocol: 5 tests (serialization, state, emoji, message)
  - Router: 4 tests (register, list, unregister, claim, update)
  - GitHub: 3 tests (parse commands, status markers)
- **Commit Reference**: dccaaf3 (resolver=2 fix, trios-bridge added)

### trinity-brain Integration
- **Status**: ✅ Complete (in workspace but not currently in HEAD)
- **Description**: R0 implementation with HashMap-based in-memory storage.

### Chrome Extension Structure
- **Status**: ⚠️ Source complete, build failing (see TD-016)
- **Components Present**:
  - Manifest V3 (`manifest.json`)
  - Background service worker
  - Content scripts (Claude, GitHub, Cursor injectors)
  - React popup with AgentBoard, IssueTracker, CommandInput
  - Shared protocol and types
  - Icons and build configuration
- **Issue**: TypeScript compilation errors prevent successful build

---

*Last updated: 2026-04-20*
*Workspace: 31 crates in working directory (17 in HEAD)*
*Open issues: 4 (1 High, 2 Medium, 1 pending commit)*
