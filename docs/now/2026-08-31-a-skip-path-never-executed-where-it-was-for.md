# NOW -- A skip path never executed where it was for (2026-08-31)

## The guard matched a message where it should have asked the OS (Refs #2990)

- the runtime leg of `verilog_real_arithmetic.rs` skips when `iverilog` is absent; the guard tested `log.contains("not found")` and the runner prints `No such file or directory (os error 2)`, so it never fired and `test-ratchet` named my own test as newly failing
- whether a tool is installed has a direct answer -- `Command::new("iverilog").arg("-V").output()` -- and a message is the tool's to change while `PATH` is not
- both legs now executed before committing: with the simulator on PATH (passes) and with it stripped (prints the reason and passes); running the test binary directly under a reduced PATH is the one command that catches this
- ci-gates 426
