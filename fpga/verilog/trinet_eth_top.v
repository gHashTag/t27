`default_nettype none
// E2e (#8770): the TRI-NET node on the ALINX AX7203 at 100 Mb/s, PHY 1. Primitives and wiring only
// (owner-approved-foreign); every decision is t27: specs/fpga/{mdio_100m,rgmii_100,eth_rx,trinet_node,
// eth_reply,jtag_status}.t27. RXC (25 MHz) -> PLL: clk samples RX; clk180 forwards TXC mid-nibble.
module trinet_eth_top (input wire clk200_p, clk200_n, e1_rxc, e1_rxctl, input wire [3:0] e1_rxd,
    output wire e1_rst_n, e2_rst_n, e1_txc, e1_txctl, e1_mdc, e1_mdio, output wire [3:0] e1_txd, led);
    wire c200i, c200, fb, c0, c1, clk, clk180, lk, cap, drck, sel, sft, tdi;
    wire [7:0] rst, mdc, mdio, rxs, dv, b, pay_v, done, ok, why, pv, dn, pb, jgo, rv, y, st, tick, take, go;
    wire [7:0] txd, txe, ten, nib, ld, tdo; wire [15:0] pi, port;
    wire [31:0] ip, rn, nok, ndrop, nsent; wire [63:0] mac, tag, status;
    IBUFDS ib (.I(clk200_p), .IB(clk200_n), .O(c200i));
    BUFG gc (.I(c200i), .O(c200));
    Mdio100m m (.clk(c200), .rst_n(1'b1), .en(1'b1), .phy_rst_n(rst), .mdc(mdc), .mdio(mdio));
    assign {e1_rst_n, e2_rst_n, e1_mdc, e1_mdio} = {rst[0], rst[0], mdc[0], mdio[0]};
    PLLE2_ADV #(.CLKIN1_PERIOD(40.0), .CLKFBOUT_MULT(40), .CLKOUT0_DIVIDE(40), .CLKOUT1_DIVIDE(40),
        .CLKOUT1_PHASE(180.0)) pll (.CLKIN1(e1_rxc), .CLKIN2(1'b0), .CLKINSEL(1'b1), .CLKFBIN(fb), .CLKFBOUT(fb),
        .CLKOUT0(c0), .CLKOUT1(c1), .LOCKED(lk), .PWRDWN(1'b0), .RST(~rst[0]), .DADDR(7'd0), .DCLK(1'b0),
        .DEN(1'b0), .DI(16'd0), .DWE(1'b0));
    BUFG g0 (.I(c0), .O(clk));
    BUFG g1 (.I(c1), .O(clk180));
    Rgmii100 r (.clk(clk), .rst_n(lk), .en(1'b1), .rxctl({7'd0, e1_rxctl}), .rxd({4'd0, e1_rxd}), .pay_v(pay_v),
        .rdone(done), .j_go(jgo), .resp_v(rv), .txd(txd), .txe(txe), .n_ok(nok), .n_drop(ndrop), .n_sent(nsent),
        .why(why), .rx_en(rxs), .rx_dv(dv), .rx_b(b), .pv(pv), .dn(dn), .tick(tick), .take(take), .go(go),
        .tx_en(ten), .tx_nib(nib), .led(ld), .status(status));
    EthRx x (.clk(clk), .rst_n(lk), .en(rxs[0]), .dv(dv), .b(b), .pay_v(pay_v), .pay_b(pb), .pay_i(pi), .done(done),
        .ok(ok), .why(why), .src_mac(mac), .src_ip(ip), .src_port(port), .n_ok(nok), .n_drop(ndrop));
    TrinetNode t (.clk(clk), .rst_n(lk), .en(1'b1), .pv(pv), .pb(pb), .pi(pi), .dn(dn), .okf(ok), .resp_v(rv),
        .r_y(y), .r_st(st), .r_n(rn), .r_tag(tag), .j_go(jgo));
    EthReply e (.clk(clk), .rst_n(lk), .en(tick[0]), .take(take), .smac(mac), .sip(ip), .sport(port), .go(go),
        .ry(y), .rs(st), .rn(rn), .rtag(tag), .txd(txd), .tx_en(txe), .n_sent(nsent));
    assign {e1_txctl, e1_txd, led} = {ten[0], nib[3:0], ld[3:0]};
    ODDR ot (.C(clk180), .CE(1'b1), .D1(1'b1), .D2(1'b0), .R(1'b0), .S(1'b0), .Q(e1_txc));
    BSCANE2 #(.JTAG_CHAIN(3)) bs (.CAPTURE(cap), .DRCK(drck), .SEL(sel), .SHIFT(sft), .TDI(tdi), .TDO(tdo[0]));
    JtagStatus j (.clk(drck), .rst_n(1'b1), .en(1'b1), .sel({7'd0, sel}), .cap({7'd0, cap}), .shift({7'd0, sft}),
        .tdi({7'd0, tdi}), .word(status), .tdo(tdo));
endmodule
`default_nettype wire
