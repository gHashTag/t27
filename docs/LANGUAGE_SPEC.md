# LANGUAGE_SPEC — canonical semantics (skeleton)

**Status:** Skeleton (expand into SPEC-000; goal = reviewer-grade formal document)  
**SOOT:** Executable meaning remains in `specs/**/*.t27` until extracted here.

---

## 1. Scope

This document will define a **core** fragment of t27 (lexical, syntactic, type, and dynamic semantics) that matches what the **`tri`** / `t27c` compiler implements today. Extensions are **incremental** per SEED-RINGS.

---

## 2. Pointers to current artifacts

| Topic | Location |
|-------|----------|
| Incremental bootstrap story | `docs/SEED-RINGS.md`, `CANON.md` |
| ASCII / language policy | `architecture/ADR-004-language-policy.md`, `docs/SOUL.md` |
| Numeric family | `docs/NUMERIC-STANDARD-001.md`, `specs/numeric/` |
| Compiler behavior | `bootstrap/src/compiler.rs`, `compiler/**/*.t27` |

---

## 3. Planned sections (TODO)

1. Lexical grammar (tokens, comments, literals).  
2. Context-free grammar (modules, types, fn, structs, enums).  
3. Type system (static rules, error model).  
4. Operational semantics (evaluation / codegen obligations).  
5. Invariants and `test` / `invariant` blocks.  
6. Backend mapping obligations — see `docs/BACKEND_CONTRACT.md`.

---

## 4. Soundness target

State **soundness** theorems **only** for fragments that are actually proven or machine-checked; otherwise mark **conjecture** in `docs/RESEARCH_CLAIMS.md`.

---

*A formal methods reviewer should be able to start here, not only in README.*
