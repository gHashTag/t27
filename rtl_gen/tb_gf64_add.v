// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf64_add.v
// Testbench for GF64 Addition Unit (BEST phi_dist = 0.003)

`timescale 1ns / 1ps

module tb_gf64_add;
    reg [63:0] a;
    reg [63:0] b;
    wire [63:0] result;

    gf64_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf64_add.vcd");
        $dumpvars(0, tb_gf64_add);

        a = 64'h0;
        b = 64'h0;
        errors = 0;

        // Test: 0 + 0 = 0
        a = 64'h0000000000000000;  // +0.0
        b = 64'h0000000000000000;  // +0.0
        #1;
        if (result !== 64'h0000000000000000) begin
            $display("ERROR: zero_plus_zero expected 0x0 got 0x%016X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_plus_zero");
        end

        // Test: +inf + -inf = NaN
        a = 64'h7FFFFFFFFFF800;   // +inf
        b = 64'hFFFFFFFFFFF800;   // -inf
        #1;
        if (result !== 64'hFFFFFFFFFFF801) begin
            $display("ERROR: inf_minus_inf expected 0xFFFFFFFFFFF801 got 0x%016X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_minus_inf");
        end

        // Test: NaN + x = NaN
        a = 64'hFFFFFFFFFFF801;   // NaN
        b = 64'h3FFFE0000000000;
        #1;
        if (result !== 64'hFFFFFFFFFFF801) begin
            $display("ERROR: nan_plus_x expected 0xFFFFFFFFFFF801 got 0x%016X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_plus_x");
        end

        // Test: -inf + finite = -inf
        a = 64'hFFFFFFFFFFF800;   // -inf
        b = 64'h2000000000000000;
        #1;
        if (result !== 64'hFFFFFFFFFFF800) begin
            $display("ERROR: minus_inf_plus_finite expected 0xFFFFFFFFFFF800 got 0x%016X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_plus_finite");
        end

        // Summary
        $display("");
        $display("=== GF64 Add Test Summary (BEST phi_dist: 0.003) ===");
        $display("Total: 4, Passed: %0d, Failed: %0d", 4 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule