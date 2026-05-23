module uart_echo_top (
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

    localparam CLK_DIV = 175;

    wire [7:0] rx_data;
    wire rx_valid;
    reg [7:0] tx_data;
    reg tx_start;
    wire tx_done;

    uart_rx #(.CLK_DIV(CLK_DIV)) u_rx (
        .clk(osc),
        .rx(uart_rx_pin),
        .valid(rx_valid),
        .data(rx_data)
    );

    uart_tx #(.CLK_DIV(CLK_DIV)) u_tx (
        .clk(osc),
        .data(tx_data),
        .start(tx_start),
        .tx(uart_tx_pin),
        .done(tx_done)
    );

    reg [1:0] state;
    always @(posedge osc) begin
        tx_start <= 0;
        case (state)
            0: begin
                if (rx_valid) begin
                    tx_data <= 8'hAA;
                    tx_start <= 1;
                    state <= 1;
                end
            end
            1: begin
                if (tx_done) begin
                    state <= 0;
                end
            end
        endcase
    end

    assign led_r23 = ~counter[20];
    assign led_t23 = ~counter[19];

endmodule
