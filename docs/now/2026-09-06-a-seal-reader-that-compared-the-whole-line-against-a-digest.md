# NOW -- A seal reader that compared the whole line against a digest (2026-09-06)

## A seal reader that compared the whole line against a digest (Closes #3366)

- `tri reseal check` exited 1 on EVERY clean checkout: it compared `raw.trim()` -- the whole `<64-hex> <path>` line -- against a bare digest, so the mismatch branch was unconditional and it printed two identical hashes as a disagreement.
- Its stated consequence was false: `bootstrap/build.rs:246` takes `split_whitespace().next()`, so `cargo build` passes.
- Obeying it corrupted the seal with no error anywhere -- the write emitted the digest alone, deleting the path token that #3280 restored on 2026-09-05.
- Nothing caught it because every fixture in the module was a bare hash. The new fixture is the real two-token line, and a control asserts it still reproduces the defect.
