# OWNERS — repository root

## Primary

**A-Architect** — top-level layout, cross-cutting policy docs, coordination entrypoints (`README.md`, `SOUL.md`, `TASK.md`).

## Notes

- **Core language path:** `specs/` → `bootstrap/` (`t27c`) → `gen/` → `conformance/` / `tests/`.
- **Non-core adjacency:** `contrib/` (API, runners, portable setup).
- **Vendored / datasets / upstream:** `external/`.
- Each major directory has its own **`OWNERS.md`** with **Primary**, **Dependencies**, and **Outputs** where helpful.

## Agent alphabet

Full 27-agent mapping lives in **`docs/AGENTS_ALPHABET.md`**. Directory ownership uses **domains** (many agents may touch one tree); the **Primary** owner is the default reviewer for structural changes.
