# NOW -- LED D5 port registers the counter and LED (2026-10-03)

Closes #5846. Supersedes the unverified clock boundary in PR #5847.

## What was read

- Original `gHashTag/trinity:fpga/openxc7-synth/led_d5_test.v` at `cdb78c21913531ff2864fdd3391edc73b61a9475`: 24-bit increment on each rising clock and `led = blink_counter[23]`. It has no initialized power-on state.
- Queen branch `queen-5846`, source commit `7a87ef7db944669ff4f33f600dab249d5dc8653e`, published head `fc37a8415e9e2aa8754090895721fc985cb6eb9a`. The publisher changed only the NOW entry, so its source matched the judged source. Three Zig tests passed, but the generated RTL duplicated `clk`, assigned an undeclared `on_clock`, and exposed no LED.
- Compiler clock lowering uses module-level mutable scalar state as output registers and adds `rst_n`, `en`, and `ready`. Clock-function parameters become data inputs; its return does not create a register.

## What changed

- Kept the bee's masked next-count and bit-23 helper functions. Added actual counter and LED state and made `on_clock()` a void clocked update. Its LED assignment evaluates the next count before changing the host counter, so sequential Zig and nonblocking RTL agree.
- Added a real clock-entry test. All three functions now have tests. Corrected this single coordination receipt; no generated file, compiler, seal or failure ledger changed. Original bee/publisher commits and author metadata remain in the branch history.
- The generated adapter exposes a 32-bit counter whose value remains masked to 24 bits, plus reset/enable/ready ports. These are the backend interface, not the original board pinout.

## Verified

- Fresh t27c from master `6e3322918`: no parser discard, typecheck zero errors/warnings, 4 Zig tests pass, coverage 3/3. Icarus accepts the generated RTL with `-g2012`.
- A deterministic bench passed 547 observations: 540 original-versus-generated comparisons (135 explicit starting counts, four enabled edges each), plus seven reset/disabled-hold observations. Both original and generated counters were explicitly seeded; no equivalence claim is made about unknown power-on values. Original reference SHA256: `5a53a02974d67fee888016c58d41bb548adf3f94b1f9c8c810b26ee695359c1c`.
- Three changed source copies still compiled as RTL but failed that same bench: LED one-cycle lag, missing 24-bit wrap, and stuck counter. These failures establish that the comparisons observe both signals and the actual clock boundary.
- The branch includes canonical master `facfd8196d3bf642345e6cd74319eb31837e53ed`. Its compiler is unchanged from the compiler build above.

## Reproduce

- Run `t27c test-report specs/port/trinity/fpga/openxc7-synth/led_d5_test.t27` and inspect the words: this command exits zero even when BLOCKED or FAIL appears.
- Save `t27c gen-verilog specs/port/trinity/fpga/openxc7-synth/led_d5_test.t27` to `/tmp/led-generated.v`.
- Save the original Verilog at the pinned SHA to `/tmp/led-reference.v`, renaming only its module to `trinity_top_ref` for co-instantiation. Save the bench below to `/tmp/led-tb.v`. Run `iverilog -g2012 -s tb -o /tmp/led-test /tmp/led-generated.v /tmp/led-reference.v /tmp/led-tb.v && vvp /tmp/led-test`; it must print `PASS 547 RTL observations`.

```verilog
`timescale 1ns/1ps
module tb;
 reg clk=0, rst_n=0, en=1;
 wire led, ready, reference_led;
 wire [31:0] blink_counter;
 integer checks=0;
 integer i;
 reg [23:0] seed;
 trinity_top dut(.clk(clk),.rst_n(rst_n),.en(en),.led(led),.ready(ready),.blink_counter(blink_counter));
 trinity_top_ref reference(.clk(clk),.led(reference_led));
 task compare_seed(input [23:0] initial_count);
  begin
   clk=0; rst_n=1; en=1;
   dut.blink_counter=initial_count;
   dut.led=initial_count[23];
   reference.blink_counter=initial_count;
   repeat(4) begin
    #4 clk=1; #1;
    if(blink_counter !== {8'b0,reference.blink_counter} || led !== reference_led || ready !== 1'b1)
     $fatal(1,"differential seed=%h count=%h reference=%h led=%b reference_led=%b",initial_count,blink_counter,reference.blink_counter,led,reference_led);
    checks=checks+1;
    #4 clk=0;
   end
  end
 endtask
 initial begin
  #1 rst_n=1; #1 rst_n=0; #1;
  if(blink_counter !== 0 || led !== 0) $fatal(1,"reset did not initialize state");
  checks=checks+1;
  compare_seed(0); compare_seed(1); compare_seed(24'h7ffffe); compare_seed(24'h7fffff);
  compare_seed(24'h800000); compare_seed(24'hfffffe); compare_seed(24'hffffff);
  seed=24'h13579b;
  for(i=0;i<128;i=i+1) begin
   compare_seed(seed);
   seed=(seed*24'h65+24'h197);
  end
  clk=0; en=0; dut.blink_counter=24'h800000; dut.led=1;
  repeat(5) begin
   #4 clk=1; #1;
   if(blink_counter !== 24'h800000 || led !== 1) $fatal(1,"disabled clock changed state");
   checks=checks+1; #4 clk=0;
  end
  rst_n=0; #1;
  if(blink_counter !== 0 || led !== 0) $fatal(1,"asynchronous reset failed while disabled");
  checks=checks+1;
  $display("PASS %0d RTL observations (540 differential, 7 reset/hold)",checks);
  $finish;
 end
endmodule

```

## Not verified

- No physical board, synthesis, place-and-route, bitstream or timing measurement. Original uninitialized startup remains undefined; the generated reset/enable adapter was tested separately from enabled-clock differential behavior.
- Full-corpus CI has existing packets parse, type-name collision and ring/spec drift failures. This receipt does not claim that the whole repository is green. No hidden failure was added to a ledger.
