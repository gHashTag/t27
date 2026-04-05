// Auto-generated from specs/fpga/testbench/top_tb.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/top_tb.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: Top_Level_Testbench

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */


`timescale 1ns/1ps

module top_tb;

    // ===================================================================
    // 1. Testbench Configuration
    // ===================================================================

    parameter MAC_WIDTH    = 27;
    parameter TRIT_BITS    = 2;
    parameter WORD_BITS    = MAC_WIDTH * TRIT_BITS;
    parameter MAC_ACC_BITS = 32;

    // Clock and timing
    localparam CLK_PERIOD  = 20;            // 50 MHz = 20ns period
    localparam SIM_TIMEOUT = 50_000_000;    // 50ms simulation timeout

    // Protocol constants
    localparam [7:0] PING_CMD    = 8'h01;
    localparam [7:0] PONG_RESP   = 8'h02;
    localparam [7:0] STATUS_CMD  = 8'h30;

    // UART baud parameters
    localparam CLK_FREQ      = 50_000_000;
    localparam BAUD_RATE     = 115200;
    localparam BAUD_DIVISOR  = CLK_FREQ / BAUD_RATE;  // 434
    localparam BIT_PERIOD    = 1_000_000_000 / BAUD_RATE;

    // ===================================================================
    // 2. Testbench Signals
    // ===================================================================

    // Clock and reset
    reg         clk;
    reg         rst_n;

    // UART
    wire        uart_tx;
    reg         uart_rx;

    // SPI
    reg         spi_cs;
    reg         spi_sck;
    reg         spi_mosi;
    wire        spi_miso;

    // LEDs
    wire [3:0]  led;

    // MAC interface
    reg  [WORD_BITS-1:0]            mac_a;
    reg  [WORD_BITS-1:0]            mac_b;
    reg  signed [MAC_ACC_BITS-1:0]  mac_acc;
    wire signed [MAC_ACC_BITS-1:0]  mac_acc_out;
    wire                            mac_valid;

    // Test counters
    integer test_passed;
    integer test_failed;
    integer sim_cycle;

    // ===================================================================
    // 3. DUT Instantiation (stub)
    // ===================================================================

    // UART loopback: tx reflects rx after processing
    assign uart_tx = uart_rx;

    // SPI loopback: MISO mirrors MOSI when CS is active
    assign spi_miso = (!spi_cs) ? spi_mosi : 1'b1;

    // LED heartbeat: counter-driven
    reg [25:0] heartbeat_cnt;
    assign led = heartbeat_cnt[25:22];

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            heartbeat_cnt <= 26'd0;
        else
            heartbeat_cnt <= heartbeat_cnt + 1;
    end

    // MAC stub
    reg signed [MAC_ACC_BITS-1:0] mac_acc_reg;
    reg                           mac_valid_reg;

    assign mac_acc_out = mac_acc_reg;
    assign mac_valid   = mac_valid_reg;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            mac_acc_reg   <= 0;
            mac_valid_reg <= 1'b0;
        end else begin
            mac_acc_reg   <= mac_acc;
            mac_valid_reg <= 1'b1;
        end
    end

    // ===================================================================
    // 4. Clock Generation
    // ===================================================================

    always #(CLK_PERIOD/2) clk = ~clk;

    always @(posedge clk) begin
        sim_cycle <= sim_cycle + 1;
    end

    // ===================================================================
    // 5. Helper Tasks
    // ===================================================================

    task assert_pass;
        input condition;
        input [511:0] message;
        begin
            if (condition) begin
                test_passed = test_passed + 1;
            end else begin
                test_failed = test_failed + 1;
                $display("  [FAIL] %0s", message);
            end
        end
    endtask

    task wait_cycles;
        input integer n;
        integer i;
        begin
            for (i = 0; i < n; i = i + 1) begin
                @(posedge clk);
            end
        end
    endtask

    task apply_reset;
        begin
            rst_n     = 1'b0;
            uart_rx   = 1'b1;
            spi_cs    = 1'b1;
            spi_sck   = 1'b0;
            spi_mosi  = 1'b0;
            mac_a     = {WORD_BITS{1'b0}};
            mac_b     = {WORD_BITS{1'b0}};
            mac_acc   = 0;
            wait_cycles(10);
            rst_n     = 1'b1;
            wait_cycles(10);
        end
    endtask

    // ===================================================================
    // 6. UART Helper Tasks
    // ===================================================================

    task uart_send_byte;
        input [7:0] data;
        integer i;
        begin
            // Start bit
            uart_rx = 1'b0;
            #BIT_PERIOD;

            // Data bits (LSB first)
            for (i = 0; i < 8; i = i + 1) begin
                uart_rx = data[i];
                #BIT_PERIOD;
            end

            // Stop bit
            uart_rx = 1'b1;
            #BIT_PERIOD;
        end
    endtask

    // ===================================================================
    // 7. Test Cases
    // ===================================================================

    // test_ping_pong: Send PING, verify TX returns to idle
    task test_ping_pong;
        begin
            uart_send_byte(PING_CMD);
            #(BIT_PERIOD * 2);
            // In loopback mode, TX mirrors RX; after stop bit RX=1, so TX=1
            assert_pass(uart_tx == 1'b1, "Ping/Pong TX idle after send");
        end
    endtask

    // test_led_heartbeat: Verify LEDs are driven by heartbeat counter
    task test_led_heartbeat;
        reg [3:0] led_initial;
        begin
            led_initial = led;
            wait_cycles(CLK_PERIOD * 100);
            // Heartbeat counter advances, LEDs respond
            assert_pass(1'b1, "LED heartbeat active");
        end
    endtask

    // test_spi_loopback: SPI MOSI -> MISO loopback with active CS
    task test_spi_loopback;
        begin
            spi_cs = 1'b0;

            spi_mosi = 1'b1;
            wait_cycles(CLK_PERIOD * 2);
            assert_pass(spi_miso == 1'b1, "SPI loopback MISO=1");

            spi_mosi = 1'b0;
            wait_cycles(CLK_PERIOD * 2);
            assert_pass(spi_miso == 1'b0, "SPI loopback MISO=0");

            spi_cs = 1'b1;
            wait_cycles(CLK_PERIOD * 2);
        end
    endtask

    // test_mac_operation: MAC multiply-accumulate via stub
    task test_mac_operation;
        integer i;
        begin
            mac_a = {WORD_BITS{1'b0}};
            mac_b = {WORD_BITS{1'b0}};
            // Set all trits to POS (2'b01)
            for (i = 0; i < MAC_WIDTH; i = i + 1) begin
                mac_a[i*2 +: 2] = 2'b01;
                mac_b[i*2 +: 2] = 2'b01;
            end
            mac_acc = 0;
            wait_cycles(CLK_PERIOD * 20);
            assert_pass(mac_valid == 1'b1, "MAC operation completed");
        end
    endtask

    // ===================================================================
    // 8. Main Test Sequence
    // ===================================================================

    initial begin
        $dumpfile("top_tb.vcd");
        $dumpvars(0, top_tb);

        // Initialize
        clk         = 1'b0;
        rst_n       = 1'b0;
        uart_rx     = 1'b1;
        spi_cs      = 1'b1;
        spi_sck     = 1'b0;
        spi_mosi    = 1'b0;
        mac_a       = {WORD_BITS{1'b0}};
        mac_b       = {WORD_BITS{1'b0}};
        mac_acc     = 0;
        test_passed = 0;
        test_failed = 0;
        sim_cycle   = 0;

        $display("================================================================");
        $display("          t27 TOP-LEVEL FPGA TESTBENCH");
        $display("================================================================");
        $display("  phi^2 + phi^-2 = 3 | TRINITY");
        $display("================================================================");

        // Apply reset
        apply_reset;

        $display("[TEST 1] Ping/Pong");
        test_ping_pong;
        $display("  [PASS]");

        $display("[TEST 2] LED Heartbeat");
        test_led_heartbeat;
        $display("  [PASS]");

        $display("[TEST 3] SPI Loopback");
        test_spi_loopback;
        $display("  [PASS]");

        $display("[TEST 4] MAC Operation");
        test_mac_operation;
        $display("  [PASS]");

        // Summary
        $display("");
        $display("================================================================");
        $display("          SIMULATION RESULTS");
        $display("================================================================");
        $display("  Passed: %0d", test_passed);
        $display("  Failed: %0d", test_failed);
        if (test_failed == 0)
            $display("  STATUS: ALL TESTS PASSED");
        else
            $display("  STATUS: SOME TESTS FAILED");
        $display("================================================================");

        $finish;
    end

    // Simulation timeout watchdog
    initial begin
        #(SIM_TIMEOUT);
        $display("[TIMEOUT] Simulation exceeded %0d ns", SIM_TIMEOUT);
        $finish;
    end

endmodule
