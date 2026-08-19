`default_nettype none
// W799: IGLA-RACE ternary node on our die, checked by the spec's OWN claim.
//
// `specs/igla/race/ternary_node.t27` ends every step in
//     node_step_b(hi, lo, act_a, act_b, acc) = acc_b(acc, weighted_b(...))
// and the sibling spec states the accumulator's contract in words: "there is no
// normalisation stage, no rounding, and nothing for a floating-point comparison
// to be imprecise about -- the datapath holds two exact integers."
//
// That is a testable claim, and it needs no golden values:
//
//   1. EXACT ADDITIVITY   node(.., acc=K) == node(.., acc=0) + K, for every K.
//                         Any rounding, saturation or normalisation anywhere in
//                         the accumulate path breaks this for some K.
//   2. NON-TRIVIALITY     node(.., acc=0) != 0 for at least one symbol -- else a
//                         module returning `acc` unchanged passes clause 1
//                         perfectly, and clause 1 alone would be a wire test.
//
// Three instances share (hi, lo, act_a, act_b) and differ only in `acc`: 0, a
// small positive, and a large negative that would expose a saturating adder.
module ternary_node_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge cfgmclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    // sweep the two comparator bits so every symbol the slicer can name is hit
    // W815 (T537): THE ACTIVATIONS WERE CONSTANTS AND THE ARITHMETIC FOLDED AWAY.
    // The weight symbol `v` was swept, so this wrapper was never at the dead
    // floor -- but `act_a`/`act_b`/`acc` were literals, and Yosys evaluated the
    // accumulator at synthesis time. Measured: 46 LUT / **8 CARRY4**, where 8 is
    // exactly this wrapper family's prescaler-plus-reset overhead (T534) and the
    // DUT alone needs 24. Driving the activations from counters gives
    // 146 LUT / 40 CARRY4 -- five times the carry logic, because there is now
    // carry logic. The old verdict proved the symbol sweep and the compilation
    // path; it did not prove the adder.
    reg [2:0] v = 3'd0;
    reg signed [31:0] liveA = 32'sd7;
    reg signed [31:0] liveB = 32'sd11;
    reg [7:0] hi, lo;
    always @* begin
        hi = {6'b0, v[2], v[1]};
        lo = {6'b0, v[1], v[0]};
    end

    localparam signed [31:0] KP = 32'sd12345;
    localparam signed [31:0] KN = -32'sd987654321;

    wire signed [31:0] r0, rp, rn;
    wire y0, yp, yn;
    IglaRaceTernaryNode n0 (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .hi(hi), .lo(lo), .act_a(liveA), .act_b(liveB),
        .acc(32'sd0), .ready(y0), .result(r0));
    IglaRaceTernaryNode np (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .hi(hi), .lo(lo), .act_a(liveA), .act_b(liveB),
        .acc(KP), .ready(yp), .result(rp));
    IglaRaceTernaryNode nn (.clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .hi(hi), .lo(lo), .act_a(liveA), .act_b(liveB),
        .acc(KN), .ready(yn), .result(rn));

    reg swept = 1'b0;
    reg add_ok = 1'b1;   // additivity held for every symbol
    reg moved  = 1'b0;   // some symbol produced a non-zero weighted term
    reg sig    = 1'b0;
    always @(posedge cfgmclk) begin
        if (!swept && rst_n) begin
            if (rp != (r0 + KP)) add_ok <= 1'b0;
            if (rn != (r0 + KN)) add_ok <= 1'b0;
            if (r0 != 32'sd0)    moved  <= 1'b1;
            if (v == 3'd7) begin
                swept <= 1'b1;
                sig <= (add_ok & (rp == (r0 + KP)) & (rn == (r0 + KN)))
                     & (moved | (r0 != 32'sd0));
            end
            v <= v + 3'd1;
        end
    end

    reg [23:0] pre = 24'd0;
    reg        beat = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin beat <= ~beat; liveA <= liveA + 32'sd1; liveB <= liveB - 32'sd1; end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5323C;
    always @(posedge drck)
        if (sel) begin
            // W820 (T548), migrated W839: LAYOUT v3, DESIGN 8 -- the clause
            // bits ride in the word, and the design nibble names whose they are.
            // add_ok = exact additivity held for every symbol; moved = the result actually
    // changed; swept = the sweep finished. One PADDING bit.
            // Bits [11:8] are the VERSION NIBBLE: 1 is this layout, 5 was the
            // legacy 28-bit magic. W819 watched `t27c silicon` report PASS from
            // two boards carrying a different design, because a 28-bit magic
            // matches whatever follows it (T547).
            if (capture)    sr <= {16'hA5A5, 4'd3, 6'd8, add_ok, moved, swept, 1'b1,
                                   beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
