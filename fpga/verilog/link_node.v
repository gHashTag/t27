`default_nettype none
// A ternary-link NODE: JTAG-writable input, JTAG-readable output. Refs #1959
//
// ROLE = 0  ENCODER.  Host writes a 3-bit word v; the node returns on_comb(v),
//                     the 4-bit ternary codeword (two 2-bit wire symbols).
// ROLE = 1  DECODER.  Host writes a 4-bit codeword; the node sweeps v = 0..7
//                     through ITS OWN ZeroDSP_TernaryLink instance and returns
//                     the v whose encoding matches, or `nomatch` if none does.
//
// WHY THE DECODER IS A SEARCH AND NOT A SECOND IMPLEMENTATION. Writing a
// decoder by hand would create a co-authored golden model, and Knight & Leveson
// (1986) is the reason this project does not trust those. Inverting the encoder
// with the encoder cannot disagree with it about the code -- only about whether
// an inverse exists. That is exactly the question worth asking on the wire:
// the delimiter (+1,+1) = 4'b0101 has NO preimage, so a decoder that searches
// must report `nomatch` for it. Sending the delimiter across is therefore a
// silicon test of the theorem, not a round trip.
//
// The BSCANE2 register is bidirectional: CAPTURE loads the reply, SHIFT walks
// it out while the new command walks in, UPDATE latches the command. One
// 32-bit DR pass is one request-and-response.
//
//   [31:4] magic 0xA5A5A5A   [3:0] payload
//
module link_node #(
    parameter integer JTAG_CHAIN_N = 3,
    parameter integer ROLE         = 0
);
    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge cfgmclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    // ---- the command latched from JTAG ----
    reg [3:0] cmd = 4'd0;

    // ---- the single shared encoder instance ----
    reg  [2:0] probe = 3'd0;
    wire [7:0] code;
    wire       dut_ready;
    wire [2:0] enc_in = (ROLE == 0) ? cmd[2:0] : probe;
    ZeroDSP_TernaryLink dut (
        .clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .v({5'b0, enc_in}), .ready(dut_ready), .result(code));

    reg [3:0] reply   = 4'd0;
    reg       nomatch = 1'b0;

    generate
    if (ROLE == 0) begin : g_enc
        always @(posedge cfgmclk) reply <= code[3:0];
    end else begin : g_dec
        // Exhaustive inverse: eight cycles, restarted whenever cmd changes.
        reg [3:0] last = 4'hF;
        reg       hit  = 1'b0;
        reg [2:0] probe_found = 3'd0;
        always @(posedge cfgmclk) begin
            if (cmd != last) begin
                last  <= cmd;
                probe <= 3'd0;
                hit   <= 1'b0;
                probe_found <= 3'd0;
            end else begin
                if (code[3:0] == cmd) begin
                    hit   <= 1'b1;
                    probe_found <= probe;
                end
                probe <= probe + 3'd1;
            end
            // W720: `nomatch` MUST reach the wire. Without it the reply 0 means
            // both "no preimage" and "recovered v = 0", and the delimiter test
            // cannot tell a theorem from a coincidence. Bit 3 is free -- v is
            // three bits -- so the reply is {nomatch, v}.
            reply   <= hit ? {1'b0, probe_found} : 4'b1000;
            nomatch <= ~hit;
        end
    end
    endgenerate

    reg [23:0] pre = 24'd0;
    reg        beat = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) beat <= ~beat;
    end

    // ---- bidirectional BSCANE2 ----
    wire drck, sel, shift, capture, update, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(update), .TDO(tdo));

    reg [31:0] sr = 32'hA5A5A5A0;
    always @(posedge drck)
        if (sel) begin
            if (capture)    sr <= {28'hA5A5A5A, reply};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    // UPDATE is in the TCK domain; a 2-flop sync is enough at these rates.
    reg u1 = 1'b0, u2 = 1'b0;
    always @(posedge cfgmclk) begin
        u1 <= update; u2 <= u1;
        if (u1 & ~u2) cmd <= sr[3:0];
    end
    assign tdo = sr[0];
endmodule
`default_nettype wire
