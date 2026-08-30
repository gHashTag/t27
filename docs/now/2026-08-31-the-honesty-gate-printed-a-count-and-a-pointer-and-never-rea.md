# NOW -- The honesty gate printed a count and a pointer, and never read what it measured (2026-08-31)

## The honesty gate printed a count and a pointer, and never read what it measured (Closes #2981)

- rings-rust's summary job printed the crate count and a pointer to rings/COMPILE_STATUS.md, and never read the matrix result. Seven master runs from 2026-05-23 to 2026-08-20 had all 17 crate jobs failing to compile, and every one concluded success.
- COMPILE_STATUS.md, which the summary points at and which calls itself the honest living per-crate status, was last updated 2026-05-22 and said throughout that they compile. Two instruments, three months, both silent; the crates were repaired by 2026-08-28 with neither having said they were broken.
- continue-on-error is NOT the defect and is not touched: the workflow states why in its own header, and a gate that lands red on the default branch is one nobody can merge past. The defect is that the state was computed 17 times a run and discarded.
- Each matrix job now appends its own verdict line using steps.<id>.outcome -- the result taken BEFORE continue-on-error is applied. A list line, not a table row: matrix jobs finish in no fixed order and GitHub concatenates summaries in completion order, so a header would not stay above the rows. Exercised on all four outcome combinations.
- Found by a six-lens adversarial sweep, and the skeptic corrected it twice: continue-on-error has a written reason (so not a defect), and the compile breakage is already repaired (so the repair is preventive, not urgent). 14 of 14 agents returned, 0 errors -- checked before reading the result as a sweep.
