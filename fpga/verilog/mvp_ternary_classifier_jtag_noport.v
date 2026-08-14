`default_nettype none

// mvp_ternary_classifier_jtag_noport.v -- the verdict on JTAG, with NO package pins.
//
// WHY THIS EXISTS SEPARATELY FROM `_jtag.v`.
// `mvp_ternary_classifier_jtag` declares `led_r23` and `led_t23` as outputs, so
// nextpnr must constrain them and dies on the XDC:
//
//     ERROR: Unable to constrain IO 'led_t23', device does not have a pin named ''
//
// The board pin map is the one thing this repository has never had for the
// FGG676 part (the only XDC in the tree targets CSG324). W689/T163 recovered the
// recipe from this project's OWN withdrawn upstream issue, and its decisive
// property was: **the design has no port list at all**, so no package pin is
// driven -- it needs no board schematic, no LED, and no PS. That is what makes
// the readback testable on a board whose pinout we cannot fully assert.
//
// The lamps do not disappear; they become internal wires. The verdict leaves the
// die through USER1 and nowhere else, which is the entire point: `Done 0x1` says
// the fabric was configured, and a lamp says nothing a machine can read.
//
// PROTOCOL. USER3 (IR = 0x22) is a 32-bit shift register clocked by the JTAG DRCK. On
// CAPTURE it loads the verdict word; on SHIFT it walks out LSB first:
//
//     [31:4]  28-bit magic 0xA5A5A5A
//     [3]     constant 0
//     [2]     constant 1
//     [1]     beat      -- the heartbeat, so a stuck chain is distinguishable
//     [0]     ok        -- 1 = every input classified correctly since power-up
//
// W675 established why the magic must be this wide: ten reads of a bitstream
// containing NO BSCANE2 returned the same two values in the same proportions as
// the real design (T139). Two constant bits are not enough entropy to prove
// provenance. 0xA5A5A5A cannot be produced by a TAP that is not shifting this
// register, so if the magic comes back the bits below it came from here -- and
// if it does not, `ok` means nothing regardless of its value.
//
// Target: QMTech Wukong V1 / XC7A200T-FGG676 via OpenXC7.
// Refs #1959
module mvp_ternary_classifier_jtag_noport #(
    // W693: THE CHAIN NUMBER IS NOT A LITERAL ANY MORE.
    //
    // W690 wrote `.JTAG_CHAIN(3)` because that build happened to place BSCANE2
    // at site 3. A compiler change one wave later (W692, gen-verilog stopped
    // emitting test blocks) changed the netlist, nextpnr moved the cell to site
    // 2, and the literal became wrong -- silently. A wrong chain reads ALL ZERO,
    // which is indistinguishable from a design that is not on the board at all.
    //
    // `t27c silicon` now reads the site out of the FASM and passes it in, so the
    // parameter cannot drift from the placement. Setting it by hand is exactly
    // how this failed the first time.
    parameter integer JTAG_CHAIN_N = 3
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

    // Internal, not ports. The check module still sweeps all 256 inputs and
    // re-checks them against the golden table ~250,000 times a second.
    wire led_r23, led_t23;
    mvp_ternary_classifier_check #(
        .PRESCALE_BITS(24)
    ) check (
        .clk(cfgmclk),
        .led_r23(led_r23),
        .led_t23(led_t23)
    );

    // The check module exposes its state only through the lamps, so recover the
    // two bits from them: r23 = beat & ok, t23 = ~ok.
    wire ok   = ~led_t23;
    wire beat =  led_r23;

    // ---- JTAG USER3 ----
    wire drck, sel, shift, capture, tdi;
    wire tdo;

    // W690: JTAG_CHAIN MUST MATCH THE SITE nextpnr PLACES THIS CELL AT.
    //
    // This is the defect that hid the readback for six waves, and it is one
    // parameter.
    //
    // The CFG_CENTER_MID tile has four independent chain-enable bits and 44
    // pseudo-pips split 11 apiece across sites BSCAN1..BSCAN4:
    //
    //     CFG_CENTER_MID.BSCAN.JTAG_CHAIN_1  26_2162
    //     CFG_CENTER_MID.BSCAN.JTAG_CHAIN_2  27_2162
    //     CFG_CENTER_MID.BSCAN.JTAG_CHAIN_3  26_2163
    //     CFG_CENTER_MID.BSCAN.JTAG_CHAIN_4  27_2163
    //
    // nextpnr places a lone BSCANE2 at site **BSCAN3**. With `.JTAG_CHAIN(1)`
    // the FASM then carried `BSCAN.JTAG_CHAIN_1` -- enabling chain 1 -- while
    // routing site 3's TDI/TDO/DRCK/SEL/CAPTURE/SHIFT. Chain 1 selects a site
    // that is not wired; site 3 is wired to a chain nothing selects. Measured:
    //
    //     JTAG_CHAIN(1), site BSCAN3   USER1 -> ffffffff   USER2/3/4 -> 00000000
    //     JTAG_CHAIN(3), site BSCAN3   USER3 -> a5a5a5a7   USER1/2/4 -> 00000000
    //
    // The BEL cannot be pinned instead: nextpnr routes BSCANE2 through the IO
    // packer and rejects the attribute with `Unexpected IOBUF BEL
    // BSCAN_X0Y0/BSCAN`. Matching the parameter to the placement is the fix that
    // works. **If nextpnr's placement ever changes, this constant must change
    // with it** -- check the FASM's `BSCAN.JTAG_CHAIN_n` line against the
    // `CFG_CENTER_BSCANn_*` routing lines before trusting a read.
    //
    // The register therefore answers on USER3 (IR = 0x22), not USER1.
    BSCANE2 #(
        .JTAG_CHAIN(JTAG_CHAIN_N)
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

    reg [31:0] sr = 32'hA5A5A5A4;
    always @(posedge drck) begin
        if (sel) begin
            if (capture)     sr <= {28'hA5A5A5A, 1'b0, 1'b1, beat, ok};
            else if (shift)  sr <= {tdi, sr[31:1]};
        end
    end
    assign tdo = sr[0];
endmodule

`default_nettype wire
