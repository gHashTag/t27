// Auto-generated from specs/fpga/bridge.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/bridge.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: FPGA_Bridge
// Synthesizable Verilog for FPGA communication bridge
// Combines UART and SPI with circular buffer management and packet protocol

module fpga_bridge #(
    parameter RX_BUFFER_SIZE  = 256,
    parameter TX_BUFFER_SIZE  = 256,
    parameter SPI_BUFFER_SIZE = 64,
    parameter MAX_PACKET_SIZE = 128,
    parameter PACKET_TIMEOUT  = 10000,
    parameter ADDR_BITS_RX    = 8,   // log2(RX_BUFFER_SIZE)
    parameter ADDR_BITS_TX    = 8    // log2(TX_BUFFER_SIZE)
)(
    input  wire                     clk,
    input  wire                     rst_n,

    // --- UART interface ---
    input  wire [7:0]               uart_rx_data,
    input  wire                     uart_rx_valid,
    output reg                      uart_rx_ready,
    output reg  [7:0]               uart_tx_data,
    output reg                      uart_tx_valid,
    input  wire                     uart_tx_ready,

    // --- SPI interface ---
    output reg  [7:0]               spi_tx_data,
    output reg                      spi_tx_valid,
    input  wire [7:0]               spi_rx_data,
    input  wire                     spi_rx_valid,
    input  wire                     spi_busy,

    // --- Status / config ---
    output wire [2:0]               bridge_state_out,
    output wire                     spi_enabled_out,
    output wire                     mac_enabled_out,
    input  wire [7:0]               config_data,
    input  wire                     config_valid
);

    // ===================================================================
    // 1. Bridge State Machine Constants
    // ===================================================================
    localparam [2:0] BRIDGE_IDLE  = 3'd0;
    localparam [2:0] BRIDGE_RX    = 3'd1;
    localparam [2:0] BRIDGE_PARSE = 3'd2;
    localparam [2:0] BRIDGE_TX    = 3'd3;
    localparam [2:0] BRIDGE_SPI   = 3'd4;
    localparam [2:0] BRIDGE_MAC   = 3'd5;

    // ===================================================================
    // 2. Packet Type Constants
    // ===================================================================
    localparam [7:0] PKT_UART_DATA = 8'h00;
    localparam [7:0] PKT_SPI_XFER  = 8'h10;
    localparam [7:0] PKT_MAC_OP    = 8'h20;
    localparam [7:0] PKT_STATUS    = 8'h30;
    localparam [7:0] PKT_CONFIG    = 8'h40;

    // ===================================================================
    // 3. Bridge Unit State Registers
    // ===================================================================
    reg [2:0]                   state;
    reg [ADDR_BITS_RX-1:0]     rx_head;
    reg [ADDR_BITS_RX-1:0]     rx_tail;
    reg [ADDR_BITS_TX-1:0]     tx_head;
    reg [ADDR_BITS_TX-1:0]     tx_tail;
    reg [7:0]                   packet_len;
    reg [7:0]                   packet_type;
    reg [15:0]                  timeout_cnt;
    reg                         spi_enabled;
    reg                         mac_enabled;

    // Byte counter for payload processing
    reg [7:0]                   byte_cnt;

    // SPI transfer registers
    reg [7:0]                   spi_cs_sel;
    reg [7:0]                   spi_data_l;
    reg [7:0]                   spi_data_h;
    reg [1:0]                   spi_byte_idx;

    // MAC operation registers
    reg [7:0]                   mac_op;
    reg [7:0]                   mac_unit;
    reg [1:0]                   mac_byte_idx;

    // Status response registers
    reg [1:0]                   status_byte_idx;

    assign bridge_state_out = state;
    assign spi_enabled_out  = spi_enabled;
    assign mac_enabled_out  = mac_enabled;

    // ===================================================================
    // 4. Circular Buffer Memory
    // ===================================================================
    reg [7:0] rx_buffer [0:RX_BUFFER_SIZE-1];
    reg [7:0] tx_buffer [0:TX_BUFFER_SIZE-1];

    // ===================================================================
    // 5. Buffer Count Logic
    // ===================================================================
    wire [ADDR_BITS_RX:0] rx_count;
    wire [ADDR_BITS_TX:0] tx_count;
    wire [ADDR_BITS_TX:0] tx_space;

    assign rx_count = (rx_head >= rx_tail)
                    ? (rx_head - rx_tail)
                    : (rx_head + RX_BUFFER_SIZE[ADDR_BITS_RX:0] - rx_tail);

    assign tx_count = (tx_head >= tx_tail)
                    ? (tx_head - tx_tail)
                    : (tx_head + TX_BUFFER_SIZE[ADDR_BITS_TX:0] - tx_tail);

    assign tx_space = TX_BUFFER_SIZE[ADDR_BITS_TX:0] - tx_count;

    // ===================================================================
    // 6. Bridge State Machine
    // ===================================================================
    integer i;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state        <= BRIDGE_IDLE;
            rx_head      <= {ADDR_BITS_RX{1'b0}};
            rx_tail      <= {ADDR_BITS_RX{1'b0}};
            tx_head      <= {ADDR_BITS_TX{1'b0}};
            tx_tail      <= {ADDR_BITS_TX{1'b0}};
            packet_len   <= 8'd0;
            packet_type  <= 8'd0;
            timeout_cnt  <= 16'd0;
            spi_enabled  <= 1'b1;
            mac_enabled  <= 1'b1;
            byte_cnt     <= 8'd0;
            spi_cs_sel   <= 8'd0;
            spi_data_l   <= 8'd0;
            spi_data_h   <= 8'd0;
            spi_byte_idx <= 2'd0;
            mac_op       <= 8'd0;
            mac_unit     <= 8'd0;
            mac_byte_idx <= 2'd0;
            status_byte_idx <= 2'd0;
            uart_rx_ready <= 1'b1;
            uart_tx_data  <= 8'd0;
            uart_tx_valid <= 1'b0;
            spi_tx_data   <= 8'd0;
            spi_tx_valid  <= 1'b0;
            for (i = 0; i < RX_BUFFER_SIZE; i = i + 1)
                rx_buffer[i] <= 8'd0;
            for (i = 0; i < TX_BUFFER_SIZE; i = i + 1)
                tx_buffer[i] <= 8'd0;
        end else begin
            uart_tx_valid <= 1'b0;
            spi_tx_valid  <= 1'b0;

            // --- Config port (always active) ---
            if (config_valid) begin
                spi_enabled <= config_data[0];
                mac_enabled <= config_data[1];
            end

            // --- UART RX: write to RX circular buffer ---
            if (uart_rx_valid && uart_rx_ready) begin
                rx_buffer[rx_head] <= uart_rx_data;
                rx_head <= rx_head + 1'b1;  // Wraps naturally with ADDR_BITS_RX width
            end

            // --- UART TX: read from TX circular buffer ---
            if (uart_tx_ready && (tx_head != tx_tail)) begin
                uart_tx_data  <= tx_buffer[tx_tail];
                uart_tx_valid <= 1'b1;
                tx_tail <= tx_tail + 1'b1;
            end

            // --- Main state machine ---
            case (state)
                BRIDGE_IDLE: begin
                    timeout_cnt <= 16'd0;
                    byte_cnt    <= 8'd0;
                    if (rx_count >= 2) begin
                        // Parse header: read type and length
                        packet_type <= rx_buffer[rx_tail];
                        packet_len  <= rx_buffer[rx_tail + 1'b1];
                        rx_tail     <= rx_tail + 2'd2;
                        state       <= BRIDGE_PARSE;
                    end
                end

                BRIDGE_PARSE: begin
                    // Validate packet length
                    if (packet_len > MAX_PACKET_SIZE[7:0]) begin
                        state <= BRIDGE_IDLE;
                    end else if (rx_count >= {1'b0, packet_len}) begin
                        // Enough data, dispatch to handler
                        byte_cnt      <= 8'd0;
                        spi_byte_idx  <= 2'd0;
                        mac_byte_idx  <= 2'd0;
                        status_byte_idx <= 2'd0;
                        case (packet_type)
                            PKT_UART_DATA: state <= BRIDGE_TX;
                            PKT_SPI_XFER:  state <= BRIDGE_SPI;
                            PKT_MAC_OP:    state <= BRIDGE_MAC;
                            PKT_STATUS:    state <= BRIDGE_TX;
                            PKT_CONFIG: begin
                                // Read config byte directly
                                spi_enabled <= rx_buffer[rx_tail][0];
                                mac_enabled <= rx_buffer[rx_tail][1];
                                rx_tail <= rx_tail + 1'b1;
                                state   <= BRIDGE_IDLE;
                            end
                            default: begin
                                // Unknown packet type, discard
                                rx_tail <= rx_head;
                                state   <= BRIDGE_IDLE;
                            end
                        endcase
                    end else begin
                        // Wait for more data, check timeout
                        timeout_cnt <= timeout_cnt + 1'b1;
                        if (timeout_cnt >= PACKET_TIMEOUT[15:0]) begin
                            rx_tail <= rx_head;  // Clear buffer
                            state   <= BRIDGE_IDLE;
                        end
                    end
                end

                BRIDGE_TX: begin
                    if (packet_type == PKT_UART_DATA) begin
                        // Echo UART data: read from RX, write to TX
                        if (byte_cnt < packet_len) begin
                            if (tx_space > 0) begin
                                tx_buffer[tx_head] <= rx_buffer[rx_tail];
                                tx_head <= tx_head + 1'b1;
                                rx_tail <= rx_tail + 1'b1;
                                byte_cnt <= byte_cnt + 1'b1;
                            end
                        end else begin
                            state <= BRIDGE_IDLE;
                        end
                    end else if (packet_type == PKT_STATUS) begin
                        // Send status response
                        if (status_byte_idx < 2'd3) begin
                            if (tx_space > 0) begin
                                case (status_byte_idx)
                                    2'd0: tx_buffer[tx_head] <= {7'd0, spi_enabled};
                                    2'd1: tx_buffer[tx_head] <= {7'd0, mac_enabled};
                                    2'd2: tx_buffer[tx_head] <= 8'd0;  // Reserved
                                    default: tx_buffer[tx_head] <= 8'd0;
                                endcase
                                tx_head <= tx_head + 1'b1;
                                status_byte_idx <= status_byte_idx + 1'b1;
                            end
                        end else begin
                            // Write final reserved byte
                            if (tx_space > 0) begin
                                tx_buffer[tx_head] <= 8'd0;
                                tx_head <= tx_head + 1'b1;
                                state <= BRIDGE_IDLE;
                            end
                        end
                    end else begin
                        state <= BRIDGE_IDLE;
                    end
                end

                BRIDGE_SPI: begin
                    if (!spi_enabled) begin
                        state <= BRIDGE_IDLE;
                    end else if (spi_byte_idx < 2'd3) begin
                        // Read SPI packet bytes: CS_SEL, DATA_L, DATA_H
                        case (spi_byte_idx)
                            2'd0: spi_cs_sel <= rx_buffer[rx_tail];
                            2'd1: spi_data_l <= rx_buffer[rx_tail];
                            2'd2: spi_data_h <= rx_buffer[rx_tail];
                            default: ;
                        endcase
                        rx_tail      <= rx_tail + 1'b1;
                        spi_byte_idx <= spi_byte_idx + 1'b1;
                    end else begin
                        // Initiate SPI transfer
                        if (!spi_busy) begin
                            spi_tx_data  <= spi_data_l;
                            spi_tx_valid <= 1'b1;
                            state        <= BRIDGE_IDLE;
                        end
                    end
                end

                BRIDGE_MAC: begin
                    if (!mac_enabled) begin
                        state <= BRIDGE_IDLE;
                    end else if (mac_byte_idx < 2'd2) begin
                        // Read MAC packet bytes: OP, UNIT
                        case (mac_byte_idx)
                            2'd0: mac_op   <= rx_buffer[rx_tail];
                            2'd1: mac_unit <= rx_buffer[rx_tail];
                            default: ;
                        endcase
                        rx_tail      <= rx_tail + 1'b1;
                        mac_byte_idx <= mac_byte_idx + 1'b1;
                    end else begin
                        // MAC operation placeholder
                        state <= BRIDGE_IDLE;
                    end
                end

                default: begin
                    state <= BRIDGE_IDLE;
                end
            endcase
        end
    end

endmodule
