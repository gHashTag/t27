# NOW — the gate that exited 0 when the compiler refused to emit (2026-08-23)

Counting the copies of `skip()` corrected yesterday's entry (**four**, not three) and found the one that mattered.

- `verify_emit_bitexact.py` — the gate whose entire job is *"prove the generated RTL equals the model bit-exactly"* — called `skip()` when `t27c gen-verilog` returned **non-zero**. A code-generation failure in the exact thing being verified made this check **exit 0**, in CI and locally, with or without any flag.
- That is not a missing prerequisite. iverilog absent is an incomplete environment; the compiler refusing to emit is the product being broken, and it is the loudest thing this gate could find. It was the quietest.
- **Two words for two states now.** `skip()` keeps its meaning — environment incomplete, tolerated locally, fatal under `--require`. A new `broken()` says the product failed and is fatal always, naming which of the two happened so a reader never guesses.
- **The fourth copy also lacked `--require`**, so it was the last of the four that could silently pass on a missing simulator. All four now agree.

The control's third case is the one that would have caught it: a planted `t27c` that refuses to emit, asserting exit 1 and naming `SKIP` as forbidden — because reporting a broken compiler as a missing tool is exactly the defect.

**The rule:** every `skip` is a claim that the thing missing is *not the subject of the test*. Read them as that sentence and the wrong ones become obvious — "we skipped because the compiler under test would not compile" does not survive being said out loud.
