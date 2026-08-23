# NOW — a control with resolution and no wiring (2026-08-23)

`verify_exhaustive.py` scored **0/2, 0/1, 0/9, 0/3** — a control that exists, passes, and kills nothing. Worse than no control: the gate read as covered.

- **Its control was good at the wrong thing.** It perturbs one input and proves the digest changes — the comparison has *resolution*. It never leaves the function. Every verdict lives in `main()`, which the control never ran.
- **Reaching main() found a product defect.** With no C compiler on PATH the gate raised `FileNotFoundError` — exit 1 and not one word of verdict. A traceback is not a verdict: CI sees red and the reader cannot tell "the tool is missing" from "the arithmetic is wrong".
- **Guarding the crash exposed a worse one.** Once the absence was caught, the gate announced `FAIL: 1 of 1 targets DISAGREED` — about arithmetic it never performed. `check()` returned `False` where it meant `UNRUN`.
- And main()'s own comment says *"check() already distinguishes them — None is 'could not run', False is 'ran and disagreed'"*. **True of the Verilog arm, false of the C/Rust arm, in the same function.** A third path returned `None`, in neither tally, so it exited 1 having printed nothing at all.

Three end-to-end cases now, one per reachable verdict: an empty selection fails, a missing compiler is UNRUN and never DISAGREED, and a clean target exits 0 *saying what it proved*. The last is the mirror the other two need — both demand exit 1, so a gate rewritten to fail unconditionally satisfied every case in the file, and `--loud` showed exactly that.

Result: **2/2, 1/1, 7/9, 0/3**. Named and left: the disagreement verdicts need a backend that genuinely differs.

**The rule:** a control that never leaves the function it tests measures the function. Ask of every control — *which process exit does this case observe?* If the answer is none, it is a property test wearing a gate's clothes.
