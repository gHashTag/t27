# NOW -- The Admitted control had no _CoqProject to read (2026-09-05)

## The Admitted control had no _CoqProject to read (Refs #3286)

- `Untrusted Input Gate` has been red on master since about 06:00Z; the last green run
  was 05:53Z.
- `f46050296` widened the Admitted gate from two hardcoded paths to the files
  `coq/_CoqProject` names. Its control extracts that step body and runs it in a temp tree
  it builds itself, and that tree carried `coq/Kernel/*.v` and no `coq/_CoqProject`.
- So the body exited 2 at its first command, before reaching its subject, in all five cases.
- The two cases asserting `rc == 2` were passing for the wrong reason: their 2 came from
  the unreadable operand LIST, not from the unreadable operand. Only the three that assert
  something else went red, which is why the failure showed as 2 of 5 rather than 5 of 5.
- The fixture now writes `coq/_CoqProject` naming both operands always, while still
  writing only the files each case asks for. That is what preserves the defect the test
  exists for: a gate that names a file it cannot read must say so, not say OK.
- Mutations, both killed: a gate body reading only `${VFILES[0]}` gives 2 failures, and
  removing `_CoqProject` from the fixture gives 5, reproducing master exactly.
- `coq/_CoqProject` is present on master. Only the fixture lacked it.
