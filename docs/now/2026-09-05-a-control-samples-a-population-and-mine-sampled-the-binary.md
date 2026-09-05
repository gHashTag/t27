# NOW -- A control samples a population, and mine sampled the binary (2026-09-05)

Skill section for the sharpest failure of the pass, and the audit that closed it.

## The audit, which is the part that matters now (Refs #3263)

- after finding that one pull request had merged its note without its fix, the question is whether it was the only one
- grepped master's `bootstrap/src/compiler.rs` for a distinguishing symbol from **every** fix shipped this pass: serde gating, the `string` alias, dotted foreign types, module type parameters, borrowed `str`, keyword field names, array length, module-level bools, `[]const u8` in a const, and `null`/`.?`
- **all ten present.** The eleventh, the one that had shipped empty, is present again after its restoration merged
- master measured on the rebuilt binary with a control on the generated output: **333 accepted**, and the control prints `if u {` rather than `if (u) != 0`

## Why the guard I already had would not have caught it (Refs #3263)

- earlier the same day I built a helper that refuses to push unless the change is verified present in the rebuilt binary
- the binary was correct at every moment anyone asked it. The edit was destroyed **after** the measurement and **before** the commit, by the `git reset --hard` that an honest baseline measurement requires
- so the guard sampled the binary while the claim was about the repository -- the same shape this skill names in a dozen other sections

## The two guards, and the ordering rule that removes the hazard (Refs #3263)

- `git diff --cached --stat` must list the source file, before the commit
- `git show --stat HEAD` must list it, before the push
- and better than either: **commit before measuring the baseline**, because an honest baseline needs a `checkout` or a `reset` and both destroy a working tree. Edit → commit → measure → re-seal → push
- the helper now refuses to push when `git diff --stat origin/master...HEAD -- bootstrap/src cli/tri/src` is empty

## And a collision of my own making, fixed in the same commit (Refs #3263)

- the three sections I filed an hour ago as §573-575 collided with a neighbour's §573-575, which landed **nine minutes earlier** -- so the duplicates were mine and mine moved, to 579/580/581
- I first tried to compute the free numbers by hand and my counter said the maximum was 548, because its fence tracking miscounts and headings quoted inside code blocks are not sections. It renumbered onto numbers already taken and the duplicate count did not move
- `tri skill renumber` could not help either, and says so honestly: it moves sections the BRANCH appended, and mine were already in the base
- the working answer was to anchor on the exact heading text, which is unique, and take the free range from the tool's own printed tail. This is the third time this pass a hand-rolled parser has been wrong where the repository's own reader was right
