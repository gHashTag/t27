`default_nettype none
// Minimal DSP48E1 probe: the SAME product computed two ways in ONE design.
//
//   path A : a hand-instantiated DSP48E1 primitive -- yosys does not infer it,
//            so this tests the PRIMITIVE path (nextpnr placement + prjxray
//            frames) with a configuration this file states explicitly.
//   path B : a plain `*`, built with `-nodsp` so it lands in LUTs and CARRY4.
//
// `-nodsp` blocks INFERENCE; it does not remove an explicit instance. One
// bitstream therefore carries both, and the die compares them itself.
//
//   ok = 1  the DSP primitive is driven correctly by this flow, and the defect
//           in tri-net's gft16_mul lies in the FASM emitted for an INFERRED DSP
//   ok = 0  openXC7 cannot drive a DSP48E1 at all, and the address is general
//
// Refs #1959, gHashTag/tri-net#381
module dsp_probe #(parameter integer JTAG_CHAIN_N = 3);
    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    localparam [29:0] A_VAL = 30'd12345;
    localparam [17:0] B_VAL = 18'd6789;
    localparam [47:0] EXPECT = 48'd83810205;      // 12345 * 6789

    wire [47:0] p_dsp;
    // OPMODE 7'b0000101 : X = M, Y = M, Z = 0  -> P = A*B
    // ALUMODE 4'b0000   : Z + X + Y + CIN
    // INMODE  5'b00000  : A from the A port, B from the B port
    DSP48E1 #(
        // W726: PREG(1) gave p_dsp = 0 in simulation -- MY configuration, caught
        // before the die. yosys's own working instance (net_dsp.v) sets EVERY
        // register attribute to 0 and drives the part combinationally, so the
        // probe copies that and registers the COMPARISON instead.
        .AREG(0), .BREG(0), .MREG(0), .PREG(0), .ADREG(0), .DREG(0),
        .ACASCREG(0), .BCASCREG(0), .CREG(0), .ALUMODEREG(0),
        .OPMODEREG(0), .INMODEREG(0), .CARRYINREG(0), .CARRYINSELREG(0),
        .A_INPUT("DIRECT"), .B_INPUT("DIRECT"), .USE_DPORT("FALSE"),
        .USE_MULT("MULTIPLY"), .USE_SIMD("ONE48")
    ) u_dsp (
        .CLK(cfgmclk),
        .A(A_VAL), .B(B_VAL), .C(48'd0), .D(25'd0),
        .OPMODE(7'b0000101), .ALUMODE(4'b0000), .INMODE(5'b00000),
        .CARRYIN(1'b0), .CARRYINSEL(3'b000),
        .CEA1(1'b0), .CEA2(1'b0), .CEB1(1'b0), .CEB2(1'b0), .CEC(1'b0),
        .CED(1'b0), .CEAD(1'b0), .CEM(1'b0), .CEP(1'b0), .CECARRYIN(1'b0),
        .CECTRL(1'b0), .CEALUMODE(1'b0), .CEINMODE(1'b0),
        .RSTA(1'b0), .RSTB(1'b0), .RSTC(1'b0), .RSTD(1'b0), .RSTM(1'b0),
        .RSTP(1'b0), .RSTCTRL(1'b0), .RSTALLCARRYIN(1'b0),
        .RSTALUMODE(1'b0), .RSTINMODE(1'b0),
        .ACIN(30'd0), .BCIN(18'd0), .PCIN(48'd0),
        .CARRYCASCIN(1'b0), .MULTSIGNIN(1'b0),
        .P(p_dsp), .ACOUT(), .BCOUT(), .PCOUT(),
        .CARRYOUT(), .CARRYCASCOUT(), .MULTSIGNOUT(), .OVERFLOW(),
        .UNDERFLOW(), .PATTERNDETECT(), .PATTERNBDETECT()
    );

    // Built with -nodsp, so this multiply lands in LUTs and CARRY4.
    reg [47:0] p_lut = 48'd0;
    always @(posedge cfgmclk) p_lut <= A_VAL[16:0] * B_VAL[16:0];

    reg [7:0] step = 8'd0;
    reg dsp_ok = 1'b0, lut_ok = 1'b0, agree = 1'b0, done = 1'b0;
    always @(posedge cfgmclk) if (!done) begin
        step <= step + 8'd1;
        if (step == 8'd32) begin
            dsp_ok <= (p_dsp  == EXPECT);
            lut_ok <= (p_lut  == EXPECT);
            agree  <= (p_dsp  == p_lut);
            done   <= 1'b1;
        end
    end

    reg [23:0] pre = 24'd0;
    reg beat = 1'b0;
    always @(posedge cfgmclk) begin
        pre <= pre + 24'd1;
        if (pre == 24'd0) beat <= ~beat;
    end

    wire drck, sel, shift, capture, tdi;
    wire tdo;
    BSCANE2 #(.JTAG_CHAIN(JTAG_CHAIN_N)) bscan (
        .CAPTURE(capture), .DRCK(drck), .RESET(), .RUNTEST(), .SEL(sel),
        .SHIFT(shift), .TCK(), .TDI(tdi), .TMS(), .UPDATE(), .TDO(tdo));
    reg [31:0] sr = 32'hA5A5A5A0;
    // reply = {dsp_ok, lut_ok, agree, done} -- every case distinguishable
    always @(posedge drck)
        if (sel) begin
            if (capture)    sr <= {28'hA5A5A5A, dsp_ok, lut_ok, agree, done};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
