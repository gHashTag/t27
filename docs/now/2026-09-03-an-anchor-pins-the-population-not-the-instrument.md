# NOW -- An anchor pins the population, not the instrument (2026-09-03)

## `find` answers a different question than the rule asked (Refs #2994)

- `window_markers` asked `low.find("last ")` -- the FIRST occurrence and no other -- then whether a digit followed. Section 439 says *"reads the last COMMIT message"* on its line 18, where no digit follows, and *"Over the last 20 commit messages on master"* on line 27. The rule stopped at the first and returned nothing
- so **section 439 was absent from its own population**, and it is the section that produced the 4-against-33 row of section 457's own table. The one-variable probe is the whole proof: same tree, same command, `find` against `match_indices`, **19 against 20**
- `find` answers *"does the FIRST occurrence satisfy this"* and the question is *"does ANY"*. On one line they agree; on a page of prose they do not, and a page of prose is the only text this rule reads

## The same anchor, a different instrument, a different number (Refs #2994)

- section 457 published `12 of 420` anchored to `c039ebebe`, and that number reproduces exactly at that commit -- an audit re-took every anchored figure, **twelve of twelve, none failed**
- re-run at the same anchor with the FIXED tool it is **13 of 420**. The anchor was right and the instrument was not
- **a figure over a fixed population is re-takeable only by someone holding the same binary**, and nothing in "over the 20 commits ending at `<sha>`" says which binary. The data anchor and the tool version are two different anchors; section 457 named one
- this does not weaken anchoring, it completes it. An unanchored figure cannot be re-taken at all; an anchored one can be re-taken and **disagreed with**, which is what happened here. Section 457 corrected in place

## Two process notes (Refs #2994)

- **the probe was not mine.** A read-only fan-out instructed to attack the previous pass's own numbers wrote it, and its refuter then narrowed the charge correctly: section 457 never claimed 439 was among the twelve, so the membership complaint falls and the matcher defect stands
- the first mutant written to prove the fix **did not compile** (`break` outside a loop, left over from the loop it removed). It was reported as *never built* rather than scored as a kill -- section 455's fourth arm arriving in a new place
- `git stash` was used for the one-variable probe and **stashes are shared across worktrees**: the list held 19 other entries, one on branch `w790`. Only `stash@{0}`, verified as mine by its branch line, was dropped
