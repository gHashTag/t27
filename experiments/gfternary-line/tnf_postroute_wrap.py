#!/usr/bin/env python3
"""Port-less harness for one TNF cost-sweep arm, so nextpnr can route it.

WHY A HARNESS AT ALL. The generated arms declare in_a/in_b/out_y as package
pins. nextpnr needs an XDC to constrain them and this repository has never had
the FGG676 pin map, so a ported design dies with
`Unable to constrain IO 'x', device does not have a pin named ''` (T163).
A design with NO port list needs no pin map at all.

WHY THE COMPARISON SURVIVES IT. The harness adds an LFSR, an accumulator and a
BSCANE2 register -- the SAME fixed overhead for every arm. Q1 and Q2 are both
questions about FIRST DIFFERENCES between arms, and a constant cancels in a
difference. Absolute post-route LUT for one arm is not reported.
"""
import sys

def wrap(top, width):
    hi = width - 1
    return f"""`default_nettype none
// Port-less post-route harness for {top}. Refs #1959
module {top}_h #(parameter integer JTAG_CHAIN_N = 3);
    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    reg [63:0] lfsr = 64'h0123456789ABCDEF;
    reg [63:0] lfs2 = 64'hFEDCBA9876543210;
    always @(posedge cfgmclk) begin
        lfsr <= {{lfsr[62:0], lfsr[63] ^ lfsr[62] ^ lfsr[60] ^ lfsr[59]}};
        lfs2 <= {{lfs2[62:0], lfs2[63] ^ lfs2[62] ^ lfs2[60] ^ lfs2[59]}};
    end

    reg  rst = 1'b1;
    reg [3:0] rc = 4'd0;
    always @(posedge cfgmclk) begin
        if (rc != 4'hF) begin rc <= rc + 4'd1; rst <= 1'b1; end
        else rst <= 1'b0;
    end

    wire [{hi}:0] out_y;
    wire out_valid, in_ready;
    {top} u (
        .clk(cfgmclk), .rst(rst), .in_valid(1'b1),
        .in_a(lfsr[{hi}:0]), .in_b(lfs2[{hi}:0]),
        .in_ready(in_ready), .out_valid(out_valid),
        .out_y(out_y), .out_ready(1'b1));

    reg [31:0] acc = 32'd0;
    always @(posedge cfgmclk) if (out_valid) acc <= acc + {{{{32-{width}{{1'b0}}}}, out_y}};

    reg [23:0] pre = 24'd0;
    reg beat = 1'b0, sig = 1'b0, frozen = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) beat <= ~beat;
        if (!frozen && pre == 24'hFFFF) begin sig <= ^acc; frozen <= 1'b1; end
    end

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5A5A4;
    always @(posedge drck)
        if (sel) begin
            if (capture)    sr <= {{28'hA5A5A5A, 1'b0, 1'b1, beat, sig}};
            else if (shift) sr <= {{tdi, sr[31:1]}};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
"""

if __name__ == "__main__":
    print(wrap(sys.argv[1], int(sys.argv[2])))
