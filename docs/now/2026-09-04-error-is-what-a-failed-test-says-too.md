# NOW -- `error:` is what a failed test says too (2026-09-04)

## A census that ranked the emptiest trees highest, and a harness that scored 8 kills as 0 (Refs #2994)

- `tri worktrees` (new) censuses the checkouts on this disk: **122**, free space down from 45 GiB to 29 in one session, after a fan-out already died of a full disk this week
- its first version counted `git status --porcelain` lines, and the two trees at the top were **all deletions**: `t27-om` 7639 of 7639 tracked files gone with 55 entries on disk, `t27w` 7414 gone. **The census ranked the emptiest checkouts as the most valuable ones**
- fixed with **no threshold**: deletions get their own field and are never summed into the work decision. A percentage tuned until two known trees land on the right side of it is a constant that decides the answer
- **then the mutation harness scored all eight kills as refusals.** It called a mutant "did not compile" by grepping `^error:` -- and `cargo test` prints `error: test failed, to rerun pass ...` when a **test** fails. All eight had been killed
- the tell was the count: **eight edits in five functions all failing to compile is not a plausible reading.** One re-run by hand settled it
- repair reads the channel that only exists after compilation: no `test result:` line = did not compile; `" 0 failed"` = survived; otherwise killed
- **a refusal bucket is the dangerous one to get wrong** -- a harness that cannot score is indistinguishable from a suite that cannot fail, and only one of those is a reason to stop
- the command deletes nothing and takes no flag that would: 96 of the 122 trees belong to one other session's scratchpad
