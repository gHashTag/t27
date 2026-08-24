`default_nettype none
// LAYOUT v3, DESIGN 17 (migrated W841 from the legacy 24-bit magic).
// NOTE: this design's silicon verdict was WITHDRAWN in W814 -- it fails timing
// at the measured 70.77 MHz CFGMCLK and meets it only at 11.26 MHz. The wrapper
// is migrated so the bench speaks one language; the verdict stays withdrawn.
// W814: a four-tap BitNet neuron on our die, checked by ALGEBRA, no golden table.
//
// `specs/ternary/gft_bitnet_neuron.t27` computes
//     on_comb(w1,a1, w2,a2, w3,a3, w4,a4) = sum_i contrib(w_i, a_i)
// over signed GF-T activations with round-to-nearest-even, four ternary taps.
//
// THIS SPEC COULD NOT BE SYNTHESISED UNTIL W813. It is one of the 25 that
// `yosys`'s `share` pass -- SAT-based resource sharing -- never finished on
// (T527), because TNF float normalisation compiles to variable-amount shifts and
// `share` enumerates their control conditions without bound. Removing that one
// pass took the family from "never" to seconds (T529/T532), and this file is the
// first of them to reach silicon.
//
// THE WEIGHT ALPHABET IS NOT THE CANONICAL ONE, and getting this wrong would
// produce a wrapper that computes the NEGATION of the spec and still passes a
// trivial check. Read out of `contrib` in the spec itself:
//
//     w == 2  ->  +a                    (positive)
//     w == 0  ->  a ^ 65536             (negate: flip the sign bit)
//     w == 1  ->  0                     (zero)
//
// `specs/numeric/gfternary.t27` -- the canonical source -- says
// `GAT_ZERO=0, GAT_POS=1, GAT_NEG=2`. The whole `gft_*` family inverts it
// (T533). This wrapper follows the SPEC IT INSTANTIATES, and the disagreement is
// recorded rather than silently reconciled, because reconciling it would change
// what the hardware computes.
//
// THE CHECKS. No expected-value table is used; three algebraic properties the
// spec's own tests assert, and the third is the one that makes the first two
// mean anything:
//
//   1. CANCELLATION   (+,+,-,-) on four equal activations sums to zero.
//                     This is the spec's `test cancel`.
//   2. ANNIHILATION   all-zero weights give zero whatever the activations are.
//   3. NON-TRIVIALITY a single positive tap returns that activation unchanged,
//                     and it is NOT zero. Without this, a module that outputs 0
//                     for everything satisfies 1 and 2 perfectly -- the same trap
//                     `tnf17_jtag.v` had at T473a.
//
//   4. ANTISYMMETRY   a single negative tap returns the same activation with the
//                     sign bit flipped: TNF_ONE 20480 -> 86016.
//
// TNF_ONE = 20480 is offset 40, mantissa 0, per `specs/numeric/tnf17.t27`.
//
// Structure copied from `e8m0_jtag.v`: STARTUPE2 supplies the clock so no
// package pin is needed, a reset counter releases the DUT once the clock runs,
// and BSCANE2 returns {A5A5A5, c_can, c_ann, c_non, c_ant, 0, 1, beat, ok} --
// the magic is 24 bits here, and the four freed bits are the clause results.
module gft_bitnet_neuron_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge cfgmclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    localparam [31:0] TNF_ONE     = 32'd20480;   // +1.0
    localparam [31:0] TNF_MINUS_1 = 32'd86016;   // 20480 ^ 65536
    localparam [7:0]  W_POS = 8'd2;
    localparam [7:0]  W_ZER = 8'd1;
    localparam [7:0]  W_NEG = 8'd0;

    wire rdy_c, rdy_z, rdy_s, rdy_n;
    wire [31:0] r_cancel, r_zeroes, r_single, r_neg;

    // W999 (T839, T863a): every constant operand below reached the DUT as a
    // literal, so yosys evaluated the clause at compile time and the die read a
    // folded 1 -- a PASS in every build, including the failing ones (T836).
    // `Z0` is identically zero at runtime and opaque to the optimiser: two
    // counters with the same seed and step, whose equality no mapper will try to
    // prove. `K + Z0` is K on silicon and an unknown to `opt`.
    reg [31:0] opq_a = 32'd1;
    reg [31:0] opq_b = 32'd1;
    always @(posedge cfgmclk) begin
        opq_a <= opq_a + 32'd1;
        opq_b <= opq_b + 32'd1;
    end
    wire [31:0] Z0 = opq_a - opq_b;

    // 1. CANCELLATION: (+1,+1,-1,-1) on 1.0 each -> 0
    GftBitnetNeuron u_cancel (
        .clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .w1(W_POS + Z0), .a1(live),    .w2(W_POS + Z0), .a2(live),
        .w3(W_NEG + Z0), .a3(live),    .w4(W_NEG + Z0), .a4(live),
        .ready(rdy_c), .result(r_cancel));

    // 2. ANNIHILATION: every weight zero -> 0, whatever the activations
    GftBitnetNeuron u_zeroes (
        .clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .w1(W_ZER + Z0), .a1(TNF_ONE + Z0), .w2(W_ZER + Z0), .a2(32'd511 + Z0),
        .w3(W_ZER + Z0), .a3(32'd40960 + Z0), .w4(W_ZER + Z0), .a4(32'd65536 + Z0),
        .ready(rdy_z), .result(r_zeroes));

    // 3. NON-TRIVIALITY: one positive tap returns that activation, non-zero
    GftBitnetNeuron u_single (
        .clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .w1(W_POS + Z0), .a1(TNF_ONE + Z0), .w2(W_ZER + Z0), .a2(TNF_ONE + Z0),
        .w3(W_ZER + Z0), .a3(TNF_ONE + Z0), .w4(W_ZER + Z0), .a4(TNF_ONE + Z0),
        .ready(rdy_s), .result(r_single));

    // 4. ANTISYMMETRY: one negative tap returns the sign-flipped activation
    GftBitnetNeuron u_neg (
        .clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .w1(W_NEG + Z0), .a1(TNF_ONE + Z0), .w2(W_ZER + Z0), .a2(TNF_ONE + Z0),
        .w3(W_ZER + Z0), .a3(TNF_ONE + Z0), .w4(W_ZER + Z0), .a4(TNF_ONE + Z0),
        .ready(rdy_n), .result(r_neg));

    wire cancels   = (r_cancel == 32'd0);
    wire annihil   = (r_zeroes == 32'd0);
    wire nontriv   = (r_single == TNF_ONE) && (r_single != 32'd0);
    wire antisym   = (r_neg    == TNF_MINUS_1);

    // W814 (T535): ONE BIT CANNOT LOCALISE A FOUR-CLAUSE CONJUNCTION. The first
    // run of this wrapper returned magic with `ok=0` and `beat=1` -- the design
    // was alive on the die and something was false, and nothing on the wire said
    // what. That is the same defect this month found three times in the host
    // tooling (T500, T513, T523a): a reporting layer collapsing states the run
    // distinguishes. The BSCAN word has 28 spare bits; four of them are now the
    // clauses themselves, so a failure names itself on the first read.
    //
    // Each clause is LATCHED STICKY-LOW: once false it stays false, because
    // `live` moves and a clause that holds at one activation and fails at
    // another must be reported as failing.
    reg sig = 1'b0;
    reg c_can = 1'b1, c_ann = 1'b1, c_non = 1'b1, c_ant = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge cfgmclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!cancels) c_can <= 1'b0;
            if (!annihil) c_ann <= 1'b0;
            if (!nontriv) c_non <= 1'b0;
            if (!antisym) c_ant <= 1'b0;
            sig <= c_can & c_ann & c_non & c_ant
                 & cancels & annihil & nontriv & antisym;
        end
    end

    // W814 (T534): THE ACTIVATION MUST BE LIVE, or there is nothing on the die.
    // With every input a constant, Yosys evaluates the whole neuron at synthesis
    // time and the design folds to 43 LUT -- which is the wrapper's own
    // STARTUPE2 + counter + BSCAN overhead and nothing else. Driving ONE
    // activation from a counter takes it to 2,078 LUT, a factor of 48. The
    // constant version proves the toolchain carried a synthesis-time answer to
    // the die; only this version proves the datapath computes there.
    reg [31:0] live = 32'd20480;
    reg [23:0] pre = 24'd0;
    reg        beat = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin beat <= ~beat; live <= live + 32'd1; end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5347C;
    always @(posedge drck)
        if (sel) begin
            if (capture)    sr <= {16'hA5A5, 4'd3, 6'd17,
                                   c_can, c_ann, c_non, c_ant, beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
