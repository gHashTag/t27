// Auto-generated from specs/fpga/top_level.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/top_level.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: Trinity_FPGA_Top
// Synthesizable Verilog for Trinity FPGA top-level module
// Target: xc7a100t (Artix-7), Package: fgg676c, Clock: 50 MHz

module trinity_fpga_top #(
    parameter CLK_FREQ       = 50_000_000,
    parameter UART_BAUD      = 115200,
    parameter MAC_DATA_WIDTH = 27,
    parameter NUM_LEDS       = 4,
    parameter TRIT_BITS      = 2,
    parameter WORD_BITS      = MAC_DATA_WIDTH * TRIT_BITS  // 54 bits
)(
    // Clock and reset
    input  wire                     clk,        // E19 - 50 MHz system clock
    input  wire                     rst_n,      // C12 - Active-low reset

    // UART
    input  wire                     uart_rx,    // L20 - UART RX from host
    output wire                     uart_tx,    // K20 - UART TX to host

    // SPI
    input  wire                     spi_miso,   // J13 - SPI MISO from device
    output wire                     spi_cs,     // G13 - SPI chip select
    output wire                     spi_sck,    // K13 - SPI clock
    output wire                     spi_mosi,   // H13 - SPI MOSI to device

    // Status LEDs (active-low)
    output reg  [NUM_LEDS-1:0]      led,        // R5, T5, T8, T9

    // MAC interface
    input  wire [WORD_BITS-1:0]     mac_a,      // MAC operand A (27 trits)
    input  wire [WORD_BITS-1:0]     mac_b,      // MAC operand B (27 trits)
    input  wire signed [31:0]       mac_acc_in, // MAC accumulator input
    input  wire                     mac_op_valid,
    input  wire [2:0]              mac_unit_sel,
    input  wire [1:0]              mac_op_sel,
    input  wire                     mac_acc_reset,
    output wire [WORD_BITS-1:0]     mac_mul_result,
    output wire                     mac_mul_valid,
    output wire signed [31:0]       mac_acc_out,
    output wire                     mac_acc_valid,
    output wire [1:0]              mac_status
);

    // =================================================================
    // LED indices
    // =================================================================
    localparam LED_UART_TX  = 0;
    localparam LED_SPI_CS   = 1;
    localparam LED_MAC_BUSY = 2;
    localparam LED_HEARTBEAT = 3;

    // =================================================================
    // Internal state
    // =================================================================
    reg        reset_pending;
    reg  [3:0] led_state;           // Internal LED state (active-low)
    reg [31:0] heartbeat_cnt;       // Heartbeat counter

    localparam [31:0] BLINK_THRESHOLD = CLK_FREQ / 2;  // 1 Hz blink

    // =================================================================
    // Submodule: UART Bridge
    // =================================================================
    wire       uart_tx_wire;
    wire       uart_tx_busy;

    uart_bridge #(
        .CLK_FREQ   (CLK_FREQ),
        .BAUD       (UART_BAUD)
    ) u_uart_bridge (
        .clk        (clk),
        .rst_n      (rst_n),
        .rx         (uart_rx),
        .tx         (uart_tx_wire),
        .tx_busy    (uart_tx_busy)
    );

    assign uart_tx = uart_tx_wire;

    // =================================================================
    // Submodule: SPI Master
    // =================================================================
    wire       spi_cs_wire;
    wire       spi_sck_wire;
    wire       spi_mosi_wire;
    wire       spi_busy;

    spi_master u_spi_master (
        .clk        (clk),
        .rst_n      (rst_n),
        .miso       (spi_miso),
        .cs         (spi_cs_wire),
        .sck        (spi_sck_wire),
        .mosi       (spi_mosi_wire),
        .busy       (spi_busy)
    );

    assign spi_cs   = spi_cs_wire;
    assign spi_sck  = spi_sck_wire;
    assign spi_mosi = spi_mosi_wire;

    // =================================================================
    // Submodule: ZeroDSP MAC
    // =================================================================

    zerodsp_mac #(
        .MAC_WIDTH       (MAC_DATA_WIDTH),
        .MAC_ACC_BITS    (32),
        .NUM_MAC_UNITS   (8),
        .PIPELINE_STAGES (4)
    ) u_mac (
        .clk         (clk),
        .rst_n       (rst_n),
        .a_word      (mac_a),
        .b_word      (mac_b),
        .unit_sel    (mac_unit_sel),
        .op_sel      (mac_op_sel),
        .op_valid    (mac_op_valid),
        .acc_reset   (mac_acc_reset),
        .mul_result  (mac_mul_result),
        .mul_valid   (mac_mul_valid),
        .acc_result  (mac_acc_out),
        .acc_valid   (mac_acc_valid),
        .unit_status (mac_status)
    );

    // =================================================================
    // LED activity indicators
    // =================================================================

    // LED 0: UART TX activity (active when transmitting)
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            led_state[LED_UART_TX] <= 1'b1;  // Off (active-low)
        else
            led_state[LED_UART_TX] <= ~uart_tx_busy;  // Active-low: invert
    end

    // LED 1: SPI CS active
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            led_state[LED_SPI_CS] <= 1'b1;   // Off (active-low)
        else
            led_state[LED_SPI_CS] <= spi_cs_wire;  // CS is active-low too
    end

    // LED 2: MAC busy
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            led_state[LED_MAC_BUSY] <= 1'b1; // Off (active-low)
        else
            led_state[LED_MAC_BUSY] <= ~(mac_status == 2'b01);  // Invert busy
    end

    // =================================================================
    // Heartbeat logic (LED 3 blinks at 1 Hz)
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            heartbeat_cnt <= 32'd0;
            led_state[LED_HEARTBEAT] <= 1'b1;  // Off (active-low)
        end else begin
            heartbeat_cnt <= heartbeat_cnt + 32'd1;
            if (heartbeat_cnt >= BLINK_THRESHOLD) begin
                heartbeat_cnt <= 32'd0;
                led_state[LED_HEARTBEAT] <= ~led_state[LED_HEARTBEAT];
            end
        end
    end

    // =================================================================
    // Reset pending logic
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            reset_pending <= 1'b1;
        else
            reset_pending <= 1'b0;
    end

    // =================================================================
    // LED output assignment
    // =================================================================
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            led <= {NUM_LEDS{1'b1}};  // All off (active-low)
        else
            led <= led_state;
    end

endmodule
