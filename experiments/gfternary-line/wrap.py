#!/usr/bin/env python3
"""Wrap one GFTernary rung in the port-less BSCANE2 harness (W690/T172).

No package pins: the design has no port list, so nextpnr needs no XDC and the
FGG676 pinout -- which this repository has never had -- is not required.

The layer must not be optimisable away, so it is DRIVEN by an LFSR with state
and OBSERVED through an accumulator whose parity reaches the JTAG register.
A constant-folded layer would report a LUT count that measures nothing.
"""
import sys

def wrap(arm, m, accw):
    outs = []
    conn = []
    paired = arm != "gft0" and arm != "q4"
    for j in range(m):
        outs.append(f"    wire signed [{accw-1}:0] a{j};")
        conn.append(f".a{j}(a{j})")
        if paired:
            outs.append(f"    wire signed [{accw-1}:0] b{j};")
            conn.append(f".b{j}(b{j})")
    sums = " + ".join([f"$signed(a{j})" for j in range(m)] +
                      ([f"$signed(b{j})" for j in range(m)] if paired else []))
    return f"""`default_nettype none
// GFTernary rung {arm} -- port-less BSCANE2 harness. Refs #1959
module gft_{arm}_jtag #(parameter integer JTAG_CHAIN_N = 3);
    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // 64-bit maximal-length LFSR: gives the layer a changing input with state,
    // so nothing upstream of it can be constant-folded.
    reg [63:0] lfsr = 64'h0123456789ABCDEF;
    always @(posedge cfgmclk)
        lfsr <= {{lfsr[62:0], lfsr[63] ^ lfsr[62] ^ lfsr[60] ^ lfsr[59]}};

{chr(10).join(outs)}
    layer_{arm} inst (.x(lfsr), {", ".join(conn)});

    // Observe every accumulator. The parity of a running sum cannot be
    // predicted without computing the layer, so the layer must survive synthesis.
    reg [31:0] acc = 32'd0;
    always @(posedge cfgmclk) acc <= acc + {sums};

    // W716: `ok` MUST be a deterministic function of this rung's arithmetic.
    // The first version used `^acc` on a free-running accumulator, so the bit
    // sampled a counter and changed between two reads of the SAME bitstream --
    // it discriminated the read time, not the rung. Freeze the parity after a
    // fixed number of clocks and it becomes a per-rung fingerprint.
    reg [23:0] pre  = 24'd0;
    reg        beat = 1'b0;
    reg        sig  = 1'b0;
    reg        frozen = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) beat <= ~beat;
        if (!frozen && pre == 24'hFFFF) begin
            sig    <= ^acc;
            frozen <= 1'b1;
        end
    end
    wire ok = sig;

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));

    reg [31:0] sr = 32'hA5A5A5A4;
    always @(posedge drck)
        if (sel) begin
            if (capture)    sr <= {{28'hA5A5A5A, 1'b0, 1'b1, beat, ok}};
            else if (shift) sr <= {{tdi, sr[31:1]}};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
"""

if __name__ == "__main__":
    print(wrap(sys.argv[1], int(sys.argv[2]), int(sys.argv[3])))
