// Auto-generated from specs/fpga/spi.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/spi.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: SPI_Master
// Synthesizable Verilog for SPI Master, Mode 0 (CPOL=0, CPHA=0)
// SCK idle low, sample on rising edge, shift on falling edge

module spi_master #(
    parameter CLK_FREQ          = 50000000, // 50 MHz system clock
    parameter MAX_DATA_WIDTH    = 32,       // Max bits per transfer
    parameter CS_ASSERT_DELAY   = 100,      // CS to SCK delay (ns)
    parameter CS_DEASSERT_DELAY = 100       // SCK to CS delay (ns)
)(
    input  wire        clk,
    input  wire        rst_n,

    // --- SPI bus ---
    input  wire        miso,
    output reg         cs,
    output reg         sck,
    output reg         mosi,

    // --- Control interface ---
    input  wire [2:0]  prescaler_sel,       // Prescaler selection (0-7)
    input  wire [4:0]  data_width_cfg,      // Data width (1-32)
    input  wire [31:0] tx_data,             // Transmit data
    input  wire        start,               // Start transfer pulse
    output reg  [31:0] rx_data,             // Received data
    output reg         busy                 // Transfer in progress
);

    // =================================================================
    // SPI states
    // =================================================================
    localparam [1:0] ST_IDLE       = 2'd0;
    localparam [1:0] ST_CS_ASSERT  = 2'd1;
    localparam [1:0] ST_TRANSFER   = 2'd2;
    localparam [1:0] ST_CS_DEASSERT = 2'd3;

    // Transfer sub-states
    localparam [1:0] TX_BIT    = 2'd0;
    localparam [1:0] RX_BIT    = 2'd1;
    localparam [1:0] WAIT_EDGE = 2'd2;

    // =================================================================
    // Prescaler divider lookup (combinational)
    // =================================================================
    reg [8:0] prescaler_div;
    always @(*) begin
        case (prescaler_sel)
            3'd0: prescaler_div = 9'd2;
            3'd1: prescaler_div = 9'd4;
            3'd2: prescaler_div = 9'd8;
            3'd3: prescaler_div = 9'd16;
            3'd4: prescaler_div = 9'd32;
            3'd5: prescaler_div = 9'd64;
            3'd6: prescaler_div = 9'd128;
            3'd7: prescaler_div = 9'd256;
            default: prescaler_div = 9'd16;
        endcase
    end

    // Half-period = prescaler_div / 2
    wire [7:0] half_period = prescaler_div[8:1];

    // =================================================================
    // CS delay cycle counts
    // Delay (ns) * CLK_FREQ / 1e9
    // =================================================================
    localparam CS_ASSERT_CYCLES  = (CS_ASSERT_DELAY * (CLK_FREQ / 1000000)) / 1000;
    localparam CS_DEASSERT_CYCLES = (CS_DEASSERT_DELAY * (CLK_FREQ / 1000000)) / 1000;

    // =================================================================
    // State registers
    // =================================================================
    reg [1:0]  state;
    reg [1:0]  tx_state;
    reg [31:0] tx_shift;        // TX shift register
    reg [4:0]  data_w;          // Latched data width
    reg [4:0]  bit_count;       // Bits transferred
    reg [31:0] bit_counter;     // Half-cycle counter
    reg [31:0] cs_assert_cnt;   // CS assert delay counter
    reg [31:0] cs_deassert_cnt; // CS deassert delay counter

    // =================================================================
    // Main state machine
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state          <= ST_IDLE;
            tx_state       <= TX_BIT;
            cs             <= 1'b1;     // CS active low, idle high
            sck            <= 1'b0;     // Mode 0: idle low
            mosi           <= 1'b0;
            busy           <= 1'b0;
            rx_data        <= 32'd0;
            tx_shift       <= 32'd0;
            data_w         <= 5'd8;
            bit_count      <= 5'd0;
            bit_counter    <= 32'd0;
            cs_assert_cnt  <= 32'd0;
            cs_deassert_cnt <= 32'd0;
        end else begin
            case (state)
                // -------------------------------------------------
                // IDLE: wait for start pulse
                // -------------------------------------------------
                ST_IDLE: begin
                    sck  <= 1'b0;  // Mode 0: idle low
                    mosi <= 1'b0;
                    if (start && !busy) begin
                        tx_shift       <= tx_data;
                        data_w         <= data_width_cfg;
                        rx_data        <= 32'd0;
                        bit_count      <= 5'd0;
                        bit_counter    <= 32'd0;
                        cs_assert_cnt  <= 32'd0;
                        busy           <= 1'b1;
                        state          <= ST_CS_ASSERT;
                    end
                end

                // -------------------------------------------------
                // CS_ASSERT: drive CS low, wait delay
                // -------------------------------------------------
                ST_CS_ASSERT: begin
                    cs <= 1'b0;  // Assert CS (active low)
                    cs_assert_cnt <= cs_assert_cnt + 32'd1;
                    if (cs_assert_cnt >= CS_ASSERT_CYCLES) begin
                        cs_assert_cnt <= 32'd0;
                        tx_state      <= TX_BIT;
                        bit_counter   <= 32'd0;
                        // Drive first bit on MOSI (MSB first)
                        mosi  <= tx_shift[data_w - 1];
                        state <= ST_TRANSFER;
                    end
                end

                // -------------------------------------------------
                // TRANSFER: clock data in/out
                // -------------------------------------------------
                ST_TRANSFER: begin
                    bit_counter <= bit_counter + 32'd1;

                    case (tx_state)
                        TX_BIT: begin
                            // SCK low phase: setup MOSI
                            sck <= 1'b0;
                            mosi <= tx_shift[data_w - bit_count - 1];
                            if (bit_counter >= {24'd0, half_period} - 1) begin
                                bit_counter <= 32'd0;
                                tx_state    <= RX_BIT;
                            end
                        end

                        RX_BIT: begin
                            // SCK high phase: sample MISO
                            sck <= 1'b1;
                            if (bit_counter >= {24'd0, half_period} - 1) begin
                                // Sample MISO on falling edge
                                rx_data   <= (rx_data << 1) | {31'd0, miso};
                                bit_count <= bit_count + 5'd1;
                                bit_counter <= 32'd0;

                                if (bit_count + 5'd1 >= data_w) begin
                                    tx_state <= WAIT_EDGE;
                                end else begin
                                    tx_state <= TX_BIT;
                                end
                            end
                        end

                        WAIT_EDGE: begin
                            // Final SCK low half-cycle
                            sck <= 1'b0;
                            if (bit_counter >= {24'd0, half_period} - 1) begin
                                bit_counter     <= 32'd0;
                                cs_deassert_cnt <= 32'd0;
                                state           <= ST_CS_DEASSERT;
                            end
                        end

                        default: begin
                            sck      <= 1'b0;
                            tx_state <= TX_BIT;
                        end
                    endcase
                end

                // -------------------------------------------------
                // CS_DEASSERT: wait delay, release CS
                // -------------------------------------------------
                ST_CS_DEASSERT: begin
                    sck <= 1'b0;
                    cs_deassert_cnt <= cs_deassert_cnt + 32'd1;
                    if (cs_deassert_cnt >= CS_DEASSERT_CYCLES) begin
                        cs_deassert_cnt <= 32'd0;
                        cs              <= 1'b1;  // Deassert CS
                        busy            <= 1'b0;
                        state           <= ST_IDLE;
                    end
                end

                default: begin
                    state <= ST_IDLE;
                end
            endcase
        end
    end

endmodule
