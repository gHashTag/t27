# NOW — feeding the degenerate inputs to my own new flag (2026-08-23)

`--dir` shipped one iteration ago. Four degenerate inputs; **two of them exited 0**.

| input | before | after |
|---|---|---|
| a directory that does not exist | refused, names the flag | unchanged |
| a **file** instead of a directory | died with `git status failed` | refused as *not a directory* |
| an **empty** directory | header, no rows, **exit 0** | refused |
| a directory **outside any git work tree** | ran, **exit 0** | refused |

- The gate now refuses each of these by name, and the refusal says which mistake was made rather than which instrument noticed.
- The NOW entry this replaced had no bullet at all — a table instead — and `check-now-freshness` rejected it. Its four requirements are a heading, a bullet, a dated filename and a date in the UTC window; a table is none of them. Third time this campaign that a gate caught me rather than the other way round.

**The empty directory is the vacuous pass, in the command whose subject is vacuous passes.** A table with a header and no rows reads exactly like a clean suite. I had widened the file filter the day before *because* a wrong filter produced this table — and never made the empty result say so, so the hazard survived the fix aimed at it.

**The one outside a work tree has teeth.** This command rewrites each gate in place and restores it, and that restore is only a promise because `git checkout` can undo an interrupted run. Outside a repository there is no undo — and the dirty-tree guard that exists for exactly this passed silently, because `git status` *fails* there and its empty stdout reads as clean. A guard whose failure mode is indistinguishable from its success.

**And then I wrote the bug into its own fix.** The first version of the empty-directory refusal printed `FAIL: no gate scripts under …` and returned `Ok(())` — announcing that nothing had been measured, and exiting 0. Measured rather than noticed: exit 0 before, exit 1 after.

Test verified RED on the restored defect. Default behaviour unchanged, 14 unit tests green.

**The rule:** after adding an option, spend one iteration giving it the inputs you had no reason to try — nothing, empty, wrong type, wrong place. The author of an option builds the case that motivated it, which is by construction none of those.
