# t27 — Bootstrap and testing roadmap (Rust seed → `.t27` tests)

**Status:** Planning charter  
**Date:** 2026-04-06  
**Audience:** Compiler, QA, and ring owners  

This document turns the **bootstrap compiler** story into an executable roadmap: when tests may be **authored** in `.t27`, when they are **executed** by Rust vs by t27 itself, and how issues should be structured. It complements **`docs/GOLDEN-RINGS-CANON.md`** and **`docs/MULTI-MODEL-TRUST-CHAIN-ANALYSIS.md`**.

---

## Principle boundary

```text
Rust seed = CONSTRUCTION TOOL
.t27      = WHAT WE BUILD

Tests expressed in .t27 appear exactly when .t27 can express:
  input → computation → assert
```

Until then, **Rust** owns the runner and the harness; `.t27` files may still exist as **specs** (parse/codegen/seal) under `specs/`, which is the current t27 model.

---

## Stage 0 — Ring 0: Rust seed (today’s shape)

**Reality:** Parser + codegen (and any evaluation logic) live in **Rust** (`bootstrap/`).  
**Tests:** Primarily **Rust** (`cargo test`, `t27c suite`, etc.).  
**Goal:** Prove the seed correctly handles a **minimal** `.t27` surface (parse, gen, seals).

### Proposed issue spine (illustrative numbering)

These are **backlog templates**, not reserved GitHub numbers:

| Range | Theme |
|-------|--------|
| #1–#10 | Rust-only hardening: CI, parser/eval/snapshot smoke, reproducible builds, optional commit/issue hooks |

Examples (rename/re-scope when opening real issues):

- **#1** `cargo nextest` + `clippy` CI gate  
- **#2** Parse empty file → `Ok`  
- **#3** Parse minimal binding → stable AST shape  
- **#4** Eval `1 + 1` → `2` (only if/when an eval path exists in seed)  
- **#5** Domain check for a small numeric type (e.g. GF4)  
- **#6–#7** AST / diagnostic snapshots  
- **#8** Optional: commit hook requiring issue reference (align with **`.github/workflows/issue-gate.yml`** for PRs)  
- **#9** Locked / reproducible toolchain for CI  
- **#10** SPEC-000 style “hello world”: parse (+ gen) canonical fixture  

**Exit criterion (conceptual):** a documented **`tri`-family command** (or equivalent) can run a minimal `hello.t27` through the seed **without panic** and with bounded, tested behavior. Exact command names evolve with the CLI.

---

## Stage 1 — Ring 1: `.t27` fixtures, Rust runner

**What changes:** `.t27` files appear as **first-class test fixtures** (expressions, asserts, small programs). **Rust** loads them, runs them, captures stdout/exit, compares to golden output.

