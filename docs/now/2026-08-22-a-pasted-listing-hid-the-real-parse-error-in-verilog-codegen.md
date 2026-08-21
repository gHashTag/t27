# NOW -- A pasted listing hid the real parse error in verilog/codegen.t27 for four months (2026-08-22)

## A pasted listing hid the real parse error in verilog/codegen.t27 for four months (Closes #2372)

- Seven lines of compiler/codegen/verilog/codegen.t27 begin with two concatenated line-number columns before their code (632-635 and 39650-39653 and 4015-4017 are three independent consecutive counters). The same paste left runs of consecutive integers inside a comment (9,340 chars) and a string literal (128,657 chars), which is why a 1,067-line file is 228 KB. Present since 2026-04-05.
- The cost was the diagnosis, not the failure: the compiler read 63339651 as a number and reported 'unexpected token after expression statement: Ident', which reads as a grammar bug. Stripping the prefixes moves the error three lines on to the true cause, Zig's payload capture 'if (x) |y|'. One defect stood in front of the other. A sweep of every tracked .t27 finds the pattern nowhere else.
