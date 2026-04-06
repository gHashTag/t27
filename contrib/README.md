# contrib/ — Non-core project adjacency

This directory holds **first-party** code and scripts that are **not** the TRI-27 language core (specs → `t27c` → `gen/`).

| Path | Role |
|------|------|
| `contrib/backend/` | Control plane API, agent runner, sandbox images, legacy Zig shims (see `OWNERS.md` there). |
| `contrib/portable-claude-setup/` | Local agent/IDE key rotation and settings templates. |

**Vendored upstream trees** (OpenCode submodule, Kaggle datasets, etc.) live under **`external/`**, not here.

Domain ownership uses **`OWNERS.md`** files (see repository root and each major directory). This matches the **27-agent alphabet** conceptually: agents share trees; directories declare a **primary** owner, not “one folder per agent.”
