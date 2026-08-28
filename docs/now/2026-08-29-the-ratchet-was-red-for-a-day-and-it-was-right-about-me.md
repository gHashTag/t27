# NOW -- The ratchet was red for a day and it was right about me (2026-08-29)

## The ratchet was red for a day and it was right about me (Refs #2754)

- my own commit fixed asp_solver parse and left its ledger entry; the ratchet reported UNEXPECTED PASSES 1 and failed twelve master runs
- I saw ACCEPTABLE no that day and read the failure list instead of the one-line verdict
- removed, and max_entries brought from 219 to 178 -- forty slots of slack had made the cap arm inert since 2026-08-23
- Vec<const u8> is not Rust: the Zig slice spelling carried its const through, in 84 of 559 files; rustc accepts 173 to 214 with Zig and cc unmoved
