---
name: tri-git-coverage
description: >-
  Get a t27 spec to 100% coverage, distinguish real t27c failures from the
  three known false alarms, and build an unattended improvement loop that
  cannot destroy finished work. Use when the owner says "покрытие 100%",
  "три гит", "замкни цикл", "авто-мердж", "не сломай прошлую работу", or when
  a t27c check reports a problem you suspect is a tooling bug rather than a
  spec defect.
---

# tri git coverage and safe loops

Hard-won facts from the ring-31 session. Three `t27c` checks lie, coverage
collecting misses real tests, and a running enhancement loop may be pure echo.

## The coverage recipe (0% -> 100% on `compiler/cli/git.t27`)

`t27c coverage` counts a function as covered only when an `ExprCall` with that
**bare** name appears inside a `TestBlock | InvariantBlock | BenchBlock`.
Consequences:

1. **Tests must sit inside the module.** Tests outside the module had to be
   qualified (`git_commands.fn`), so `ExprCall.name` was `"git_commands.fn"`,
   which never matches the bare `fn_names`. Move tests inside; close the
   module with `}` at EOF.
2. **`test` blocks must be braced** to register as a test block for coverage.
   A no-brace `test` inside a module parses but scores 0 (experiment A).
   No-brace `invariant`/`bench` *do* register (experiment B) — but brace
   everything for consistency.
3. **A reference is not a call.** `_ = git_commit;` is an `ExprCall` only if
   written as `git_commit();`. Bare references score nothing. Replace mock
   stubs with real smoke calls.
4. Cover every helper: after the real calls, check
   `t27c coverage <file>` and add tests for each name still listed under
   `Untested`.

Then run the chain: `parse -> spec-status -> test -> coverage -> gen -> seal`.
A fresh spec should show `Implemented: N, Sealed: N (MATCH)`.

## The three known false alarms (do NOT "fix" a spec to silence them)

| Check | Lie | Truth |
|-------|-----|-------|
| `t27c lint` | `WARN: fn 'x' has no test or invariant`, `issues == fn_count` for every file | Lint looks for `TestBlock` as a **direct child** of `FnDecl`; tests are always siblings. Fires on 100% coverage. **Tooling bug** — filed, not silenced. |
| `t27c test` | Passes assertions that are never evaluated | A planted `assert == 999` passed. Treat a green run as "parsed and counted", not "proved". Cross-check with coverage and by deliberately breaking an assertion. |
| `run_coverage` in `bootstrap/src/main.rs` | Only scans the 3 top-level `ast.children` for fn/test blocks | Nested `fn`s inside another block are invisible; `pub fn` shape differences shift results. If numbers look wrong, read `run_coverage` before doubting the spec. |

Never rewrite a spec to satisfy a check you have proved is wrong — that is
metric-gaming, and it hides the real defect.

## Honest anomaly logging

When you log a finding, count real counts. In one pass "34 issues" was
actually **27 unique** (7 duplicates). Say `34 (27 unique)`, or the report is
noise.

## Auto-merge (L1 + green gate)

- Every change needs `Closes #N` (L1). Create the issue *first*.
- Dry-run before trusting it:
  `gh workflow run auto-merge-ready-prs.yml --dry-run` then
  `gh run list --workflow=auto-merge-ready-prs.yml` and `gh run view <id>`.
- It only merges green PRs; zero green PRs is a correct no-op, not a failure.

## The safe continuous loop

- **Logic lives in a `.t27` spec** (L2 + L4), scheduler is a declarative
  GitHub workflow calling `t27c` (L7: no new `.sh` on the critical path).
  See `specs/pipeline/loop_guard.t27`: idempotency, green-only,
  preservation of pending work, issue binding, anomaly severity.
- **Read a running loop before you kill it.** The legacy
  `COSMIC-ENHANCEMENT-LOOP-*.sh` processes were confirmed to do only
  `echo` + `sleep 300` + `grep` — no `git`, no writes. They are harmless to
  files but print fabricated metrics ("100% ACHIEVED", "Anomalies: 0").
  They are the owner's processes: **document and recommend, do not kill
  without asking.**
- A loop must **refuse to act** when: work was already applied, any check is
  red, uncommitted work sits in scope, or no issue is bound. That refusal is
  what makes "the new cron must not break previous work" true.

## tri git subcommands

Parse/help work: `t27c run specs/git/tri_workflow.t27 release-plan` and <!-- not built as a t27c subcommand -->
`status-report` (ignore the "please install release build" note — the stub
prints it by design, so confirm by overriding, never by trusting a bare run).
The git release rules live in `specs/git/release_rules.t27`.

## Traps

- 2026-10-24 target/release/t27c was rebuilt by an unseen process mid-measurement (8.1MB Oct23 to 10.6MB Oct24), flipping from rejecting test-report to accepting it; record the binary mtime+size beside every tri test number or two runs of the same command measure different programs (12001)
- 2026-10-24 a 0-based index assert is wrong by one more often than not: bar_at over ' | ' at offset 7 not 8, and starts_at('xghp_',1,5,'ghp_') is TRUE not false; run the test before trusting the sign of your own negative case (12001)
- 2026-10-24 coverage-to-100 is safe when the spec already compiles and passes (lesson/tick/e2e/experience: just add bare-name calls); it is NOT a fix for a BLOCKED spec -- operations.t27 (5%) and diff.t27 (37%) do not compile, so raise the compiling ones first and file the broken ones (12001)
