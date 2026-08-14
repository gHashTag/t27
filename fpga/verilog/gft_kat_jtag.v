`default_nettype none
// tri-net's GF-T16 multiplier, on OUR die, checked by tri-net's OWN KAT. Refs #1959
//
// The RTL is gHashTag/tri-net `fpga/gft/gft16_mul.v` verbatim, and the two
// vectors are the ones its own testbench uses -- phi^1 * phi^1 = phi^2, and
// 1.5 * 1.5. Nothing here re-implements the multiplier or re-derives an
// expected value, so there is no co-authored golden model: the check is
// tri-net's, the fabric is ours.
//
// tri-net ran GF-T16 on an ALINX AX7203; this is a QMTech Wukong. Both are
// XC7A200T -- same die, different package -- and by T230 a port-less design
// cannot see the difference.
module gft_kat_jtag #(parameter integer JTAG_CHAIN_N = 3);
    wire cfgmclk;
    STARTUPE2 #(.PROG_USR("FALSE"), .SIM_CCLK_FREQ(10.0)) startup (
        .CFGCLK(), .CFGMCLK(cfgmclk), .EOS(), .PREQ(),
        .CLK(1'b0), .GSR(1'b0), .GTS(1'b0), .KEYCLEARB(1'b0),
        .PACK(1'b0), .USRCCLKO(1'b0), .USRCCLKTS(1'b0),
        .USRDONEO(1'b1), .USRDONETS(1'b1));

    // vector 0: (41,0)   x (41,0)   -> (42,0)
    // vector 1: (41,256) x (41,256) -> (43,64)
    reg sel_v = 1'b0;
    wire [6:0] a_off  = 7'd41;
    wire [8:0] a_mant = sel_v ? 9'd256 : 9'd0;
    wire [6:0] e_off  = sel_v ? 7'd43  : 7'd42;
    wire [8:0] e_mant = sel_v ? 9'd64  : 9'd0;
    wire [6:0] o_off;
    wire [8:0] o_mant;
    gft16_mul dut (.a_off(a_off), .a_mant(a_mant),
                   .b_off(a_off), .b_mant(a_mant),
                   .out_off(o_off), .out_mant(o_mant));

    // W723: the first version sampled two cycles after changing the input and
    // passed in iverilog while the die said ok=0. Behavioural simulation runs
    // the RTL; the die runs what yosys MAPPED, and yosys inferred a DSP48E1,
    // which can carry a register stage the RTL does not show. Give each vector
    // 16 cycles and the comparison stops depending on mapping latency.
    reg  v0_ok = 1'b0, v1_ok = 1'b0, done = 1'b0, sig = 1'b0;
    reg [7:0] step = 8'd0;
    always @(posedge cfgmclk) if (!done) begin
        step <= step + 8'd1;
        case (step)
            8'd0:  sel_v <= 1'b0;
            8'd16: v0_ok <= (o_off == e_off) && (o_mant == e_mant);
            8'd17: sel_v <= 1'b1;
            8'd40: v1_ok <= (o_off == e_off) && (o_mant == e_mant);
            8'd48: begin
                sig  <= v0_ok & ((o_off == e_off) && (o_mant == e_mant));
                done <= 1'b1;
            end
            default: ;
        endcase
    end

    reg [23:0] pre = 24'd0;
    reg beat = 1'b0;
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
    reg [31:0] sr = 32'hA5A5A5A4;
    always @(posedge drck)
        if (sel) begin
            // W723: ok=0 alone says nothing about WHICH vector failed. Put the
            // whole state on the wire -- third time this lesson has been needed.
            if (capture)    sr <= {28'hA5A5A5A, v0_ok, v1_ok, done, sig};
            else if (shift) sr <= {tdi, sr[31:1]};
        end
    assign tdo = sr[0];
endmodule
`default_nettype wire
