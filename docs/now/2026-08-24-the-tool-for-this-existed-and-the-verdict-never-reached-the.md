# NOW -- The tool for this existed, and the verdict never reached the exit code (2026-08-24)

## The tool for this existed, and the verdict never reached the exit code (Closes #2161)

- I merged with checks running and called it discipline. tri pr ready already prints VERDICT: WAIT and counts incomplete checks, and its --merge flag's help documents this exact failure — 'it prints WAIT, the merge runs anyway, and nobody reads the line. That happened four times in one session.' An earlier me built the defence; I typed past it.
- Reading it to use it found a real defect: ready ends in Ok(()), so WAIT, CANNOT TELL, DO NOT MERGE and safe all exited 0. tri pr ready N && gh pr merge N merges on WAIT.
- Now 0 safe, 1 do-not-merge, 2 wait, 3 cannot-tell, with a test that the four are distinct and that pending outranks every verdict computed from an incomplete list.
