# NOW -- The front door routes without the compiler (2026-09-04)

## The front door routes without the compiler (Closes #3090)

- 37 of the Rust tri binary's 47 subcommands are not t27c subcommands and compile nothing; scripts/tri checked for t27c before routing, so all 37 were refused when t27c was absent
- Four arms, one variable each: with t27c hidden and target/release/tri present, 'tri now --help' answered 'cannot run now -- t27c is not built' and named a 2m20s build of the wrong binary
- Routing now happens before the compiler guard when t27c is not there at all; a name the tri binary does not list still exits 2, and with neither binary built the refusal names both
- The comment claiming the fresh-clone case was verified by moving BOTH binaries aside was wrong: with both aside the guard answers first and that block never runs
