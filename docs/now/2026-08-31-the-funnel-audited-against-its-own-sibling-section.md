# NOW -- The funnel, audited against its own sibling section (2026-08-31)

## An unplaceable failure was being filed as a generation refusal (Refs #2987)

- `tri vsim funnel` landed in #2993 with two arms that returned `Stage::Gen`: a process that could not be SPAWNED, and a failure whose message matched none of the four patterns -- both an unmeasured state wearing a verdict about the spec, which is what ci-gates 428 says not to do, shipped in the same session
- new `Unattributed` stage with its own printed row; the attribution moved into `attribute_failure(&log)` so it can be tested without spawning anything
- two new unit tests: a message matching no arm is `Unattributed` and NOT `Gen`, and each of the four arms is reachable by the compiler's own words
- measured over 650 specs: the new row reads **0** and every other row is identical -- 69 / 410 / 0 / 9 / 107 / 55 / 0. **Nothing was being misfiled today**; the change is preventive and buys the reader the ability to tell, which is why the zero is printed beside the hits rather than omitted
