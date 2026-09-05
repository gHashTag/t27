# NOW -- The Admitted gate reads two of the nine files the build compiles (2026-09-05)

## The Admitted gate reads two of the nine files the build compiles (Refs #3237)

- coq-kernel.yml grepped Admitted in coq/Kernel/Phi.v and coq/Kernel/PhiFloat.v. coq/_CoqProject names nine files; seven compiled proof files including all three Theorems/ were outside the gate.
- Nothing else covers them: coqc compiles a file containing Admitted without complaint (it becomes an axiom), and coqchk is run for PhiFloat alone. The grep was the sole defence.
- The population now comes from coq/_CoqProject, the same file the build reads. An empty _CoqProject exits 2 rather than passing.
- Controls: planting Admitted in Kernel/Trit.v is caught by the new gate and PASSES the old one, which prints 'OK ... both files read'. Phi.v still caught. Empty and missing-file cases both exit 2.
