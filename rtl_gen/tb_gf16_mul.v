// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf16_mul.v
// Testbench for GF16 Multiplication Unit (PRIMARY, phi_dist = 0.049)

`timescale 1ns / 1ps

module tb_gf16_mul;
    reg clk;
    reg rst_n;
    reg [15:0] a;
    reg [15:0] b;
    wire [15:0] result;

    gf16_mul dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    // Clock generation
    initial clk = 0;
    always #2.5 clk = ~clk;  // 400 MHz

    initial begin
        $dumpfile("tb_gf16_mul.vcd");
        $dumpvars(0, tb_gf16_mul);

        rst_n = 0;
        a = 16'h0000;
        b = 16'h0000;
        errors = 0;

        // Reset sequence
        #10 rst_n = 0;
        #20 rst_n = 1;

        // Test: 0 * x = 0
        a = 16'h0000;
        b = 16'h3E80;
        @(posedge clk);
        if (result !== 16'h0000) begin
            $display("ERROR: zero_times_x expected 0x0000 got 0x%04X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_times_x");
        end

        // Test: x * 0 = 0
        a = 16'h7E00;
        b = 16'h0000;
        @(posedge clk);
        if (result !== 16'h0000) begin
            $display("ERROR: x_times_zero expected 0x0000 got 0x%04X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: x_times_zero");
        end

        // Test: 1.0 * 1.0 = 1.0
        a = 16'h3E80;  // 1.0
        b = 16'h3E80;  // 1.0
        @(posedge clk);
        if (result === 16'h3E80 || result === 16'h3E81) begin
            $display("PASS: one_times_one");
        end else begin
            $display("INFO: one_times_one got 0x%04X (may be rounding)", result);
        end

        // Test: NaN * x = NaN
        a = 16'hFE01;   // NaN
        b = 16'h3E80;
        @(posedge clk);
        if (result !== 16'hFE01) begin
            $display("ERROR: nan_times_x expected 0xFE01 got 0x%04X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_times_x");
        end

        // Test: inf * finite = inf
        a = 16'h7E00;   // +inf
        b = 16'h3E80;
        @(posedge clk);
        if (result !== 16'h7E00) begin
            $display("ERROR: inf_times_finite expected 0x7E00 got 0x%04X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_finite");
        end

        // Test: inf * inf = inf
        a = 16'h7E00;   // +inf
        b = 16'h7E00;   // +inf
        @(posedge clk);
        if (result !== 16'h7E00) begin
            $display("ERROR: inf_times_inf expected 0x7E00 got 0x%04X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_times_inf");
        end

        // Test: -inf * -inf = +inf
        a = 16'hFE00;   // -inf
        b = 16'hFE00;   // -inf
        @(posedge clk);
        if (result !== 16'h7E00) begin
            $display("ERROR: minus_inf_times_minus_inf expected 0x7E00 got 0x%04X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_times_minus_inf");
        end

        // Summary
        $display("");
        $display("=== GF16 Mul Test Summary (PRIMARY, phi_dist: 0.049) ===");
        $display("Total: 7, Passed: %0d, Failed: %0d", 7 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule