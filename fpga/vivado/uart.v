module uart_rx #(
    parameter CLK_DIV = 31250
)(
    input  wire        clk,
    input  wire        rx,
    output reg         valid,
    output reg  [7:0]  data
);

    localparam HALF_DIV = CLK_DIV / 2;

    reg [15:0] cnt;
    reg [3:0]  bit_cnt;
    reg [7:0]  shift_reg;
    reg        rx_sync;
    reg        rx_prev;
    reg [1:0]  state;

    always @(posedge clk) begin
        rx_sync <= rx;
        rx_prev <= rx_sync;

        case (state)
            0: begin
                valid <= 0;
                if (rx_prev && !rx_sync) begin
                    // wait 1.5 bit-times so the first sample is the centre of d0
                    // (skip the start bit); sampling only HALF_DIV captured the
                    // start bit as a data bit and dropped d7.
                    cnt <= CLK_DIV[15:0] + HALF_DIV[15:0] - 16'd1;
                    bit_cnt <= 0;
                    state <= 1;
                end
            end
            1: begin
                if (cnt == 0) begin
                    cnt <= CLK_DIV[15:0] - 16'd1;
                    shift_reg <= {rx_sync, shift_reg[7:1]}; // LSB-first: new bit into MSB
                    bit_cnt <= bit_cnt + 1;
                    if (bit_cnt == 7) begin                 // 8th data bit (d7) sampled
                        data <= {rx_sync, shift_reg[7:1]};
                        valid <= 1;
                        state <= 0;
                    end
                end else begin
                    cnt <= cnt - 16'd1;
                end
            end
        endcase
    end

endmodule

module uart_tx #(
    parameter CLK_DIV = 31250
)(
    input  wire        clk,
    input  wire [7:0]  data,
    input  wire        start,
    output reg         tx,
    output reg         done
);

    reg [15:0] cnt;
    reg [3:0]  bit_cnt;
    reg [10:0] shift_reg;
    reg [1:0]  state;

    always @(posedge clk) begin
        case (state)
            0: begin
                done <= 0;
                tx <= 1;
                if (start) begin
                    shift_reg <= {1'b1, data, 1'b0};
                    bit_cnt <= 0;
                    cnt <= CLK_DIV[15:0] - 16'd1;
                    state <= 1;
                end
            end
            1: begin
                if (cnt == 0) begin
                    tx <= shift_reg[0];
                    shift_reg <= {1'b1, shift_reg[10:1]};
                    cnt <= CLK_DIV[15:0] - 16'd1;
                    bit_cnt <= bit_cnt + 1;
                    if (bit_cnt == 10) begin
                        done <= 1;
                        state <= 0;
                    end
                end else begin
                    cnt <= cnt - 16'd1;
                end
            end
        endcase
    end

endmodule
