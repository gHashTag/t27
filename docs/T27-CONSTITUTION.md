# Trinity S³AI / t27 — Repository constitution

**Status:** Active  
**Version:** 1.1  
**Date:** 2026-04-06  

---

## Preamble

The Trinity S³AI repository is built around the **t27** specification language and the **`tri` / `t27c`** toolchain. Mathematics, numerics, and physics formulas that participate in verification must not be split between “the spec” and “side scripts.” The following article establishes a single normative principle.

---

## Article SSOT-MATH — single source of truth for mathematics and physics

**Article SSOT-MATH.** The mathematical, numeric, and physical meaning of Trinity S³AI / t27 has **one normative source of truth**: specifications in the **t27** language (`*.t27` files), exercised through the official **`tri` / `t27c`** pipeline and tied to **`.trinity/experience/`** artifacts where run experience is recorded.

It is **forbidden** to introduce new **Python** dependencies (or equivalent script bypasses) on the **critical path** of verification, conformance, or “verdict,” except for **explicitly marked legacy** code with a removal date and a tracked migration into `.t27`.

Target backends (**Zig, C, Verilog**) are **compiler output**, not hand-written application languages; hand-written Zig outside the generated pipeline is allowed only in **bootstrap** (compiler implementation) and related build infrastructure.

The numeric formalism relies on repository standards (**NUMERIC-STANDARD-001**, GoldenFloat, Strand I in `specs/math/sacred_physics.t27` and related specs). Extensions for precision or new numeric primitives are delivered through the **t27 language and compiler**, not external interpreters.

---

## Article LANG-EN — English for first-party code and documentation

**Article LANG-EN.** All **first-party** Markdown under `docs/`, `specs/`, `architecture/`, `clara-bridge/`, `conformance/`, and root project Markdown (`README.md`, `AGENTS.md`, `CLAUDE.md`, `task.md`, `SOUL.md`) **MUST** be written in **English**. Source files (`.t27`, `.zig`, etc.) **MUST** use **English** for comments and identifiers, and remain **ASCII-only** per **ADR-004** and **docs/SOUL.md** Law #1.

Grandfathered non-English paths are listed only in **`docs/.legacy-non-english-docs`** until translated; **do not expand** that list without Architect approval. Vendored content under **`external/`** is exempt.

**Enforcement:** `scripts/check-first-party-doc-language.sh` (runs `scripts/check_first_party_doc_language.py`).

---

## Related documents

| Document | Purpose |
|----------|---------|
| `docs/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md` | Technical specification for critical-path migration |
| `docs/TDD-CONTRACT.md` | TDD and conformance from specs |
| `docs/SOUL.md` | Constitutional laws (Law #1 language) |
| `architecture/ADR-004-language-policy.md` | ASCII source + English first-party docs |
| `docs/NUMERIC-STANDARD-001.md` | GoldenFloat family, φ structure |
| `.cursor/rules/t27-ssot-math.mdc` | Cursor rule for AI agents |

---

## Amendments

Amendments to this constitution are made via pull request with an explicit charter version bump and rationale.
