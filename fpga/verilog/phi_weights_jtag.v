`default_nettype none
// W799: IGLA-RACE phi weights on our die, checked by ANTISYMMETRY, no constants.
//
// `specs/igla/race/phi_weights.t27` sets
//     on_comb(code) = weight_apply_b(code, PHI_A, PHI_B)
// with the three-symbol alphabet
//     GAT_ZERO = 0 -> 0        GAT_POS = 1 -> +phi        GAT_NEG = 2 -> -phi
//
// The check needs no expected values, only the algebra the alphabet claims:
//
//   1. ANTISYMMETRY   applying -phi is the negation of applying +phi:
//                         r(GAT_NEG) == -r(GAT_POS)
//   2. ANNIHILATION   the zero symbol yields zero, and so does every code
//                         outside the alphabet:
//                         r(GAT_ZERO) == 0  and  r(3) == 0  and  r(255) == 0
//   3. NON-TRIVIALITY r(GAT_POS) != 0 -- without this, a module that returns 0
//                         for everything satisfies 1 and 2 perfectly.
//
// Clause 3 is the one that matters. This is the same trap the TNF17 involution
// had (T473a): a property test that a dead wire passes is not a test. Here the
// dead answer is "always zero", and the alphabet's own claim -- that +phi and
// -phi are DISTINCT weights -- is exactly what rules it out.
//
// Five instances, one per probe, all combinational; STARTUPE2 supplies the clock
// so no package pin is needed, and BSCANE2 returns {A5A5A5A, 0, 1, beat, ok}.
module phi_weights_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge cfgmclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    wire signed [31:0] r_zero, r_pos, r_neg, r_three, r_ff;
    wire z0, z1, z2, z3, z4;
    IglaRacePhiWeights w0 (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
                           .code(8'd0),   .ready(z0), .result(r_zero));
    IglaRacePhiWeights w1 (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
                           .code(8'd1),   .ready(z1), .result(r_pos));
    IglaRacePhiWeights w2 (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
                           .code(8'd2),   .ready(z2), .result(r_neg));
    IglaRacePhiWeights w3 (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
                           .code(8'd3),   .ready(z3), .result(r_three));
    IglaRacePhiWeights w4 (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
                           .code(8'd255), .ready(z4), .result(r_ff));

    wire antisym  = (r_neg == -r_pos);
    wire annihil  = (r_zero == 32'sd0) && (r_three == 32'sd0) && (r_ff == 32'sd0);
    wire nontriv  = (r_pos != 32'sd0);

    reg sig = 1'b0;
    reg [3:0] settle = 4'd0;
    always @(posedge cfgmclk) begin
        if (rst_n && settle != 4'hF) settle <= settle + 4'd1;
        if (settle == 4'hF) sig <= antisym & annihil & nontriv;
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
            // antisym = -phi is the negation of +phi; annihil = zero and out-of-alphabet codes
    // give zero; nontriv = +phi is not zero, without which a dead module satisfies
    // the first two. One PADDING bit.
            // Bits [11:8] are the VERSION NIBBLE: 1 is this layout, 5 was the
            // legacy 28-bit magic. W819 watched `t27c silicon` report PASS from
            // two boards carrying a different design, because a 28-bit magic
            // matches whatever follows it (T547).
            if (capture)    sr <= {20'hA5A5A, 4'd1, antisym, annihil, nontriv, 1'b1,
                                   1'b0, 1'b1, beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
