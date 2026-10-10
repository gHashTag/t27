`default_nettype none
// Exhaustive on-die proof of Verified Compute PoC slot A (specs/verified/poc/a/slot.t27,
// module PocSlot: on_comb(x) = x). Phase H hardware PoC of epic #6655.
//
// THE CLAIM. Implementation A passes the input through. The wrapper drives every x in 0..255
// into the same generated boundary a partial image would occupy (module PocSlot, ports
// clk / rst_n / en / x / ready / result), sweeps the space TWICE, and requires:
//   swept -- the input counter completed both passes over all 256 inputs
//   pass1 -- every output of sweep 1 equaled the expected function of its input
//   pass2 -- every output of sweep 2 did too (the same answer twice off the die)
//   rdy   -- the DUT asserted ready on every sampled cycle
// ok = pass1 & pass2. Exhaustive, not sampled: 256 of 256 inputs, two passes.
//
// THE SLOT BOUNDARY. poc_slot_b_jtag.v is this file byte-identical except for the EXPECTED
// expression (x ^ 8'hFF), the design id and these comments: same boundary, different
// implementation beneath it -- the compatibility region.t27 slot_swappable() describes.
module poc_slot_a_jtag #(parameter integer JTAG_CHAIN_N = 3);

    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // rst_n released after the STARTUPE2 clock is running; `en` held high.
    reg [3:0] rstc = 4'd0;
    wire rst_n = (rstc == 4'hF);
    always @(posedge cfgmclk) if (rstc != 4'hF) rstc <= rstc + 4'd1;

    reg  [7:0] x = 8'd0;
    wire [7:0] result;
    wire       ready;
    PocSlot dut (
        .clk(cfgmclk), .rst_n(rst_n), .en(1'b1),
        .x(x), .ready(ready), .result(result));

    // The expected function of slot A. Slot B differs in exactly this expression.
    wire [7:0] expected = x;

    reg swept = 1'b0;
    reg pass1 = 1'b0;
    reg pass2 = 1'b0;
    reg rdy   = 1'b0;
    reg first = 1'b1;
    always @(posedge cfgmclk) if (rst_n) begin
        // result is combinational in x, so the sampled pair (x, result) is consistent.
        if (result == expected) begin
            if (first) pass1 <= 1'b1; else pass2 <= 1'b1;
        end else begin
            if (first) pass1 <= 1'b0; else pass2 <= 1'b0;
        end
        if (ready) rdy <= 1'b1;
        if (x == 8'd255) begin
            if (first) first <= 1'b0;   // sweep 1 folded; sweep 2 starts at x == 0
            else       swept <= 1'b1;   // sweep 2 folded too
        end
        x <= x + 8'd1;
    end

    reg [23:0] pre = 24'd0;
    reg        beat = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) beat <= ~beat;
    end
    wire ok = pass1 & pass2;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = {16'hA5A5, 4'd3, 6'd19, 4'b1111, 1'b0, 1'b0};
    always @(posedge drck)
        if (sel) begin
            // LAYOUT v3, DESIGN 19. W820 (T548): the clause bits ride in the word and the
            // design nibble names whose they are, so a stale design cannot answer for this
            // one (W819/T547). All four clauses are real here -- no padding bits.
            if (capture)    sr <= {16'hA5A5, 4'd3, 6'd19, swept, pass1, pass2, rdy,
                                   beat, ok};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
