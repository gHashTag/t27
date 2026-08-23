# NOW — why that control killed nothing (2026-08-23)

`verify_emit_bitexact.py` scored **0 killed of 17**. Reading the survivor lines rather than counting them:

- `sys.exit(0 if ok else 1)` survived **both** return operators
- every FAIL branch of the comparison — timeout, step count, mismatch, resource count, synth error — survived inversion

**All three of my cases leave through `skip()` or `broken()`.** The gate's own verdict was never observed at all. A control that covers only preconditions is a control *for* preconditions, and I wrote those three knowing that class.

**The plant that reaches the verdict moves one arm only.** The Python side comes from the interpreter `g.run()`; the Verilog is emitted from the microcode `steps`, not from `run()`. Perturbing the interpreter makes the model disagree with an RTL that is unchanged — a real bit-exactness failure. Perturbing shared arithmetic would move both arms together and plant nothing.

Two cases now: a clean tree exits 0 saying `RTL == model BIT-EXACT`, and a perturbed model exits 1 naming the disagreeing step.

## The cost is real

The mutation loop runs a gate's **whole control per mutant**. This control now spawns two ~45-second whole-program runs, so seventeen sites cost roughly half an hour — the measurement timed out at ten minutes.

**That is not a reason to make the control cheaper.** A control that exercises only what is fast to exercise is how this gate got to 0/17. The tension is inherent: a control worth having is expensive, and the loop pays that cost once per mutant. The cache is the answer, and `[cached]` is what keeps a reused row honest.

**The guard chain worked end to end for the first time:** timeout → marker names the gate → `git checkout -- tools/` → tree clean → committed diff contains only the intended change. Three iterations ago the same sequence leaked two mutants into a branch.
