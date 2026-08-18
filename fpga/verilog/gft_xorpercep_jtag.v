`default_nettype none
// W828: a XOR perceptron's weight update on our die. LAYOUT v2, DESIGN 4.
//
// `specs/ternary/gft_xorpercep.t27` is one training step of a two-hidden-unit
// perceptron in signed GF-T float: form `s = x0 + x1`, take `h0 = relu(s)` and
// `h1 = relu(s - 1)`, predict from `z = v0*h0 + v1*h1`, and step both weights
// against the error. `on_comb` returns `(v0' << 32) | v1'`.
//
// THE CLAUSE THAT IS WORTH BUILDING THIS FOR. The update rule is
//
//     v0' = v0 - smul(eta, smul(g, h0))
//
// so whether "do not learn" is expressible in this numeric line depends entirely
// on whether `smul(0, x)` is zero -- and the corpus does not agree with itself
// about that (W836: two normalised forms, nineteen specs guard zero, two do not).
//
// FORECAST AS ORIGINALLY WRITTEN HERE: c_eta0 comes back ZERO. It was derived
// from T552, which measured `gft_signed_dot4` -- the OTHER form. See below.
//
// W838 -- REFUTED ON THREE DICE. c_eta0 came back ONE. The paragraph above
// derives its forecast from T552, and T552 measured `gft_signed_dot4`, whose
// `smul` is the MINORITY form with no zero guard (W836: two normalised forms
// across the corpus, 8d3af2b6 in two specs, 7c0755a0 in nineteen). THIS spec
// carries the majority form:
//     fn smul(a,b) { if (a==0) return 0; if (b==0) return 0; ... if (mag==0) return 0; }
// so `smul(eta=0, x)` is exactly zero here and a zero learning rate DOES leave
// the weights alone. Icarus predicted 1111 over 64 cycles before the build; the
// die read 0xa5a5a1f7 -- clauses 1111, ok=1.
//
// The error was checking one file's claim against another file's arithmetic --
// the fourth instance of it in this project. The two specs genuinely disagree
// about whether 0*x = 0, and W838 demonstrated both answers on the same three
// boards within minutes of each other.
//
// THE FOUR CLAUSES:
//   1. GOLD      the spec's own `test upd`: on_comb(0,0, 1.0,0, 1.0, 0.25)
//                = (19456 << 32) | 0, i.e. v0 moves to 0.25 and v1 stays put.
//                A golden value is legitimate here because it is the SPEC's, not
//                one I derived -- the same standing `e8m0_jtag.v` uses.
//   2. ETA-ZERO  eta = 0 must leave (v0, v1) unchanged. MEASURED W838: HOLDS.
//   3. NON-TRIVIAL the gold case must actually MOVE v0 -- without this, a module
//                returning its inputs satisfies clause 2 and looks correct.
//   4. INDEPENDENT a live, independently-driven input yields a non-zero result,
//                so the datapath is on the die rather than in Yosys's folder.
//
// INDEPENDENCE, NOT LIVENESS (T555). Four sources with no provable relationship:
// counters at strides 1, 3 and 7 from unequal seeds plus a 32-bit LFSR. Deriving
// one from another -- `x ^ 65536` -- let Yosys share almost everything and DROPPED
// the measured arithmetic from 1.96 to 1.53 DUT-equivalents in W823.
//
// TNF constants per `specs/numeric/tnf17.t27`: 20480 = 1.0, 19456 = 0.25.
module gft_xorpercep_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // The datapath is a float chain; W818/W821 both measured this family at
    // 7-11 MHz against a 70.77 MHz CFGMCLK, so the wrapper runs on /16 and the
    // ratio is DECLARED in gft_xorpercep_jtag.xdc -- a divider alone tells the
    // timing engine nothing (T541).
    // /32, not /16: this chain is relu -> multiply -> add -> multiply -> add in
    // series and measures **2.93 MHz**, against 7.16 for the four-term dot product
    // (T551). Depth, not width, sets the period here -- 10,893 LUT is smaller than
    // gft_signed_dot4's 12,724 and yet 2.4x slower.
    reg [4:0] dv = 5'd0;
    always @(posedge cfgmclk) dv <= dv + 5'd1;
    wire slowclk;
    BUFG bufg_slow (.I(dv[4]), .O(slowclk));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge slowclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    localparam [31:0] ONE  = 32'd20480;   // 1.0
    localparam [31:0] QTR  = 32'd19456;   // 0.25
    localparam [31:0] Z    = 32'd0;
    localparam [63:0] GOLD = 64'd83562883710976;   // (19456 << 32) | 0

    // Four independent sources (T555).
    reg [23:0] pre   = 24'd0;
    reg        beat  = 1'b0;
    reg [31:0] liveA = 32'd20480;
    reg [31:0] liveB = 32'd20736;
    reg [31:0] liveC = 32'd21504;
    reg [31:0] lfsr  = 32'h1ACE5EED;
    always @(posedge slowclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin
            beat  <= ~beat;
            liveA <= liveA + 32'd1;
            liveB <= liveB - 32'd3;
            liveC <= liveC + 32'd7;
            lfsr  <= {lfsr[30:0], lfsr[31]^lfsr[21]^lfsr[1]^lfsr[0]};
        end
    end

    wire y_g, y_e, y_i;
    wire [63:0] r_gold, r_eta0, r_ind;

    // 1. GOLD: the spec's own test vector
    GftXorPercep u_gold (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .v0(Z), .v1(Z), .x0(ONE), .x1(Z), .y(ONE), .eta(QTR),
        .ready(y_g), .result(r_gold));

    // 2. ETA-ZERO: a zero learning rate must not move the weights.
    //    W838 on three dice: it HOLDS -- this spec's smul guards zero.
    GftXorPercep u_eta0 (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .v0(ONE), .v1(QTR), .x0(ONE), .x1(Z), .y(ONE), .eta(Z),
        .ready(y_e), .result(r_eta0));

    // 4. INDEPENDENT: every port from a different source
    GftXorPercep u_ind (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .v0(liveA), .v1(liveB), .x0(liveC), .x1(lfsr), .y(liveA), .eta(liveB),
        .ready(y_i), .result(r_ind));

    wire gold_ok = (r_gold == GOLD);
    // unchanged means v0' == v0 and v1' == v1, i.e. the packed word is the input pair
    wire eta0_ok = (r_eta0 == {ONE, QTR});
    wire non_ok  = (r_gold[63:32] != 32'd0);          // v0 actually moved off zero
    wire ind_ok  = (r_ind != 64'd0);

    reg sig = 1'b0;
    reg c_gold = 1'b1, c_eta0 = 1'b1, c_non = 1'b1, c_ind = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!gold_ok) c_gold <= 1'b0;
            if (!eta0_ok) c_eta0 <= 1'b0;
            if (!non_ok)  c_non  <= 1'b0;
            if (!ind_ok)  c_ind  <= 1'b0;
            sig <= c_gold & c_eta0 & c_non & c_ind
                 & gold_ok & eta0_ok & non_ok & ind_ok;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A524F4;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd2, 4'd4, c_gold, c_eta0, c_non, c_ind,
                                1'b0, 1'b1, beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
