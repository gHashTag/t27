`default_nettype none
// W821: a signed four-term dot product on our die, checked by algebra. LAYOUT v3, DESIGN 3.
//
// `specs/ternary/gft_signed_dot4.t27` computes
//     on_comb = sadd(dot2(a1,b1, a2,b2), dot2(a3,b3, a4,b4))
//     dot2    = sadd(smul(a1,b1), smul(a2,b2))
// -- four TNF-float multiplies and three adds. It is one of the 25 specs that
// could not be synthesised at all until W813 removed yosys's `share` pass.
//
// THE CLAUSES, all from properties the spec's own test asserts or that any
// correct dot product must have. The fourth is the one that makes the rest mean
// something:
//
//   1. CANCELLATION  (+1·1) + (−1·1) + (+1·1) + (−1·1) = 0.
//                    This is `test cancel`, with 86016 = −1.0 (TNF_ONE with the
//                    sign bit set).
//   2. ANNIHILATION  every operand zero gives zero.
//   3. COMMUTATIVITY a·b must equal b·a. A dot product that silently drops one
//                    operand, or swaps a sign per position, fails here and
//                    passes cancellation -- which is symmetric under exactly
//                    those errors.
//   4. NON-TRIVIALITY a single non-zero product is NOT zero. Without it a module
//                    returning 0 for everything satisfies 1, 2 and 3 perfectly --
//                    the trap `tnf17_jtag.v` had at T473a and
//                    `gft_bitnet_neuron_jtag.v` at T534.
//
// ONE OPERAND IS LIVE. W815-W817 measured that a wrapper whose probes are all
// constants folds its DUT away: CARRY4 == 8 is this family's prescaler floor and
// means no arithmetic reached the fabric (T534/T536). `t27c silicon` now refuses
// to load such a build (T539/T540), so the live drive is not decoration -- it is
// what makes the gate pass honestly.
//
// WORD LAYOUT v3, DESIGN 3 (T548; migrated W839):
//     {16'hA5A5, 4'd3, 6'd3, c_can, c_ann, c_com, c_non, beat, ok}
// Bits [11:8] are the version nibble; 5 was the legacy 28-bit magic. Every
// wrapper on this bench now speaks v1, so a reader cannot attribute one design's
// verdict to another (T547/T549).
//
// TNF constants per `specs/numeric/tnf17.t27`: 20480 = +1.0, 86016 = −1.0
// (20480 ^ 65536, the sign bit), 0 = zero.
module gft_signed_dot4_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // W821 (T551): all four probes live measure 7.16 MHz, so the whole wrapper
    // runs on CFGMCLK/16 through a BUFG. The ratio is DECLARED in
    // `gft_signed_dot4_jtag.xdc` -- a divider alone tells the timing engine
    // nothing (T541), which is the mistake W818 made and W819 fixed.
    reg [3:0] dv = 4'd0;
    always @(posedge cfgmclk) dv <= dv + 4'd1;
    wire slowclk;
    BUFG bufg_slow (.I(dv[3]), .O(slowclk));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge slowclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    localparam [31:0] P1 = 32'd20480;   // +1.0
    localparam [31:0] N1 = 32'd86016;   // -1.0
    localparam [31:0] Z  = 32'd0;

    reg [23:0] pre  = 24'd0;
    reg        beat = 1'b0;
    reg [31:0] live  = 32'd20480;
    reg [31:0] live2 = 32'd20736;
    always @(posedge slowclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin beat <= ~beat; live <= live + 32'd1; live2 <= live2 - 32'd1; end
    end

    wire y_can, y_ann, y_ab, y_ba, y_one;
    wire [31:0] r_can, r_ann, r_ab, r_ba, r_one;

    // 1. CANCELLATION, on a LIVE operand so the datapath cannot be folded away
    GftSignedDot4 u_can (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(P1), .b1(live), .a2(N1), .b2(live),
        .a3(P1), .b3(live), .a4(N1), .b4(live),
        .ready(y_can), .result(r_can));

    // 2. ANNIHILATION
    GftSignedDot4 u_ann (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(Z), .b1(live), .a2(Z), .b2(live2),
        .a3(Z), .b3(live), .a4(Z), .b4(live2),
        .ready(y_ann), .result(r_ann));

    // 3. COMMUTATIVITY: the same four products with each pair swapped
    GftSignedDot4 u_ab (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(live), .b1(N1),    .a2(Z),     .b2(live2),
        .a3(N1),   .b3(live2), .a4(live), .b4(live),
        .ready(y_ab), .result(r_ab));
    GftSignedDot4 u_ba (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(N1),    .b1(live), .a2(live2), .b2(Z),
        .a3(live2), .b3(N1),  .a4(live), .b4(live),
        .ready(y_ba), .result(r_ba));

    // 4. NON-TRIVIALITY: one product, and it must not be zero
    GftSignedDot4 u_one (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(live), .b1(P1), .a2(Z), .b2(Z),
        .a3(Z),    .b3(Z),  .a4(Z), .b4(Z),
        .ready(y_one), .result(r_one));

    wire can_ok = (r_can == 32'd0);
    wire ann_ok = (r_ann == 32'd0);
    wire com_ok = (r_ab  == r_ba);
    wire non_ok = (r_one != 32'd0) && (r_one == live);

    reg sig = 1'b0;
    reg c_can = 1'b1, c_ann = 1'b1, c_com = 1'b1, c_non = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!can_ok) c_can <= 1'b0;
            if (!ann_ok) c_ann <= 1'b0;
            if (!com_ok) c_com <= 1'b0;
            if (!non_ok) c_non <= 1'b0;
            sig <= c_can & c_ann & c_com & c_non
                 & can_ok & ann_ok & com_ok & non_ok;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A530FC;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd3, 6'd3, c_can, c_ann, c_com, c_non,
                                beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
