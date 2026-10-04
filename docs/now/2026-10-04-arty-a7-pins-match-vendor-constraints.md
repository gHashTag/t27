# NOW -- Arty A7 pins match vendor constraints (2026-10-04)

The board profile's old pin assertions passed while reset, both FPGA UART pins
and all four LEDs were assigned incorrectly. RX C9 also shared a package pin
with button1. Every modeled bank was0. The count-only change from PR5747 did
not expose these defects. Closes #5884. Closes #5722.

The source corrects seven package assignments and twelve physical I/O bank
IDs. Existing function signatures, port names, input/output/clock flags and
FPGA part constants remain. The original count_buttons test and all three
original source/publisher/Vasilev commits remain in the integration history;
the original publisher receipt is preserved.

## Vendor mapping and UART viewpoint

Both Digilent Master XDC variants agree on these pins, LVCMOS33 and banks:

| FPGA signal | Package pin | Physical bank | Direction |
| --- | --- | --- | --- |
| clk | E3 |35| input, clock |
| rst_n | C2 |35| input |
| uart_tx | D10 |16| output |
| uart_rx | A9 |16| input |
| led0 | H5 |35| output |
| led1 | J5 |35| output |
| led2 | T9 |14| output |
| led3 | T10 |14| output |
| btn0 | D9 |16| input |
| btn1 | C9 |16| input |
| btn2 | B9 |16| input |
| btn3 | B8 |16| input |

The XDC uart_rxd_out and uart_txd_in labels describe the bridge/PC viewpoint.
They must not be copied into FPGA-facing TX/RX names without resolving
physical direction. Independently joining the vendor board.xml logical
TxD/RxD direction to part0_pins.xml gives FPGA TX D10/output and RX A9/input.
The same join for clock/reset/LED/button interfaces confirms all twelve pins.
The XDC FPGA IO comments supply the physical bank numbers.

Primary evidence:

- [35T XDC](https://github.com/Digilent/digilent-xdc/blob/00a3404901f35aa9567b01ecb3f2c233b6efe9f4/Arty-A7-35-Master.xdc),
  [100T XDC](https://github.com/Digilent/digilent-xdc/blob/00a3404901f35aa9567b01ecb3f2c233b6efe9f4/Arty-A7-100-Master.xdc).
  The35T file explicitly covers RevD/RevE;100T likewise covers RevD/RevE.
- Digilent/vivado-boards new/board_files/arty-a7-35/E.0/1.1:
  board.xml blobf0ebb7386933dbd9cff7a2b30283e367595baea8;
  part0_pins.xml blob3f6de551304064f26f61f9a8b61b928f3ac61949.
- Digilent/vivado-boards new/board_files/arty-a7-100/E.0/1.1:
  board.xml blob1bffd15b522cc12dee32802d830365b8c9eb1346;
  part0_pins.xml blobdc32e54efd754ba1df183231cb20ae6341ff9c47.

## Executed verification

TDD first applied the vendor-backed assertions to the old source:12 of23
actual Zig tests failed, and a direct generated Zig execution panicked in the
reset assertion. The independently executed old Rust/C declarations disagreed
with both vendor tables for all12records and detected the RX/button1 collision.
After the source correction:

- All23 generated Zig tests and all11 existing compile-time invariants pass;
  five of five functions are covered, including count_buttons. Typecheck has
  zero errors/warnings. The old API/function bodies are unchanged.
- Unmodified generated Rust declarations compile and execute a pin/count
  driver. An existing unused-parentheses warning remains in clock_freq_mhz.
  The Rust backend does not lower spec tests; its execution is compared
  independently with the vendor table, not presented as a backend test suite.
- Existing C string-assertion lowering prevents full raw C compilation both
  before and after: CONFIG_VOLTAGE equality becomes a nonintegral
  _Static_assert, and string tests use pointer comparison. A SOURCE fixture
  replaces spec checks with one numeric sanity test, then regenerates C.
  Its type/constant/function declaration bytes exactly match canonical C;
  generated files are not edited. This declaration-only C driver compiles
  with C11 -Wall -Wextra -Werror and executes against the independent oracle.
- Both drivers compare twelve pins and six vendor fields against both35T and
  100T tables,288 scalar field comparisons in total. Each checks all66 pairs
  for package-pin uniqueness. Count returns4. No mismatches/collisions remain.
- Five type-correct source mutants compile to C/Rust declarations but are
  rejected by actual generated Zig runtime assertions and the vendor/count
  oracle: wrong button count, wrong LED pin, swapped UART pins, RX/button1
  collision and bank0. Tests do execute; wrapper exit0 alone is not a verdict.
- The compiler refreshes both existing seal aliases naming this same spec,
  BoardArtyA7.json and boards_BoardArtyA7.json, and verifies all hashes MATCH.
  Both record23/23 unforced tests. No force seal, deleted alias or hand-edited
  hash is used.
- Fast corpus ratchet remains95/95CLEAN, with no unexpected failure/pass,
  discarded input or gate drift and no expectation changes. Types77/77CLEAN;
  duplicate bodies590/4779 in169groups across1174specs, no new/growing group.

Reproduction: t27c typecheck/test-report/coverage/seal --verify on
specs/boards/arty_a7.t27; t27c suite --repo-root . --ratchet --fast;
tri types ratchet and the existing duplicate-body gate. Local independent
vendor-driver and mutation evidence is retained with the campaign checkpoint.

This repairs modeled pin/bank metadata. It does not place these pins in a
bitstream, prove timing, flash/test a board, validate radio traffic or perform
complete inference. Existing FPGA part identifiers are retained: the35T vendor
board.xml and pin-map part metadata differ, so SKU/revision selection needs its
own review. The95 known corpus failures and the raw C assertion bug remain;
skipped long suite phases are not reported as executed.
