# CANON.md — Golden rings, seals, and project dashboard

**Status:** Active (root standard — read with `AGENTS.md`, `SOUL.md`, `CLAUDE.md`)  
**Companion:** `FROZEN.md` (normative freeze standard), `docs/SEED-RINGS.md`, `**docs/RINGS.md` (Rings 32+ review-grade roadmap — constitutional law)**, `stage0/FROZEN_HASH`, `docs/T27-CONSTITUTION.md`, `docs/TECHNOLOGY-TREE.md`

This file is the **single source and dashboard** for: where **GOLD** lives, what **REFACTOR-HEAP** must be migrated out, **recorded compiler seals**, and the **ring roadmap**. **Nothing outside the golden cycle is product truth.**

---

## 0. Live seal status (`stage0/FROZEN_HASH` vs working tree)

**Normative rules:** `**FROZEN.md`** (format, ceremony, threat model, verification ladder).  
**CI / local gate:** `cargo build` or `cargo build --release` in `**bootstrap/`** — enforced in `**bootstrap/build.rs**` (Rust only; no shell verifier).

### 0.1 Recorded seal (what the repo file commits to)

The file `stage0/FROZEN_HASH` **must** follow `**FROZEN.md` §4**: one operational line — 64 lowercase hex + whitespace + **repository-relative** path (no absolute paths).

**Parsed canonical hash (first field of the operational line):**

`af208c1bcd8361092fe6303313c94729c67a71e0eb24de1b9ba7c3d992d8e215`

**Operational line as stored today:**

```text
af208c1bcd8361092fe6303313c94729c67a71e0eb24de1b9ba7c3d992d8e215  bootstrap/src/compiler.rs
```

### 0.2 Working tree drift check

Run on every machine (must match §0.1 until M5 updates the file):

```bash
cd bootstrap && cargo build
```

If **build.rs** reports **FROZEN drift**, the compiler core does not match `stage0/FROZEN_HASH`. **Do not** silently edit `FROZEN_HASH` — update only via **freeze ceremony (M5)** per `**FROZEN.md` §5** (use `cargo run --release -- frozen-digest` from `bootstrap/` to print a fresh line).

### 0.3 Recovering older ring seals

Per-ring history lives in Git:

```bash
git log --oneline -- stage0/FROZEN_HASH
git show <commit>:stage0/FROZEN_HASH
```

---

## 1. Compiler seal registry (hashes recorded at historical ring freezes)

These rows are **reconstructible from the repository history** of `stage0/FROZEN_HASH`. Rings **18–31** (and later) are tracked as product milestones in `docs/TECHNOLOGY-TREE.md` and `README.md`; **this Git log does not show further updates** to `FROZEN_HASH` after Ring 17 until maintainers advance the seal again. The **current** §0.1 value may therefore **differ** from the last SEED-era row below — Git history remains authoritative for **past** freezes; `**FROZEN.md` + `bootstrap/build.rs`** are authoritative for **current** drift.


| Ring (tag in commit) | Git commit | `bootstrap/src/compiler.rs` seal at freeze (SHA-256)               |
| -------------------- | ---------- | ------------------------------------------------------------------ |
| SEED-0               | `c3356a4`  | *(line was a comment only — first numeric seal at Ring 5)*         |
| SEED-5               | `91b6e24`  | `c14b8e4e325e89d359f671fd10295fc4cd060081c6eba53845aa33da40d579b3` |
| SEED-6               | `90914e4`  | `27b5d1acdd640222f6fb75cab04afd6666edd732b2695506e5cfbc7f804d434c` |
| SEED-7               | `caedb84`  | `97d86174b01ca2b2779f89db77325b673c2f2e351c491c637e9279e9c2d735ff` |
| SEED-9               | `e590519`  | `5244fbad946b76dc81bd02e30563b0ecdefc705fca424b1e0200887122c3681d` |
| SEED-10              | `570a247`  | `8c2a34a720ff83df75f16820c9c14f45d5966102fb91265e6019ad17abaf9779` |
| SEED-11              | `5859baf`  | `b6d82cd9f3ef8abbc65127ccaa2bbc3a03d1393097f9e8235741f0a52774650e` |
| SEED-13              | `a8c9c2c`  | `ec2e84d72900de78ad77a0b3ec27e21637a86c61d251d63ab5a186b38ac36562` |
| SEED-15              | `33bc17c`  | `9d6165ae377f6e10cbf78ad33242a1ea1820941bdce0e3d71467adff34326c44` |
| SEED-17 (CANOPY)     | `7c84a0d`  | `9d6165ae377f6e10cbf78ad33242a1ea1820941bdce0e3d71467adff34326c44` |


