`default_nettype none
// W839: does 0 * x == 0? The MINORITY form, on the die. LAYOUT v3, DESIGN 13.
//
// The companion to `gft_smul_jtag.v` (DESIGN 12). Same question, same four
// clause slots, opposite `smul`: `gft_signed_mac` is one of the only two specs
// in the corpus whose multiply carries no zero guard and derives its sign by
// XOR rather than by branch (W836's `8d3af2b6`; the other is
// `gft_signed_dot4`, whose annihilation clause W838 measured false on board
// 1:4 -- 0xa5a5a1b4, clauses 1011).
//
//     fn smul(a, b) {
//         var sa = a >> 16; var sb = b >> 16;
//         var mag = magmul(a & 65535, b & 65535);
//         return ((sa ^ sb) << 16) | mag;      // no a==0, no b==0, no mag==0
//     }
//
// FORECAST REGISTERED BEFORE SYNTHESIS: c_zero comes back **0**, and c_comm
// comes back **1**.
//
// Both halves matter. T596 read `magmul` and found that with `am = 0` the
// product is `512*(512+bm)`, so `q = 512+bm` and `mant = bm` -- zero times a
// number returns that number's mantissa. But every place the operands enter is
// still a commutative operation, so the multiply remains exactly commutative.
// **The two forms differ in their zero, and in nothing else this wrapper can
// see.** A wrapper that only asked about zero could not say that.
//
// `on_comb(a1,b1,a2,b2) = sadd(smul(a1,b1), smul(a2,b2))`, so each clause is
// phrased as a two-term MAC:
//
//   1. ZERO   mac(0,live, 0,live2) == 0. Under the guarded form both products
//             vanish and the sum is zero; under this one both products return
//             mantissas and the sum cannot be.
//   2. COMM   mac(live,TWO, live2,ONE) == mac(TWO,live, ONE,live2). The addition
//             order is untouched, so this isolates the multiply.
//   3. CANCEL (+1*live) + (-1*live) == 0. Cancellation is the property the
//             arithmetic is FOR, and it does not depend on the zero guard --
//             which is why it belongs beside the clause that does.
//   4. IND    a live-driven MAC is non-zero, so the datapath is on the die and
//             not in yosys's constant folder (T534/T555).
//
// /16: two multiplies and an add in series. The four-term version of this chain
// measured 7.16 MHz (T551) and the two-term one cannot be slower, so CFGMCLK
// 70.77 MHz (T495) / 16 = 4.42 MHz, a period of 226.1 ns, DECLARED in the
// companion .xdc (T541).
//
// WORD v3: {16'hA5A5, 4'd3, 6'd13, c_zero, c_comm, c_cancel, c_ind, beat, ok}
module gft_signed_mac_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // W844: /32, NOT /16. Its sibling gft_signed_dot4 measured 4.73 MHz against
    // the 4.42 MHz /16 declares -- a 1.07x margin. This wrapper carries the same
    // DUT and derived its period the same way, so it is slowed with it rather
    // than left to be discovered separately (T625).
    reg [4:0] dv = 5'd0;
    always @(posedge cfgmclk) dv <= dv + 5'd1;
    wire slowclk;
    BUFG bufg_slow (.I(dv[4]), .O(slowclk));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge slowclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    localparam [31:0] ONE = 32'd20480;   // +1.0
    localparam [31:0] NEG = 32'd86016;   // -1.0  (20480 ^ 65536)
    localparam [31:0] TWO = 32'd20992;   // 2.0
    localparam [31:0] Z   = 32'd0;

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
    // W984 (T839): a constant operand reaches the DUT as a literal and the clause
    // is evaluated at compile time -- the die then reads a folded 1, a PASS in
    // every build including the failing ones (T836). `Z0` is identically zero at
    // runtime and opaque to the optimiser: two counters, same seed, same step.
    reg [31:0] opq_a = 32'd1;
    reg [31:0] opq_b = 32'd1;
    always @(posedge slowclk) begin
        opq_a <= opq_a + 32'd1;
        opq_b <= opq_b + 32'd1;
    end
    wire [31:0] Z0 = opq_a - opq_b;


    wire y_z, y_c1, y_c2, y_x, y_i;
    wire [31:0] r_zero, r_c1, r_c2, r_cancel, r_ind;

    // 1. ZERO
    GftSignedMac u_z (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(Z + Z0), .b1(live), .a2(Z + Z0), .b2(live2), .ready(y_z), .result(r_zero));

    // 2. COMM
    GftSignedMac u_c1 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(live), .b1(TWO + Z0), .a2(live2), .b2(ONE + Z0), .ready(y_c1), .result(r_c1));
    GftSignedMac u_c2 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(TWO + Z0), .b1(live), .a2(ONE + Z0), .b2(live2), .ready(y_c2), .result(r_c2));

    // 3. CANCEL
    GftSignedMac u_x (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(ONE + Z0), .b1(live), .a2(NEG + Z0), .b2(live), .ready(y_x), .result(r_cancel));

    // 4. IND
    GftSignedMac u_i (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a1(live), .b1(ONE + Z0), .a2(live2), .b2(ONE + Z0), .ready(y_i), .result(r_ind));

    wire zero_ok   = (r_zero   == 32'd0);
    wire comm_ok   = (r_c1     == r_c2);
    wire cancel_ok = (r_cancel == 32'd0);
    wire ind_ok    = (r_ind    != 32'd0);

    reg sig = 1'b0;
    reg c_zero = 1'b1, c_comm = 1'b1, c_cancel = 1'b1, c_ind = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!zero_ok)   c_zero   <= 1'b0;
            if (!comm_ok)   c_comm   <= 1'b0;
            if (!cancel_ok) c_cancel <= 1'b0;
            if (!ind_ok)    c_ind    <= 1'b0;
            sig <= c_zero & c_comm & c_cancel & c_ind
                 & zero_ok & comm_ok & cancel_ok & ind_ok;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5337C;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd3, 6'd13,
                                c_zero, c_comm, c_cancel, c_ind,
                                beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
