# NOW -- The roadmap had no tasks, so the Queen had nothing to choose (2026-09-27)

## A feeder for the roadmap, and a brief that tells a bee what the game is (Closes #3977)

- **The Queen was healthy and idle.** Read from the trios-agent-server deploy log on Railway: every round since the 02:14Z redeploy answered `chosen=null refusal="nothing to choose" candidates=497` with 0 running and 4 free slots. The 497 are every open issue here: 188 have no `## Boundary`, including the eight roadmap goals #4543-#4550, which have none on purpose. The other 309 do have one. 16 of them already have a bee pull request (12 open, 4 closed unmerged), and 293 are held by earlier attempts.
- **The roadmap was never turned into work.**
  - Stage 1 had `Port tools/...` issues: 23 completed, 45 closed as not planned, 10 open.
  - Stages 2-8 had none. The Queen files nothing herself: queen-tick.ts says "IT DOES NOT FILE ANYTHING".
  - The feeders that exist, `feed_untested` and `feed_empty_bodies`, read `specs/` and never the roadmap.
- **A task about another repository's file is not workable as written.** On 2026-09-26 at 09:32Z the re-file of #4813-#4832 put 20 bees on work that pointed at `src/hslm/cli.zig`, `apps/queen/Package.swift` and `src/tri-api/session_store.zig`. Those files live in gHashTag/trinity, and a bee sees only this checkout. The deploy log shows `ENOENT` on each of them, and the attempts came back as empty branches. Over the following 17 hours the Queen restarted #4611 about 12 times and #4827 about 8 times. Both kept writing outside their boundary: `debug_output.txt`, `stdout.txt` and `brain/unified_state.zig`.
- **`tools/queen/feed_roadmap.py` files one port task per hand-written file, round robin across stages 1-7.** Each task has one Boundary, the `specs/port/<repo>/<path>.t27` it creates. It also carries:
  - the source quoted verbatim behind a line-number gutter;
  - the six acceptance commands of the stage 1 port issues;
  - `docs/ROADMAP_BRIEF.md`;
  - the language and instruments from `docs/BEE_TOOLBELT.md`.
- **The gutter protects the Queen's criteria.** She reads criteria as the bullets under any line that starts with `#`. A quoted Python comment is such a line, and so is a C comment line ` * ...`. The self-test quotes a hostile source that has both, plus a line that looks like a Boundary heading. It reads the result with twins of her boundary and criteria parsers and still gets exactly one path and six criteria.
- **`docs/ROADMAP_BRIEF.md` is the goal of the game, in every task:**
  - law L0;
  - the eight stages with their repositories and targets;
  - how progress is measured (`roadmap-stack.mjs`);
  - how one file moves from task to pull request;
  - the four ways a turn is lost, each one measured.

  The second half is how to rewrite code into `.t27`. Every line was measured today with `t27c test-report` and Zig 0.16.0:
  - port the decision and pass the plumbing in as parameters;
  - a table from each construct in the original to its `.t27` idiom;
  - what compiles, and what does not with the symptom it gives (`match` is silently dropped, `"\r"` is not an escape, and string `==` compiles only against a literal, a parameter or a field);
  - a complete example that passes 3/3.
- **Of the 35 files already under `specs/port/tools/`, 3 build.** 7 do not parse, and 25 generate Zig that does not compile, although `spec-status` calls 24 of them IMPLEMENTED. That is why the brief says only `test-report` is proof.

## Measured before filing anything

- Dry run over the real repositories: 14 tasks. The stage 2 tasks are the Queen's own code (`queen-lease.ts`, `queen-brief-shape.ts`, `queen-report-lines.ts`). The others are trinity's `src/vsa_simple` and `src/vm`, the trios-doctor crate, and `fpga/` Verilog. Passed over:
  - 62 files already claimed by an issue;
  - 5 generated files;
  - 10 files with no function;
  - 1 file too big to quote.
- The criteria were checked against real ports, not only by reading them. I wrote a `.t27` for one generated Zig task and one Verilog task, then ran each task's commands exactly as printed. Every command printed what the task says it should: `present`, the function count, `0`, `IMPLEMENTED`, the test count and `0`.
- Stages 3 and 4 live in private repositories, 999-multibots-telegraf and vibee-gleam. A run without the `ROADMAP_READ_TOKEN` secret names them and feeds the rest. Stage 8 is refused with its reason: t27c has no interface target.

## The ledgers this moves, in the same commit

- `tools/census/quiet.txt` and `shell.txt`:
  - workflow files 63 -> 64;
  - jobs 84 -> 85;
  - run steps 278 -> 280, of which runner-run steps 257 -> 259;
  - "named a path but not quiet" 147 -> 150.

  Each change is the new `queen-feed-roadmap.yml` and nothing else.
- `check_pr_branch_filters.py`:
  - the new workflow is classified as not merge-critical;
  - `needs-boundary.yml` and `queen-export-push.yml` are classified too. Master already carried 26 unclassified workflows against a ceiling of 24, so this brings the count back to 24.
- `pusher.py` now dispatches the roadmap feeder with the other two feeders when the swarm runs out of fuel.
