# NOW -- A note shipped without its fix, and every control I ran asked the wrong question (2026-09-05)

#3264 merged as `49e5fff28`: **one file, 24 insertions, the docs/now note alone**. The
compiler change it describes is in no commit anywhere in the repository.

## The evidence, which is unambiguous (Refs #3263)

- `git log --all --oneline -S expr_is_bool_syntactically -- bootstrap/src/compiler.rs` returns **nothing**
- `git show --stat 49e5fff28` is one markdown file
- master therefore carried a note claiming a **+3** that did not exist -- a claim without its subject, which is this repository's central defect, committed by me

## What happened, step by step (Refs #3263)

1. edited `collect_bool_locals`, built, and the control printed `if u {` -- correct
2. measured **333** against a stored baseline. Real measurement, real number, change present only in the WORKING TREE
3. to re-measure the master baseline honestly, ran `git checkout master && git reset --hard origin/master` -- **which discarded the edit**
4. returned to the branch, wrote the note, `git add -A && git commit`, and shipped what was there: the note
5. every step passed. Every control I had asked about the **binary**; not one asked about the **commit**

## The guard that was missing (Refs #3263)

- I built a helper earlier today that refuses to push unless the change is verified present in the rebuilt binary. It would not have caught this either -- the binary was correct at every moment I checked it
- the missing control is one line, before the push: `git diff --cached --stat` must list the source file, or `git show --stat HEAD` must
- restored here with **both** controls run: the rebuilt binary emits `if u {`, and the index carries `bootstrap/src/compiler.rs` with 37 changed lines
- re-measured on top of master: **330 -> 333, +3, 0 regressions** -- the original number was right, only its delivery was missing

## Why this one stings (Refs #3263)

- the pass has been about measuring the subject rather than the instrument, and about gates that pass while their subject is gone
- I then shipped a note whose subject was gone, and the reason is exactly the one the skill names: a control tells you about the population it samples, and mine sampled the binary while the claim was about the repository
