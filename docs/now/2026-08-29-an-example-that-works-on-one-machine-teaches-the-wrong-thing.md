# NOW -- An example that works on one machine teaches the wrong thing (2026-08-29)

## An example that works on one machine teaches the wrong thing (Refs #2804)

- examples/fpga/qmtech_minimal/build.sh named one developer's home SIX times -- an example is the one kind of file whose purpose is to be copied
- scripts/fpga-build.sh held three one-machine-path defaults; now derived from T27_OPENXC7, the same variable the compiler reads since W706
- my own first version exited 128 in silence: set -e kills the script on a failing git rev-parse inside a substitution, before the friendly message can print. The out-of-repository control caught it
- baseline 32/47 -> 30/38, and the three kinds of entry are now named in it: configuration to fix, records not to rewrite, experiments to leave
