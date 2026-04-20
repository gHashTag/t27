# 🎯 TRIOS DASHBOARD — Issue #143
**Updated:** 2026-04-21T03:00:00Z
**Branch:** main
**HEAD:** 036da0cf

---

## 📊 PRIORITY QUEUE

### P0 — COMPLETED (2026-04-21)

| Issue | Task | Status | Owner | Commit |
|-------|------|--------|-------|--------|
| #138 | L8: Push First Law | ✅ DONE | CLAUDE | 036da0cf |
| #152 | Chrome icons + popup | ✅ DONE | BRAVO | 786f31ee |
| #121 | trios-ext web-sys fix | ✅ DONE | CHARLIE | 966e1964 |
| #118 | trios-server MCP WebSocket | ✅ DONE | DELTA | 174c50e9 |
| #142 | anti-ban audit | ✅ DONE | ECHO | 2b9c0346 |

### 🚨 P0 VIOLATION — NEW

| Issue | Task | Status | Priority |
|-------|------|--------|----------|
| #156 | **JS in extension/ — MUST be Rust→WASM** | 🔴 **VIOLATION** | **CRITICAL** |
| #155 | trios-ext proper implementation | ⏸️ BLOCKED | by #156 |

### P1 — NEXT

| Issue | Task | Deadline | Status |
|-------|------|----------|--------|
| #106 | trios-claude bridge | — | ⏳ TODO |
| #110 | Parameter Golf P2–P7 | 30.04.2026 | 🟡 RUNNING |
| #109 | PhD Monograph | 15.06.2026 | ⏳ TODO |

---

## 🟢 AGENT ROSTER (NATO) — UPDATED

| NATO | Issue | Role | Status | Commit |
|------|-------|------|--------|--------|
| ALFA | #122 | igla-trainer skeleton | ✅ DONE | — |
| BRAVO | #152 | Chrome icons | ✅ DONE | 786f31ee |
| CHARLIE | #121 | web-sys fix | ✅ DONE | 966e1964 |
| DELTA | #118 | WebSocket server | ✅ DONE | 174c50e9 |
| DELTA | #156 | **trios-ext Rust→WASM rewrite** | 🔴 **CLAIM NOW** | — |
| ECHO | #142 | anti-ban audit | ✅ DONE | 2b9c0346 |

---

## 📦 INFRASTRUCTURE STATUS

| Crate | Status | Tests | Notes |
|-------|--------|-------|-------|
| trios-proto | ✅ DONE | — | Envelope, PhiPriority, RoutingKey |
| trios-bus | ✅ DONE | 35 | EventBus, actors, replay |
| trios-orchestrator | ✅ DONE | — | boot(), autodiscovery |
| trios-sdk | ✅ DONE | — | Trios::boot(), run(), publish(), one_shot |
| trios-ext | 🔴 **VIOLATION** | — | #156 — JS must be Rust→WASM |
| trios-server | ✅ DONE | 17 | MCP WebSocket on port 9005 |
| anti-ban-audit | ✅ DONE | 4 | 8 anti-ban checks |
| trios-claude | ⏳ TODO | — | Process bridge (#106) |
| igla-oracle | 🆕 TODO | — | Rust PBT controller (#150) |

---

## 🌐 EXTENSION STATUS

| File | Status | Violation |
|------|--------|-----------|
| manifest.json | ✅ | — |
| icons/* (16/32/48/128) | ✅ | — |
| sidepanel.html | ✅ | — |
| sidepanel.js | 🔴 **VIOLATION** | #156 — must be Rust→WASM |
| background.js | 🔴 **VIOLATION** | #156 — must be Rust→WASM |

---

## 🚨 ISSUE #156 — TRINITY STACK LAW VIOLATION

**Problem:** Extension contains hand-written JavaScript (`sidepanel.js`, `background.js`)

**Trinity Stack Law:**
```
Layer          | Rust  | JS  | Notes
---------------|-------|-----|-------------------
Extension UI   | ✅    | ❌  | Leptos/Yew/Dioxus + wasm-bindgen
Extension DOM  | ✅    | ❌  | web-sys only
Extension WS   | ✅    | ❌  | gloo-net / web-sys::WebSocket
```

**Fix Required:**
1. Delete `extension/sidepanel.js`, `extension/background.js`
2. Implement in `crates/trios-ext/src/` using Rust + wasm-bindgen
3. `wasm-pack build --target web`
4. CI rule: `extension/**/*.js` outside `dist/` = build fail

**Agent:** DELTA (call-sign reassigned for this task)

---

## ⚖️ LAWS (Mandatory)

| Law | Rule | Status |
|-----|------|--------|
| **L1** | No `.sh` files. Rust + TypeScript only | ✅ Followed |
| **L2** | Every PR must contain `Closes #N` | ✅ Followed |
| **L3** | `cargo clippy -D warnings` = 0 | ✅ Passing |
| **L4** | `cargo test` passes before merge | ✅ Passing |
| **L5** | Port 9005 is trios-server | ✅ Fixed |
| **L7** | Write experience log | ✅ Writing |
| **L8** | PUSH FIRST LAW | ✅ ENFORCED |
| **L9** | BLOCKED must have `BLOCKER:` comment | ✅ ENFORCED |

---

_Last updated: 2026-04-21T03:00:00Z_
_Source: Issue #143 dashboard_
