# NOW — fifty boundary survivors, and a PR that got 7 of 35 checks (2026-08-23)

The boundary column carried **50 survivors** — the largest and last unexamined number in the table. Read rather than counted, they sort into four kinds, and only one is a verdict:

- **proven equivalence** (6) — `sig = … if r.returncode < 0 else ""`, reached only after a returncode check
- **fixture generation** (8) — `cls = int((xs[0] > 0) != …)`, `if p < 0.15:`, `while len(v) < N:`
- **display truncation** (4) — `if len(out) > 6:` guarding a "… N more" line
- **possibly real thresholds** (3) — `if total < 200:`, `space > budget`, `if b < 0:`

Plus 26 in one file's arithmetic internals, a separate surface.

**The operator has no scope discipline, and this is where that shows.** `invert` restricts itself to conditions whose body carries a verdict; `boundary` takes every comparison. In *verifier*-style gates most comparisons are in test-data generation and reporting. On checker-style gates the same operator found six real thresholds. **The same operator is sharp on one shape of gate and noisy on another.**

**Five of the six equivalences are one line, copied five times** — the `signal` message, each reached only after a returncode check, so the value cannot be zero and `< 0` ≡ `<= 0`. Fifth duplication family this campaign has found. All five marked; rows still read SURVIVED and still count.

## A pull request that silently received 7 checks of 35

PR #2541 changed a file explicitly listed in a gate's `pull_request` paths, and **that gate did not run.** Measured rather than assumed: an empty commit did not re-trigger it; a second branch touching the **same files** got a normal check list; the repository had no queue backlog.

**Not diagnosed.** Re-opened as a fresh pull request rather than merged without checks — the new one has 25 checks and the gate running.

**The rule:** a green check list is evidence; a *short* check list is a finding. Count the checks against a sibling before reading the colours — a gate that never ran is invisible in exactly the way a gate that passed is.
