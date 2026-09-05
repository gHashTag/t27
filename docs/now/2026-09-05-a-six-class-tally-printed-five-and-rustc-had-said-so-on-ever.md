# NOW -- A six-class tally printed five, and rustc had said so on every build (2026-09-05)

## A six-class tally printed five, and rustc had said so on every build (Refs #3229)

- tri types redef closed with a sentence reading as a partition: it summed to 345 for 346 rows. The dropped row was the only SIGNATURE one -- two delete definitions in specs/file/operations.t27 whose parameter lists disagree.
- signature was incremented in the match and left out of the println argument list. rustc printed 'value assigned to signature is never read' on every build for as long as the line existed.
- The detector written this pass could not have found it: its shape required the variable be READ by a control-flow test, and the defect is the absence of any read.
- Adds tri gates warnings, classifying the crate's own warnings into DISCARDED/dead/cosmetic/other; --gate holds the discarded class at zero. It touches the crate root first, because a cached unit emits no warnings and would read clean regardless of the code.
