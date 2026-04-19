# TRIOS ECOSYSTEM — Architecture

**Date:** 2026-04-19
**Branch:** `main` (trios)
**Status:** DRAFT — Planning Document
**SSOT:** BUILD_STATUS.md (detailed), this file (high-level)

---

## 1. Existing Zig Repositories (LIVE)

| # | Module / Repo | Zig ver | Build | TRIOS FFI | Status | SSOT link README | Submodule |
|---|---|---|---|---|---|---|---|
| 1 | Golden Float / GF16 | `zig-golden-float` | 0.16 ✅ | ✅ | ✅ | PARTIAL — symbol mismatch | [docs](https://github.com/gHashTag/zig-golden-float) |
| 2 | Physics / Quantum | `zig-physics` | 0.16 ✅ | ✅ | ✅ | GREEN | [docs](https://github.com/gHashTag/zig-physics) |
| 3 | HDC / VSA | `zig-hdc` | 0.16 ✅ | ✅ | ❌ | FAIL — vendor missing | — | — |
| 4 | Sacred Geometry | `zig-sacred-geometry` | ⚠️ NO VENDOR | ❌ | ❌ | BLOCKED — repo 404 | — |
| 5 | Crypto-mining | `zig-crypto-mining` | 0.16 ✅ | ✅ | ❌ | FAIL — sha256 linker error | — | — |

**Note:** Sacred geometry (φ-attention) is already implemented in `zig-physics/src/gravity/sacred_geometry/`, separate Zig repository not required. Status: **deferred, awaiting explicit signal on merge vs resurrect** (choice A1 in audit).

---

## 2. Candidates for Integration — HIGH priority

### 2.1 `trinity-fpga` — FPGA Synthesis

| Parameter | Status | Description |
|-----------|--------|-------------|
| Source | DRAFT | [t27](https://github.com/gHashTag/trinity-fpga) — VIBEE T27 synthesis for FPGA |
| Integration | PENDING | Integration with TRIOS server MCP tools |
| Dependencies | DRAFT | VIBEE Core, Triton FPGAs (external) |
| Complexity | HIGH | Requires deep FPGA synthesis knowledge |

**Integration steps:**
1. Study VIBEE spec and existing trinity infrastructure
2. Create integration layer in trios-server for FPGA control
3. Develop pipeline: CLI request → FPGA compilation → verification → deployment

---

## 3. Candidates for Integration — MEDIUM priority

### 3.1 `trinity-bio` — Biological Computation

| Parameter | Status | Description |
|-----------|--------|-------------|
| Source | DRAFT | [t27](https://github.com/gHashTag/trinity-bio) — Bio-computation primitives |
| Integration | PENDING | Integration with zig-physics (quantum simulations) |
| Dependencies | DRAFT | VIBEE Core, zig-physics |
| Complexity | LOW | Specialized domain, low priority |

**Integration steps:**
1. Create stub for bio-operations in trios-physics
2. Develop protocols for classical-quantum hybrid computing

---

### 3.2 `trinity-brain` — Neural Interface

| Parameter | Status | Description |
|-----------|--------|-------------|
| Source | DRAFT | [t27](https://github.com/gHashTag/trinity-brain) — Brain simulation interface |
| Integration | PENDING | Integration with trios-agents (AI agents) |
| Dependencies | DRAFT | VIBEE Core, Neural network libraries (external) |
| Complexity | MEDIUM | Bridges to existing AI systems |

**Integration steps:**
1. Study trios-agents architecture
2. Implement bidirectional communication agents ↔ brain
3. Create protocols for hardware neural inference

---

### 3.3 `trinity-websocket` — WebSocket Layer

| Parameter | Status | Description |
|-----------|--------|-------------|
| Source | DRAFT | — WebSocket transport for TRIOS |
| Integration | PENDING | Integration into trios-server |
| Dependencies | DRAFT | trios-server |
| Complexity | LOW | Low complexity, pure infrastructure |

**Integration steps:**
1. Create `crates/trios-ws` crate
2. Implement WebSocket handlers for MCP
3. Test performance and security

---

### 3.4 `trinity-rpc` — RPC Layer

| Parameter | Status | Description |
|-----------|--------|-------------|
| Source | DRAFT | — RPC protocol for trios |
| Integration | PENDING | Integration into trios-server |
| Dependencies | DRAFT | trios-server |
| Complexity | MEDIUM | Standard RPC, requires integration tests |

**Integration steps:**
1. Select RPC framework (json-rpc, tarpc)
2. Implement bidirectional RPC over MCP
3. Create middleware for authorization and rate limiting

---

## 4. Candidates for Integration — LOW priority

### 4.1 `trinity-io` — I/O Layer

| Parameter | Status | Description |
|-----------|--------|-------------|
| Source | DRAFT | — Async I/O for TRIOS |
| Integration | PENDING | Integration with trios-server |
| Dependencies | DRAFT | trios-server, async runtime |
| Complexity | LOW | Can defer until RPC/WebSocket complete |

**Integration steps:**
1. Study async I/O patterns in trios-core
2. Implement file MCP operations (async read/write)
3. Test performance

---

### 4.2 `trinity-tests` — Testing Framework

| Parameter | Status | Description |
|-----------|--------|-------------|
| Source | DRAFT | — Unit and integration tests for TRIOS |
| Integration | PENDING | Integration into CI/CD |
| Dependencies | DRAFT | — |
| Complexity | LOW | Infrastructure task |

**Integration steps:**
1. Create `crates/trios-test` crate
2. Add property-based tests for all modules
3. Integration with GitHub Actions

---

## 5. DO NOT Extract (Core System, Hard Coupled)

| Module | Reason |
|---|---|
| `tri/` | Core of CLI, tri-stack doesn't work without it |
| `phi-engine/` | Tightly coupled with tri runtime |
| `hslm/` | Already part of trinity-training |
| `cli/` | Internals of trinity-cli |
| `trinity-firebird` | LLM inference, core of entire system — critical component, DO NOT extract |

---

## 6. Candidates for Extraction — LOWEST priority

### 6.1 `trinity-mcp` — Model Context Protocol

| Parameter | Status | Description |
|-----------|--------|-------------|
| Source | DRAFT | — MCP (Model Context Protocol) for TRIOS |
| Integration | — | Built into trios-server |
| Dependencies | DRAFT | — |
| Complexity | LOW | Already implemented, no integration needed |

---

## 7. Technical Debt

| Area | Status | Action |
|------|--------|--------|
| TRIOS FFI link | ❌ 5 FAIL | Zig vendor 4/5 ✅ but TRIOS FFI link 0/13 ❌ (linker errors) |
| Sacred geometry in zig-physics | ✅ IMPLEMENTED | Current implementation works, status A1 (merge, not resurrect) |
| zig-sacred submodule 404 | ⏸️ DEFERRED | Sacred geometry already in zig-physics, decision A1 made |

---

## 8. Integration Strategy

### 8.1 Phased Approach

1. **Phase 1 — FFI Stabilization (IN PROGRESS)**
   - Zig vendor builds: 4/5 ✅ (sacred 404)
   - TRIOS FFI stub mode: 13/13 ✅
   - TRIOS FFI link mode: 0/13 ❌ (5 FAIL: crypto sha256, golden-float gf16_*, hdc vendor, physics vendor, sacred sacred_*)
   - Status: **NOT complete**, Phase 1 requires resolution of 5 FAIL before Phase 2

2. **Phase 2 — Core Stack (NEXT)**
   - Select one HIGH priority candidate for MVP
   - Recommendation: `trinity-fpga` (only HIGH priority)
   - Reason: FPGA synthesis — maximum hardware control

3. **Phase 3 — Expansion (FUTURE)**
   - Add MEDIUM priority candidates one by one
   - Create integrated architecture (websocket, rpc, io, tests)

---

## 9. Action Priority Today

| # | Action | Priority | Status |
|---|--------|----------|--------|
| 1 | Resolve 5 TRIOS FFI FAIL | CRITICAL | ❌ IN PROGRESS |
| 2 | Architecture documentation | HIGH | ✅ SAVED |
| 3 | zig-sacred geometry resolution | LOW | ✅ A1 (merge in zig-physics) |

---

**Note:** ARCHITECTURE.md is created as planning foundation and should not contain operational tasks. For specific candidate implementation, create separate files (e.g., `.trinity/ARCHITECTURE-FPGA.md` for trinity-fpga).

---

**Last updated:** 2026-04-19 18:30
