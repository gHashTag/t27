# Contributing to T27

Thank you for helping improve T27. This repository is **spec-first**: behavior lives in `.t27` specs; generated Zig / C / Verilog must not be hand-edited.

## Before you change code or specs

1. Read **[`SOUL.md`](SOUL.md)** at repo root — **canonical** constitutional law. Use **[`docs/SOUL.md`](docs/SOUL.md)** only as **expanded** reference (especially Law #1 detail); if they disagree, **root `SOUL.md` wins**.
2. Open or reference a **GitHub Issue**; pull requests should satisfy the project **Issue Gate** where applicable (`Closes #N`).
3. Multi-agent coordination: **[`TASK.md`](TASK.md)** and **[`docs/TASK_PROTOCOL.md`](docs/TASK_PROTOCOL.md)**.

## Specs and tests

- New or changed `.t27` files should include **`test`**, **`invariant`**, and/or **`bench`** blocks as required by SOUL (TDD mandate).
- Run **`cargo build --release`** in `bootstrap/` after compiler changes.
- When present, run **`bash tests/run_all.sh`** before pushing (CI runs this on Ubuntu).

## Language

First-party Markdown and source comments must follow **English-first** policy (see root **`SOUL.md`** Article I; **`docs/SOUL.md`** Law #1 for expansion; **`architecture/ADR-004-language-policy.md`**).

## Security

See **[`SECURITY.md`](SECURITY.md)** for reporting vulnerabilities.