**Note:** SEED-15 and SEED-17 share the same compiler hash; only the path formatting in `FROZEN_HASH` changed at SEED-17.

---

## 2. Ring roadmap dashboard (product rings 0–40+)

High-level status aligned with `docs/TECHNOLOGY-TREE.md` (detail lives there). Use PR tags `**[GOLD-RING]`** vs `**[REFACTOR-HEAP]`** as in §5.


| Rings | Layer / theme                                                   | Status (per tech tree) |
| ----- | --------------------------------------------------------------- | ---------------------- |
| 0–4   | SEED — lexer/parser, const, types → Zig                         | Complete               |
| 5–8   | ROOT — fn bodies, tests, invariants, conformance                | Complete               |
| 9–12  | TRUNK — Zig / Verilog / C backends, seal CLI                    | Complete               |
| 13–15 | BRANCH — AR pipeline, Queen+NN, full spec suite                 | Complete               |
| 16–17 | CANOPY — self-hosting fixed point                               | Complete               |
| 18–24 | CLARA AR integration                                            | Complete               |
| 25–31 | Gen backends + conformance hardening                            | Complete               |
| 32–35 | Hardening (docs, validation scripts, CI)                        | In progress            |
| 36+   | Zig/C/Verilog compile in CI, cross-backend conformance, benches | Planned                |


**Normative detail for Rings 32+ (scientific credibility, FAIR/JOSS-style bars, epics TASK-1.x–9.x):** `**docs/RINGS.md`**. A PR that claims **Ring 32+** or **hardening** progress **must** align with an open or closed task there and **must** update `**docs/STATE_OF_THE_PROJECT.md`** when subsystem status changes.

**Module / spec seals:** `.trinity/seals/*.json` — gold for “this spec revision verified under policy.”

---

## 3. Golden cycle — micro-iterations (M1–M6)

Each **ring increment** is a **micro-iteration**. Minimum bar before a commit claims “ring progress”:


| Step | Command / artifact                                           | Pass criterion                                                                                                         |
| ---- | ------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| M1   | `cd bootstrap && cargo build` (or `--release`)               | **Must succeed** — runs `build.rs` language guard + builds `t27c`.                                                     |
| M2   | `./bootstrap/target/release/t27c parse <new-or-touched.t27>` | **Parse OK** for every spec touched in the PR.                                                                         |
| M3   | `cargo test` in `bootstrap/`                                 | **All tests green** for compiler changes.                                                                              |
| M4   | `bash tests/run_all.sh` (CI)                                 | Full spec parse/gen sweep as defined by the repo.                                                                      |
| M5   | Update `**stage0/FROZEN_HASH`**                              | **Only when intentionally sealing a ring** — SHA-256 of `bootstrap/src/compiler.rs` (see `docs/SEED-RINGS.md` step 8). |
| M6   | Seal / experience                                            | `.trinity/seals/*.json` updated where required; optional `.trinity/experience/` record.                                |


If **M1–M4** are not green, the change is **not gold** — use a draft branch or revert.

---

## 4. What is GOLD (canonical)


