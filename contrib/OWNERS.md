# OWNERS — contrib/

## Primary

**A-Architect** (layout) with **B-Builder** for `backend/` CI and images.

## Contents

| Subtree | Primary | Role |
|---------|---------|------|
| `backend/` | **B-Builder** | Rust-adjacent services, Docker, legacy Zig shims |
| `portable-claude-setup/` | **A-Architect** / tooling | Local keys and editor templates |

## Dependencies

- Core specs and compiler are **not** defined here.
- Docker workflows under `.github/workflows/` point at `contrib/backend/`.
