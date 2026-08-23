# NOW — a null result, and a selector that was right by accident (2026-08-23)

Two measurements, and the honest one is the boring one.

- **The assert blind spot costs nothing here.** `is_gate_by_property` recognises `return`, `sys.exit` and `raise SystemExit`; a tool failing only through an uncaught `AssertionError` is invisible to it. Measured across every CI-invoked tool: **zero** fail only that way. The hole is real and empty, and saying so is worth more than the fix I was about to write.
- **But the selector has a false positive, and it fired.** `gft_backprop_microcode.py` was classified as a gate on the strength of `return 0 if ys[0] >= ys[1] else 1` — a **class label from a classifier**, not a verdict. The file is a gate anyway (sixteen `assert`s), so the classification is right **by accident**, which is the least useful way to be right.
- **The last of the four verifiers has a control.** `verify_trainer_c.py` — the one whose `skip()` every other copied — is 1/1, 1/1, 3/3, with a planted divergence in its C arm and the clean direction asserted beside it. The plant scopes its edit to the text after `def run_c(`, because the same plant elsewhere hit `run_model` three functions earlier and a case passed on a divergence planted where it was never meant to be.

Gates with no control in any form: **1**, and it is the file the selector was right about by accident.

**The rule:** a predicate reused outside the question it was written for will be right often enough to look correct. `verdict_literals` answers *"is this return a verdict?"*; it does not answer *"can this program fail?"*, and the difference only shows up on a classifier that returns 1 for a class.
