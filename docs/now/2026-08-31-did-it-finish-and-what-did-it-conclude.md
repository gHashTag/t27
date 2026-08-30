# NOW -- "Did it finish" and "what did it conclude" are two fields (2026-08-31)

## Three status readers of mine were wrong in one session, all toward reassurance (Refs #2987)

- `grep -c '^ERROR'` over yosys output returned 0 for six files that all exit 1: yosys writes `<path>:<line>: ERROR: …`, so nothing starts with the anchor, and an empty count read as "no error"
- `cmd | head -12; echo rc=$?` printed rc=0 for a run that exits 1 -- `$?` is head's status, which is already section 245 of the skill, met again through a pipe I wrote myself
- a filter of `conclusion not in ('success','skipped',None)` printed `FAIL: FPGA E2E Build` for a run that was still going: an in-progress run has `conclusion: ''`, not null, and master was in fact clean
- the general rule: `status` says whether it finished and `conclusion` says what it decided; filter on `status == completed` FIRST, and print the empty value as its own count rather than reassigning it
- same shape as the `none == none` finding one layer up -- a sentinel meaning "no answer" compared as though it were an answer -- and in all three the wrong reading was the reassuring one
- ci-gates 428
