# Rolling integration plan — Seed → Tests → Queen brain

**Status:** Operational backlog (planning). **Language:** English (**LANG-EN**).  
**Date:** 2026-04-06  

**Paired with:** [`NOW.md`](../../NOW.md) (snapshot, repo root), [`docs/T27-CONSTITUTION.md`](../T27-CONSTITUTION.md) (**tri** / **`t27c`** as canonical CLI), [`docs/nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md`](../nona-03-manifest/T27-BOOTSTRAP-TESTING-PLAN.md).

---

## Executive summary — what to do first

**Main hole (if you want a Zig self-host):** **`bootstrap/main.zig`** can **parse** `.t27` → AST but has **no codegen**, so **this Zig-only** chain is **open**:

```text
bootstrap/main.zig  ── parse only ──→  AST  ── ??? ──→  .zig  ── ??? ──→  zig test
   works                                   no Zig emitter here        no E2E on this path
```

**Product reality:** **Rust `t27c`** already implements **`gen`**, **`compile`**, **`compile-all`**, etc. (**`t27c --help`**). The **`gen/zig/**`** tree is produced by **`t27c`**, not by **`bootstrap/main.zig`**. The **first** hardening work is often **tests + conformance + CI** on the **`t27c`** path—not pretending codegen is missing globally.

**Separate gap (Zig bootstrap):** **`bootstrap/main.zig`** is a **single-file Zig** parser with **no** codegen—so the **self-hosted Zig compiler** story is **not** closed. If the project wants **two** emitters, add Zig codegen **or** demote `main.zig` to parse-only demo.

**Queen brain:** Needs **E2E proof** (generated **Lotus** / **HSLM** + tests + honest experience logs), not only “files exist.”

**Rolling snapshot (verify on your machine / CI — do not treat as SSOT):** e.g. ring 44 in **`.trinity/experience/clara_track*.jsonl`**, **GREEN** **`queen-health.json`**, ~52 generated `.zig` files, **`gen/zig/queen/lotus.zig`** + **`gen/zig/nn/hslm.zig`** generated but not proven E2E.

### Critical path (illustrative)

| Phase | Working days | Issue slots (titles only — open **real** `#N`) | Unblocks |
|-------|----------------|--------------------------------------------------|----------|
| **0** Seed audit | 2 | 2 (Rust `t27c` + golden **5** `.t27` seeds) | Everything |
| **1** Numerics | 3 | 7 (GF16 vectors, family, constants, TF3, GF smoke, φ helpers, experience) | Brain numerics |
| **2** Codegen | 5 | 5 (const → struct → fn → test blocks → diff vs `gen/zig`) | E2E on chosen emitter |
| **3** VSA + AR | 3 | 3 | Reasoning layer |
| **4** HSLM + Lotus | 4 | 4 | Queen unit behavior |
| **5** E2E + Queen CI | 5 | 4 | Full ring + logs |
| **6** Science | ongoing | 3 (+) | Publication / repro |

**~22 working days** for phases **0–5** if run sequentially; **25** issue-sized tasks for phases **0–5** (plus **3** science issues in phase **6** = **28** total if all are filed).

---

## 0. Repository facts (verify; do not assume)

| Item | Observed in t27 |
|------|------------------|
| **Zig parser bootstrap** | **`bootstrap/main.zig`** (~1314 LOC) — self-contained lexer/parser/AST-oriented prototype. |
| **Rust compiler / suite** | **`bootstrap/`** Cargo project — **`t27c`** binary (see README: `./scripts/tri` wraps it). **Canonical tooling path** per constitution. Subcommands include **`parse`**, **`gen`**, **`compile`**, **`compile-all`**, **`suite`**, **`validate-conformance`**. |
| **Generated Zig** | **`gen/zig/**`** — emitted by **`t27c gen` / `t27c compile-all`** (and **`tri`** wrappers), **not** by **`bootstrap/main.zig`**. |
| **Conformance contract** | **`conformance/gf16_vectors.json`** (and family JSON). There is **no** `conformance/vectors.json` — do not reference it as SSOT. Target **33** vectors is a **growth goal**, not current file fact. |
| **Trinity float check** | `PHI_SQ + PHI_INV_SQ == 3.0` is **not** bitwise exact in IEEE `f64` — use **tolerance** or rational proof in docs (see [`NUMERIC-CORE-PALETTE-REGISTRY.md`](../nona-02-organism/NUMERIC-CORE-PALETTE-REGISTRY.md)). |
| **Queen / experience** | **`.trinity/state/queen-health.json`**, **`.trinity/experience/*.jsonl`** — update only with **real** run results. |

---

## 1. Critical gaps (product vs self-host)

