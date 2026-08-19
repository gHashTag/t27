`default_nettype none
// W830: the absorption boundary, tested on silicon. LAYOUT v2.
//
// W829 derived, by simulating `magadd` out of the spec source, that a spurious
// term at offset 0 with mantissa 511 -- the worst `smul(0,x)` can produce (T570) --
// moves an operand at offsets 0 through 9 and is absorbed exactly at offset 10
// and above (T571). The mechanism is one clamp:
//
//     d = ho - lo;  if (d > 11) { d = 11; }   and   512 >> 11 == 0
//
// **That is a simulation, not a measurement.** Every number in T570/T571 came
// from re-implementing the spec in Python, which is the same source the RTL is
// generated from and therefore shares any misreading I made of it. This file
// puts the boundary on three dice, where an independent tool chain has to agree.
//
// `specs/ternary/gft_sadd.t27` exposes exactly what is needed:
// `on_comb(a, b) = sadd(a, b)`, two ports, nothing else in the way.
//
// THE CLAUSES, with values computed BEFORE the file was written:
//
//   1. MOVE      sadd(2560, 511) == 2591.  2560 is offset 5, mantissa 0; the
//                spurious term is four offsets inside the clamp, so it survives
//                the shift and changes the result.
//   2. ABSORB    sadd(7680, 511) == 7680.  7680 is offset 15, ten offsets clear;
//                `512 >> 11` is zero and the term vanishes entirely.
//   3. GOLD      the spec's own `test a1`: sadd(1.0, 1.0) == 20992.
//   4. INDEPENDENT two independently-driven live inputs give a non-zero result,
//                so the adder is on the die and not in Yosys's constant folder
//                (T534/T539). Independence, not liveness -- deriving one source
//                from another dropped measured arithmetic by a quarter in W823.
//
// CLAUSES 1 AND 2 ARE THE POINT. Together they bracket the boundary: one says
// the defect reaches inside it, the other says it cannot reach outside. Either
// failing would refute the model, and they fail in opposite directions, so a
// module that simply returns its first argument fails clause 1 while passing
// clause 2 -- which is why both are here rather than only the absorbing one.
//
// WORD LAYOUT v3, and the version nibble is why:
//
//     {16'hA5A5, 4'd3, 6'd1, c_move, c_absorb, c_gold, c_ind, beat, ok}
//
// v1 carried a version but no DESIGN identity, and W828 read three dice showing
// `v=1, clauses=1111` where two of them held `ternary_node` rather than the
// design just programmed (T569/lesson 1134). Bits [11:8] are now the layout
// version (**2**) and bits [7:4] a DESIGN ID -- **1 = gft_sadd boundary probe**.
// A reader can tell not only which format a board speaks but which experiment it
// is running.
module gft_sadd_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // /4 only: `sadd` alone measures **24.59 MHz**, against 2.93 for the
    // perceptron (T568) and 7.16 for the four-term dot product (T551). One
    // operation deep instead of five, and 8.4x faster -- depth sets the period.
    // The ratio is DECLARED in gft_sadd_jtag.xdc; a divider alone tells the
    // timing engine nothing (T541).
    // W843: /8, NOT /4. The audit that T619a's three-seed rule made possible
    // measured this wrapper at 17.39-17.53 MHz against the 17.70 MHz that /4
    // declares -- a MISS, on two seeds of three, by about one percent. `sadd`
    // ALONE measures 24.59 MHz (T568); four instances of it in one wrapper do
    // not. The standing verdict here (T572/T575, the absorption boundary) was
    // built at essentially zero timing margin and the pipeline could not say so
    // until T603 made Fmax a measurement instead of a label.
    reg [2:0] dv = 3'd0;
    always @(posedge cfgmclk) dv <= dv + 3'd1;
    wire slowclk;
    BUFG bufg_slow (.I(dv[2]), .O(slowclk));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge slowclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    localparam [31:0] IN_BAND  = 32'd2560;   // offset 5,  mantissa 0
    // W831 (T574): 2592, not 2591. W829's Python model of `magadd` OMITTED the
    // round-to-nearest-even branch that runs when `s < 1024` --
    //     t = rem << 1; hf = 1 << d; if (t > hf) mant++;
    // -- and here rem = 31, t = 62, hf = 32, so the mantissa rounds up. The RTL,
    // Icarus and all three dice say 2592; three readings of the spec by me said
    // 2591 because they shared one omission. **The clause was right to fail.**
    localparam [31:0] MOVED    = 32'd2592;   // verified: RTL, Icarus, silicon
    localparam [31:0] OUT_BAND = 32'd7680;   // offset 15, mantissa 0
    localparam [31:0] SPUR     = 32'd511;    // offset 0,  mantissa 511
    localparam [31:0] ONE      = 32'd20480;  // 1.0
    localparam [31:0] TWO      = 32'd20992;  // 2.0, the spec's `test a1`

    // Two independent sources (T555): coprime strides, unequal seeds.
    reg [23:0] pre   = 24'd0;
    reg        beat  = 1'b0;
    reg [31:0] liveA = 32'd20480;
    reg [31:0] liveB = 32'd21504;
    always @(posedge slowclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) begin
            beat  <= ~beat;
            liveA <= liveA + 32'd1;
            liveB <= liveB + 32'd7;
        end
    end

    wire y_m, y_a, y_g, y_i;
    wire [31:0] r_move, r_abs, r_gold, r_ind;

    // 1. INSIDE the band: offset 5 against a term at offset 0 -- must move
    GftSadd u_move (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(IN_BAND), .b(SPUR), .ready(y_m), .result(r_move));

    // 2. OUTSIDE the band: offset 15 -- must be exactly unchanged
    GftSadd u_abs (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(OUT_BAND), .b(SPUR), .ready(y_a), .result(r_abs));

    // 3. GOLD: the spec's own test
    GftSadd u_gold (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(ONE), .b(ONE), .ready(y_g), .result(r_gold));

    // 4. INDEPENDENT: the adder must exist in the fabric
    GftSadd u_ind (.clk(slowclk), .rst_n(rst_n), .en(1'b1),
        .a(liveA), .b(liveB), .ready(y_i), .result(r_ind));

    wire move_ok = (r_move == MOVED) && (r_move != IN_BAND);
    wire abs_ok  = (r_abs  == OUT_BAND);
    wire gold_ok = (r_gold == TWO);
    wire ind_ok  = (r_ind  != 32'd0);

    reg sig = 1'b0;
    reg c_move = 1'b1, c_abs = 1'b1, c_gold = 1'b1, c_ind = 1'b1;
    reg [4:0] settle = 5'd0;
    always @(posedge slowclk) begin
        if (rst_n && settle != 5'h1F) settle <= settle + 5'd1;
        if (settle == 5'h1F) begin
            if (!move_ok) c_move <= 1'b0;
            if (!abs_ok)  c_abs  <= 1'b0;
            if (!gold_ok) c_gold <= 1'b0;
            if (!ind_ok)  c_ind  <= 1'b0;
            sig <= c_move & c_abs & c_gold & c_ind
                 & move_ok & abs_ok & gold_ok & ind_ok;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5307C;
    always @(posedge drck)
        if (sel) begin
            if (capture) sr <= {16'hA5A5, 4'd3, 6'd1,
                                c_move, c_abs, c_gold, c_ind,
                                beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
