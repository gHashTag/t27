`default_nettype none
// W798: TNF17e on our die, checked by an INVOLUTION with no golden constants.
//
// `specs/numeric/tnf17.t27` sets `on_comb(x) = tnf_negate(x)`, and negation of a
// sign-magnitude float is an involution: applying it twice must return the input
// exactly, for every code. So the known-answer test needs no expected values at
// all -- it needs a second instance:
//
//     x --> [TNF17] --> r1 --> [TNF17] --> r2      require r2 == x
//
// That is worth more than a table of constants. A table has to be derived from
// somewhere, and if it is derived from the same spec it checks nothing; this
// checks an algebraic property the hardware either has or does not, and no golden
// model was written to produce it.
//
// THE SECOND HALF, and it is the half that makes the first mean anything: an
// involution test passes trivially if `on_comb` is the IDENTITY. So a separate
// bit requires that at least one probe actually MOVED -- r1 != x. Both must hold.
//
// Probes sweep the sign bit and the offset field: TNF17e is [s(1)|offset(7)|
// mantissa(9)] and TNF_ONE = 20480 (offset 40, mantissa 0), whose negation
// tnf17.t27's own `comb_surface_negates` test puts at 86016. That value is NOT
// used here -- it is quoted only to say which bit the probes are moving.
module tnf17_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge cfgmclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    // eight probes across sign, offset and mantissa
    reg  [2:0]  v = 3'd0;
    reg  [31:0] x;
    always @* begin
        case (v)
            3'd0: x = 32'd20480;   // TNF_ONE: offset 40, mantissa 0
            3'd1: x = 32'd86016;   // its negation
            3'd2: x = 32'd0;       // all zero
            3'd3: x = 32'd511;     // mantissa full, offset 0
            3'd4: x = 32'd16384;   // offset 32
            3'd5: x = 32'd40960;   // offset 80, the top rung
            3'd6: x = 32'd65536;   // sign bit alone
            3'd7: x = 32'd131071;  // every field saturated
        endcase
    end

    wire        rdy1, rdy2;
    wire [31:0] r1, r2;
    TNF17 dut_a (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
                 .x(x),  .ready(rdy1), .result(r1));
    TNF17 dut_b (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
                 .x(r1), .ready(rdy2), .result(r2));

    reg swept   = 1'b0;
    reg inv_acc = 1'b1;   // every probe returned to itself
    reg moved   = 1'b0;   // at least one probe actually changed
    reg sig     = 1'b0;
    always @(posedge cfgmclk) begin
        if (!swept && rst_n) begin
            if (r2 != x) inv_acc <= 1'b0;
            if (r1 != x) moved   <= 1'b1;
            if (v == 3'd7) begin
                swept <= 1'b1;
                // fold the last probe into both halves
                sig <= (inv_acc & (r2 == x)) & (moved | (r1 != x));
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
    reg [31:0] sr = 32'hA5A5A5A4;
    always @(posedge drck)
        if (sel) begin
            if (capture)    sr <= {28'hA5A5A5A, 1'b0, 1'b1, beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
