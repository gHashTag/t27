`default_nettype none
// W818: ONE GRADIENT-DESCENT STEP on our die, checked by the algebra of learning.
//
// `specs/ternary/gft_train1.t27` is a single training update in signed GF-T
// float: predict `y = w*x`, take the error `e = y - t`, form the gradient
// `g = e*x`, and step the weight against it by `eta`. It is one of the 25 specs
// that could not be synthesised at all before W813 removed yosys's `share` pass
// (T527/T529).
//
// WHAT MAKES THIS WORTH PUTTING ON SILICON. The other wrappers check an
// arithmetic identity. This one checks that LEARNING BEHAVES LIKE LEARNING, and
// all three properties come from the spec's own tests:
//
//   1. FIXED POINT   at the optimum the update leaves the weight alone.
//                    `test optimum`: on_comb(20480, 20480, 20480, 19968) = 20480,
//                    i.e. w = 1.0 predicts t = 1.0 exactly, so there is no error,
//                    no gradient, and no step.
//   2. ASCENT        when the target exceeds the prediction the weight moves UP.
//                    `test ascend`: t = 20992 > 20480 gives 20736 > 20480.
//   3. MOVEMENT      a weight below the optimum moves, and moves toward it.
//                    `test learn`: w = 19968 becomes 20224, which is strictly
//                    between the start and the optimum 20480.
//
//   4. NON-TRIVIALITY the outputs of 1 and 2 DIFFER. Without this a module that
//                    returns its first argument satisfies 1 and 3 and looks like
//                    a fixed point everywhere -- the trap `tnf17_jtag.v` had at
//                    T473a and `gft_bitnet_neuron_jtag.v` at T534.
//
// THE ACTIVATION IS LIVE, and that is not decoration. W815-W817 measured that a
// wrapper whose probes are all constants folds its DUT away: `CARRY4 == 8` is
// this family's prescaler-and-reset floor and means no arithmetic reached the
// fabric at all (T534/T536). `t27c silicon` now refuses to load such a build
// (T539/T540). The learning-rate input is driven from the prescaler so the
// multiplier and the adder exist on the die rather than in Yosys's constant
// folder.
//
// WORD LAYOUT, AND IT CARRIES A VERSION (T535a).
//
//     {16'hA5A5, 4'd3, 6'd5, c_fix, c_asc, c_mov, c_non, beat, ok}
//
// Every earlier wrapper used `{28'hA5A5A5A, beat, ok}`, and W814 found the
// two indistinguishable when three dice held different builds -- two boards were
// decoded with the wrong layout and reported an arithmetically impossible result.
// Bits [11:8] are now a VERSION NIBBLE: **1** is this layout, and the old one
// reads **5** there, because 0xA5A5A5A shifts a `5` into that position. A reader
// can tell them apart from the word alone, which was the whole complaint.
//
// TNF values, per `specs/numeric/tnf17.t27`: 20480 = 1.0, 19968 and 20992 are its
// neighbours one offset step away, 20224 and 20736 lie between them.
module gft_train1_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // W818 (T541): THE DATAPATH CANNOT CLOSE TIMING AT THE RAW CLOCK.
    // nextpnr measured the combinational chain through one GftTrain1 --
    // multiply, subtract, multiply, step, all in TNF float -- at **7.53 MHz**,
    // against a 12 MHz default target, and CFGMCLK actually runs at 68.8 MHz on
    // these dice (T495). Every earlier wrapper was small enough that timing never
    // arose; this is the first that fails it.
    //
    // The design is genuinely MULTICYCLE: `live` advances once per 2^24 ticks and
    // nothing needs to settle within one raw period. A clock ENABLE would not
    // help -- static timing still analyses the full path at the source frequency.
    // So the whole wrapper runs on a divided clock through a BUFG, which gives
    // the combinational path 16 raw periods and makes the timing question honest
    // rather than suppressed.
        // W844: /32, NOT /16. Measured 4.82 MHz against the 4.42 MHz /16 declares --
    // a 1.09x margin, THINNER than the seed-to-seed spread W842 measured on a
    // single netlist (15.83 to 18.29 MHz, about 15%). A margin smaller than the
    // placer's own variance is not a margin; this one would fail on some seeds
    // and pass on others, which is indistinguishable from the T616 defect and
    // would have been misread as it.
    reg [4:0] div = 5'd0;
    always @(posedge cfgmclk) div <= div + 5'd1;
    wire slowclk;
    BUFG bufg_slow (.I(div[4]), .O(slowclk));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge slowclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    localparam [31:0] ONE      = 32'd20480;   // 1.0
    localparam [31:0] BELOW    = 32'd19968;   // one offset step below 1.0
    localparam [31:0] ABOVE    = 32'd20992;   // one offset step above 1.0
    localparam [31:0] LEARNED  = 32'd20224;   // `test learn`   expects this
    localparam [31:0] ASCENDED = 32'd20736;   // `test ascend`  expects this

    // The live drive: the prescaler advances it, so nothing below is constant.
    reg [23:0] pre  = 24'd0;
    reg        beat = 1'b0;
    reg [31:0] live = 32'd19968;              // starts at BELOW, walks upward
    always @(posedge slowclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin beat <= ~beat; live <= live + 32'd1; end
    end

    wire rdy_f, rdy_a, rdy_l, rdy_v;
    wire [31:0] r_fix, r_asc, r_learn, r_live;

    // 1. FIXED POINT: w = t = x = 1.0 -> the weight does not move
    // W984 (T839): every constant operand below used to reach the DUT as a
    // literal, so yosys evaluated the clause at compile time and the die read a
    // folded 1 -- a PASS in every build, including the failing ones (T836).
    // `Z0` is identically zero at runtime and opaque to the optimiser: two
    // counters with the same seed and step, whose equality no mapper will try to
    // prove. `K + Z0` is K on silicon and an unknown to `opt`.
    reg [31:0] opq_a = 32'd1;
    reg [31:0] opq_b = 32'd1;
    always @(posedge slowclk) begin
        opq_a <= opq_a + 32'd1;
        opq_b <= opq_b + 32'd1;
    end
    wire [31:0] Z0 = opq_a - opq_b;

    GftTrain1 u_fix   (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
                       .w(ONE + Z0),   .x(ONE + Z0), .t(ONE + Z0),   .eta(BELOW + Z0),
                       .ready(rdy_f), .result(r_fix));

    // 2. ASCENT: target above the prediction -> the weight rises
    GftTrain1 u_asc   (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
                       .w(ONE + Z0),   .x(ONE + Z0), .t(ABOVE + Z0), .eta(BELOW + Z0),
                       .ready(rdy_a), .result(r_asc));

    // 3. MOVEMENT: a weight below the optimum moves toward it
    GftTrain1 u_learn (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
                       .w(BELOW + Z0), .x(ONE + Z0), .t(ONE + Z0),   .eta(BELOW + Z0),
                       .ready(rdy_l), .result(r_learn));

    // The live instance: exists so the datapath cannot be folded away (T539).
    GftTrain1 u_live  (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
                       .w(live),  .x(ONE + Z0), .t(ONE + Z0),   .eta(BELOW + Z0),
                       .ready(rdy_v), .result(r_live));

    wire fix_ok = (r_fix   == ONE);
    wire asc_ok = (r_asc   == ASCENDED) && (r_asc > ONE);
    wire mov_ok = (r_learn == LEARNED)  && (r_learn > BELOW) && (r_learn < ONE);
    // Non-triviality: the fixed point and the ascent must DISAGREE, or the module
    // is returning its first argument and every clause above is vacuous.
    wire non_ok = (r_fix != r_asc) && (r_live != 32'd0);

    reg sig = 1'b0;
    reg c_fix = 1'b1, c_asc = 1'b1, c_mov = 1'b1, c_non = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!fix_ok) c_fix <= 1'b0;
            if (!asc_ok) c_asc <= 1'b0;
            if (!mov_ok) c_mov <= 1'b0;
            if (!non_ok) c_non <= 1'b0;
            sig <= c_fix & c_asc & c_mov & c_non
                 & fix_ok & asc_ok & mov_ok & non_ok;
        end
    end

    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5317C;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd3, 6'd5, c_fix, c_asc, c_mov, c_non,
                                beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
