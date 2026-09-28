# Auto-Loop 2026-09-29 — continuous improvement, one bounded iteration at a time

Started by the owner's overnight instruction (2026-09-28 evening, user asleep):

> исследуй слабые места задачи, исследуй конкурентов по теме, создай
> декомпозированный план и реализуй все и в конце отчет и три варианта
> сотрудничества для следующего лупа. автономно все сам делай без остановки!
> создай непрерывный цикл улучшений… и чтобы новый цикл крона не ломал прошлую
> работу. Самокритично к своей работе и поиск аномалий и самовосстановления.

This directory is that loop's permanent record: the charter (`README.md`, this
file), the **live state** (`state.md` — updated at the end of every iteration),
the decomposed plan (`plan.md`), the measured baseline (`baseline.md`), and one
report per iteration (`iterations/NN-*.md`).

## How a cron firing finds its place (the not-breaking-prior-work protocol)

Every 15 minutes the same instruction arrives again. Each firing:

1. Reads `state.md` first. If it names an in-flight task, that task is resumed
   or safely closed out — never duplicated.
2. Checks `git -C /tmp/t27_carry status` — a dirty worktree means a crashed
   iteration; the files are inspected, then committed or reverted (only files
   this loop created may be reverted).
3. Takes the first open item from `plan.md` whose blockers are clear.
4. Ends by updating `state.md`, pushing the branch, and appending a
   self-critique line to the iteration report.

## The safety constitution (hard rules, not guidance)

1. Work lands **only** on branch `loop/auto-2026-09-29`. Never on master, never
   a force-push to any shared branch, never inside PRs #5078 / #5081 (owner's).
2. The remote claim `tri loop claim auto-2026-09-29` is held for the whole
   pass and released at the end. If it is found released, re-claim or stop.
3. Suite reds may only **shrink**. `baseline.md` holds the measured starting
   inventory; an iteration that grows it reverts itself.
4. No mass reseal. A seal is rewritten only for a spec whose new generated
   output has been verified deliberately, one batch at a time.
5. Anything needing the `workflows` OAuth scope or a PR merge is filed as
   BLOCKED-user in `state.md` and skipped, not forced.
6. One `docs/now/` entry per PR via `tri now add` (and revert its
   `.gitattributes` side-write before committing).

## The measured weak points this loop attacks

All numbers were measured on 2026-09-28/29, not assumed — full detail in
`plan.md`:

- master's Lean CI red since Sep 24 (carried fix sits in PR #5081, unmerged);
- 9 corpus specs unparseable by the current compiler (typecheck cluster);
- `verilog_bench_harness.t27` truncated since Jul 31 (#5079);
- 574 stale seals on the master base;
- `.tri` codegen emits silently-wrong output; `pub type` aliases silently
  dropped; control-flow bodies degrade to `@compileError` (measured, 28/400
  sampled specs);
- `t27c gen` accepts unknown flags by creating literal `--out/` directories;
- coq-kernel / emit-bitexact CI phases (status checked per iteration via
  `tri red`).

## Three cooperation variants for the next loop

Collected at close-out in the final iteration report — see
`iterations/` for the latest full list with evidence.
