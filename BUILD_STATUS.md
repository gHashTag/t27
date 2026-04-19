# BUILD_STATUS.md — Trinity Ecosystem Full Architecture Map

**Updated:** 2026-04-19 18:00 +07  
**Zig:** 0.16.0 | **Rust:** stable | **Workspace:** 12 crates

---

## Layer 1: T27 — Trinity Specification Layer (SSOT)

Repo: `gHashTag/t27`

| # | Module | Path | Type | Status | Priority |
|---|--------|------|------|--------|----------|
| 1 | t27 language core | `specs/tri/*.t27` | Spec | ✅ LIVE, PHI-loop, 30+ modules | core |
| 2 | Meta-compiler | `specs/compiler/meta_compile.t27` | Spec | 🟡 4-5 backends in PR (#521/#529/#532) | critical |
| 3 | GF16 / numeric core | `specs/numeric/gf16.t27` | Spec | 🟡 GF16 helpers + tests (PR #521) | critical |
| 4 | Native Memory System | `specs/memory/*.t27` | Spec | 🟡 Phase 0 done (#517) | high |
| 5 | Rings (RING spec) | `specs/base/ring_*.t27` | Spec | ✅ up to Ring 32 | high |
| 6 | COA / AR / DARPA CLARA | `specs/ar/coa_planning.t27` | Spec | ✅ linked with CLARA docs | critical |
| 7 | specs/trios/*.t27 | `specs/trios/` | Spec | 📋 PLANNED | P2 |
| 8 | specs/clara/*.t27 | `specs/clara/` | Spec | 📋 PLANNED | P3 |
| 9 | specs/agi/*.t27 | `specs/agi/` | Spec | 📋 PLANNED | P3 |
| 10 | ARCHITECTURE-MULTIREPO.md | root | docs | ✅ committed | done |
| 11 | TECH_DEBT.md | root | docs | 📋 create | NOW |
| 12 | TS codegen (PR #529) | tooling | tool | 🟡 PR pending | CI queue |
| 13 | bootstrap (PR #524) | tooling | tool | 🟡 PR pending | CI queue |
| 14 | GF16 backend (PR #521) | tooling | tool | 🟡 PR pending | CI queue |
| 15 | All backends (PR #532) | tooling | tool | 🟡 PR pending | CI queue |
| 16 | Coq formal verification | research | verification | 📋 planned | long-term |

---

## Layer 2: Zig Ecosystem

### 2.1 Live Zig Repositories

| # | Module | Repo | Zig ver | Build | Static Lib | Submodule in TRIOS | Status |
|---|--------|------|:-------:|:-----:|:----------:|:------------------:|--------|
| 1 | Golden Float / GF16 | `zig-golden-float` | 0.16 ✅ | ✅ | ✅ `libgolden_float.a` | ✅ | PARTIAL — FFI symbol mismatch |
| 2 | Physics / Quantum | `zig-physics` | 0.16 ✅ | ✅ | ✅ `libphysics.a` | ✅ | GREEN |
| 3 | HDC / VSA | `zig-hdc` | 0.16 ✅ | ✅ | ✅ `libhdc.a` | ✅ | GREEN |
| 4 | Sacred Geometry | `zig-sacred-geometry` | ⚠️ | — | N/A | ❌ 404 | BLOCKED — repo needs restoration |
| 5 | Crypto-mining | `zig-crypto-mining` | 0.16 ✅ | ✅ | ✅ `libcrypto_mining.a` | ✅ | GREEN |
| 6 | Agents | `zig-agents` | 0.16 ✅ | ✅ | ✅ | separate | GREEN |
| 7 | Knowledge Graph | `zig-knowledge-graph` | ✅ | ✅ | — | not touched | LIVE |

### 2.2 Zig 0.16 Migration Applied

| Package | Changes |
|---------|---------|
| zig-golden-float | Disabled `tri_gen` (`std.process.argsAlloc` removed in 0.16); added static lib target |
| zig-hdc | Migrated `build.zig` → `createModule`/`root_module`; `build.zig.zon` fingerprint + hash; created `src/c_abi.zig` |
| zig-physics | Rewrote `build.zig` for 0.16 API; `build.zig.zon` fingerprint; created `src/c_abi.zig` (CHSH Bell + constants) |
| zig-crypto-mining | Created missing `build.zig`; `build.zig.zon` fingerprint; created `src/c_abi.zig` (SHA-256 + mining) |

### 2.3 Planned New Zig Repos (P3)

| # | Repo | Purpose | Priority |
|---|------|---------|----------|
| N1 | `zig-training` | Training utils in Zig | P3 |
| N2 | `zig-ensemble` | Ensemble orchestration | P3 D8-9 |
| N3 | `zig-agi-eval` | AGI benchmark Zig layer | P3 parallel |

---

## Layer 3: TRIOS — Rust Workspace + FFI

Repo: `gHashTag/trios`

### 3.1 Current Modules

| # | Module | Type | stub | FFI | test | Status | SSOT spec |
|---|--------|------|:----:|:---:|:----:|--------|-----------|
| 1 | trios-core | lib | ✅ | N/A | ✅ | GREEN | `specs/trios/core.t27` 📋 |
| 2 | trios-git | lib | ✅ | N/A | ⚠️ | GREEN (build) | `specs/trios/git.t27` 📋 |
| 3 | trios-gb | lib | ✅ | N/A | ✅ | GREEN | `specs/trios/gitbutler.t27` 📋 |
| 4 | trios-server | bin (MCP) | ✅ | N/A | ✅ | GREEN | `specs/trios/server.t27` 📋 |
| 5 | trios-kg | lib (HTTP) | ✅ | N/A | ✅ | GREEN | `specs/trios/kg.t27` 📋 |
| 6 | trios-agents | lib (HTTP) | ✅ | N/A | ✅ | GREEN | `specs/trios/agents.t27` 📋 |
| 7 | trios-training | lib (HTTP) | ✅ | N/A | ✅ | GREEN | `specs/trios/training.t27` 📋 |
| 8 | trios-crypto | FFI wrapper | ✅ | ✅ | ✅ | GREEN — `libcrypto_mining.a` linked | `specs/crypto/mining.t27` |
| 9 | trios-golden-float | FFI wrapper | ✅ | ⚠️ | ⚠️ | PARTIAL — missing `compress_weights` exports | `specs/golden-float/gf16.t27` |
| 10 | trios-hdc | FFI wrapper | ✅ | ✅ | ✅ | GREEN — `libhdc.a` linked | `specs/hdc/core.t27` |
| 11 | trios-physics | FFI wrapper | ✅ | ✅ | ✅ | GREEN — `libphysics.a` linked | `specs/physics/constants.t27` |
| 12 | trios-sacred | FFI wrapper | ✅ | ❌ | ❌ | FAIL — no vendor (repo 404) | `specs/sacred-geometry/phi.t27` |
| 13 | trios-zig-agents | lib (FFI) | ✅ | ✅ | GREEN — FFI wrapper for zig-agents | `specs/agents/agents.t27` |
### 3.2 Planned Modules

| # | Module | Purpose | Priority |
|---|--------|---------|----------|
| 13 | trios-clara | MCP bridge for CLARA / ParameterGolf | P2 |
| 14 | trios-zig-agents | FFI wrapper for zig-agents | P1 ✅ |
| 15 | trios-hdc-bridge | HDC→CLARA bridge | P3 D2-3 |
| 16 | trios-phi-quant | φ-quantization | P3 D4-5 |
| 17 | trios-fibonacci-attn | Fibonacci attention | P3 D6-7 |
| 18 | trios-ensemble | Ensemble orchestrator | P3 D8-9 |
| 19 | trios-agi-bench | 5 AGI tracks wrapper | P3 parallel |

### 3.3 TRIOS Summary

- **stub:** 12/12 ✅ (all crates compile without FFI)
- **FFI linked:** 4/5 ✅ (golden-float, hdc, physics, crypto — all have `.a` files)
- **FFI full:** 3/5 ✅ (hdc, physics, crypto — all symbols resolve)
- **Blocked:** 1/5 (sacred-geometry — repo 404)
- **`cargo build`:** ✅ All 12 crates
- **`cargo test`:** 7 green, 5 FFI debt (symbol mismatches + missing vendor)

---

## RED List — Blockers

| # | Blocker | Impact | Action |
|---|---------|--------|--------|
| 1 | zig-sacred-geometry repo 404 | trios-sacred FFI blocked | Restore GitHub repo |
| 2 | zig-golden-float missing `compress_weights`/`decompress_weights` | trios-golden-float test failure | Add exports to `src/c_abi.zig` |
| 3 | External concurrent file modifications | Build instability | Coordinate across agents |
| 4 | trios-git async test break | Test regression | Fix `group_files_smart` test |

---

## Build Verification

```bash
cargo build                                    # ✅ All 12 crates
cargo test                                     # 7 green, 5 FFI debt

# Zig vendor builds
cd crates/trios-golden-float/vendor/zig-golden-float && zig build   # ✅
cd crates/trios-hdc/vendor/zig-hdc && zig build                     # ✅
cd crates/trios-physics/vendor/zig-physics && zig build             # ✅
cd crates/trios-crypto/vendor/zig-crypto-mining && zig build        # ✅
```
