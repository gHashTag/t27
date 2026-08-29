# NOW -- The set was seeded empty (2026-08-29)

## One spurious declaration per variable, and the emitter contradicting itself (Refs #2834)

- the C test-block emitter declares the FIRST assignment to a plain identifier, tracking what it has bound in a HashSet -- seeded EMPTY
- so a variable the block already declared as a local was read as a fresh binding: `PackedTrit packed = 0;` then `uint64_t packed = ...;` then plain assignments
- exactly one spurious declaration per variable, and `cc` calls it "redefinition of 'packed' with a different type"
- the Verilog path has seeded that set from the block's locals since #1894 and says so in a comment; only the C path was missing it
- measured: redefinition 45 specs / 375 errors -> 33 / 318, and cc accepts 163 -> 166 across the whole corpus
- the first measurement said 0 errors because the BUILD had failed on the M5 hash and the binary did not exist -- an empty output has no errors in it
- what remains is a different defect: 262 of the 318 are duplicate `test_*` function names, 7 are the discard placeholder `_` emitted as a real name
