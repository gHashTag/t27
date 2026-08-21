`default_nettype none
// W840: DESIGN 15 PLUS FOLDABLE INSTANCES. LAYOUT v3, DESIGN 0.
//
// NUMBERED 0, NOT 16, AND THAT IS A FINDING. The v2 design field is FOUR BITS.
// This file was written as `4'd16`, Verilog truncated it to 0 without a word,
// and the die answered as design 0 while the service looked for 16 -- which it
// correctly refused to resolve rather than reporting a neighbour's PASS. Ids
// 1-15 were already spent, so 0 is the last one v2 has. **The next wrapper this
// bench adds has nowhere to go, and the word format needs a wider field before
// it gets one.**
//
// Two hypotheses died this wave, each to one cheap build:
//   "instance-vs-instance comparisons fail"  -- design 14 passes c_comm on two
//                                               dice with a comparison textually
//                                               identical to design 12's
//   "a second counter is the differentiator" -- design 15 adds one and passes
//
// What is left dividing the sample is what the OTHER instances in the wrapper
// look like. Design 12 holds three that fold to constants -- smul(0,live) and
// smul(live2,0) vanish under the zero guard, smul(1.0,1.0) has no live operand
// at all -- and reports 1.57 DUT-equivalents of surviving arithmetic. Designs 14
// and 15 hold none and report 2.21 and 2.31.
//
// This file is design 15 with two foldable instances added and nothing else
// changed. FORECAST REGISTERED BEFORE SYNTHESIS: c_comm comes back 0, and
// DUT-equivalents drops toward design 12's 1.57.
//
// If both happen, the finding is that FOLDABLE NEIGHBOURS CORRUPT THEIR
// NON-FOLDABLE SIBLINGS, which is a mechanism a netlist diff can chase in 800
// LUT. If c_comm holds, the third hypothesis dies too and what remains is that
// design 12 differs from design 14 in some way I have still not named -- which
// is worth knowing before another wave builds on either.
//
// Design 14 passed all four clauses on two dice. Designs 12 and 13 failed
// `c_comm` on two dice each -- with a comparison that is TEXTUALLY IDENTICAL to
// design 14's. So the failure is a property of the surrounding wrapper, not of
// the comparison, and the question is which property.
//
// Sharing does not divide the sample: 1.57 DUT-equivalents (12, fails), 3.10
// (13, fails), 2.21 (14, passes) is not monotone. One thing does divide it
// without exception:
//
//     designs 12 and 13 drive TWO counters.  design 14 drives ONE.
//
// This file is design 14 with a second counter added and NOTHING else changed --
// same instances, same comparisons, same clauses, same divider, same XDC. It is
// the smallest edit that can carry the hypothesis.
//
// FORECAST REGISTERED BEFORE SYNTHESIS: c_comm comes back 0.
//
// If it does, the second counter is the differentiator and the next wave has a
// mechanism to chase in one 800-LUT netlist instead of six thousand. If it does
// not, the counter hypothesis dies here and the sample is back to being divided
// by something I have not named -- which is worth one 40-second build to learn.
//
// (The original design-14 header follows, since every clause is unchanged.)
//
// W840: do two IDENTICAL instances of one function agree on the die?
// ORIGINALLY LAYOUT v3, DESIGN 14.
//
// W839 measured, on three designs and two arithmetic forms, that clauses
// comparing one DUT instance against another fail on silicon while clauses
// comparing against a constant hold -- with the arithmetic excluded by proof and
// by Icarus, and timing excluded by a measured 3.7x margin (T604/T605a). The
// mechanism was not identified and this file exists to narrow it by one step.
//
// THE READING W839 PUBLISHED HAS A COMPETITOR, AND I OWE IT A TEST. Both failing
// clauses in design 12 needed `live` to hold its seeded value; both passing ones
// did not -- `c_zero` returns 0 for any `live` because the guard fires, and
// `c_gold` has no `live` at all. So "instance vs constant" and "depends on the
// counter's value" fit the same four bits. Only one thing separates them:
//
//     `smul` is exactly commutative for EVERY input (T605: the operands enter
//     only through (512+am)*(512+bm), ao+bo and sa^sb). So a wrong `live` cannot
//     make c_comm false -- both instances would be wrong in the same way.
//
// That argument rescues the instance reading for c_comm and leaves c_ind fitting
// either story. Hence the control this bench has never had:
//
//   u_self_a and u_self_b are the SAME function with the SAME operand order.
//   Nothing distinguishes them but their existence as two instances.
//
// FORECASTS REGISTERED BEFORE SYNTHESIS:
//   c_init   1   -- register INIT survives the flow. Evidence: design 12's
//                   c_zero and c_gold are sticky registers seeded 1'b1 with no
//                   path that ever sets them, and both read back 1.
//   c_self   1   -- two identical instances must agree, or the flow duplicates
//                   incorrectly, which would be a far larger finding than the
//                   one being chased.
//   c_comm   0   -- reproducing W839 on the smallest design that can carry it.
//
// WHAT EACH OUTCOME MEANS:
//   c_self=1, c_comm=0  the divergence is ORDER-DEPENDENT: swapping operands
//                       produces a netlist that is not equivalent. Next step is
//                       a netlist diff of u_comm_a against u_comm_b.
//   c_self=0            the flow miscompiles DUPLICATION itself, independent of
//                       operands. Every multi-instance verdict on this bench is
//                       void, including W838's and W832's sweeps.
//   c_self=1, c_comm=1  W839's failures were not about instances at all, and the
//                       `live`-value reading wins. c_init then says whether the
//                       counter's seed is why.
//
// c_init is a register written NOWHERE. If the openXC7 flow drops FDRE INIT
// values, it reads back zero and the whole `live`-value story gains its
// mechanism; if it reads back its seed, that story loses it. Either way the
// answer is worth one clause, and no build so far has asked.
//
// /8: six GftSmul instances. A single multiply measured Fmax 32.87/47.37 MHz
// through the stage repaired in T603, against 8.85 MHz declared -- so the margin
// here is known rather than assumed, for the first time on this bench.
//
// WORD v3: {16'hA5A5, 4'd3, 6'd16, c_init, c_self, c_comm, c_ind, beat, ok}
module gft_dup3_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg [2:0] dv = 3'd0;
    always @(posedge cfgmclk) dv <= dv + 3'd1;
    wire slowclk;
    BUFG bufg_slow (.I(dv[2]), .O(slowclk));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge slowclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    localparam [31:0] TWO = 32'd20992;   // 2.0
    localparam [31:0] ONE = 32'd20480;   // 1.0
    localparam [31:0] Z   = 32'd0;

    // ---- the INIT probe: seeded, and written by nothing ----
    // 0x5A5A1234 is not the shift register's magic and not any TNF constant, so
    // a match cannot come from a neighbouring net being read by mistake.
    reg [31:0] initprobe = 32'h5A5A1234;
    always @(posedge slowclk) initprobe <= {initprobe[30:0], initprobe[31]};

    reg [23:0] pre  = 24'd0;
    reg        beat = 1'b0;
    reg [31:0] live = 32'd20480;
    // THE ONLY CHANGE FROM DESIGN 14: a second counter, seeded and strided the
    // way designs 12 and 13 seed and stride theirs.
    reg [31:0] live2 = 32'd21504;
    always @(posedge slowclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin
            beat  <= ~beat;
            live  <= live  + 32'd1;
            live2 <= live2 + 32'd7;
        end
    end

    // W984 (T839): a constant operand reaches the DUT as a literal and the clause

    // is evaluated at compile time -- the die then reads a folded 1, a PASS in

    // every build including the failing ones (T836). `Z0` is identically zero at

    // runtime and opaque to the optimiser: two counters, same seed, same step,

    // whose equality no mapper will try to prove.

    reg [31:0] opq_a = 32'd1;

    reg [31:0] opq_b = 32'd1;

    always @(posedge slowclk) begin

        opq_a <= opq_a + 32'd1;

        opq_b <= opq_b + 32'd1;

    end

    wire [31:0] Z0 = opq_a - opq_b;


    wire y1, y2, y3, y4, y5, y6, y7;
    wire [31:0] r_self_a, r_self_b, r_comm_a, r_comm_b, r_ind, r_fold1, r_fold2;

    // ---- THE CONTROL: identical function, identical operand order ----
    GftSmul u_self_a (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(live), .b(TWO + Z0), .ready(y1), .result(r_self_a));
    GftSmul u_self_b (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(live + Z0), .b(TWO + Z0), .ready(y2), .result(r_self_b));

    // ---- THE TEST: identical function, operands swapped ----
    GftSmul u_comm_a (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(live), .b(TWO + Z0), .ready(y3), .result(r_comm_a));
    GftSmul u_comm_b (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(TWO + Z0), .b(live), .ready(y4), .result(r_comm_b));

    // ---- liveness, so nothing above can be a folded constant (T534) ----
    GftSmul u_ind (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(live), .b(live2), .ready(y5), .result(r_ind));

    // THE ONLY CHANGE FROM DESIGN 15: two instances yosys can fold away. Their
    // results are folded into c_ind so they cannot be deleted outright.
    GftSmul u_fold1 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(Z), .b(live), .ready(y6), .result(r_fold1));
    GftSmul u_fold2 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(ONE + Z0), .b(ONE + Z0), .ready(y7), .result(r_fold2));

    wire init_ok = (initprobe != 32'd0);   // rotation-invariant, not foldable
    wire self_ok = (r_self_a == r_self_b);
    wire comm_ok = (r_comm_a == r_comm_b);
    wire ind_ok  = (r_ind != 32'd0) && (r_fold1 == 32'd0) && (r_fold2 == ONE);

    reg sig = 1'b0;
    reg c_init = 1'b1, c_self = 1'b1, c_comm = 1'b1, c_ind = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!init_ok) c_init <= 1'b0;
            if (!self_ok) c_self <= 1'b0;
            if (!comm_ok) c_comm <= 1'b0;
            if (!ind_ok)  c_ind  <= 1'b0;
            sig <= c_init & c_self & c_comm & c_ind
                 & init_ok & self_ok & comm_ok & ind_ok;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5343C;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd3, 6'd16,
                                c_init, c_self, c_comm, c_ind,
                                beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
