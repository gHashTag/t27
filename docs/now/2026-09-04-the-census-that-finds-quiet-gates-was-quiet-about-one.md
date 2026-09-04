# NOW -- The census that finds quiet gates was quiet about one (2026-09-04)

## A false clear, found by probing rather than reading (Refs #2994)

- `tri gates quiet` cleared `coq-kernel.yml:121` -- it printed the line under *NAMED A PATH AND WAS NOT QUIET*. A read-only fan-out probed it instead of reading it, and the clear was wrong
- re-probed here with a positive control: with both files present and with both deleted, stdout is **byte-identical**, stderr is **empty**, both exit **0** -- while a real `Admitted.` still exits 1, so the gate works exactly when its subject is there
- **the shape is multi-line and the rule was line-scoped.** `grep` exits 2, the `if` merges that with "no match", and the `echo` after `fi` is unconditional. Nothing on the `if` line says so; the evidence is three lines below it
- the new clause takes the following lines: the condition silences stderr, the THEN branch exits non-zero, and the block ends without an `else`. An `else` takes it out of scope -- then the missing-file path has its own branch, and whether THAT passes is a different question
- **it fires zero times today, and the reason is the point.** Three lines of that syntactic shape remain and all three have an `else`. The one it was written for was repaired on master while this was being built, by another session quoting this file's own rule, with a comment opening *"GREP HAS THREE ANSWERS AND THIS USED TO KEEP ONE"* -- and recording that an `[ -f ]` loop was written first and **removed** because mutation showed it redundant

## A census with a third, invisible bucket (Refs #2994)

- the same command counted 32 and refused 18, and its refusal rule was "names a path AND (silences stderr OR has an `||`)". A line like `[ ! -f build/x.json ] && echo skip` names a path and does neither, so it appeared in **neither list**
- measured: `grep -nE '\[ *! *-[fd] ' .github/workflows/*.yml` returns **11** lines and exactly **one** was anywhere in `--list --excluded`. Ten were invisible
- **an omission a reader cannot see is an omission a reader cannot argue with**, and section 464's rule -- print the list, the list is the check -- does not hold if the list is drawn from a narrower population than the subject
- the population is now named once, in one function, and both lists draw from it. Totals moved from `32 + 18` to **`32 + 122 = 154`**, and the 90 lines that appeared are not new defects: they are what the first version was silently declining to mention
