# MIGRATION_AUDIT.md

Audit of `.py` and `.sh` files in the t27 repository for migration to Rust /
centralization in the `tri` CLI, per issue #592.

**Scope:** every `.py`/`.sh` file outside `target/`, `node_modules/`,
`contrib/solana/node_modules/`, `contrib/portable-claude-setup/`,
`research/trinity-pellis-paper/`, and `.git/`.

**Classes:**

- **A — Infra critical path:** pre-commit / push hooks, CI gates, scripts
  invoked from `.githooks/` or `.github/workflows/`. Must be rewritten in
  Rust and exposed via `tri hooks <name>`.
- **B — FPGA tooling duplicates:** Python implementations of cable/flash
  programming whose behaviour now lives in `cli/dlc10` (Rust, silicon-
  verified). Delete.
- **C — Research / examples / contrib backend:** standalone scripts not on
  any commit / push / CI critical path. Keep as-is for this migration.
- **D — Bootstrap stubs:** `bootstrap/t27c.py`,
  `bootstrap/src/memory/ace_step_wrapper.py`. The real `t27c` is the
  Rust binary built from `bootstrap/`. Stubs are unused at the critical
  path; keep for this migration, address separately.

## Decisions for this PR (#593, Closes #592)

| Path | Class | Decision | Rationale |
|------|-------|----------|-----------|
| `tools/dlc10_jtag.py` | B | **Delete** | Behaviour reimplemented in `cli/dlc10` lib (silicon-verified: IDCODE `0x13631093`, SRAM blink, SPI flash). |
| `tools/tri_fpga/__init__.py` | B | **Delete** | Same as above; package empty. |
| `tools/tri_fpga/cli.py` | B | **Delete** | Same as above; replaced by `tri fpga ...`. |
| `.claude/hooks/check-l1-traceability.sh` | A | **Rewrite + keep stub** | Wrapped via `tri hooks l1-check`; the `.sh` becomes a one-line forwarder so any existing harness wiring keeps working. |
| `.claude/hooks/session-gate.sh` | A | Keep | Calls into `cargo run`; not on commit/push path, harness-only. Out of scope for this PR. |
| `.claude/hooks/stop-hook-guard.sh` | A | Keep | Session-stop accounting only; not on commit/push gate. |
| `.claude/hooks/inject-notebook-context.sh` | A | Keep | NotebookLM telemetry, not a gate. |
| `.githooks/pre-commit` | A | Keep | Already delegates to `scripts/tri check-now` (Rust `t27c`). Out of scope. |
| `.githooks/pre-push` | A | Keep | NotebookLM gate, no Rust replacement available yet. |
| `.githooks/post-merge` | A | Keep | NotebookLM sync; non-blocking. |
| `scripts/tri` | A | Keep | Already a 17-line forwarder to the Rust `t27c` binary. |
| `scripts/ci/now-sync-gate-diff.sh` | A | Keep | CI-only diff check against GitHub event env; thin glue. |
| `scripts/ci/phi-loop-last-failure.sh` | A | Keep | Diagnostic only, off the merge gate. |
| `scripts/aggregate-experience.sh` | A | Keep | Triggered by `brain-seal-refresh.yml` workflow; not commit-gate. |
| `.claude/skills/tri/scripts/*.sh` | A | Keep | Skill-internal helpers, not invoked from any gate. |
| `scripts/fpga/build.sh`, `scripts/fpga/flash.sh` | C | Keep | Vivado wrappers — orthogonal to the DLC10 USB driver. |
| `examples/fpga/qmtech_minimal/build.sh` | C | Keep | Example; not on critical path. |
| `bootstrap/t27c.py`, `bootstrap/src/memory/ace_step_wrapper.py` | D | Keep | Out of scope; handled separately. |
| `contrib/backend/**/*.py` | C | Keep | NotebookLM / music-generator backends; not on commit gate. |
| `clara-bridge/**/*.py` | C | Keep | Research bridge. |
| `benchmarks/**/*.py`, `research/**/*.py`, `scripts/ultra_engine_v*.py`, `scripts/pysr_*.py`, `scripts/pslq_*.py`, `scripts/trinity-pellis-pipeline/**/*.py`, `external/kaggle/**/*.py`, `docs/clara/examples/*.py` | C | Keep | Research / examples; orthogonal. |
| `bindings/python/**/*.py` | C | Keep | Python bindings to golden-float crate. |
| `conformance/kepler_newton_tests.py` | C | Keep | Conformance helper not on gate. |
| `test_notebooklm.py`, `test_notebooklm_venv.sh` | C | Keep | Manual smoke tests at repo root. |
| `scripts/tri-*.py`, `scripts/audit_discovery.py`, `scripts/check_first_party_doc_language.py`, `scripts/verify_*.py`, `scripts/lee_*.py`, `scripts/compare_*.py`, `scripts/fix_*.py`, `scripts/overnight_research_agent.py`, `scripts/print_pellis_seal_decimal.py`, `scripts/unified_search_all.py`, `scripts/wrapup/*.py` | C | Keep | Standalone helpers; none referenced from `.githooks/` or commit-path workflows. |
| `scripts/install-*.sh`, `scripts/setup-git-hooks.sh`, `scripts/auto-*.sh`, `scripts/check-conflicts.sh`, `scripts/bulk-create-notebooks.sh`, `scripts/generate_episodes.sh`, `scripts/git_commands_tasks_1_4.sh`, `scripts/mcp-wrapper.sh`, `scripts/phi-loop-stack.sh`, `scripts/run_v51_multiple.sh`, `scripts/test-agent-bridge.sh`, `scripts/verify-notebooklm.sh`, `scripts/verify-ssot-integration.sh` | C | Keep | One-shot setup / human-invoked utilities; not gated. |

