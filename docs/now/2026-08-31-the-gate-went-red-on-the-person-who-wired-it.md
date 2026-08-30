# NOW -- The gate went red on the person who wired it (2026-08-31)

## The gate went red on the person who wired it (Closes #2972)

- Harness scratch directories failed on master at corpus_zig_bodies.rs -- three tests, a pid-only scratch key, and a remove_dir_all of it
- added by #2966, the same pass that wired the gate; the race cannot fire today because only one of the three tests uses the directory
- the gate refuses the SHAPE not the outcome, which is why it fired on the commit that introduced it rather than on a later innocent one
