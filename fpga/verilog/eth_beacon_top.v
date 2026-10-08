`default_nettype none
// E1 of the Ethernet bring-up on the ALINX AX7203: primitives and wiring only (owner-approved-foreign,
// 2026-10-08). Every decision is specs/fpga/eth_beacon.t27. 200 MHz -> PLL -> 125 MHz for the data and a
// 90-degree copy for TXC; both PHYs get the same frames, so either RJ-45 port works.
module eth_beacon_top (
    input wire clk200_p, input wire clk200_n,
    output wire e1_rst_n, output wire e1_txc, output wire e1_txctl, output wire [3:0] e1_txd,
    input wire e1_rxctl, input wire [3:0] e1_rxd,
    output wire e2_rst_n, output wire e2_txc, output wire e2_txctl, output wire [3:0] e2_txd,
    input wire e2_rxctl, input wire [3:0] e2_rxd,
    output wire [3:0] led);
    wire clk200, fb, c0, c90, clk, clk90, locked;
    wire [7:0] txd, tx_en, rst, ld;
    IBUFDS ib (.I(clk200_p), .IB(clk200_n), .O(clk200));
    PLLE2_ADV #(.CLKIN1_PERIOD(5.0), .CLKFBOUT_MULT(5), .DIVCLK_DIVIDE(1), .CLKOUT0_DIVIDE(8),
        .CLKOUT1_DIVIDE(8), .CLKOUT1_PHASE(90.0)) pll (.CLKIN1(clk200), .CLKIN2(1'b0), .CLKINSEL(1'b1),
        .CLKFBIN(fb), .CLKFBOUT(fb), .CLKOUT0(c0), .CLKOUT1(c90), .LOCKED(locked), .PWRDWN(1'b0),
        .RST(1'b0), .DADDR(7'd0), .DCLK(1'b0), .DEN(1'b0), .DI(16'd0), .DWE(1'b0));
    BUFG g0 (.I(c0), .O(clk));
    BUFG g1 (.I(c90), .O(clk90));
    EthBeacon b (.clk(clk), .rst_n(locked), .en(1'b1), .c1({7'd0, e1_rxctl}), .d1({4'd0, e1_rxd}),
        .c2({7'd0, e2_rxctl}), .d2({4'd0, e2_rxd}), .txd(txd), .tx_en(tx_en), .phy_rst_n(rst), .led(ld));
    assign {e1_rst_n, e2_rst_n} = {rst[0], rst[0]};
    assign led = ~ld[3:0];  // the AX7203 user LEDs light on a low output
    // ODDR takes D2 on the falling edge, after txd has moved on: hold the high nibble one byte back.
    reg [3:0] hi = 4'd0;
    reg ctl = 1'b0;
    always @(posedge clk) begin hi <= txd[7:4]; ctl <= tx_en[0]; end
    wire [4:0] d1 = {tx_en[0], txd[3:0]}, d2 = {ctl, hi}, q1, q2;
    assign {e1_txctl, e1_txd} = q1;
    assign {e2_txctl, e2_txd} = q2;
    genvar k;
    generate for (k = 0; k < 5; k = k + 1) begin : lane
        ODDR o1 (.C(clk), .CE(1'b1), .D1(d1[k]), .D2(d2[k]), .R(1'b0), .S(1'b0), .Q(q1[k]));
        ODDR o2 (.C(clk), .CE(1'b1), .D1(d1[k]), .D2(d2[k]), .R(1'b0), .S(1'b0), .Q(q2[k]));
    end endgenerate
    ODDR t1 (.C(clk90), .CE(1'b1), .D1(1'b1), .D2(1'b0), .R(1'b0), .S(1'b0), .Q(e1_txc));
    ODDR t2 (.C(clk90), .CE(1'b1), .D1(1'b1), .D2(1'b0), .R(1'b0), .S(1'b0), .Q(e2_txc));
endmodule
`default_nettype wire
