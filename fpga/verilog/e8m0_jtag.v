`default_nettype none
// W797: E8M0 on our die, checked by a known-answer sweep read back over JTAG.
//
// `specs/numeric/e8m0.t27` defines the OCP Microscaling shared scale (T436) and
// its `on_comb` round-trips a code through decode and encode:
//
//     on_comb(x) = e8m0_encode(e8m0_exponent(x))   for finite x
//     on_comb(255) = 255                            (the NaN code passes through)
//
// so on the whole 8-bit code space `on_comb` is the IDENTITY, and out of range it
// saturates to the NaN code. That makes a known-answer test cheap to state and
// impossible to pass by accident: eight probes, each with its own expected word,
// and one bit read off the die.
//
// The eight probes are chosen to hit every boundary the spec names:
//   0    smallest finite code, 2^-127
//   1    one above it
//   126  just below unity
//   127  unity, the bias
//   128  just above unity
//   254  largest finite code, 2^127
//   255  the NaN code -- passes through unchanged
//   256  OUT OF RANGE -- e8m0_exponent(256) = 129 > 127, so the guard fires
//        and the answer must be the NaN code, not a wrapped one. This is the
//        probe that would catch a missing range check, and the spec's own
//        `encode_out_of_range_is_nan` test asserts the same thing in software.
//
// Structure copied from `ternary_link_jtag.v`: STARTUPE2 supplies the clock so
// the design needs no package pin, a reset counter releases the DUT after the
// clock is running, and BSCANE2 returns {A5A5A5A, 0, 1, beat, ok}.
module e8m0_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge cfgmclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    // probe index -> input code, and the word the spec says must come back
    reg  [2:0]  v = 3'd0;
    reg  [31:0] x;
    reg  [31:0] want;
    always @* begin
        case (v)
            3'd0: begin x = 32'd0;   want = 32'd0;   end
            3'd1: begin x = 32'd1;   want = 32'd1;   end
            3'd2: begin x = 32'd126; want = 32'd126; end
            3'd3: begin x = 32'd127; want = 32'd127; end
            3'd4: begin x = 32'd128; want = 32'd128; end
            3'd5: begin x = 32'd254; want = 32'd254; end
            3'd6: begin x = 32'd255; want = 32'd255; end
            3'd7: begin x = 32'd256; want = 32'd255; end  // out of range -> NaN
        endcase
    end

    wire        dut_ready;
    wire [31:0] got;
    E8M0 dut (
        .clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .x(x), .ready(dut_ready), .result(got));

    // AND every probe together; a single wrong word clears `sig` for good.
    reg swept = 1'b0;
    reg acc   = 1'b1;
    reg sig   = 1'b0;
    always @(posedge cfgmclk) begin
        if (!swept && rst_n) begin
            if (got != want) acc <= 1'b0;
            if (v == 3'd7) begin
                swept <= 1'b1;
                sig   <= acc & (got == want);   // fold the last probe in
            end
            v <= v + 3'd1;
        end
    end

    reg [23:0] pre = 24'd0;
    reg        beat = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) beat <= ~beat;
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A526F4;
    always @(posedge drck)
        if (sel) begin
            // W820 (T548), migrated W839: LAYOUT v2, DESIGN 6 -- the clause
            // bits ride in the word, and the design nibble names whose they are.
            // acc = every probe matched its expected word; swept = the sweep finished.
    // Two clauses and two PADDING bits. A padding bit is not a check; it is
    // written as a constant one so it can never mask a real failure.
            // Bits [11:8] are the VERSION NIBBLE: 1 is this layout, 5 was the
            // legacy 28-bit magic. W819 watched `t27c silicon` report PASS from
            // two boards carrying a different design, because a 28-bit magic
            // matches whatever follows it (T547).
            if (capture)    sr <= {16'hA5A5, 4'd2, 4'd6, acc, swept, 1'b1, 1'b1,
                                   1'b0, 1'b1, beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
