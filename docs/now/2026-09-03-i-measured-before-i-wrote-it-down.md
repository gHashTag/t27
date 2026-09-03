# NOW -- I measured before I wrote it down (2026-09-03)

## "19 against 20" is 19 against 21 at the commit that shipped it (Refs #2994)

- section 465 and the body of the PR carrying it both said *same tree, same command, `find` against `match_indices` -- **19 against 20***. At `d448b1864`, the commit that shipped them, it is **19 against 21**
- not the instrument and not an outside population: **section 465 itself carries the shape it documents.** It quotes section 439 -- *"reads the last COMMIT message"* before *"the last 20 commit messages on master"* -- so under the old `find` it would have been masked too, and it is the second masked section the fixed rule now sees
- the order was: take the reading, write the section, ship. The reading described the tree before the section existed and was published as a description of the tree that shipped
- **that is section 457 word for word** -- *the figure moved because writing it moved the population* -- unlearned one pass after being written, by the author, in the section that cites it

## Two things made it findable and only one was mine (Refs #2994)

- the number was re-taken by a read-only fan-out told to attack the previous pass's own figures
- and the anchor in the PR body was the words **"same tree"** rather than a sha. Had it named `cfa32871c` the pair would have been exactly right and merely stale, instead of wrong about the commit it shipped in. **"Same tree" does not name a tree**
- section 465 is corrected in place; the rule that follows is mechanical: a figure describing the state AFTER a change is taken as the LAST action before the commit, from the tree that is committed, and written with that commit's sha. Any earlier reading describes a different tree, however few minutes earlier
