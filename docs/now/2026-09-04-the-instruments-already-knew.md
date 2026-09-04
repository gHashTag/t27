# NOW -- The instruments already knew (2026-09-04)

## Two empty hunts in a row is a fact about method

- Pass 64 looked for gates reading a slice of their subject: **none of 20**. Pass 65 looked for
  gates with no master baseline and found the question asked, tooled and **withdrawn** five days
  earlier. Both times the thing that stopped a wrong publication was reading what this repository
  had already written down.
- So this pass ran the ten `tri gates` commands and the loop helpers instead of choosing a
  twelfth class by intuition. The consolidated reading is #3157.
- **Largest open item, unchanged: 19 MERGE_CRITICAL claims, 15 hollow.** A comment says a gate
  blocks a merge; the ruleset requires four contexts and none of the fifteen is among them.
  `tri gates required` names why it persists: *"A required check is named in repository SETTINGS.
  No file in the tree can read it, so a comment claiming a gate blocks cannot go stale against
  anything -- this is the only drift here with no detector."*

## What shipped, from that list rather than from a hunch

`tri gates fetches` flags **5 sites that print what they got** as though it were a total, and two
are in `prcheck.rs::ready` -- the function every merge in this loop goes through.

The fetch asks for `baseline * 3` closed pull requests and keeps up to `baseline` merged ones.
Nothing guarantees the page holds that many. The sentence said *"the last {baseline} merged PRs"*
regardless, so a comparison against two could be reported as a comparison against five.

`baseline_phrase(compared, asked)` now says what was read: `the 2 merged PRs found (fewer than the
5 asked for)`, and for zero, `no merged PR (none were found to compare against)` -- which is the
case the surrounding comment in that file was written about. Five tests, three mutants, three dead.
606 crate tests pass.

Refs #3157
