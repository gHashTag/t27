`default_nettype none
// Exhaustive on-die proof of the 3B2T delimiter theorem. Refs #1959
//
// THE WHOLE CLAIM IN ONE EQUALITY. The encoder maps 8 data words onto ternary
// symbol PAIRS. Nine pairs exist; 3B2T spends eight, and the ninth -- (+1,+1),
// wire code 4'b0101 = 5 -- is the frame delimiter. Sweep every input, set one
// bit per codeword produced, and the resulting 16-bit map must equal
//
//     {0,1,2,4,6,8,9,10} = 16'd1879
//
// That single comparison certifies ALL THREE properties at once:
//   * injectivity  -- exactly 8 bits set, so no two words collide
//   * delimiter absence -- bit 5 clear, so (+1,+1) cannot arise from data
//   * symbol validity   -- bits 3,7,11..15 clear, so no illegal wire code
//
// The input space is 8 values, so this is EXHAUSTIVE, not sampled. There is no
// golden model to co-author (Knight & Leveson 1986): the expected constant is
// derived from the ternary alphabet itself, not from a second implementation.
module ternary_link_jtag #(parameter integer JTAG_CHAIN_N = 3);
    localparam [15:0] EXPECT = 16'd1879;

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg  [2:0]  v      = 3'd0;
    reg  [15:0] seen   = 16'd0;
    reg         swept  = 1'b0;
    reg         sig    = 1'b0;
    wire [7:0]  code;

    wire dut_ready;
    // rst_n released after the STARTUPE2 clock is running; `en` held high so the
    // combinational surface is evaluated every cycle.
    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge cfgmclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;
    ZeroDSP_TernaryLink dut (
        .clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .v({5'b0, v}), .ready(dut_ready), .result(code));

    always @(posedge cfgmclk) begin
        if (!swept && rst_n) begin
            seen <= seen | (16'd1 << code[3:0]);
            if (v == 3'd7) begin
                swept <= 1'b1;
                // compare AFTER the last word is folded in
                sig   <= ((seen | (16'd1 << code[3:0])) == EXPECT);
            end
            v <= v + 3'd1;
        end
    end

    reg [23:0] pre = 24'd0;
    reg        beat = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) beat <= ~beat;
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5A1F4;
    always @(posedge drck)
        if (sel) begin
            // W820 (T548): LAYOUT v1 -- the clause bits ride in the word.
            // swept = every input symbol was presented. This wrapper folds its validity checks
    // into `sig` through a 16-bit `seen` mask rather than into named wires, so only
    // ONE clause can be surfaced honestly; the other three are PADDING and are
    // marked as such rather than invented.
            // Bits [11:8] are the VERSION NIBBLE: 1 is this layout, 5 was the
            // legacy 28-bit magic. W819 watched `t27c silicon` report PASS from
            // two boards carrying a different design, because a 28-bit magic
            // matches whatever follows it (T547).
            if (capture)    sr <= {20'hA5A5A, 4'd1, swept, 1'b1, 1'b1, 1'b1,
                                   1'b0, 1'b1, beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
