# NOW -- A zero-length array asked for a register of usize::MAX bits (2026-08-24)

## A zero-length array asked for a register of usize::MAX bits (Closes #2566)

- var xs : [0]T is a legal empty list. total_width = dims[0] * elem_w is then 0, and total_width - 1 on a usize is 18446744073709551615, emitted as reg [18446744073709551615:0] at exit 0. Clamped the way range_decl clamps; 650 specs regenerated, 1 file differs and it is the target.
- The third absurd width is not an underflow. Str is 4096*8+32 = 32800 bits and Map is 2*64*32800+32 = 4198432 — 513 KiB in one packed register, computed faithfully from a spec with no hardware form. Whether such a type should be lowerable is a design decision, filed with the same question about zero-width types.
- Searching for the underflow found 96 sites formatting a width inline with a bare minus one. That is a candidate list: exactly one produced a bad width over the corpus. Fixing 95 unverifiable sites in a compiler is a change nobody can defend later.
- The arithmetic is now written into the ledger beside the remaining entry, so the next reader does not look for a subtraction that is not there.
