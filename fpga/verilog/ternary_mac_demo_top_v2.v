`default_nettype none

// ternary_mac_demo_top_v2.v -- board wrapper for the IGLA RACE ternary MAC demo.
//
// Supersedes ternary_mac_demo_top.v, whose on-silicon behaviour was not
// checkable.  That design had three defects, each sufficient on its own to
// make a successful flash indistinguishable from a broken one:
//
//   1. Clock was a 20-stage LUT1 ring oscillator closed with
//      ALLOW_COMBINATORIAL_LOOPS.  Its frequency is PVT-dependent and
//      unconstrained, so no Fmax can be reported and no timing claim survives.
//   2. LEDs were driven from acc_out[0] and acc_out[1], which toggle at
//      f_osc/2 and f_osc/4 -- ~10^8 Hz.  Both LEDs sit at ~50 % brightness;
//      the eye cannot distinguish "working" from "stuck".
//   3. w_code was tied to 2'b01 (+1) and acc_in to 0.  The minus-weight and
//      zero-weight decode paths were never exercised, and the accumulator
//      never accumulated -- synthesis is free to constant-fold both away.
//      The design therefore proved only that the toolchain emits a loadable
//      bitstream, not that the ternary MAC computes anything.
//
// This module is deliberately thin: it supplies a real, characterized clock
// and nothing else.  All behaviour lives in ternary_mac_demo_core, which takes
// an ordinary clock port and is therefore both simulatable without a STARTUPE2
// stub and reachable by yosys model checking (see fpga/formal/).
//
// Expected on-board signature (this is the pass criterion):
//   led_r23 blinks at ~1 Hz with a 50 % duty cycle; led_t23 stays dark.
//   A steady or dark led_r23 means the MAC is not accumulating.  A lit led_t23
//   contradicts theorem T3 and means the minus-weight path is wrong.
//
// Target: QMTech Wukong V1 / XC7A200T-FGG676 via OpenXC7 or Vivado-in-Docker.
// Board SSOT: fpga/HARDWARE_SSOT.md

module ternary_mac_demo_top_v2 #(
    // CFGMCLK (~65 MHz) / 2^PRESCALE_BITS ~= 3.9 steps/s at the default.
    parameter integer PRESCALE_BITS = 24
) (
    output wire led_r23,
    output wire led_t23
);
    // The internal configuration oscillator.  Unlike a ring oscillator this is
    // a real primitive with a datasheet frequency, so the step rate is a number
    // we can state rather than guess, and the clock net can be constrained
    // (see ternary_mac_demo_top_v2.xdc).
    wire cfgmclk;
    STARTUPE2 #(
        .PROG_USR("FALSE"),
        .SIM_CCLK_FREQ(10.0)
    ) startup (
        .CFGCLK(),
        .CFGMCLK(cfgmclk),
        .EOS(),
        .PREQ(),
        .CLK(1'b0),
        .GSR(1'b0),
        .GTS(1'b0),
        .KEYCLEARB(1'b0),
        .PACK(1'b0),
        .USRCCLKO(1'b0),
        .USRCCLKTS(1'b0),
        .USRDONEO(1'b1),
        .USRDONETS(1'b1)
    );

    ternary_mac_demo_core #(
        .PRESCALE_BITS(PRESCALE_BITS)
    ) core (
        .clk(cfgmclk),
        .led_r23(led_r23),
        .led_t23(led_t23)
    );
endmodule