**Gap A — Product path (codegen exists; proof must deepen):**  
`specs/**/*.t27` → **`t27c gen` / `compile*` / `suite`** → **`gen/zig/**`** + **`validate-conformance`**, **`validate-gen-headers`**, CI.  
**Work:** expand **GF16 vector** coverage toward **33+**, add **E2E** tests for **Lotus/HSLM**, and bind PRs to issues.

**Gap B — Zig-only bootstrap (`bootstrap/main.zig`):**  
Parse-only prototype; **no** Zig emitter.  
**Work (optional):** implement **`codegenZig`** there **only if** the project wants a **second** compiler; otherwise keep it explicitly **non-canonical** and invest in **`t27c`** instead.

**Do not** claim **`bootstrap/main.zig`** is the **sole** compiler SSOT — **`t27c`** is canonical per **`docs/T27-CONSTITUTION.md`**.

---

## 2. Law (aligned with repo)

1. **No mutation without a GitHub issue** — every PR **`Closes #N`**; **`issue-gate`** enforces binding.  
2. **Commit messages** should reference the issue (e.g. `feat: … #123`).  
3. **Do not hand-edit `gen/**`** — regenerate via **`tri gen`** / **`t27c`**.  
4. **SSOT is `.t27` / `.tri`** — Zig/C/Verilog are backends ([`SOUL.md`](../../SOUL.md), ADR-001).  
5. **Conformance golden file** is **`conformance/gf16_vectors.json`** (and related JSON), **not** `conformance/vectors.json` (that path does not exist here). Grow vector count toward **33+** via schema work (**Ring #133**).  
6. After each **phase**, append a **verifiable** JSON line to **`.trinity/experience/clara_track*.jsonl`** when work actually completed.  
7. **`queen-health.json`** should reflect **real** health (≥ 0.9 if you claim GREEN).  
8. **Compiler SSOT:** canonical CLI is **`./scripts/tri` → `t27c`** (Rust bootstrap). **`bootstrap/main.zig`** is a **parallel** Zig parser prototype — **not** the single authority for emission unless the project explicitly promotes it (see **§1 Gap A/B**).

---

## 3. Phase 0 — Seed audit (~2 days)

**Goal:** Reproducible **`t27c`** + tiny **golden** `.t27` corpus.

| Track | Suggested issue title (open real #) | Acceptance |
|-------|-------------------------------------|------------|
| **Rust** | Verify bootstrap compiles (`SEED-001`-class) | `cd bootstrap && cargo build --release`; `./target/release/t27c help`; `./target/release/t27c parse <file.t27>` → AST / success (there is **no** `t27c --version` today) |
| **Zig** | Verify Zig bootstrap parser still runs | Document `zig build` / `zig test` command for **`bootstrap/main.zig`** if present in repo scripts |
| **Golden specs** | Add **5** minimal files under **`specs/seed/`** or **`seed/`** (`SEED-002`-class) | (1) `hello.t27` — minimal `fn`/`main` shape as accepted by parser (2) `phi_const.t27` — `PHI` const (3) `gf16_type.t27` — struct layout (4) `trinity_test.t27` — test with **tolerant** Trinity check (5) `queen_stub.t27` — stub `observe`/context |

*Replace labels like `SEED-001` with real GitHub issue numbers when created (`gh issue create` requires `gh` auth).*

---

## 4. Phase 1 — Ring 0: Numerics (~3 days)

| # | Focus | Acceptance (corrected) |
|---|--------|-------------------------|
| 1 | **GF16 vectors** | **`zig test gen/zig/numeric/gf16.zig`** (in-repo: **17** unit tests passed at last check); add runners that consume **`conformance/gf16_vectors.json`**; grow toward **33+** vectors; optional **`conformance/gf16_results.jsonl`**. |
| 2 | **GoldenFloat family** | `get_format_by_name("GF16")` / φ-distance matches **`FORMAT-SPEC-001.json`**. |
| 3 | **Constants** | Comptime checks with **epsilon** for `PHI`; Trinity identity documented with tolerance. |
| 4 | **TF3** | Roundtrip tests on `gen/zig/numeric/tf3.zig` (or spec-driven gen) per **`specs/numeric/tf3.t27`**. |
| 5 | **GF4/GF8/GF12 smoke** | `fromF32(1.0)` style smoke where APIs exist. |
| 6 | **φ-rounding helpers** | If present in **`gf16.t27`**, test **documented** error bounds. |
| 7 | **Experience** | Log numeric ring completion to **`clara_track*.jsonl`**. |

**Zig test note:** `gen/zig/` may lack a root **`build.zig`** — add a **small test harness** issue or test via **`t27c` / `tri test`**.

---

## 5. Phase 2 — Ring 1: Codegen closure (~5 days)

**This phase unblocks “everything” on the **self-hosted Zig** narrative.** Pick **one** primary strategy; the other can lag.

| Issue theme | Acceptance (Zig path) | Acceptance (Rust path) |
|-------------|----------------------|-------------------------|
| **COMP-001** Minimal emit | Add `codegenZig(node, writer)` (or CLI `codegen`) in **`bootstrap/main.zig`**; `pub const NAME: T = val;` round-trip | Extend **`t27c`** / **`tri gen`** for the same minimal surface |
| **COMP-002** Structs | `struct GF16 { … }` → valid Zig | Same |
| **COMP-003** Functions | `fn add(…) GF16` → valid Zig | Same |
| **COMP-004** Test blocks | `test "trinity" { … }` → **`zig test`** passes on emitted file | Same via generated test harness |
| **COMP-005** Parity | Emit from **5** specs; diff vs **`gen/zig/numeric/*.zig`** ≤ whitespace / stable formatting | **`t27c`** regen matches committed **`gen/zig`** |

**CLI note:** Examples using `./t27c codegen` assume a **new subcommand** or a **Zig entrypoint** — implement whichever the issue specifies; do not assume it exists today.

---

## 6. Phase 3 — Ring 2: VSA + AR (~3 days)

- **VSA:** `bind` / `bundle` / `similarity` tests on **`gen/zig/vsa/**`** (or spec-driven targets).  
- **AR:** **`ternary_logic.t27`** truth tables; composition / ASP smoke where specs exist.

---

## 7. Phase 4 — Ring 3: HSLM / Queen Lotus (~4 days)

- **HSLM:** Comptime or unit checks for **243 / 3 / 81 / 6** only if **specs/nn/hslm.t27** defines them (do not invent Fibonacci lore without spec text).  
- **Attention:** forward pass **finite** outputs; softmax sum ≈ 1 where applicable.  
- **GF16 weights:** differential vs f32 with stated tolerance.  
- **Queen Lotus:** six-phase cycle produces a **recorded** episode object (test stub).

---

## 8. Phase 5 — Ring 4: E2E (~5 days)

1. **Spec → generated Zig → test** using **canonical** `tri`/`t27c` path.  
2. **GF16 conformance** via the same pipeline + JSON results artifact.  
3. **Queen / CI:** optional automation issue — verify **experience log** append under controlled fixtures.  
4. **`phi-loop-ci`:** full smoke documented in workflow; **queen-health** unchanged or updated with **real** verdict.

---

## 9. Phase 6 — Science / publication (ongoing)

- **`RESEARCH_CLAIMS.md`** + **`CLAIM_TIERS.md`** rows for physics coincidence claims.  
- **Zenodo / CITATION.cff** when E2E stable.  
- **`make repro`** or **`repro/`** bundle (see existing repro docs).

---

## 10. Priority summary

| Phase | Proves | Blocks |
|-------|--------|--------|
| 0 | Toolchains + golden parse | All |
| 1 | Numeric truth + vectors | Brain numeric deps |
| 2 | Codegen story **chosen** (Rust vs Zig) | True E2E |
| 3 | Reasoning layer smoke | Integrated brain |
| 4 | NN + Lotus unit behavior | Queen E2E |
| 5 | Closed loop spec→test in CI | Publication-grade repro |
| 6 | External evidence | — |

**Critical path:** highly dependent on whether **Strategy X** (Rust/`tri`) is accepted as the **only** required emitter for Ring 5; **Strategy Y** adds parallel work.

---

## 11. Next 48 hours (suggested commands)

```bash
# Issues (needs: gh auth, repo remote)
gh issue create --title "SEED-001: Verify bootstrap (cargo) + t27c help/parse" --label "seed,p0"
gh issue create --title "SEED-002: Add 5 golden seed .t27 specs" --label "seed,p0"

# Rust bootstrap
cd bootstrap && cargo build --release
./target/release/t27c help
./target/release/t27c --repo-root . suite   # or documented subset

# Zig parser bootstrap (if you maintain it)
# zig build …  # only if build.zig exists for bootstrap/main.zig

# Numerics — prefer repo-root paths; gen/zig may need a harness issue if zig test fails standalone
zig test gen/zig/numeric/gf16.zig 2>&1 | tee conformance/gf16_run1.log || true
zig test gen/zig/math/constants.zig 2>&1 | tee conformance/constants_run1.log || true

# Conformance
./target/release/t27c validate-conformance   # or: ./scripts/tri validate-conformance

# NOW gate
./scripts/tri check-now
```

**Experience log (only after real verdict):**

```bash
echo '{"ring":45,"task":"audit-seed","verdict":"…","ts":"…"}' >> .trinity/experience/clara_track1.jsonl
```

---

*Non-English drafts of this plan should be translated into this file or linked from a personal notes repo — not duplicated as competing SSOT under **`docs/`**.*
