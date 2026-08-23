# NOW -- tri vectors: the executed-vector registry as a command (2026-08-22)

## run and debt, with the three-verdict doctrine in the tool itself (Refs #2241)

- `tri vectors run <module>` generates the module's Verilog and executes its
  registered vectors in one step -- the hand-typed gen-verilog-then-python
  pair that ran a dozen times today.
- `tri vectors debt` inventories all 34 vector files against the registry:
  1 executed, 33 not. The output states in-band that not-executed splits into
  DEBT (numbered: #2410, #2413) and ASPIRATIONAL (no interface exposes the
  behaviour), and that neither counts as coverage -- the confusion this
  command exists to end is "a file exists, therefore something checks it".
- Negative controls, all measured without a pipe swallowing the exit code:
  missing spec -> exit 1 with the path; module outside the registry -> exit 1
  naming what v1 covers; a planted codegen fault -> 7 FAILs and exit 1.
  173 tri tests pass.
