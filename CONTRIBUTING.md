# Contributing to T27

Thank you for helping improve T27. This repository is **spec-first**: behavior lives in `.t27` specs; generated Zig / C / Verilog must not be hand-edited.

## Before you change code or specs

1. Read **[`SOUL.md`](SOUL.md)** at repo root — **canonical** constitutional law. Use **[`docs/SOUL.md`](docs/SOUL.md)** only as **expanded** reference (especially Law #1 detail); if they disagree, **root `SOUL.md` wins**.
2. Check **`OWNERS.md`** in the directory you touch (and the repo root **[`OWNERS.md`](OWNERS.md)**) for the **primary** Trinity agent / domain owner.
3. Open or reference a **GitHub Issue**; pull requests should satisfy the project **Issue Gate** where applicable (`Closes #N`).
4. Multi-agent coordination: **[`TASK.md`](TASK.md)** and **[`docs/TASK_PROTOCOL.md`](docs/TASK_PROTOCOL.md)**.

## NOW.md sync gates (Ring 033)

Keep **`docs/NOW.md`** current: it is the rolling snapshot for humans and agents (see [#141](https://github.com/gHashTag/t27/issues/141)).

1. **Local pre-commit:** run once after clone: **`bash scripts/setup-git-hooks.sh`** (sets `core.hooksPath` to **`.githooks/`**). Every commit is blocked unless `docs/NOW.md` contains **`Last updated: <today>`** (local calendar date).
2. **CI:** **`.github/workflows/now-sync-gate.yml`** requires **`docs/NOW.md`** in each PR/push to `master` and checks the date (UTC today or yesterday). **`.github/workflows/phi-loop-ci.yml`** runs **`scripts/check-now-sync.sh`** before the Rust build (must match **runner’s “today”**, typically UTC).
3. **`tri`:** **`./scripts/tri check-now`** runs the same script; **`gen*`** and **`compile*`** subcommands run it automatically before invoking `t27c`.

## Specs and tests

- New or changed `.t27` files should include **`test`**, **`invariant`**, and/or **`bench`** blocks as required by SOUL (TDD mandate).
- Run **`cargo build --release`** in `bootstrap/` after compiler changes.
- Before pushing, run **`./scripts/tri test`** (same as CI: `t27c suite`).

## Language

First-party Markdown and source comments must follow **English-first** policy (see root **`SOUL.md`** Article I; **`docs/SOUL.md`** Law #1 for expansion; **`architecture/ADR-004-language-policy.md`**).

## Security

See **[`SECURITY.md`](SECURITY.md)** for reporting vulnerabilities.
