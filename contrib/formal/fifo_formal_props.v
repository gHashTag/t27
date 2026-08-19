// v1 (2026-08-20, #2265): first DUT-bound property set; see uart_formal_props.v
// header for the history. The generated Fifo currently exposes NO data ports
// (#2238), so the one non-vacuous provable property is the handshake constant.
// This file exists to keep the harness real and ready to grow with the ports.
module fifo_formal_props (
    input wire clk,
    input wire rst_n,
    input wire en
);
    wire ready;
    Fifo dut (.clk(clk), .rst_n(rst_n), .en(en), .ready(ready));

    // P1: the handshake line is constant-high in the current lowering.
    always @(*) assert (ready == 1'b1);
endmodule
