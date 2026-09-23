# NOW -- the board had one dispatchable issue, and now it has forty-four (2026-09-23)

## Where "never write a boundary" has a floor (Closes #4640)

- The instruments said capacity 20, active 1, `refusal="nothing to choose"`. Of **648 open issues only 82 carried a boundary at all** -- and of those 40 were accepted-and-never-closed, 19 escalated, 19 sent back or empty. **One was free.** The 167 drafts posted as comments could not help: the Queen reads the BODY.
- `propose_boundary.py` refuses to edit a body, and that rule is right -- a wrong boundary reserves files the work does not own and blocks whatever really owns them until the claim expires. But the argument has a floor. **An issue naming exactly one path in the whole of its title and body, where that path is a `.t27`, has no second candidate to be wrong about.** #2835 "`specs/file/operations.t27` declares fn delete twice with different arity" cannot be about a file it never mentions.
- Two paths is already a judgement about which one the work owns, and that judgement stays a person's. So `--write-body` refuses two paths, refuses a single non-spec path, refuses a `docs/` citation, and leaves alone any issue that already has the section. Each refusal is a case in `--self-test`.
- **44 issues met the bar and were written.** Measured after: `missingBoundary` 565 -> 521, `refusal` -> `None`, **active 1 -> 18 of 20**. The rest of the 553 still wait for a person, which is the point.
