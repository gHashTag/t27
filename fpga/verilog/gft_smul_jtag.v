`default_nettype none
// W839: does 0 * x == 0? The MAJORITY form, on the die. LAYOUT v3, DESIGN 12.
//
// W836 normalised every function body in the corpus and found that `smul` has
// exactly TWO forms, not the fourteen a raw hash reported:
//
//     7c0755a0   19 specs   if (a==0) return 0; if (b==0) return 0;
//                           ... if (mag==0) return 0;   branch sign
//     8d3af2b6    2 specs   no guards at all;           XOR sign
//
// W838 read one spec of each pair on two boards within an hour and they gave
// opposite answers about zero. That was a by-product of testing two unrelated
// designs; THIS wrapper and its companion `gft_signed_mac_jtag.v` (DESIGN 13)
// ask the one question directly, with the same four clauses, on the smallest
// design that can carry it -- `gft_smul.on_comb(a,b) = smul(a,b)`, two ports and
// nothing else in the way.
//
// FORECAST REGISTERED BEFORE SYNTHESIS:
//     DESIGN 12 (this file, majority form)   c_zero comes back 1
//     DESIGN 13 (companion, minority form)   c_zero comes back 0
//
// The mechanism for the second is T596: with no guard, `smul(0, x)` reduces to
// `magmul(0, x&65535)`, and there `am = 0` gives `prod = 512*(512+bm)`,
// `q = 512+bm`, `mant = bm` -- the OTHER operand's mantissa, returned intact.
//
// CLAUSES:
//   1. ZERO   smul(0, live) == 0 and smul(live, 0) == 0. Both orders, because a
//             guard on only one argument would pass a one-sided test.
//   2. COMM   smul(live, TWO) == smul(TWO, live). Every place the operands enter
//             `magmul` is commutative -- (512+am)*(512+bm), ao+bo, sa^sb -- so
//             this must hold in BOTH forms, and it separates "the guard differs"
//             from "the whole multiply differs".
//   3. GOLD   the spec's own `test m1`: smul(1.0, 1.0) == 1.0 (20480).
//   4. IND    smul(live, ONE) is non-zero and equals live, so the multiplier is
//             on the die and not in yosys's constant folder (T534/T555).
//
// /8: `gft_sadd` alone measured 24.59 MHz at 1,335 LUT (T568) and a single
// multiply is the same order. CFGMCLK 70.77 MHz (T495) / 8 = 8.85 MHz, a period
// of 113.0 ns, DECLARED in the companion .xdc -- a divider alone tells the
// timing engine nothing (T541).
//
// WORD v3: {16'hA5A5, 4'd3, 6'd12, c_zero, c_comm, c_gold, c_ind, beat, ok}
module gft_smul_jtag #(parameter integer JTAG_CHAIN_N = 3);

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

    localparam [31:0] ONE = 32'd20480;   // 1.0
    localparam [31:0] TWO = 32'd20992;   // 2.0
    localparam [31:0] Z   = 32'd0;

    // Two independent sources (T555): coprime strides, unequal seeds.
    reg [23:0] pre   = 24'd0;
    reg        beat  = 1'b0;
    reg [31:0] live  = 32'd20480;
    reg [31:0] live2 = 32'd21504;
    always @(posedge slowclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin
            beat  <= ~beat;
            live  <= live  + 32'd1;
            live2 <= live2 + 32'd7;
        end
    end

    wire y_z1, y_z2, y_c1, y_c2, y_g, y_i;
    wire [31:0] r_z1, r_z2, r_c1, r_c2, r_gold, r_ind;

    // 1. ZERO, both operand orders
    GftSmul u_z1 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(Z), .b(live), .ready(y_z1), .result(r_z1));
    GftSmul u_z2 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(live2), .b(Z), .ready(y_z2), .result(r_z2));

    // 2. COMM
    GftSmul u_c1 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(live), .b(TWO), .ready(y_c1), .result(r_c1));
    GftSmul u_c2 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(TWO), .b(live), .ready(y_c2), .result(r_c2));

    // 3. GOLD: the spec's own test m1
    GftSmul u_g (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(ONE), .b(ONE), .ready(y_g), .result(r_gold));

    // 4. IND
    GftSmul u_i (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(live), .b(ONE), .ready(y_i), .result(r_ind));

    wire zero_ok = (r_z1 == 32'd0) && (r_z2 == 32'd0);
    wire comm_ok = (r_c1 == r_c2);
    wire gold_ok = (r_gold == ONE);
    wire ind_ok  = (r_ind != 32'd0) && (r_ind == live);

    reg sig = 1'b0;
    reg c_zero = 1'b1, c_comm = 1'b1, c_gold = 1'b1, c_ind = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!zero_ok) c_zero <= 1'b0;
            if (!comm_ok) c_comm <= 1'b0;
            if (!gold_ok) c_gold <= 1'b0;
            if (!ind_ok)  c_ind  <= 1'b0;
            sig <= c_zero & c_comm & c_gold & c_ind
                 & zero_ok & comm_ok & gold_ok & ind_ok;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5333C;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd3, 6'd12,
                                c_zero, c_comm, c_gold, c_ind,
                                beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