**Key idea:** the test is **written** in `.t27` but **executed** by the Rust runner — standard [bootstrapping](https://en.wikipedia.org/wiki/Bootstrapping_(compilers)) practice.

### Proposed issue spine #11–#25

| ID | Direction |
|----|-----------|
| #11 | Runner: `tri run <file.t27>` (or `t27c run`) → capture stdout / exit code |
| #12 | **First `.t27` fixture:** e.g. `ring1/assert_eq.t27` |
| #13 | GF4 (or chosen domain) arithmetic fixture |
| #14 | Parse / round-trip fixture |
| #15 | Property tests on Rust generating `.t27` snippets → run under runner |
| #16 | Golden/snapshot outputs for all Ring 1 fixtures |
| #17 | CI: every Ring 1 `.t27` fixture passes via seed |
| #18 | Differential: same fixture → Rust eval vs reference (e.g. high-precision library), where applicable |
| #19 | Parser fuzzing (`cargo-fuzz` or equivalent) |
| #20 | Formal / exhaustive checks where cheap (e.g. small finite domains) |
| #21–#23 | First conformance-style vectors expressed as `.t27` or JSON sidecars |
| #24 | Experience log: Ring 1 learnings → `.trinity/experience/` |
| #25 | **Milestone:** Ring 1 sealed (seal event + green suite) |

---

## Stage 2 — Ring 2: `.t27` evaluates `.t27`

**This is the main inflection point (“Bootstrap Day”).** The language is rich enough to:

1. Read `.t27` as data (source string)  
2. Parse inside t27  
3. Evaluate  
4. Assert  

### Proposed issue spine #26–#40

| ID | Direction |
|----|-----------|
| #26 | Meta-eval / exec in `.t27` |
| #27 | **First self-referential test:** e.g. `ring2/self_eval.t27` — `eval("let x = 1 + 1")` → `2` |
| #28 | Minimal test framework in `.t27` (`test`, `assert_eq`, failure reporting) |
| #29–#31 | GF4 / parser / type-system tests **authored and run in .t27** |
| #32–#33 | Metamorphic relations (e.g. commutativity, round-trip) in `.t27` |
| #34 | Differential / reference oracles where still needed |
| #35 | Ring 2 conformance suite (vector set) |
| #36 | CI: Ring 2 suite runs via **.t27 runner** entrypoint |
| #37 | Rust runner only on the cold-start / bootstrap path |
| #38 | Coverage / metamorphic-coverage targets (policy TBD) |
| #39 | Experience log v2 |
| #40 | **Milestone:** Ring 2 sealed — **Bootstrap Day** |

---

## Stage 3 — Ring 3+: `.t27` compiles `.t27`

The **test framework** and most suites live in `.t27`. Rust is needed mainly to build the **first** self-hosting compiler binary and for host integration.

### Proposed issue spine #46–#55 (sample)

- **#46** `stdlib/test.t27` (or equivalent): `it`, `assert_eq`, `assert_property` hooks for PBT-style checks  
- **#47** `test_compiler.t27` — self-compile smoke  
- **#48–#50** Codegen tests (Zig, C, differential Zig vs C)  
- **#51** Property-based generators in `.t27`  
- **#52** First **brain** reasonableness tests (e.g. metamorphic paraphrase consistency — charter-level)  
- **#53** Sacred physics / φ-ratio tests with dimensional or metamorphic oracles  
- **#54** Experience log v3  
- **#55** Ring 3 sealed  

(Adjust numbering when merging with real GitHub milestones.)

---

## Handoff of responsibility (intent)

```text
Ring 0      Ring 1           Ring 2                Ring 3+
  │            │                │                     │
  │ Rust       │ .t27 fixtures  │ .t27 evaluates      │ .t27 compiles
  │ tests      │ Rust runner    │ .t27                │ .t27
  │            │                │                     │
  ▼            ▼                ▼                     ▼
100% Rust    ~90% Rust        ~50% Rust             ~5% Rust
             ~10% .t27        ~50% .t27             ~95% .t27
                              ↑                     ↑
                         First tests            Full framework
                         “on .t27”              “on .t27”
```

Percentages are **communicative**, not measured.

---

## Issue template (recommended fields)

Every substantive issue should carry:

```text
ring:         [0 | 1 | 2 | 3+]
language:     [rust | .t27 | both]
test_type:    [unit | snapshot | pbt | metamorphic | differential | formal | e2e]
oracle:       [reference | golden | metamorphic_relation | formal_proof | seal]
acceptance:   concrete pass criteria (command + expected outcome)
```

**Policy alignment:** “No issue → no merge” is already enforced for PRs via **`.github/workflows/issue-gate.yml`** (linked issues / `Closes #N`). Extending the same discipline to **every local commit** (git hook) is optional and should be tracked as its own issue.

---

## Canonical answer: “On which ring can we write tests in `.t27`?”

| Ring | Write test in `.t27`? | Executes via |
|------|------------------------|----------------|
| 0 | Only as **specs** consumed by Rust (`specs/`, `compiler/`) | Rust (`t27c suite`, etc.) |
| 1 | **Yes** — as **fixtures** | **Rust runner** |
| 2 | **Yes** — full programs + framework | **t27 interpreter on t27** |
| 3+ | **Yes** — compiled test binaries / modules | **t27-compiled code** (+ minimal Rust shim) |

Moving tests to `.t27` is not stylistic preference — it is an **architectural milestone**. Ring 1 starts when the first **executable** `.t27` fixture lands (#12-class work). Ring 2 marks **autonomy** (#27 / #40-class work).

---

## References

- Bootstrapping (compilers): [Wikipedia — Bootstrapping](https://en.wikipedia.org/wiki/Bootstrapping_(compilers))  
- Readable metacompiler / bootstrap narrative: [tmewett — Bootstrapping](https://tmewett.com/bootstrapping-metacompiling/)  

---

**φ² + 1/φ² = 3 | TRINITY**
