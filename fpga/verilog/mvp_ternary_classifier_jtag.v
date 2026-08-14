`default_nettype none

// mvp_ternary_classifier_jtag.v -- the MVP's verdict, exported to JTAG.
//
// WHY.  Everything this project has put on silicon rests on `led_r23`. The
// equivalence miter proves the arithmetic for all inputs (T110), and the
// on-chip sweep re-checks all 256 of them ~250,000 times a second -- but the
// only channel carrying the answer out of the die is a lamp, and no machine
// reads it. `Done 0x1` says the fabric was configured; it says nothing about
// what the fabric computed.
//
// BSCANE2 is the one readback path that needs no pin, no wire and no pin map.
// Verified present in the open flow before this file was written:
// nextpnr-openxc7 `pack_io_xc7.cc:1236` maps `id_BSCANE2` to `id_BSCAN`, and
// `X(BSCANE2)` is in its constids.
//
// PROTOCOL.  USER1 is a 4-bit shift register clocked by the JTAG DRCK. On
// CAPTURE it loads the verdict word; on SHIFT it walks out LSB first:
//
//     bit 0  ok        1 = every input classified correctly since power-up
//     bit 1  beat      the heartbeat, so a stuck chain is distinguishable
//     bit 2  1'b1      a constant one   \  a fixed pattern, so an all-zero or
//     bit 3  1'b0      a constant zero  /  all-one chain reads as BROKEN
//
// The two constant bits are the discriminating part. A JTAG chain that returns
// all zeroes or all ones -- the two ways a readback fails silently -- cannot
// produce `x1` in bits 3:2, so a correct-looking `ok` cannot be manufactured by
// a dead chain. That is the same rule as the wrong-part bitstream used to make
// `Done 0->1` mean something.
//
// Target: QMTech Wukong V1 / XC7A200T-FGG676 via OpenXC7.
// Refs #1959

module mvp_ternary_classifier_jtag (
    output wire led_r23,
    output wire led_t23
);
    wire cfgmclk;
    STARTUPE2 #(
        .PROG_USR("FALSE"),
        .SIM_CCLK_FREQ(10.0)
    ) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1)
    );

    wire ok, beat;
    mvp_ternary_classifier_check #(
        .PRESCALE_BITS(24)
    ) check (
        .clk(cfgmclk),
        .led_r23(led_r23),
        .led_t23(led_t23)
    );

    // The check module exposes its state only through the lamps, so recover the
    // two bits from them: r23 = beat & ok, t23 = ~ok.
    assign ok   = ~led_t23;
    assign beat = led_r23;

    // ---- JTAG USER1 ----
    wire drck, sel, shift, capture, tdi;
    wire tdo;

    BSCANE2 #(
        .JTAG_CHAIN(1)
    ) bscan (
        .CAPTURE(capture),
        .DRCK(drck),
        .RESET(),
        .RUNTEST(),
        .SEL(sel),
        .SHIFT(shift),
        .TCK(),
        .TDI(tdi),
        .TMS(),
        .UPDATE(),
        .TDO(tdo)
    );

    reg [3:0] sr = 4'b0100;
    always @(posedge drck) begin
        if (sel) begin
            if (capture)     sr <= {1'b0, 1'b1, beat, ok};
            else if (shift)  sr <= {tdi, sr[3:1]};
        end
    end
    assign tdo = sr[0];
endmodule

`default_nettype wire
