// Auto-generated from specs/fpga/uart.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/uart.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: UART_Bridge
// Synthesizable Verilog for UART Bridge with 8-N-1 protocol
// Baud rate: 115200 @ 50 MHz clock

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */


module uart_bridge #(
    parameter CLK_FREQ     = 50000000,
    parameter BAUD_RATE    = 115200,
    parameter BAUD_DIVISOR = CLK_FREQ / BAUD_RATE,  // ~434
    parameter DATA_BITS    = 8,
    parameter STOP_BITS    = 1
)(
    input  wire       clk,
    input  wire       rst_n,

    // --- UART physical lines ---
    input  wire       uart_rx,
    output reg        uart_tx,

    // --- TX data interface ---
    input  wire [7:0] tx_data,
    input  wire       tx_valid,
    output wire       tx_ready,

    // --- RX data interface ---
    output reg  [7:0] rx_data,
    output reg        rx_valid,
    input  wire       rx_ack,

    // --- Status ---
    output wire       framing_error
);

    // =================================================================
    // TX State Machine States
    // =================================================================
    localparam [1:0] TX_IDLE  = 2'd0;
    localparam [1:0] TX_START = 2'd1;
    localparam [1:0] TX_DATA  = 2'd2;
    localparam [1:0] TX_STOP  = 2'd3;

    // =================================================================
    // RX State Machine States
    // =================================================================
    localparam [1:0] RX_IDLE  = 2'd0;
    localparam [1:0] RX_START = 2'd1;
    localparam [1:0] RX_DATA  = 2'd2;
    localparam [1:0] RX_STOP  = 2'd3;

    // =================================================================
    // Protocol Commands
    // =================================================================
    localparam [7:0] CMD_PING      = 8'h01;
    localparam [7:0] CMD_PONG      = 8'h02;
    localparam [7:0] CMD_WRITE_REG = 8'h10;
    localparam [7:0] CMD_READ_REG  = 8'h11;
    localparam [7:0] CMD_MAC_OP    = 8'h20;
    localparam [7:0] CMD_STATUS    = 8'h30;
    localparam [7:0] RESP_OK       = 8'h00;
    localparam [7:0] RESP_ERROR    = 8'hFF;

    // =================================================================
    // TX Unit Registers
    // =================================================================
    reg [1:0]  tx_state;
    reg        tx_busy;
    reg [2:0]  tx_bit_index;
    reg [7:0]  tx_shift_reg;
    reg [15:0] tx_baud_counter;

    assign tx_ready = !tx_busy;

    // =================================================================
    // TX State Machine
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            tx_state        <= TX_IDLE;
            tx_busy         <= 1'b0;
            tx_bit_index    <= 3'd0;
            tx_shift_reg    <= 8'd0;
            tx_baud_counter <= 16'd0;
            uart_tx         <= 1'b1;  /* Idle high */
        end else begin
            case (tx_state)
                TX_IDLE: begin
                    uart_tx <= 1'b1;  /* Idle high */
                    if (tx_valid && !tx_busy) begin
                        tx_shift_reg    <= tx_data;
                        tx_bit_index    <= 3'd0;
                        tx_baud_counter <= 16'd0;
                        tx_busy         <= 1'b1;
                        tx_state        <= TX_START;
                    end
                end

                TX_START: begin
                    uart_tx <= 1'b0;  /* Start bit low */
                    tx_baud_counter <= tx_baud_counter + 16'd1;
                    if (tx_baud_counter >= BAUD_DIVISOR[15:0] - 16'd1) begin
                        tx_baud_counter <= 16'd0;
                        tx_state        <= TX_DATA;
                    end
                end

                TX_DATA: begin
                    uart_tx <= tx_shift_reg[tx_bit_index];
                    tx_baud_counter <= tx_baud_counter + 16'd1;
                    if (tx_baud_counter >= BAUD_DIVISOR[15:0] - 16'd1) begin
                        tx_baud_counter <= 16'd0;
                        if (tx_bit_index == DATA_BITS[2:0] - 3'd1) begin
                            tx_state <= TX_STOP;
                        end else begin
                            tx_bit_index <= tx_bit_index + 3'd1;
                        end
                    end
                end

                TX_STOP: begin
                    uart_tx <= 1'b1;  /* Stop bit high */
                    tx_baud_counter <= tx_baud_counter + 16'd1;
                    if (tx_baud_counter >= BAUD_DIVISOR[15:0] - 16'd1) begin
                        tx_baud_counter <= 16'd0;
                        tx_busy         <= 1'b0;
                        tx_state        <= TX_IDLE;
                    end
                end

                default: begin
                    tx_state <= TX_IDLE;
                end
            endcase
        end
    end

    // =================================================================
    // RX Unit Registers
    // =================================================================
    reg [1:0]  rx_state;
    reg [2:0]  rx_bit_index;
    reg [7:0]  rx_shift_reg;
    reg [15:0] rx_baud_counter;
    reg [2:0]  rx_sync_reg;      /* 3-bit input synchronizer */
    reg        rx_framing_error;

    assign framing_error = rx_framing_error;

    // =================================================================
    // RX Input Synchronizer (2-FF metastability guard + 1 history)
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rx_sync_reg <= 3'b111;  /* Idle high */
        end else begin
            rx_sync_reg <= {rx_sync_reg[1:0], uart_rx};
        end
    end

    wire rx_synced = rx_sync_reg[2];  /* Stable synchronized input */

    // =================================================================
    // RX State Machine
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rx_state         <= RX_IDLE;
            rx_bit_index     <= 3'd0;
            rx_shift_reg     <= 8'd0;
            rx_baud_counter  <= 16'd0;
            rx_framing_error <= 1'b0;
            rx_data          <= 8'd0;
            rx_valid         <= 1'b0;
        end else begin
            /* Clear rx_valid when acknowledged */
            if (rx_ack) begin
                rx_valid <= 1'b0;
            end

            case (rx_state)
                RX_IDLE: begin
                    if (!rx_synced) begin  /* Falling edge: start bit */
                        rx_baud_counter <= BAUD_DIVISOR[15:0] / 16'd2;  /* Sample at middle */
                        rx_state        <= RX_START;
                    end
                end

                RX_START: begin
                    rx_baud_counter <= rx_baud_counter + 16'd1;
                    if (rx_baud_counter >= BAUD_DIVISOR[15:0] - 16'd1) begin
                        rx_baud_counter <= 16'd0;
                        if (!rx_synced) begin  /* Verify start bit still low */
                            rx_state     <= RX_DATA;
                            rx_bit_index <= 3'd0;
                        end else begin
                            rx_state <= RX_IDLE;  /* False start */
                        end
                    end
                end

                RX_DATA: begin
                    rx_baud_counter <= rx_baud_counter + 16'd1;
                    if (rx_baud_counter >= BAUD_DIVISOR[15:0] - 16'd1) begin
                        rx_baud_counter <= 16'd0;
                        rx_shift_reg[rx_bit_index] <= rx_synced;
                        if (rx_bit_index == DATA_BITS[2:0] - 3'd1) begin
                            rx_state <= RX_STOP;
                        end else begin
                            rx_bit_index <= rx_bit_index + 3'd1;
                        end
                    end
                end

                RX_STOP: begin
                    rx_baud_counter <= rx_baud_counter + 16'd1;
                    if (rx_baud_counter >= BAUD_DIVISOR[15:0] - 16'd1) begin
                        rx_baud_counter <= 16'd0;
                        if (rx_synced) begin
                            /* Valid stop bit: latch data */
                            rx_framing_error <= 1'b0;
                            rx_data          <= rx_shift_reg;
                            rx_valid         <= 1'b1;
                        end else begin
                            /* Stop bit not high: framing error */
                            rx_framing_error <= 1'b1;
                        end
                        rx_state <= RX_IDLE;
                    end
                end

                default: begin
                    rx_state <= RX_IDLE;
                end
            endcase
        end
    end

endmodule
