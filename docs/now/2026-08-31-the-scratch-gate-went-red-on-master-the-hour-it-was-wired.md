# NOW -- The scratch gate went red on master the hour it was wired (2026-08-31)

## The scratch gate went red on master the hour it was wired (Closes #2968)

- tri harness scratch --gate has been red on master since 994f3c8a3: bootstrap/tests/corpus_zig_bodies.rs keys its scratch directory by std::process::id(), which is one value for the whole test binary.
- Measured, and stated as measured: only one of that file's three tests creates or deletes the directory today, so nothing collides on this tree. The gate is structural, and right to be -- the second test to use that key would collide, and would pass the first time it ran.
- Fixed as the gate's own message prescribes: an AtomicUsize counter, unique per call. Gate now prints none and exits 0; the three tests still pass.
- The gate found this within an hour of being wired, on a file written after it. That is the reading worth keeping: it went red on new code rather than on history.
