## a diagnosis ages, and two of my four did not survive the log

Four workflows were carried forward from one pass to the next as recorded FIXes,
each with a written cause. Re-reading the actual runs before editing anything,
**two of the four were wrong**:

| workflow | recorded | what the evidence said |
|---|---|---|
| `coq-proofs.yml` | OPAMROOT + `opam env` | correct |
| `brain-seal-refresh.yml` | drop the push step | correct |
| `lean-proofs.yml` | "two Lean targets, keep `--wfail`" | **wrong** |
| `release.yml` | never green, restrict the trigger | **wrong premise** |

`lean-proofs` is not misconfigured. 8571 of 8574 targets build, and the two that
fail carry comments in the source saying so:

```
-- LEFT FAILING, DELIBERATELY.                           H4Lagrangian.lean:73
-- LEFT FAILING, DELIBERATELY, AND THIS IS THE ONLY ONE. H4Lagrangian.lean:108
```

`release.yml` was on a *never-green* list while run `33180327861` had succeeded
on 2026-08-28, publishing `t27c 0.2.0`. And one of its two recent failures is a
`release` whose tag names no product — the product gate working as designed.

A diagnosis is a reading taken at a moment, and it decays two ways: the subject
changes, and the diagnosis was never right. Neither is visible from the note.
The cost of re-reading the log is one command per workflow; the cost of not
doing it is shipping an edit against a cause that is not there.

**Re-read the failure before applying a stored fix — including one you wrote.**
The list to be suspicious of is the one where every entry is marked FIX, because
that is the shape a list takes when it was written to be actioned rather than
measured.
