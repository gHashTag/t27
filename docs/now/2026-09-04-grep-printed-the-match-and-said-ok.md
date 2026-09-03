# NOW -- grep printed the match and said OK (2026-09-04)

## The Admitted gate kept one of grep's three answers

- `coq-kernel.yml:121` used `if grep -n 'Admitted' A B 2>/dev/null; then … fi;
  echo OK`. grep answers 0 matched, 1 no match, **2 cannot open**, and `if`
  merges the last two. With `Phi.v` absent and one `Admitted.` in `PhiFloat.v`
  the step **prints the match** and then prints `OK: no Admitted`, exit 0.
- One ordinary rename away: `grep -rn "Kernel/Phi.v"` finds three hits in the
  tree -- this line and two Coq `Require`s resolved by module name -- so moving
  the file and updating `_CoqProject` leaves everything green and this gate dead.
- Now three-way: 0 fails, 1 passes, anything else exits **2 = could not run**.
  `2>/dev/null` is gone so grep names the file it could not open.
- An `[ -f ]` existence loop was written first and **removed**: mutation showed
  it redundant -- deleting it left every test green, because the three-way case
  already turns grep's 2 into exit 2. Two rules with one testable consequence
  are one rule and a decoration.
- The shape's population was measured before anything was proposed:
  `if … 2>/dev/null … then` matches **4 lines in 49 workflow files** -- one
  defect, two milder instances in `fpga-build.yml`, one that is fine. Reported
  on the issue rather than turned into a detector on someone else's file.
- The test extracts the step body from the YAML and asserts the extraction
  matched. Three mutants, three kills, including one showing that dropping
  `2>/dev/null` is itself load-bearing.