| Asset                                                                        | Meaning                                                                            |
| ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `**specs/**/*.t27` that parse + gen in CI**                                  | **Source of truth** for Trinity semantics.                                         |
| `**bootstrap/src/compiler.rs` (+ lexer/parser/codegen in `bootstrap/src/`)** | **Only** allowed hand-written compiler implementation until self-host ring.        |
| `**stage0/FROZEN_HASH`**                                                     | Cryptographic **seal** of the compiler snapshot for the current ring baseline.     |
| `**.trinity/seals/*.json`**                                                  | Module seals — gold for verified spec revisions.                                   |
| `**docs/SEED-RINGS.md` + this file (`CANON.md`)**                            | Process gold — rings, micro-iterations, dashboard.                                 |
| `**docs/RINGS.md`**                                                          | Process gold — **Rings 32+** review-grade repository law (epics, tasks, timeline). |
| `**docs/T27-CONSTITUTION.md` + `docs/SOUL.md` Law #1**                       | Policy gold — language and SSOT.                                                   |


**Golden rule:** If it is not `**.t27` spec**, `**t27c`**, **frozen hash**, or **documented policy**, it is **not** where “the math lives” — it is implementation or debt.

---

## 5. What is REFACTOR-HEAP (explicit debt — plan to extract)

Everything here is **acknowledged non-gold**. Do **not** copy patterns into new features; **migrate or delete** per linked plans.


| Bucket                               | Pointer                                   | Summary                                                               |
| ------------------------------------ | ----------------------------------------- | --------------------------------------------------------------------- |
| Non-t27 languages on critical path   | `docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md` | Python CLARA runner, Kepler tests, legacy `t27c.py`, etc.             |
| IEEE f32/f64 instead of GF16 primary | `docs/NUMERIC-GF16-DEBT-INVENTORY.md`     | nn/, vsa/, math/, physics/, AR composition `f32`, etc.                |
| GF4–GF32 spec files                  | Same inventory §1                         | `**[REFERENCE]`** only — not an excuse to add `f64` in product paths. |
| Vendored forests                     | `external/opencode/`                      | Not Trinity gold; submodule or delete policy.                         |
| Research sidecars                    | `research/tba/*.py`, `kaggle/`            | Quarantined from ring gates.                                          |


---

## 6. Extraction plan (REFACTOR-HEAP → GOLD)

**Goal:** shrink critical-path surface until **only** `.t27` / `tri` / `t27c` / Rust bootstrap / generated outputs remain.


| Phase | Name     | Actions                                                                                                                                                              |
| ----- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0     | Observe  | Refresh inventories; `cargo build` in `bootstrap/`; list Python/JS on critical path (`QUEEN-LOTUS` §3).                                                              |
| 1     | Recall   | Read `docs/T27-CONSTITUTION.md`, `docs/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md`, ADR-004/005, `NUMERIC-GF16-DEBT-INVENTORY.md`.                                        |
| 2     | Evaluate | Tag paths P0 / P1 / P2 / ALLOW / OUTPUT per `QUEEN-LOTUS`.                                                                                                           |
| 3     | Plan     | One PR per tier; do not mix “delete external/” with “migrate kepler” in one commit.                                                                                  |
| 4     | Act      | Replace Python verdict with `.t27` + `tri`; move orchestration to `t27c`/`tri` subcommands; retire `bootstrap/t27c.py`; align numerics to GF16 per inventory.        |
| 5     | Record   | Update seals / `.trinity/experience/`; on compiler milestone, run **M5** and commit `stage0/FROZEN_HASH` with **repo-relative** path to `bootstrap/src/compiler.rs`. |


**Ordered priorities (suggested):**

