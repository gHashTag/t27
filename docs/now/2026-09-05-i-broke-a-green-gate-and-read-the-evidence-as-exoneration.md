# NOW -- I broke a green gate and then read the evidence as exoneration (2026-09-05)

The gate `Documented t27c subcommands exist` has been red on master since 02:21Z. I
put it there, and when I looked I concluded the opposite.

## What happened (Refs #3241)

- #3231 corrected a claim of mine, and its text explains that `t27c gen-zig` is not a subcommand -- with the name in backticks, as a document naming a command
- the gate reads backticked `t27c <sub>` mentions in live documents and fails on any that does not exist; it accepts a disclaimer, matching `not implemented|not built|does not exist|never existed|no such subcommand|...`
- my wording was "which is not a subcommand", which says the right thing and matches none of those patterns
- **#3231 merged at 02:21:25Z; the gate's first failure on master is at 02:21:28Z** -- the run its own merge triggered. Before it: **seven successes**. After it: five failures, all mine

## The misreading, which is the part worth keeping (Refs #3241)

- I saw three consecutive master failures and wrote "this is a pre-existing red gate, not something my PR caused"
- `tri pr ready` agreed in its own words: "also failing in 4 other place(s) -- pre-existing"
- both readings are correct and neither means what I took it to mean. **A defect merged to master fails on every subsequent pull request, which is exactly what "pre-existing" looks like to a tool that compares a branch against master**
- the check that separates them is one command: when did the gate last pass, and what landed between then and the first failure

## The repair (Refs #3241)

- one phrase: "which is not a subcommand" becomes "which does not exist as a subcommand"
- the checker then reports `ok: every t27c subcommand a live document names exists, and the 141 dead `tri` mentions are the recorded ceiling`
- the gate is well built: it names the file, the line, the phrase it wants, and where the disclaimer may sit -- on the line, in the paragraph, or in the heading above