## New `tri` subcommands added by this PR

| Command | Behaviour |
|---------|-----------|
| `tri fpga idcode` | Read DLC10 JTAG IDCODE (was `tools/dlc10_jtag.py idcode` / `dlc10 idcode`). |
| `tri fpga sram <bit> [--verbose]` | Program FPGA SRAM (volatile). |
| `tri fpga program <bit> [--no-verify]` | Program SPI flash (persistent). |
| `tri fpga flash-id` | Read SPI flash JEDEC ID. |
| `tri fpga status` | Raw CFG_OUT status. |
| `tri fpga debug [--no-jstart]` | Decode 7-series CFG registers. |
| `tri hooks l1-check` | Pure-Rust port of `.claude/hooks/check-l1-traceability.sh` (commit-message issue-reference gate). |
| `tri hooks now-gate` | Verifies `docs/NOW.md` "Last updated" is today's date (UTC). |
| `tri hooks pre-commit` | Runs the migrated gates in sequence (currently `now-gate` + `l1-check`). |

## Crate restructuring

- `cli/dlc10` is now a **lib crate** (`lib.rs` already exposed all primitives;
  the `[lib]` target is now consumed by both `cli/flash-spi` and `cli/tri`).
  The `dlc10` binary stays as a thin diagnostic wrapper.
- `cli/flash-spi` continues to ship a `flash-spi` binary that re-exports the
  same logic; for new work users are pointed at `tri fpga program`.
- `cli/tri` gains a `fpga` subcommand backed by `dlc10::Dlc10` directly
  (no shell-out, no Python).

## What was NOT changed and why

- `scripts/tri` (the Bash forwarder to `t27c`): already a 17-line thin
  wrapper around a Rust binary. Rewriting it to Rust would be circular
  (Rust binary launching a Rust binary). Constitution allows it under
  L7-UNITY since it never implements logic.
- `.githooks/pre-commit` and `pre-push`: still call `scripts/tri check-now`
  and a NotebookLM ID guard. These remain Bash because they're glue and
  the Rust replacement for the NotebookLM client is out of scope.
- `bootstrap/t27c.py`: scaffolding stub; deletion would force a
  bootstrap-tooling change that is orthogonal to this issue.

---

**Issue:** #592 — Closes #592 in the corresponding commit.
