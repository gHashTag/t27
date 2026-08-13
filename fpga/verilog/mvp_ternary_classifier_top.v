`default_nettype none

// mvp_ternary_classifier_top.v -- board wrapper for the IGLA RACE MVP:
// a complete ternary neural network layer, self-checking on silicon.
//
// WHY THIS EXISTS.  WAVE_LOOP_656 loaded the MVP onto three boards and could
// only report `Done 0x0 -> 0x1`.  That proves the fabric was CONFIGURED; it
// says nothing about what the fabric COMPUTES.  Section 7 of that report lists
// "no function readback" as the first thing not done, and Option 1 recommends
// fixing it.  This wrapper fixes it without a single new pin: instead of
// exporting the result and checking it off-chip, it puts the reference table
// from specs/igla/race/mvp_ternary_classifier.t27 INTO the silicon and lets the
// silicon check itself.
//
// PASS CRITERION -- stated before the flash, and discriminating:
//
//   led_r23 BLINKS ~2 Hz, led_t23 DARK    -> every vector classified correctly
//   led_r23 DARK,        led_t23 LIT      -> a mismatch was latched; the
//                                            network computes a wrong class
//   both DARK                             -> no clock / not configured
//   both LIT steady                       -> contradiction; harness is wrong
//
// The failure lamp is sticky: `ok` can only ever fall, never rise, so a single
// wrong class at any point in the cycle is latched permanently and cannot be
// blinked away.  This is the property the first demo lacked -- a design whose
// failure looks like its success proves nothing.  A blinking r23 is the only
// state that requires the arithmetic to be right, ten times over, forever.
//
// This module is deliberately thin: it supplies a real, characterized clock and
// nothing else.  All behaviour lives in mvp_ternary_classifier_check, which
// takes an ordinary clock port and is therefore simulatable without a STARTUPE2
// stub -- the verdict is checked in iverilog before it is trusted on silicon.
//
// Target: QMTech Wukong V1 / XC7A200T-FGG676 via OpenXC7.
// Board SSOT: fpga/HARDWARE_SSOT.md
// Refs #1959

module mvp_ternary_classifier_top #(
    // CFGMCLK (~65 MHz) / 2^PRESCALE_BITS ~= 3.9 vectors/s at the default.
    parameter integer PRESCALE_BITS = 24
) (
    output wire led_r23,
    output wire led_t23
);
    // The internal configuration oscillator.  Unlike a ring oscillator this is
    // a real primitive with a datasheet frequency, so the vector rate is a
    // number we can state rather than guess, and the clock net can be
    // constrained (see mvp_ternary_classifier_top.xdc).
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

    mvp_ternary_classifier_check #(
        .PRESCALE_BITS(PRESCALE_BITS)
    ) check (
        .clk(cfgmclk),
        .led_r23(led_r23),
        .led_t23(led_t23)
    );
endmodule

`default_nettype wire
