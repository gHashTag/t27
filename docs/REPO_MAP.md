# Repository map — for external reviewers

**Purpose:** In under **10 minutes**, locate **source of truth**, **generated**, **frozen**, **experimental**, and **peripheral** material.

---

## Source of truth (authoritative)

| Path | What |
|------|------|
| `specs/**/*.t27`, `specs/**/*.tri` | Normative language and domain semantics (SSOT-MATH). |
| `compiler/**/*.t27` | Compiler-facing meta-specs. |
| `docs/T27-CONSTITUTION.md`, `SOUL.md`, `CANON.md`, `FROZEN.md` | Law, rings, freeze. |
| `architecture/ADR-*.md` | Recorded architectural decisions. |
| `stage0/FROZEN_HASH` | Sealed bootstrap `compiler.rs` hash. |
| `conformance/*.json` | Conformance inputs (prefer spec-driven generation per `docs/TDD-CONTRACT.md`). |

---

## Generated (do not hand-edit)

| Path | Rule |
|------|------|
| `gen/zig/**`, `gen/c/**`, `gen/verilog/**` | Emitted by `t27c`; mirror spec paths. Default `t27c compile-all` → `gen/zig`. |
| Future: provenance trailer per file | Planned (see `docs/REPOSITORY_EXCELLENCE_PROGRAM.md` P2). |

---

## Frozen / integrity

| Path | What |
|------|------|
| `stage0/FROZEN_HASH` | Cryptographic baseline for bootstrap compiler core. |
| `.trinity/seals/*.json` | Per-module seal records. |
| `.trinity/experience/*.jsonl` | Append-only run experience (schema as documented). |

---

## Experimental / research / non-core

| Path | Note |
|------|------|
| `research/**`, `kaggle/**` | Not ring-gold; quarantine from critical path. |
| `external/**` | Vendored third parties; not Trinity SOOT. |
| `backend/**`, `clara-bridge/**`, `portable-claude-setup/**` | Operational / bridge infrastructure; distinguish from **language proof obligations**. |
| `specs/math/**` (physics-flavored) | May mix **reference constants** and **empirical phi models** — read `docs/WHAT_REMAINS_SPECULATIVE.md`. |

**Policy (target):** split tree into `specs/stable`, `specs/experimental`, `specs/research` — **not yet enforced**; until then, use claim labels in `docs/RESEARCH_CLAIMS.md`.

---

## Bootstrap implementation (temporary)

| Path | Role |
|------|------|
| `bootstrap/**` | Only hand-written **Rust** for `t27c` until self-host; `build.rs` enforces LANG-EN + FROZEN + required docs. |

---

## Community and umbrella project

t27 is part of **Trinity S³AI** ([`gHashTag/trinity`](https://github.com/gHashTag/trinity)). **Social and docs site** match the Trinity README: [Reddit r/t27ai](https://www.reddit.com/r/t27ai/), [Telegram @t27_lang](https://t.me/t27_lang), [X @t27_lang](https://x.com/t27_lang), site [gHashTag.github.io/trinity](https://gHashTag.github.io/trinity). Full table: root **`README.md`** § Community and contact.

---

## Publications (Trinity Framework)

- **DOI catalog + series** → `publications/README.md`  
- **Pipeline / policy** → `docs/PUBLICATION_PIPELINE.md`  
- **Readiness audit** → `docs/PUBLICATION_AUDIT.md`

---

## One-page navigation

- **Roadmap / NOW / queue** → `docs/ROADMAP.md`, `docs/NOW.md`, `docs/PUBLICATION_QUEUE.md`  
- **Pinned issue + Project setup** → `docs/PINNED_ROADMAP_ISSUE.md`, `docs/GITHUB_EPIC_ISSUES.md`, `docs/GITHUB_PROJECT_TRACKER.md`  
- **Why claims?** → `docs/RESEARCH_CLAIMS.md`  
- **Honest status?** → `docs/STATE_OF_THE_PROJECT.md`  
- **Physics boundaries?** → `docs/PHYSICS_REVIEW_PROTOCOL.md`, `docs/WHAT_REMAINS_SPECULATIVE.md`  
- **Reproduce?** → `repro/README.md`
