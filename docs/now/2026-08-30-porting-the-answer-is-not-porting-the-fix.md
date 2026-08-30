# NOW -- Porting the answer is not porting the fix (2026-08-30)

## Porting the answer is not porting the fix (Refs #2931)

- Verilog's bare `0` for the scaffold call is correct THERE because its bindings are already declared `reg` of the right width; C has no such declaration
- copying it gave: undeclared-call 86 -> 13, int-to-pointer 0 -> 68, `cc accepts` 174 -> 174 -- one family traded for another
- the right sibling was Zig, which recovers the type from the consumer's declared parameter; only that moved the number
- one construct needed THREE separate answers, and fixing any two left the accept count exactly where it was -- a partial fix on a multi-defect construct measures as no fix
- `-> void` is written two ways: no `fn_return_types` entry, or an entry whose value is `"void"`; a presence check saw only the first and 80 specs write the second
- the reseal belongs in the SAME commit as the emitter change; two consecutive passes left master red by deferring it, both mine
