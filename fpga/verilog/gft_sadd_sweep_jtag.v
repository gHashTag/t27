`default_nettype none
// W832: the absorption boundary swept across the whole band, on silicon.
// LAYOUT v2, DESIGN 2.
//
// W830 tested the boundary at TWO points and W831 found one of them carried a
// wrong expected value -- my Python model of `magadd` had dropped its
// round-to-nearest-even branch, and three readings of the spec agreed with each
// other because all three were mine (T575, lesson 1141). The corrected boundary
// is offset **11**, not 10.
//
// Two points verified a corrected model at the two places it was checked. This
// sweeps all twenty-one, so the boundary becomes MEASURED rather than computed
// by the model that already erred once.
//
// THE SWEEP. A counter walks `off` from 0 to 20; the probe is
// `sadd(off << 9, 511)` -- an operand at that offset with a zero mantissa,
// against the largest spurious term `smul(0, x)` can produce (T570: offset 0,
// mantissa 511). The predicate at each step is
//
//     off <= 10   ->  the result must DIFFER from the base   (the term survives)
//     off >= 11   ->  the result must EQUAL the base         (512 >> 11 == 0)
//
// and both halves are latched sticky-low, so a single disagreement anywhere in
// the band is visible at the end. Offset 0 is a special case and is included
// deliberately: `sadd` guards `if (a == 0) return b`, so it returns 511 by the
// guard rather than by the shift, and the predicate still holds.
//
// PREDICTED BY ICARUS BEFORE THIS FILE WAS BUILT, on the same generated RTL:
// every offset 0-10 MOVED, every offset 11-20 absorbed, first absorbed = 11.
// Icarus is a different tool from the Python model and therefore independent of
// it -- but it is still a simulation of the same RTL, so the die remains the
// only third party.
//
// CLAUSES:
//   1. LOW    every offset 0..10 moved
//   2. HIGH   every offset 11..20 was absorbed exactly
//   3. SWEEP  the counter actually reached 20 -- without this, a wrapper that
//             never advances satisfies 1 and 2 vacuously
//   4. IND    an independently-driven adder is live in the fabric (T534/T555)
//
// WORD v2: {16'hA5A5, 4'd2, 4'd2, c_low, c_high, c_swept, c_ind, 0, 1, beat, ok}
// Design 2 = sadd band sweep. Design 1 was the two-point probe (T572).
module gft_sadd_sweep_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // /4: `sadd` alone measured 24.59 MHz (T568). The ratio is declared in
    // gft_sadd_sweep_jtag.xdc -- a divider alone tells the timing engine nothing.
    reg [1:0] dv = 2'd0;
    always @(posedge cfgmclk) dv <= dv + 2'd1;
    wire slowclk;
    BUFG bufg_slow (.I(dv[1]), .O(slowclk));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge slowclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    localparam [31:0] SPUR     = 32'd511;   // offset 0, mantissa 511
    localparam [4:0]  LAST_OFF = 5'd20;
    localparam [4:0]  BOUNDARY = 5'd11;     // first absorbed offset (T575)

    // The sweep counter: one offset per prescaler tick, so each probe has ample
    // time to settle before it is judged.
    reg [23:0] pre  = 24'd0;
    reg        beat = 1'b0;
    reg [4:0]  off  = 5'd0;
    reg        swept = 1'b0;
    always @(posedge slowclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin
            beat <= ~beat;
            if (off == LAST_OFF) swept <= 1'b1;
            else                 off   <= off + 5'd1;
        end
    end

    wire [31:0] base = {19'd0, off, 9'd0};   // (off << 9), mantissa zero

    wire y_s, y_i;
    wire [31:0] r_sweep, r_ind;
    GftSadd u_sweep (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(base), .b(SPUR), .ready(y_s), .result(r_sweep));

    // Independence, not liveness (T555): two coprime strides, unequal seeds.
    reg [31:0] liveA = 32'd20480;
    reg [31:0] liveB = 32'd21504;
    always @(posedge slowclk) if (pre == 24'd0) begin
        liveA <= liveA + 32'd1;
        liveB <= liveB + 32'd7;
    end
    GftSadd u_ind (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(liveA), .b(liveB), .ready(y_i), .result(r_ind));

    wire in_low   = (off <  BOUNDARY);
    wire moved    = (r_sweep != base);
    wire absorbed = (r_sweep == base);

    reg sig = 1'b0;
    reg c_low = 1'b1, c_high = 1'b1, c_swept = 1'b0, c_ind = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (in_low  && !moved)    c_low  <= 1'b0;
            if (!in_low && !absorbed) c_high <= 1'b0;
            if (r_ind == 32'd0)       c_ind  <= 1'b0;
            c_swept <= swept;
            sig <= c_low & c_high & c_ind & swept;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A522F4;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd2, 4'd2,
                                c_low, c_high, c_swept, c_ind,
                                1'b0, 1'b1, beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
