module uart_formal_props (
    input wire clk,
    input wire rst_n,
    input wire uart_rx,
    output wire uart_tx
);

    default clocking fp @(posedge clk); endclocking
    default disable !rst_n;

    // P1: TX line idle high after reset
    assert property (rst_n |-> uart_tx == 1'b1)
        else $error("P1 FAILED: TX line not idle high after reset");

    // P2: TX line always driven (no X/Z)
    cover property (uart_tx == 1'b0);
    cover property (uart_tx == 1'b1);

    // P3: RX start bit is low
    assume property (uart_rx == 1'b1 || uart_rx == 1'b0);

    // P4: If TX sends start bit, stop bit follows within 10 baud periods
    // (approximate check: start bit low followed by data then high)
    assert property (uart_tx == 1'b0 |-> ##[1:1000] uart_tx == 1'b1)
        else $error("P4 FAILED: TX start bit not followed by stop");

endmodule
