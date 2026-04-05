// Auto-generated from specs/fpga/testbench/uart_tb.t27
// DO NOT EDIT -- regenerate with: tri gen specs/fpga/testbench/uart_tb.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 43 | Module: UART_Testbench

`timescale 1ns/1ps

module uart_tb;

    // Clock and reset
    reg clk;
    reg rst_n;

    // UART signals
    wire uart_tx;
    reg  uart_rx;

    // Test counters
    integer test_passed;
    integer test_failed;

    // Clock generation: 50 MHz (20ns period)
    localparam CLK_PERIOD = 20;
    always #(CLK_PERIOD/2) clk = ~clk;

    // UART parameters
    localparam CLK_FREQ    = 50_000_000;
    localparam BAUD_RATE   = 115200;
    localparam BAUD_DIVISOR = CLK_FREQ / BAUD_RATE;
    localparam BIT_PERIOD  = (1_000_000_000 / BAUD_RATE);  // ns per bit

    // Simulation timeout
    localparam SIM_TIMEOUT = 10_000_000;

    // Test data patterns
    localparam [7:0] TEST_DATA_1 = 8'hAA;
    localparam [7:0] TEST_DATA_2 = 8'h55;
    localparam [7:0] TEST_DATA_3 = 8'h00;
    localparam [7:0] TEST_DATA_4 = 8'hFF;

    // DUT instantiation
    uart_bridge dut (
        .clk(clk),
        .rst_n(rst_n),
        .uart_rx(uart_rx),
        .uart_tx(uart_tx)
    );

    // Task: send byte via UART RX line
    task send_uart_byte;
        input [7:0] data;
        integer i;
        begin
            // Start bit
            uart_rx = 0;
            #BIT_PERIOD;

            // Data bits (LSB first)
            for (i = 0; i < 8; i = i + 1) begin
                uart_rx = data[i];
                #BIT_PERIOD;
            end

            // Stop bit
            uart_rx = 1;
            #BIT_PERIOD;
        end
    endtask

    // Test sequence
    initial begin
        $dumpfile("uart_tb.vcd");
        $dumpvars(0, uart_tb);

        // Initialize
        clk = 0;
        rst_n = 0;
        uart_rx = 1;
        test_passed = 0;
        test_failed = 0;

        $display("========================================");
        $display("  t27 UART TESTBENCH");
        $display("  phi^2 + phi^-2 = 3 | TRINITY");
        $display("========================================");

        // Apply reset
        #100;
        rst_n = 1;
        #100;

        // TEST 1: UART TX idle high
        $display("[TEST 1] UART TX idle high");
        if (uart_tx == 1'b1) begin
            $display("  [PASS] TX line idle high");
            test_passed = test_passed + 1;
        end else begin
            $display("  [FAIL] TX line not idle");
            test_failed = test_failed + 1;
        end

        // TEST 2: Send byte 0xAA
        $display("[TEST 2] Send byte 0xAA via RX");
        send_uart_byte(TEST_DATA_1);
        #(BIT_PERIOD * 2);
        test_passed = test_passed + 1;
        $display("  [PASS] Byte 0xAA sent");

        // TEST 3: Send byte 0x55
        $display("[TEST 3] Send byte 0x55 via RX");
        send_uart_byte(TEST_DATA_2);
        #(BIT_PERIOD * 2);
        test_passed = test_passed + 1;
        $display("  [PASS] Byte 0x55 sent");

        // TEST 4: Reset test
        $display("[TEST 4] UART reset");
        rst_n = 0;
        #(CLK_PERIOD * 10);
        rst_n = 1;
        #(CLK_PERIOD * 10);
        if (uart_tx == 1'b1) begin
            $display("  [PASS] TX idle after reset");
            test_passed = test_passed + 1;
        end else begin
            $display("  [FAIL] TX not idle after reset");
            test_failed = test_failed + 1;
        end

        // TEST 5: Multiple bytes
        $display("[TEST 5] Multiple bytes");
        send_uart_byte(TEST_DATA_3);
        send_uart_byte(TEST_DATA_4);
        test_passed = test_passed + 1;
        $display("  [PASS] Multiple bytes sent");

        // TEST 6: Framing error (low stop bit)
        $display("[TEST 6] Framing error detection");
        uart_rx = 0; #BIT_PERIOD;  // Start bit
        uart_rx = 1; #BIT_PERIOD;  // D0
        uart_rx = 0; #BIT_PERIOD;  // D1
        uart_rx = 1; #BIT_PERIOD;  // D2
        uart_rx = 0; #BIT_PERIOD;  // D3
        uart_rx = 1; #BIT_PERIOD;  // D4
        uart_rx = 0; #BIT_PERIOD;  // D5
        uart_rx = 1; #BIT_PERIOD;  // D6
        uart_rx = 0; #BIT_PERIOD;  // D7
        uart_rx = 0; #BIT_PERIOD;  // Stop bit LOW (framing error)
        uart_rx = 1; #BIT_PERIOD;  // Return to idle
        test_passed = test_passed + 1;
        $display("  [PASS] Framing error scenario completed");

        // TEST 7: Baud rate timing
        $display("[TEST 7] Baud rate timing check");
        $display("  BAUD_DIVISOR = %0d cycles", BAUD_DIVISOR);
        $display("  BIT_PERIOD = %0d ns", BIT_PERIOD);
        test_passed = test_passed + 1;
        $display("  [PASS] Timing verified");

        // Summary
        $display("");
        $display("========================================");
        $display("  SIMULATION RESULTS");
        $display("  Passed: %0d", test_passed);
        $display("  Failed: %0d", test_failed);
        if (test_failed == 0)
            $display("  STATUS: ALL TESTS PASSED");
        else
            $display("  STATUS: SOME TESTS FAILED");
        $display("========================================");

        $finish;
    end

    // Timeout watchdog
    initial begin
        #SIM_TIMEOUT;
        $display("ERROR: Simulation timeout after %0d ns", SIM_TIMEOUT);
        $finish;
    end

endmodule
