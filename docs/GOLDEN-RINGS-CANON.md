# Golden Rings Canon — where the gold is, what is refactor trash

**Status:** Active  
**Companion:** `docs/SEED-RINGS.md`, `stage0/FROZEN_HASH`, `docs/T27-CONSTITUTION.md`  

This document defines **GOLD** (canonical, ring-sealed, must stay green) versus **REFACTOR-HEAP** (explicit debt — code we tolerate until a ring or ADR removes it). **Nothing outside the golden cycle is product truth.**

---

## 1. The golden cycle (micro-iterations)

Each **ring increment** is a **micro-iteration** that proves the spine still works. Minimum bar before a commit claims “ring progress”:

| Step | Command / artifact | Pass criterion |
|------|-------------------|----------------|
| M1 | `cd bootstrap && cargo build` (or `--release`) | **Must succeed** — runs `build.rs` language guard + builds `t27c`. |
| M2 | `./bootstrap/target/release/t27c parse <new-or-touched.t27>` | **Parse OK** for every spec touched in the PR. |
| M3 | `cargo test` in `bootstrap/` | **All tests green** for compiler changes. |
| M4 | `bash tests/run_all.sh` (when available in CI) | Full spec parse/gen sweep as defined by repo. |
| M5 | Update **`stage0/FROZEN_HASH`** | **Only when intentionally sealing a ring** — SHA-256 of `bootstrap/src/compiler.rs` (see SEED-RINGS §9). |
| M6 | Seal / experience | `.trinity/seals/*.json` updated for modules that require sealing; optional `.trinity/experience/` record. |

If **M1–M4** are not green, the change is **not gold** — it belongs in a draft branch or must be reverted.

---

## 2. What is GOLD (canonical)

| Asset | Meaning |
|-------|---------|
| **`specs/**/*.t27` that parse + gen in CI** | **Source of truth** for Trinity semantics. |
| **`bootstrap/src/compiler.rs` (+ lexer/parser/codegen in `bootstrap/src/`)** | **Only** allowed hand-written compiler implementation until self-host ring. |
| **`stage0/FROZEN_HASH`** | Cryptographic **seal** of the compiler snapshot for the current ring baseline. |
| **`.trinity/seals/*.json`** | Module seals — **gold** for “this spec revision was verified under policy.” |
| **`docs/SEED-RINGS.md` + this file** | Process gold — how rings and micro-iterations work. |
| **`docs/T27-CONSTITUTION.md` + `docs/SOUL.md` Law #1** | Policy gold — language and SSOT. |

**Golden rule:** If it is not **`.t27` spec**, **`t27c`**, **frozen hash**, or **documented policy**, it is **not** where “the math lives” — it is implementation or debt.

---

## 3. What is REFACTOR-HEAP (explicit trash / debt)

Everything below is **acknowledged non-gold**. Agents **must not copy-paste** patterns from here into new features; they **must** migrate or delete per linked plans.

| Bucket | Pointer | Summary |
|--------|---------|---------|
| **Non-t27 languages on critical path** | `docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md` | Python CLARA runner, Kepler tests, legacy `t27c.py`, etc. |
| **IEEE f32/f64 instead of GF16 primary** | `docs/NUMERIC-GF16-DEBT-INVENTORY.md` | nn/, vsa/, math/, physics/, AR composition `f32`, etc. |
| **GF4–GF32 spec files** | Same inventory §1 | **`[REFERENCE]`** only — not an excuse to add `f64` in product paths. |
| **Vendored forests** | `external/opencode/` | Not Trinity gold; submodule or delete policy. |
| **Research sidecars** | `research/tba/*.py`, `kaggle/` | Quarantined from ring gates. |

Label in PRs: **`[REFACTOR-HEAP]`** when touching only debt; **`[GOLD-RING]`** when touching parser/specs/seals/hash.

---

## 4. Ring work vs garbage work

| Activity | Class |
|----------|-------|
| New `.t27` spec + `t27c` parse/gen + tests + optional seal | **GOLD** |
| Extending `bootstrap` lexer/parser/codegen for one capability | **GOLD** |
| Updating `FROZEN_HASH` after deliberate ring freeze | **GOLD** |
| Adding Python to “verify” physics | **REFACTOR-HEAP** (forbidden as new work) |
| Hand-writing Zig/C for domain logic outside `tri gen` | **REFACTOR-HEAP** (ADR-005 violation) |
| Patching `external/opencode` for Trinity features | **REFACTOR-HEAP** |

---

## 5. Single command cheat sheet (local micro-iteration)

```bash
cd bootstrap && cargo build --release \
  && ./target/release/t27c parse ../specs/base/types.t27
```

Add your changed spec path(s) in place of `types.t27`. For full repo sweep, use **`bash tests/run_all.sh`** when present.

---

## 6. Traceability

- Constitution: **`docs/T27-CONSTITUTION.md`** (SSOT-MATH, LANG-EN).  
- Numeric primary format: **`docs/NUMERIC-STANDARD-001.md`**.  
- Language purge procedure: **`docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md`** (Lotus phases).  

---

*phi^2 + 1/phi^2 = 3 | TRINITY — **gold** is only what passes the ring and the hash.*
