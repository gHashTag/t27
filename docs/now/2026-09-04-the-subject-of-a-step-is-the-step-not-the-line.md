# NOW -- The subject of a step is the step, not the line (2026-09-04)

## Thirteen of twenty-two gain a subject at step scope (Refs #2994)

- `tri gates quiet` reported **22 of 32** quiet steps as naming no path. That number was about the LINE, and the thing it describes is a STEP
- a GitHub step is a `run:` block, and the path a gate reads is often on a different line of it -- a `cd`, a `for f in ...` header, a variable holding the path. Searching the block moves the figures: present **1 -> 11**, builds-it **5 -> 7**, variable **4 -> 5**, no path anywhere **22 -> 9**
- **nine still name none anywhere**, and those are the ones no reader and no probe can check: a step that never says what it reads cannot be told from a step that reads nothing
- **step scope is weaker evidence and is labelled.** A path on the line is what the gate demonstrably reads; a path elsewhere in the block is what it plausibly reads. Every row prints which
- and "the first path in the block" is a choice with a known cost: a block saying `cd ffi/src` and then grepping `tools/lint.rs` names two, and the subject is really the first joined with the second. First is reported because the last would hide the `cd` -- neither is right, and the test says so rather than asserting the convenient one
- the block ends where the next key at the same indent begins, so a neighbouring step is never swallowed; a blank line inside a block does not end it

## A redirection is not a path, and this is the second time (Refs #2994)

- `>/dev/null` carries a `/`, so `subject_of` returned it as the path a gate reads -- and the command reported it as **a tracked path that is missing**, under *GUARDING NOTHING RIGHT NOW*
- same defect as the inline python one-liner two passes ago, in **the same function**, found the same way: not by the count, which was a plausible `1`, but by reading the row
- **the cure is the shape of the rule, not the size of the list.** Both fixes rule out what cannot be a path rather than trying to recognise what can, because the second is open-ended and the first is not
- with it, the honest count of tracked subjects missing today is **0** -- the same answer the previous pass gave for a different reason, and this time the reason is measured rather than lucky
