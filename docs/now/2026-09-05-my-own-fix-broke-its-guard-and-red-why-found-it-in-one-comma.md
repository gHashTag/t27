# NOW -- My own fix broke its guard, and red why found it in one command (2026-09-05)

## My own fix broke its guard, and red why found it in one command (Refs #3270)

- tri red why over the four active red workflows: scorecard is 8/8 upstream (pulling gcr.io/openssf/scorecard-action), coq-kernel is intermittent rather than a streak, and untrusted-input-gate is 8/8 at one step - 'The Admitted gate reads both files, or says it could not'.
- That step is a guard on MY change. #3238 widened the Admitted gate from two literal operands to the nine files _CoqProject names; the guard's scratch fixture builds a two-file tree with no _CoqProject, so every case exited 2 could-not-run. Red for 22 runs, unnoticed because the workflow is not required.
- The guard was right and its fixture had stopped reproducing the structure. arm() now writes a _CoqProject, and names is kept separate from files so arming ABSENCE still NAMES the missing file - the shape a deleted source produces.
- Added the assertions the widened contract deserves: an Admitted in a THIRD file is caught, and an empty _CoqProject is could-not-run. Control: restoring the old two-operand gate fails the third-file assertion with rc=0 and 'OK: no Admitted in Phi.v' - the exact defect #3238 fixed.
- Renamed the test and the step, which both still claimed 'both files'.
