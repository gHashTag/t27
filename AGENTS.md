# AGENTS — Trinity S³AI / t27

This file is the **repository entry point** for humans and coding agents. It summarizes **where law lives** and **how to work safely** in this tree.

---

## 1. Read first (constitutional stack)

| Order | File | Role |
|------:|------|------|
| 1 | [`SOUL.md`](SOUL.md) | **Canonical** constitution (language policy, TDD mandate, validation). |
| 2 | [`docs/SOUL.md`](docs/SOUL.md) | Expanded reference; if it conflicts with root `SOUL.md`, **root wins**. |
| 3 | [`docs/T27-CONSTITUTION.md`](docs/T27-CONSTITUTION.md) | **SSOT-MATH**, **LANG-EN**, and related articles. |
| 4 | [`TASK.md`](TASK.md) + [`docs/TASK_PROTOCOL.md`](docs/TASK_PROTOCOL.md) | Multi-agent coordination, locks, anchor issue. |
| 5 | [`OWNERS.md`](OWNERS.md) | Domain ownership; each major directory may have its own `OWNERS.md`. |

Supporting: [`CONTRIBUTING.md`](CONTRIBUTING.md), [`SECURITY.md`](SECURITY.md), [`architecture/ADR-004-language-policy.md`](architecture/ADR-004-language-policy.md).

---

## 2. Agent model (27-letter alphabet)

- **Full alphabet and roles:** [`docs/AGENTS_ALPHABET.md`](docs/AGENTS_ALPHABET.md) — canon for T, N, P, C, B, etc.
- **Operational agent specs (e.g. watchdogs, schemas):** [`docs/AGENTS.md`](docs/AGENTS.md) — complements the alphabet; not a second constitution.

Use **domain directories**, not “one folder per agent.” Primary contact for a path is the **Primary** listed in the nearest `OWNERS.md`.

---

## 3. Non-negotiables for changes

1. **Specs are source of truth** — behavior belongs in `.t27` / `.tri`; generated `gen/` output is not hand-edited (except documented exceptions).
2. **TDD inside specs** — new or changed specs need `test`, `invariant`, and/or `bench` where SOUL requires it.
3. **English + ASCII** — first-party Markdown and source comments per **LANG-EN** and **ADR-004**; grandfathered paths only in [`docs/.legacy-non-english-docs`](docs/.legacy-non-english-docs).
4. **No new Python on the verification critical path** — see **SSOT-MATH** and [`docs/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md`](docs/TZ-T27-001-NO-PYTHON-CRITICAL-PATH.md).
5. **Issue gate** — PRs should link issues (`Closes #N`) where project policy requires it.
6. **Ring / gold work** — follow [`docs/GOLDEN-RINGS-CANON.md`](docs/GOLDEN-RINGS-CANON.md) for parser/compiler/spec changes; compiler seal path: `bootstrap/stage0/FROZEN_HASH`.

---

## 4. Layout reminders (after repo hygiene refactors)

- **Core:** `specs/`, `compiler/`, `bootstrap/`, `gen/`, `conformance/`, `tests/`.
- **Non-core services & tooling:** `contrib/backend/`, `contrib/portable-claude-setup/`.
- **Vendored / datasets / upstream:** `external/` (e.g. OpenCode submodule, `external/kaggle/`).

---

## 5. Cursor / automation

- Rule file: [`.cursor/rules/t27-ssot-math.mdc`](.cursor/rules/t27-ssot-math.mdc) — keep in sync with **SSOT-MATH** and this entry point.

---

**φ² + 1/φ² = 3 | TRINITY**
