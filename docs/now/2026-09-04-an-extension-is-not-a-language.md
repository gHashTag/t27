# NOW -- An extension is not a language (2026-09-04)

## Four corrections, three of them about the population

- `git ls-files '*.v' | wc -l` returns **225** and **166 are Verilog**. `.v` is Coq *and* Verilog.
  A first pass was about to report "184 uncompiled Coq files"; content classification gives
  **58 Coq, 166 Verilog, 1 ambiguous**.
- **`Admitted` and `admit` are one event, not two.** `admit` is a tactic inside a proof;
  `Admitted` is the command that closes it, so they pair. Matching both gave 64 where the answer
  is **32** -- exactly double.
- **`while read` drops the last entry** of a file with no trailing newline: bash counted 17 where
  python counted 18 over the same list. Two counts of one list differing by exactly one: suspect
  the terminator before the logic.
- **`-R .` maps a logical path, it does not add files.** `coq_makefile` builds the listed files, so
  the file list *is* the population.

## What survived

| | files | `Qed` | `Admitted` |
|---|---:|---:|---:|
| named in a `_CoqProject` | 41 | 234 | **0** |
| named in none | 18 | 21 | **32** |

`proofs/trinity/Bounds_LeptonMasses.v`: 8 theorems, 8 `Admitted`, **0 `Qed`**. The `Admitted` gate
reads `coq/Kernel/Phi.v` and `coq/Kernel/PhiFloat.v` -- two of 58, both clean.

Third instance of one shape in two passes, with #3142 and the three Coq proofs a merge took in
#3150. **Work that nothing compiles cannot report its own state.**

Refs #3153
