# Contributing to T27

Thank you for helping improve T27. This repository is **spec-first**: behavior lives in `.t27` specs; generated Zig / C / Verilog must not be hand-edited.

## Before you change code or specs

1. Read **[`SOUL.md`](SOUL.md)** at repo root — **canonical** constitutional law. Use **[`docs/nona-03-manifest/SOUL.md`](docs/nona-03-manifest/SOUL.md)** only as **expanded** reference (especially Law #1 detail); if they disagree, **root `SOUL.md` wins**.
2. Check **`OWNERS.md`** in the directory you touch (and the repo root **[`OWNERS.md`](OWNERS.md)**) for the **primary** Trinity agent / domain owner.
3. Open or reference a **GitHub Issue**; pull requests should satisfy the project **Issue Gate** where applicable (`Closes #N`).
4. Multi-agent coordination: **[`docs/coordination/TASK_PROTOCOL.md`](docs/coordination/TASK_PROTOCOL.md)**. A PR may add one **`docs/now/`** entry; it is optional -- see **[NOW entries](#now-entries-optional-one-file-each)** below (coordination anchor: [#141](https://github.com/gHashTag/t27/issues/141)).

## NOW entries (optional, one file each)

No hook and no CI job asks for a `docs/now/` entry or a `NOW.md` update: the NOW gate was removed by owner decision 2026-10-04 ([#5935](https://github.com/gHashTag/t27/issues/5935)). An entry is optional. Which checks block a merge is the ruleset's answer: `gh api repos/gHashTag/t27/rules/branches/master`.

When you do write one, it is one new file, **`docs/now/<YYYY-MM-DD>-<slug>.md`**. The layout, why it replaced the single file, and the date window are in **[`docs/now/README.md`](docs/now/README.md)**. The entry shape is defined in one place, the docstring of **[`tools/check_now_entry_shape.py`](tools/check_now_entry_shape.py)**; in short: first line `# NOW -- <title> (YYYY-MM-DD)` dated like the filename, a `## ` section, at least one `- ` bullet that says something. Root **`NOW.md`** and **`docs/NOW.md`** are not read by any NOW check. **`docs/NOW.md`** is a frozen archive: do not edit it. `scripts/ci/now-sync-gate-diff.sh`, when you run it, refuses a range that edits it unless a commit message carries an **`Archive-Repair:`** trailer.

1. **Write the entry:** **`./scripts/tri now add "<title>" --bullet "<what changed>" --closes <N>`** (or `--refs <N>` for an issue that must stay open). `now` is a subcommand of the Rust **`tri`** binary (`cli/tri`), not of `t27c`, so build it first: **`cargo build --release -p tri`**. Without that binary, write the file by hand in the shape above; `tri now add` writes the section as `## <title> (Closes #N)`, and the checker requires a `## ` section but not the issue reference.
2. **Check it, if you want to.** These are tools anyone may run; no hook and no CI job runs them. From repo root, after `git fetch origin master` and committing the entry:
   ```bash
   PR_BASE_SHA=$(git merge-base origin/master HEAD) python3 tools/check_now_entry_shape.py
   GITHUB_EVENT_NAME=pull_request PR_BASE_SHA=$(git merge-base origin/master HEAD) \
     PR_HEAD_SHA=$(git rev-parse HEAD) bash scripts/ci/now-sync-gate-diff.sh
   ```
   The first checks the shape of every `docs/now/` entry your change adds. The second checks that the range **adds** an entry whose filename date is within **[yesterday .. tomorrow] UTC** and which has a heading and a bullet. With the `tri` binary built, **`tri now check`** asks the same shape question and **`tri hooks pre-push`** the same range question. **`tri hooks now-gate`** and **`t27c check-now`** (also reachable as **`./scripts/tri check-now`**, which needs a built `t27c` or **`TRI_T27C=<path>`**) are weaker: they look at the whole **`docs/now/`** directory for an entry dated in the UTC window, so any fresh entry already on `master` satisfies them.
3. **Local hooks:** run **`bash scripts/setup-git-hooks.sh`** once after clone (sets `core.hooksPath` to **`.githooks/`**). **`.githooks/pre-commit`** runs **`tri hooks pre-commit`** when it finds a `tri` binary (`target/{debug,release}/tri`, else `tri` on `PATH`): conflict markers and the other commit-time gates in `cli/tri/src/hooks.rs`. It does not look for a NOW entry.

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
- After compiler changes, build the release binary once before pushing (`cargo build --release -p t27c`); iterate with the faster builds in [Build speed](#build-speed-the-t27c-inner-loop).
- Before pushing, run **`./scripts/tri test`** (same as CI: `t27c suite`).

## Build speed: the t27c inner loop

`cargo build --release -p t27c` is the build CI and releases use, and the slowest one to iterate with: the release profile has no incremental compilation, so a one-line edit recompiles the whole `t27c` crate. While you work, use one of the first three rows.

| you need | command (from the repo root) | one-line edit, then rebuild |
|---|---|---|
| type errors only | `cargo check -p t27c` | 2.4-3.3 s |
| a binary that runs | `cargo build -p t27c` | 3.2-4.1 s |
| a release binary that runs | `CARGO_PROFILE_RELEASE_INCREMENTAL=true cargo build --release -p t27c` | 3.2-5.0 s |
| the CI / release build | `cargo build --release -p t27c` | 29.8-33.9 s |

Measured 2026-10-04 on an idle M1 Pro (8 cores), adding one function to `bootstrap/src/suite.rs`; the debug row had `CARGO_PROFILE_DEV_DEBUG=0`. Each figure is a range over repeated edits after one warm-up build. `bootstrap/src/compiler.rs` (44.6k lines) was not timed on an idle machine; splitting it is tracked in [#5905](https://github.com/gHashTag/t27/issues/5905).

- **Do not benchmark or ship an incremental release build.** Incremental mode splits the crate into many more codegen units and optimises less across them, so its speed and size are not the release build's. Run timings, `tri test` before a PR, and anything you hand to someone else on a plain `cargo build --release`.
- **`./scripts/tri` prefers `target/release/t27c`** over `target/debug/t27c` when both exist, so after a debug rebuild it can still run an older release binary. Point it at the one you just built: `TRI_T27C=target/debug/t27c ./scripts/tri test`.
- **Editing `bootstrap/src/compiler.rs` changes the seal.** `build.rs` refuses to build until `bootstrap/stage0/FROZEN_HASH` carries the new SHA-256 of that file. Moving the seal is the M5 freeze ceremony in [`FROZEN.md`](FROZEN.md) section 5. Its step 3 command, `cargo run --release -- frozen-digest`, cannot run after the edit, because the rebuild hits the same check ([#5928](https://github.com/gHashTag/t27/issues/5928)). Until that is fixed, print the line with a binary built before the edit (`./target/debug/t27c frozen-digest`), or with `shasum -a 256 bootstrap/src/compiler.rs`, which gives the same digest.

## Language

First-party Markdown and source comments must follow **English-first** policy (see root **`SOUL.md`** Article I; **`docs/nona-03-manifest/SOUL.md`** Law #1 for expansion; **`architecture/ADR-004-language-policy.md`**).

## Security

See **[`SECURITY.md`](SECURITY.md)** for reporting vulnerabilities.
