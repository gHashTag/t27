# NOW -- A reading stamp is a command, not a warning (2026-09-03)

## Every live count now hands over the line that repeats it (Refs #2994)

- section 461 made the flagless case print a sentence: *this reading is NOT anchored, pass --as-of to fix it*. A sentence is something to agree with. Both commands now print two lines, ABOVE the numbers rather than below:
- `read at 2026-09-03T18:27:56Z   NOT PINNED -- this count changes on every open and close` and `re-take:  tri issues numbers --as-of 2026-09-02 --limit 3039   (the most recent day that has ended)`
- the second line is the point. It names **the most recent day that has ended** -- the only date `--as-of` accepts, since today is refused for having a future end -- so the distance between *what I have* and *what you can check* is one paste
- **the line is a fixed point:** run what it suggests and the reading it produces carries the identical `re-take:` line. Checked by running it, not reasoned about
- `--as-of` now exists on `tri issues dated` too, over the same rule. Half a symmetry is worse than none: `dated` printed eight figures over the same live backlog with no way to pin any
- completeness is asked of the READ, before the as-of filter removes rows. Asking afterwards would see a short page and call a truncated read complete -- the guard reporting exactly backwards in the one case it exists for

## The guard from the last pass caught the suggestion from this one (Refs #2994)

- the first `re-take:` line suggested the limit the command had just used, and that was wrong in precisely the case the line exists for. Without `--as-of` the query is `--state open` and **489 rows fit under 500**; with it the query is `--state all` and there are **1486**
- so the suggestion read `--as-of 2026-09-02 --limit 500`, and running it printed `500 *** LOWER BOUND ***` and **360 open** where the true figure is **484**
- a helpful line, offered as the cure for unpinned numbers, handed the reader a wrong one -- and **section 462's truncation guard, written one pass earlier for an unrelated reason, is what said so.** Two rules from two passes; the second caught the first's mistake
- that is the argument for a guard that PRINTS rather than one that returns a bool. A predicate in an `if` protects its caller; a predicate that puts a line in the output protects everything downstream, including a suggestion written later by someone who forgot it existed
- the fix does not guess a bigger number: the suggested limit is **the largest issue number seen**, because GitHub numbers issues and pull requests from one sequence starting at 1, so the count can never exceed the largest number

## One test assertion was wrong, and the failure was mine (Refs #2994)

- `assert!(!s.contains("PINNED --"))` on the unpinned stamp failed on **correct** output: the string `NOT PINNED --` contains `PINNED --`
- replaced with the pinned form's own opening, `  as of `, which the unpinned form never prints. Four clauses mutated, four killed