1. **P0 Python on verdict path** — `conformance/kepler_newton_tests.py`, `clara-bridge/run_scenario.py` → spec + `tri` (see TZ-T27-001).
2. **Language guard convergence** — keep `build.rs` + CI; long-term single `t27c lint-lang` (Python checker is temporary duplicate).
3. **Numeric debt** — burn down `NUMERIC-GF16-DEBT-INVENTORY.md` from hottest product paths first.
4. **Vendor boundaries** — `external/opencode/` submodule or remove; never teach agents to patch for Trinity features.
5. **Next ring seals** — when Rings **32–35** or **36+** close a compiler milestone, **append or replace** `FROZEN_HASH` per M5 so §1 registry gains new rows via Git history.

---

## 7. Ring work vs garbage work


| Activity                                                   | Class                                     |
| ---------------------------------------------------------- | ----------------------------------------- |
| New `.t27` spec + `t27c` parse/gen + tests + optional seal | **GOLD**                                  |
| Extending `bootstrap` lexer/parser/codegen                 | **GOLD**                                  |
| Updating `FROZEN_HASH` after deliberate ring freeze        | **GOLD**                                  |
| Adding Python to “verify” physics                          | **REFACTOR-HEAP** (forbidden as new work) |
| Hand-writing Zig/C for domain logic outside `tri` gen      | **REFACTOR-HEAP** (ADR-005)               |
| Patching `external/opencode` for Trinity features          | **REFACTOR-HEAP**                         |


---

## 8. Single-command cheat sheet (local micro-iteration)

```bash
cd bootstrap && cargo build --release \
  && ./target/release/t27c parse ../specs/base/types.t27
```

Regenerate **canonical** Zig tree (default output `**gen/zig`**, no flags needed): from repo root, `./bootstrap/target/release/t27c compile-all`. Use `--backend verilog` / `c` for `**gen/verilog**` / `**gen/c**`.

Substitute your changed spec paths. Full sweep: `**bash tests/run_all.sh**`.

---

## 9. Traceability

- Constitution: `**docs/T27-CONSTITUTION.md**` (SSOT-MATH, LANG-EN).  
- System architecture: `**docs/ARCHITECTURE.md**` (three strands, φ-identity, `gen/` contract, umbrella lessons).  
- Freeze normative standard: `**FROZEN.md**` (format, ceremony, verification ladder, references).  
- Numeric primary: `**docs/NUMERIC-STANDARD-001.md**`.  
- Language purge: `**docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md**`.  
- No Python on critical path: `**docs/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md**`.  
- **Rings 32+ hardening law:** `**docs/RINGS.md`** (FAIR/JOSS-aligned roadmap, EPIC/TASK IDs, claim taxonomy and repro obligations).

---

## 10. RINGS law — review-grade repository (constitutional)

**Article RINGS.** For **Ring 31 and below**, closure is defined by `**docs/SEED-RINGS.md`**, `**CANON.md` §§0–8**, and `**FROZEN.md`**. For **Ring 32 and above**, closure **also** requires progress against `**docs/RINGS.md`**: reproducibility, persistent citation identity, explicit **claim status** for physics-adjacent material, formal spec depth, numeric validation, testing maturity, and supply-chain documentation — as enumerated in that file’s EPICs.

**Binding rules:**

1. **No silent hardening:** A merge to `master` that advertises **Ring 32+** or **excellence / reviewer-grade** work **must** reference the relevant **TASK-x.y** (or EPIC) in `docs/RINGS.md` in the PR description or linked issue.
2. **Honest dashboard:** When a subsystem’s maturity changes, `**docs/STATE_OF_THE_PROJECT.md`** **must** be updated in the same PR or the next immediate follow-up.
3. **English normativity:** `docs/RINGS.md` is **English-only** per **Article LANG-EN** in `docs/T27-CONSTITUTION.md` (no parallel “shadow” roadmap in another language as normative).

**Companion (non-normative index):** `docs/REPOSITORY_EXCELLENCE_PROGRAM.md` — shorter P0/P1/P2 table; `**docs/RINGS.md`** is the **authoritative** task breakdown.

---

*phi^2 + 1/phi^2 = 3 | TRINITY — **gold** is only what passes the ring and the hash.*