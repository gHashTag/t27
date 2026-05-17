// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf32_add.v
// Testbench for GF32 Addition Unit (phi_dist = 0.014)

`timescale 1ns / 1ps

module tb_gf32_add;
    reg [31:0] a;
    reg [31:0] b;
    wire [31:0] result;

    gf32_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf32_add.vcd");
        $dumpvars(0, tb_gf32_add);

        a = 32'h0;
        b = 32'h0;
        errors = 0;

        // Test: 0 + 0 = 0
        a = 32'h00000000;  // +0.0
        b = 32'h00000000;  // +0.0
        #1;
        if (result !== 32'h00000000) begin
            $display("ERROR: zero_plus_zero expected 0x00000000 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_plus_zero");
        end

        // Test: +inf + -inf = NaN
        a = 32'h7FFF8000;   // +inf
        b = 32'hFFFFF800;   // -inf
        #1;
        if (result !== 32'hFFFFF801) begin
            $display("ERROR: inf_minus_inf expected 0xFFFFF801 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_minus_inf");
        end

        // Test: NaN + x = NaN
        a = 32'hFFFFF801;   // NaN
        b = 32'h3FFE0000;
        #1;
        if (result !== 32'hFFFFF801) begin
            $display("ERROR: nan_plus_x expected 0xFFFFF801 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_plus_x");
        end

        // Test: -inf + finite = -inf
        a = 32'hFFFFF800;   // -inf
        b = 32'h20000000;
        #1;
        if (result !== 32'hFFFFF800) begin
            $display("ERROR: minus_inf_plus_finite expected 0xFFFFF800 got 0x%08X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_plus_finite");
        end

        // Test: Large + small = large (no overflow)
        a = 32'h7FFF0000;   // Large positive
        b = 32'h00000001;   // Tiny positive
        #1;
        if (result >= a) begin
            $display("PASS: large_plus_small_no_overflow");
        end else begin
            $display("ERROR: large_plus_small_no_overflow got 0x%08X", result);
            errors = errors + 1;
        end

        // Summary
        $display("");
        $display("=== GF32 Add Test Summary (phi_dist: 0.014) ===");
        $display("Total: 5, Passed: %0d, Failed: %0d", 5 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule