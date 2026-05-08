module uart_loopback_top (
    input  wire uart_rx_pin,
    output wire uart_tx_pin,
    output wire led_r23,
    output wire led_t23
);

    assign uart_tx_pin = uart_rx_pin;

    (* KEEP = "TRUE" *) wire osc;
    (* KEEP = "TRUE" *) wire chain [19:0];
    reg [22:0] counter = 0;

    assign chain[0] = ~chain[19];
    genvar i;
    generate
        for (i = 1; i < 20; i = i + 1) begin : inv_chain
            (* KEEP = "TRUE" *) LUT1 #(.INIT(2'b01)) inv (
                .I0(chain[i-1]),
                .O(chain[i])
            );
        end
    endgenerate
    assign osc = chain[19];

    always @(posedge osc) begin
        counter <= counter + 1;
    end

    assign led_r23 = ~counter[20];
    assign led_t23 = uart_rx_pin;

endmodule
