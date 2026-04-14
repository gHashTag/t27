# Contributing to T27

Thank you for helping improve T27. This repository is **spec-first**: behavior lives in `.t27` specs; generated Zig / C / Verilog must not be hand-edited.

## Before you change code or specs

1. Read **[`SOUL.md`](SOUL.md)** at repo root — **canonical** constitutional law. Use **[`docs/nona-03-manifest/SOUL.md`](docs/nona-03-manifest/SOUL.md)** only as **expanded** reference (especially Law #1 detail); if they disagree, **root `SOUL.md` wins**.
2. Check **`OWNERS.md`** in the directory you touch (and the repo root **[`OWNERS.md`](OWNERS.md)**) for the **primary** Trinity agent / domain owner.
3. Open or reference a **GitHub Issue**; pull requests should satisfy the project **Issue Gate** where applicable (`Closes #N`).
<<<<<<< Updated upstream
4. Multi-agent coordination: root **[`NOW.md`](NOW.md)** (rolling snapshot) and **[`docs/coordination/TASK_PROTOCOL.md`](docs/coordination/TASK_PROTOCOL.md)**. **CI** also requires every PR/push to touch **[`docs/NOW.md`](docs/NOW.md)** (mirror / coordination copy; see [#141](https://github.com/gHashTag/t27/issues/141)).

## NOW.md sync gates (Ring 033)

Keep **both** **`NOW.md` (repo root)** and **`docs/NOW.md`** aligned for handoffs: root is what **`t27c check-now`** reads; **`docs/NOW.md`** must appear in every PR diff for **`now-sync-gate.yml`**.

1. **Local pre-commit:** run once after clone: **`bash scripts/setup-git-hooks.sh`** (sets `core.hooksPath` to **`.githooks/`**). Every commit is blocked unless **root `NOW.md`** **Last updated** line includes **today’s calendar date `YYYY-MM-DD`** (checked against your **local** date when `tri check-now` runs). Prefer **human-readable local wall time** in that line, not UTC `Z`, unless you work in UTC.
2. **CI:** **`.github/workflows/now-sync-gate.yml`** requires **`docs/NOW.md`** in each PR/push to `master` and checks the date (UTC today or yesterday). **`.github/workflows/phi-loop-ci.yml`** builds **`t27c`**, then runs the same gates through **`./scripts/tri`** (`check-now`, `test`, `validate-conformance`, `validate-gen-headers`). Calendar date for **`tri check-now`** must match the runner’s local “today” (typically UTC on GitHub Actions).
3. **`tri`:** **`./scripts/tri check-now`** forwards to **`t27c check-now`** (root **`NOW.md`**); **`gen*`** and **`compile*`** run that gate automatically before invoking codegen.
=======
4. Multi-agent coordination: **[`NOW.md`](NOW.md)** (root) and **[`docs/coordination/TASK_PROTOCOL.md`](docs/coordination/TASK_PROTOCOL.md)**.

## NOW.md sync gates (Ring 033)

Keep **`NOW.md`** (repository root) current: rolling snapshot and coordination surface for humans and agents (see [#141](https://github.com/gHashTag/t27/issues/141)).

1. **Local pre-commit:** run once after clone: **`bash scripts/setup-git-hooks.sh`** (sets `core.hooksPath` to **`.githooks/`**). Every commit is blocked unless **`NOW.md`** **Last updated** line includes **today’s calendar date `YYYY-MM-DD`** (checked against your **local** date when `tri check-now` runs). Prefer **human-readable local wall time** in that line, not UTC `Z`, unless you work in UTC (see **`NOW.md`** header template).
2. **CI:** **`.github/workflows/now-sync-gate.yml`** requires **`NOW.md`** in each PR/push to `master` and checks the date (UTC today or yesterday). **`.github/workflows/phi-loop-ci.yml`** builds **`t27c`**, then runs the same gates through **`./scripts/tri`** (`check-now`, `test`, `validate-conformance`, `validate-gen-headers`). Calendar date for **`tri check-now`** must match the runner’s local “today” (typically UTC on GitHub Actions).
3. **`tri`:** **`./scripts/tri check-now`** forwards to **`t27c check-now`**; **`gen*`** and **`compile*`** run that gate automatically before invoking codegen.
>>>>>>> Stashed changes

## PHI Loop CI — why assistants do not “see” red builds

GitHub Actions does **not** push logs into Cursor or chat by default. To inspect failures you (or an agent with shell + `gh`) must **pull** them:

```bash
gh run list --workflow=phi-loop-ci.yml --limit 8
gh run view <run-id> --log-failed
# or, from repo root:
bash scripts/ci/phi-loop-last-failure.sh
```

Install the **GitHub Actions** extension in the editor if you want in-UI log links. After **`git push`**, run **`gh run watch`** to stream the current workflow.

**Common `tri test` failure — seal verify:** new `.t27` under `specs/` needs a saved seal:

```bash
./scripts/tri seal specs/path/to/module.t27 --save
```

If **`gen_hash_*` mismatches** appear for many specs, the compiler output changed; refresh seals intentionally (same `--save` per spec or batch policy from maintainers) and commit **`.trinity/seals/*.json`**.

## Seal discipline

1. **Every spec under `specs/`** that you add or materially change should have a matching entry under **`.trinity/seals/<module>.json`**. Generate or refresh with:
   ```bash
   ./scripts/tri seal specs/path/to/module.t27 --save
   ```
2. **Pull requests:** **[`.github/workflows/seal-coverage.yml`](.github/workflows/seal-coverage.yml)** runs when `specs/**/*.t27`, **`.trinity/seals/**`, or **`conformance/**`** change. It lists changed `specs/**/*.t27` files in the PR and runs **`t27c validate-seals --pr-files …`** so missing or stale seals fail CI.
3. **Hardening (maintainers, optional):** mark the **Seal Coverage Gate** workflow as a **required status check** under branch protection; extend trigger paths further if new layouts appear.
4. **Traceability:** seal-related fixes should reference the issue (e.g. [#131](https://github.com/gHashTag/t27/issues/131)) in the PR body when applicable.

## Specs and tests

- New or changed `.t27` files should include **`test`**, **`invariant`**, and/or **`bench`** blocks as required by SOUL (TDD mandate).
- Run **`cargo build --release`** in `bootstrap/` after compiler changes.
- Before pushing, run **`./scripts/tri test`** (same as CI: `t27c suite`).

## Language

First-party Markdown and source comments must follow **English-first** policy (see root **`SOUL.md`** Article I; **`docs/nona-03-manifest/SOUL.md`** Law #1 for expansion; **`architecture/ADR-004-language-policy.md`**).

## Security

See **[`SECURITY.md`](SECURITY.md)** for reporting vulnerabilities.
