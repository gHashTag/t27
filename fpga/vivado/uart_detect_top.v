module uart_detect_top (
    input  wire uart_rx_pin,
    output wire uart_tx_pin,
    output wire led_r23,
    output wire led_t23
);

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

    reg rx_sync;
    reg rx_prev;
    reg [22:0] pulse_count;

    always @(posedge osc) begin
        rx_sync <= uart_rx_pin;
        rx_prev <= rx_sync;
        if (rx_prev && !rx_sync) begin
            pulse_count <= pulse_count + 1;
        end
    end

    assign led_r23 = ~counter[20];
    assign led_t23 = (pulse_count > 0) ? ~counter[19] : 1'b1;

    assign uart_tx_pin = 1'b1;

endmodule
