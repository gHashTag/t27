# NOW -- `grep` has three answers and `2>/dev/null || echo OK` keeps one (2026-09-03)

## `tri gates quiet` (Refs #2994)

- `grep` exits **0** for a match, **1** for no match, **2** for no such file. `2>/dev/null` deletes the message and an `||` arm deletes the exit code, and what survives is one bit: *clean*. A missing subject and a clean subject print the same characters
- the command walks the workflows and reports the shapes: **49 files read, 32 steps in a quiet shape** -- 16 `failure branch passes`, 9 `a count that reads zero`, 7 `gated on the file existing`
- the counter is the same defect in a different costume: `ADMISSIONS=$(grep -r "^Admitted" *.v 2>/dev/null | wc -l)` reads **0** from a directory with no proofs, and 0 is the number a clean tree prints
- `[ -f X ]` is reported apart because it is often exactly right. What separates it from the defect is whether the output NAMES what it read, and nothing here does
- **the reading that matters is not the count of shapes.** Of the 32, exactly **one** names a tracked path -- `phi-loop-ci.yml:30`, subject `ffi/src/`, on disk today. So no gate is currently guarding nothing, said plainly. **Twenty-two name no path at all**, and that is the harder finding: a step that does not say what it read cannot be checked by this tool, by a reader, or by the next person to rename something

## "Cannot check" is not "absent" (Refs #2994)

- the first version reported **25 of 32** subjects missing. Four different answers had been collapsed into one: **no path on the line** (22 -- the tool cannot say anything, and *cannot say* is not *is missing*); **the run builds it** (`build/fpga/synth/synth.log` is absent from a checkout because the workflow creates it later); **a variable in the path** (`specs/fpga/${m}.v`); and one the tool invented outright
- that last: `subject_of` took the first token carrying a `/`, so from an inline python one-liner it returned `json;print(len(json.load(open('/tmp/r.json'))['checks']` and reported **that** as a tracked path that is missing. Punctuation which cannot appear in a path now rules the token out
- with the four separated, the honest count of tracked paths missing today is **0**
- **a detector that cannot distinguish its own ignorance from a finding will always find something.** 25 of 32 is 78%, just under the 80% line, so the ratio alone would not have caught it. **What caught it was reading the list** -- the first row was python source
- section 464 for the third time: the list is the check. `--list` prints every step counted and `--excluded` every line refused, because a census that prints only its totals cannot be argued with, and this one was wrong in its totals while every total looked plausible
