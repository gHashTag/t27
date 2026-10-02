# NOW -- The Zig target of verify_multitarget runs (2026-10-02)

## Make the fourth target real (Refs #5497)

- The Zig branch added by the salvage commit #4736 never ran: it called `t27c gen-` (there is no such subcommand; the Zig generator is `t27c gen`), required a `pub fn` the generator never writes, and appended Zig that does not parse (`[_:u32;N]`, `std.io.getStdOut()`). The emit-bitexact job does not install zig, so under `--require` the step would also have failed as "zig not on PATH".
- `_gen` maps an empty mode to `t27c gen`, the harness looks for `fn <name>(`, prints with `std.debug.print` (the same API in every Zig release, unlike stdout) and `_run_bin` can read stderr. The job installs Zig 0.16.0 the way queen-doctor.yml and oracle-nightly.yml do.
- Measured on master 5c5c2aff with Zig 0.16.0: `smul` agrees in C, Rust and Zig over 1744 pairs; `sadd` agrees in C and Rust, and Zig traps (integer does not fit) because `sadd` shifts by a negative count before it orders its operands and because `as u32` wraps in C and Rust but is a checked `@intCast` in Zig. The step now fails for that reason instead of a missing subcommand.
