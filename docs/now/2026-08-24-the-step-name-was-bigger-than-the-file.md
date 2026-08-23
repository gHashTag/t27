# NOW -- The step name was bigger than the file (2026-08-24)

## The step name was bigger than the file (Closes #2161)

- tri gate-sweep selected files by name (check_ or gate), so it measured the naming convention: thirteen checkers including every verify_* had never been swept. It now takes every non-private .py in tools/ — thirty files, CRASH 0.
- Among the thirteen: five correct skips that are fatal under --require, one generator raising a traceback on the catalog it reads (now a verdict naming the file), and one claim.
- A CI step called 'Prove the trainer LEARNS' runs a pure-Python tool: no t27c, no spec, no iverilog. It generates microcode, runs it on a GF-T interpreter, and asserts on emitted Verilog text. The file's docstring was honest; the overclaim lived entirely in the step name, which is the claim most people read.
- Renamed to what it proves, with a 'what this does not establish' block in the tool naming the three claims it is not. When writing a step name, write what would still be true if the environment were bare.
