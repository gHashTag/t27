# External / third-party trees

Third-party and vendored upstream code lives **only** under this directory (not at the repo root).

| Path | What |
|------|------|
| `external/opencode/` | [OpenCode](https://github.com/anomalyco/opencode) — **git submodule**. After clone: `git submodule update --init --recursive`. |

Project-local tooling (e.g. `portable-claude-setup/`) stays at the repository root; it is not an upstream library.
