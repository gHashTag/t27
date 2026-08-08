`default_nettype none

// ternary_mac_demo_top_v2.v -- observable on-board demo for ternary_mac_top.
//
// Supersedes ternary_mac_demo_top.v, whose on-silicon behaviour was not
// checkable.  That design had three defects, all of which made a successful
// flash indistinguishable from a broken one:
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
// This version fixes all three:
//   * Clock is STARTUPE2/CFGMCLK -- the same characterized primitive the
//     working phi_temporal heartbeat uses (nominal ~65 MHz on 7-series).
//   * A 24-bit prescaler steps the datapath at ~3.9 Hz, so every state change
//     is visible.
//   * Each step applies the next weight in the repeating sequence
//     {+1, 0, -1, 0} to a fixed activation and feeds acc_out back into acc_in,
//     so the accumulator genuinely accumulates and all three decode branches
//     are live.
//
// Expected on-board signature (this is the pass criterion):
//   With a = +1 and the weight sequence above, acc_out walks
//     0 -> +1 -> +1 -> 0 -> 0 -> +1 -> +1 -> 0 -> ...
//   led_r23 shows "accumulator is non-zero", led_t23 shows "accumulator is
//   negative".  Observed: led_r23 blinks at ~1 Hz with a 50 % duty cycle,
//   led_t23 stays dark.  A steady or dark led_r23 means the MAC is not
//   accumulating.
//
// Target: QMTech Wukong V1 / XC7A200T-FGG676 via OpenXC7 or Vivado-in-Docker.
// Board SSOT: fpga/HARDWARE_SSOT.md

module ternary_mac_demo_top_v2 #(
    // CFGMCLK (~65 MHz) / 2^PRESCALE_BITS ~= 3.9 steps/s at the default.
    // The testbench overrides this to keep simulation short; synthesis uses 24.
    parameter integer PRESCALE_BITS = 24
) (
    output wire led_r23,
    output wire led_t23
);
    // ------------------------------------------------------------------
    // Clock: the internal configuration oscillator.  Unlike a ring
    // oscillator this is a real primitive with a datasheet frequency, so the
    // step rate below is a number we can state rather than guess.
    // ------------------------------------------------------------------
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

    // ------------------------------------------------------------------
    // Prescaler: one datapath step per 2^PRESCALE_BITS clocks.
    // ------------------------------------------------------------------
    reg [PRESCALE_BITS-1:0] prescale = {PRESCALE_BITS{1'b0}};
    wire step = (prescale == {PRESCALE_BITS{1'b1}});

    always @(posedge cfgmclk) begin
        prescale <= prescale + 1'b1;
    end

    // ------------------------------------------------------------------
    // Reset: hold the MAC in reset for the first prescaler wrap so the
    // accumulator starts from a known zero even without an external button.
    // ------------------------------------------------------------------
    reg [1:0] por = 2'b00;
    always @(posedge cfgmclk) begin
        if (step && por != 2'b11)
            por <= por + 1'b1;
    end
    wire rst_n = (por == 2'b11);

    // ------------------------------------------------------------------
    // Weight sequence {+1, 0, -1, 0}: exercises every decode branch of
    // ternary_mac_top, including the two distinct zero encodings.
    // ------------------------------------------------------------------
    reg [1:0] phase = 2'b00;
    always @(posedge cfgmclk) begin
        if (!rst_n)
            phase <= 2'b00;
        else if (step)
            phase <= phase + 1'b1;
    end

    reg [1:0] w_code;
    always @(*) begin
        case (phase)
            2'd0:    w_code = 2'b01;  // +1
            2'd1:    w_code = 2'b00;  // zero, encoding A
            2'd2:    w_code = 2'b10;  // -1
            default: w_code = 2'b11;  // zero, encoding B
        endcase
    end

    // ------------------------------------------------------------------
    // The MAC, with its own output fed back as the accumulator input.  This
    // is what makes it an accumulator rather than a single multiply.
    // ------------------------------------------------------------------
    wire signed [7:0]  a = 8'sd1;
    wire signed [31:0] acc_out;

    ternary_mac_top mac (
        .clk(cfgmclk),
        .rst_n(rst_n),
        .en(step),
        .a(a),
        .w_code(w_code),
        .acc_in(acc_out),
        .acc_out(acc_out)
    );

    // ------------------------------------------------------------------
    // Observable state.  Both are functions of the whole 32-bit accumulator,
    // so a stuck or constant-folded datapath shows up immediately.
    // LEDs on the Wukong V1 are active-low.
    // ------------------------------------------------------------------
    assign led_r23 = ~(acc_out != 32'sd0);
    assign led_t23 = ~(acc_out[31]);
endmodule
