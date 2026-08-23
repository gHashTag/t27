# NOW -- The routing fix missed the fresh clone, which is the case that matters (2026-08-23)

## The routing fix missed the fresh clone, which is the case that matters (Closes #2559)

- Routing 17 subcommands to the Rust tri binary does nothing on a machine where target/ is empty: the gate's remediation still prints 'unrecognized subcommand now', unchanged.
- My first emulation of a fresh clone was wrong. I hid target/debug/tri and the command still worked, because target/release/tri answered. Hiding one of two binaries proves nothing.
- The front door now says the binary exists, is not built, and how to build it, then falls through so t27c's own error still prints. It does not claim which commands the unbuilt binary has, because nothing there can know.
