# NOW -- The pipeline rule is about the last command, and a verification is dated (2026-09-03)

## Two sections, both from walking into my own recorded traps (Refs #2988, #3005)

- ci-gates 434: section 428 named `$?`, and this time the trap fired through `&&` -- `git apply --check "$p" | head -3 && echo APPLIES` printed APPLIES for two patches that do not apply, because `head` succeeded; the real `error: bootstrap/stage0/FROZEN_HASH: patch does not apply` went past on the piped line
- `$?` was never the subject: `&&`, `||`, `if`, `while`, `until` and `set -e` all read the LAST command of a pipeline. The rule with no judgement in it is that a command whose exit code you care about does not go in a pipeline -- redirect, read `rc=$?` on its own line, then look at the text
- ci-gates 435: I published that two externally-authored patches apply cleanly to master and their `FROZEN_HASH` is valid, correctly hedged that any edit to `compiler.rs` would end that -- and then ended it myself four hours later by merging #3005, which rewrote the seal from `fd842146…` to `1b52250f…`
- a hedge moves the work to a reader who may never come; name the COMMIT rather than the branch, and when you merge something touching the same files, go back and say so, because you are the one person holding both facts
- the tell was a review agent reporting that the brief's premise -- "compiler.rs is byte-identical between X and master" -- had expired **during its own run**
