`default_nettype none
// W839: is `gft_signed_dot4`'s commutativity failure ARITHMETIC or a RACE?
// LAYOUT v3, DESIGN 11.
//
// W838 read `gft_signed_dot4` three times on board 1:4. The first load returned
// clauses 1001 -- c_com = 0 -- and the next two returned 1011. The clause bits
// latch sticky-low, so 1001 means commutativity was false at least once during
// that load, and 1011 means it never was during the others. One intermittent
// across three loads is not a defect and not a clean bill; it is an open
// question, and W838 recorded it as one.
//
// THE ARITHMETIC CANNOT BE THE CAUSE, and this is provable by reading `magmul`:
//
//     prod = (512 + am) * (512 + bm);    <- symmetric in am, bm
//     s    = ao + bo + carry;            <- symmetric in ao, bo
//     sign = sa ^ sb;                    <- symmetric in sa, sb
//
// Every place the two operands enter is a commutative operation, so
// `smul(a,b) == smul(b,a)` identically, for every input, with no exceptions and
// no dependence on the zero guard that distinguishes this spec's `smul` from the
// other nineteen. FORECAST REGISTERED BEFORE SYNTHESIS:
//
//     c_imm      MAY come back 0
//     c_settled  MUST come back 1
//
// -- because if the mismatch is real arithmetic it survives any settling delay,
// and if it is a race it does not. This is the discriminator W838 lacked: that
// wave could see THAT the clause fell, never WHY, and a wrapper that only says
// "it fell" cannot be run again to learn more.
//
// HOW THE TWO SAMPLES DIFFER. `k` advances once every 256 slowclk ticks (~58 us
// at 4.42 MHz), so the full 65,536-step sweep finishes in about 3.8 seconds --
// comfortably inside the load-then-read window, which matters because `ok`
// requires `swept` and a sweep still running would answer 0 vacuously (T572).
//   c_imm      compares r_ab against r_ba on EVERY slowclk edge, including the
//              edge on which the operands change. If the two structurally
//              different netlists settle at different times, this catches it.
//   c_settled  compares only when at least 8 edges have passed since `k` moved.
//              Eight periods is ~1.8 us against a 226 ns period -- far beyond any
//              combinational settling, and still 32x shorter than one step, so
//              248 of every 256 edges are sampled settled.
//
// WHAT THE FOUR OUTCOMES MEAN:
//   1 1   commutativity held everywhere in this sweep
//   0 1   A RACE. The arithmetic is symmetric; the netlists are not synchronised.
//   0 0   a genuine input pattern -- which would REFUTE the symmetry argument
//         above, and that is exactly what makes this worth building.
//   1 0   impossible by construction (settled samples are a subset of immediate
//         ones); if it ever appears, the wrapper is wrong, not the die.
//
// THE SWEEP IS AN OPERAND SWEEP, NOT A COUNTER. `opA` and `opB` walk offsets
// 35..50 and all 512 mantissas with independent signs, and their mantissas are
// bit-shuffled relative to each other so the pair is not correlated. Each of the
// four product positions holds the SAME product in both instances with only the
// operand order flipped, so the ADDITION order is identical between u_ab and
// u_ba -- this isolates multiplication, which is the claim under test. Mixing in
// a live operand keeps yosys from folding either instance away (T534/T555).
//
// CLAUSES:
//   1. IMM      commutativity held on every edge
//   2. SETTLED  commutativity held on every edge >= 8 after an operand change
//   3. SWEPT    `k` actually reached the end of the band -- without this a
//               wrapper that never advances satisfies 1 and 2 vacuously (T572)
//   4. IND      the compared instances produce a non-zero result, so neither
//               was folded to a constant (T534)
//
// WORD v3: {16'hA5A5, 4'd3, 6'd11, c_imm, c_settled, c_swept, c_ind, beat, ok}
module gft_dot4_comm_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // /16, the same divider gft_signed_dot4_jtag.v runs at, and for the same
    // measured reason: the four-term dot product's critical path is 7.16 MHz
    // (T551) against a 70.77 MHz CFGMCLK. The ratio is DECLARED in the companion
    // .xdc -- a divider alone tells the timing engine nothing (T541).
    reg [3:0] dv = 4'd0;
    always @(posedge cfgmclk) dv <= dv + 4'd1;
    wire slowclk;
    BUFG bufg_slow (.I(dv[3]), .O(slowclk));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge slowclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    // ---- the sweep ----
    reg [7:0]  pre   = 8'd0;
    reg [15:0] k     = 16'd0;
    reg        swept = 1'b0;
    reg        beat  = 1'b0;
    reg [3:0]  age   = 4'd0;      // slowclk edges since `k` last moved
    always @(posedge slowclk) begin
        pre <= pre + 8'd1;
        if (pre == 8'd0) begin
            age  <= 4'd0;
            beat <= ~beat;
            if (k == 16'hFFFF) swept <= 1'b1;
            else               k     <= k + 16'd1;
        end else if (age != 4'hF) begin
            age <= age + 4'd1;
        end
    end

    // Two operands from one counter, decorrelated: different offset fields,
    // different sign bits, and B's mantissa is a rotation of A's bit range.
    wire        sgnA = k[15];
    wire [6:0]  offA = 7'd35 + {3'd0, k[13:10]};
    wire [8:0]  manA = k[8:0];
    wire [31:0] opA  = {15'd0, sgnA, offA, manA};

    wire        sgnB = k[14];
    wire [6:0]  offB = 7'd35 + {3'd0, k[9:6]};
    wire [8:0]  manB = {k[5:0], k[8:6]};
    wire [31:0] opB  = {15'd0, sgnB, offB, manB};

    // A live operand so neither instance can be folded to a constant (T534).
    reg [31:0] live = 32'd20480;
    always @(posedge slowclk) if (pre == 8'd0) live <= live + 32'd1;

    wire y_ab, y_ba;
    wire [31:0] r_ab, r_ba;

    // Same four products, same addition order; each pair's operands flipped.
    GftSignedDot4 u_ab (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(opA),  .b1(opB),  .a2(opB), .b2(opA),
        .a3(live), .b3(opA),  .a4(opB), .b4(live),
        .ready(y_ab), .result(r_ab));
    GftSignedDot4 u_ba (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(opB),  .b1(opA),  .a2(opA), .b2(opB),
        .a3(opA),  .b3(live), .a4(live), .b4(opB),
        .ready(y_ba), .result(r_ba));

    // NO third instance. W823's independence rule (T555) exists to stop yosys
    // folding a DUT whose operands are all constants -- but `live` already drives
    // u_ab and u_ba directly, so both are unfoldable, and a third copy of a
    // 12.7k-LUT dot product would buy nothing and cost a third of the die and of
    // the place-and-route. c_ind therefore asserts that the COMPARED instances
    // produce a non-zero result, which is the property the clause was ever for.

    wire comm = (r_ab == r_ba);

    reg sig = 1'b0;
    reg c_imm = 1'b1, c_settled = 1'b1, c_swept = 1'b0, c_ind = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!comm)                      c_imm     <= 1'b0;
            if (!comm && age >= 4'd8)       c_settled <= 1'b0;
            if (r_ab == 32'd0)              c_ind     <= 1'b0;
            c_swept <= swept;
            sig <= c_imm & c_settled & c_ind & swept;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A532FC;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd3, 6'd11,
                                c_imm, c_settled, c_swept, c_ind,
                                beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
