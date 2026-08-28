# NOW -- The Rust column measured the ruler: 0 of 559 was never true (2026-08-29)

## The Rust column measured the ruler: 0 of 559 was never true (Refs #2754)

- corpus compiled with rustc -o /dev/null, and rustc writes metadata through a temp file next to the output -- so it failed on every input, valid or not
- reproduced on a 23-character valid file: fails with /dev/null, exits 0 with a real path; rustc 1.98.0 was on PATH the whole time
- corrected: rustc accepts 0 -> 144, ALL FOUR accept 0 -> 60, with Zig 217 and cc 157 unmoved as the control
- then serde behind a cfg_attr, the largest single real cause: rustc 144 -> 173, all four 60 -> 63, and 668 seals caught by the gen-drift category added this morning
