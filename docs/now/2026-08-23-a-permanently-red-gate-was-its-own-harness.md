# NOW — a permanently-red gate was its own harness (2026-08-23)

`emit-bitexact` has been red on every pull request that triggers it. It said six of eight code-generation targets "did not agree or did not run". The generated code was correct all along; the gate's own testbench wrapper was dropping the module's ports.

- **The failure.** `negate_tb.v:264: error: Unable to bind wire/reg/memory 'a0' in 'tb'`. The lifted module body contains `assign result = on_comb(a0, a1, b0, b1);` — and `a0`, `a1`, `b0`, `b1` are ports of the generated module that the wrapper never declared.

- **The generator is not at fault.** `t27c gen-verilog specs/ternary/ternary_ripple_adder.t27` emits a module header with all nine ports, and the `assign` is valid inside it. `tools/verify_exhaustive.py` lifts the body into a bare `module tb;` and declared exactly four hardcoded names: `clk`, `rst_n`, `en`, `ready`. Any spec whose module carries other ports produced an unbindable testbench.

- **The docstring said it was doing the right thing.** *"the ports declared as constants rather than instantiated"* — true of four names and of nothing else. A self-description that is measurably false today, which is question five of the gate audit, sitting in the file the audit had not reached.

- **Ports are read from the header now.** Inputs get a sized constant, outputs get a bare wire because the lifted body's own `assign` drives them. The three control ports keep the values the hardcoded version gave them — `rst_n` and `en` held, not released — so a syntax fix does not smuggle a semantic edit along with it.

- **Result: the gate passes.** `negate`, `xor2` and `sign0` now run their Verilog arm and agree with model, C and Rust **exhaustively**. What was reported as six disagreements was six things that never ran.

- **The verdict that hid it, fixed at the root.** "Did not agree **or did not run**" is one sentence for two facts. `check()` already had the distinction — `None` for could-not, `False` for disagreed — and the tally threw it away. My first attempt at this made it *worse*: I split the tally without noticing that `verilog_digest` returns `False` for build failures too, so a broken testbench would have been reported, more confidently, as a disagreement. The four could-not-run paths return an explicit `UNRUN` now. Proven both ways: breaking the wrapper prints `COULD NOT RUN`, corrupting a digest prints `DISAGREED`.

- **Why it stayed hidden for days.** It was the fourth permanently-red gate in a column with three others, and I merged four pull requests past it without opening it (#2474). A gate that reports "could not measure" in the same words as "measured and found wrong" gives its reader no reason to look, and a red row among red rows gives them no way to notice a new one.

Refs #2474
