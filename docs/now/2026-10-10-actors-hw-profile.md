# NOW -- actors_hw.t27: the actor card's decisions at hardware widths, bit-exact, one engine for the AX7203 (2026-10-10)

## specs/queen/actors_hw.t27 (Closes #8336)

- Every decision of actors.t27 as `hw_<name>`, at the widths its values need (pid exact u64; tags and counts u8; seconds u16, saturating; milliseconds u32), with no data-dependent loop, no `*`, `/` or `%` on a value and no DSP. The jitter stays the card's second: Knuth's hash in canonical signed digits, the modulo by restoring division, and a 48-cycle serial unit. `fn on_clock` is one time-multiplexed decision engine with 77 operations.
- `t27c test-report`: 17 tests, 0 FAIL, 0 vacuous, 14.7 M runtime asserts; 5 invariants pin every `HW_` constant to the card's; 21 of 21 negative controls killed. The generated Verilog of all 79 functions was simulated against the card's own (Icarus, 20,000 vectors each, 0 mismatches). That check found a gen-verilog defect the Zig tests could not see (#8341).
- yosys 0.63 `synth_xilinx -nodsp`: on today's card 70 of 79 functions synthesize (23,508 LUT + 6,103 INV; `jittered_seconds` alone 20,225 + 5,921). With this profile all 79 synthesize (2,979 LUT + 147 INV). One engine: 1,869 LUT + 388 INV, 256 FF, 146 CARRY4, 0 DSP. A board holds 37,376 actors (BRAM-bound) and about 41 engines at 70% LUT fill; three boards hold 112,128.
- The three-board split is in the PR's design note:
  - one bitstream for every board, with the node id read from the device DNA through a named table;
  - the pid's 12-bit node field routes each send (`hw_is_remote`);
  - heartbeats every 5 s, and a board silent for 20 s is down (`X_NOCONNECTION`);
  - `place_before` picks the roomiest board, and keyed actors are placed by rendezvous hash;
  - the link is an Ethernet ring over the two KSZ9031 PHYs, with a serial ring as the fallback until RGMII bring-up is done;
  - each added board brings +37,376 actors.
- Compiler defects filed, see #5980: #8338 constant dividers, #8339 loop unrolling, #8340 constants dropped as type aliases, #8341 dropped widening casts, #8342 import shadowing, #8343 Zig parentheses, #8344 entry-port names, #8346 `on_clock` locals, #8351 seal v2 closure of `use` items, #8352 one reset INV per flip-flop.
