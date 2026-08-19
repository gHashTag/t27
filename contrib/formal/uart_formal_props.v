// v1 (2026-08-20, #2265): the first property set in this repo BOUND TO THE DUT.
// The previous file used SVA (default clocking / disable) that yosys does not
// parse, mirrored a port interface the generated module never had, and never
// instantiated anything -- every assertion floated over undriven wires.
// yosys-supported subset only: DUT instantiation + immediate assertions.
module uart_formal_props (
    input wire clk,
    input wire rst_n,
    input wire en,
    input wire [7:0] data
);
    wire ready, result;
    ZeroDSP_UART dut (
        .clk(clk), .rst_n(rst_n), .en(en), .data(data),
        .ready(ready), .result(result)
    );

    // P1: the handshake line is constant-high in the current lowering.
    always @(*) assert (ready == 1'b1);

    // P2: uart_tx_send reports success for EVERY input byte -- the fresh-state
    // combinational lowering makes tx_ready's initial value reach every call.
    // Cross-checked by exhaustive simulation (256/256) before proving.
    always @(*) assert (result == 1'b1);
endmodule
