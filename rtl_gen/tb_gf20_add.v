// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf20_add.v
// Testbench for GF20 Addition Unit (phi_dist = 0.035)

`timescale 1ns / 1ps

module tb_gf20_add;
    reg [19:0] a;
    reg [19:0] b;
    wire [19:0] result;

    gf20_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf20_add.vcd");
        $dumpvars(0, tb_gf20_add);

        a = 20'h0;
        b = 20'h0;
        errors = 0;

        // Test: 0 + 0 = 0
        a = 20'h00000;  // +0.0
        b = 20'h00000;  // +0.0
        #1;
        if (result !== 20'h00000) begin
            $display("ERROR: zero_plus_zero expected 0x00000 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: zero_plus_zero");
        end

        // Test: +inf + -inf = NaN
        a = 20'h7F800;   // +inf
        b = 20'hFF800;   // -inf
        #1;
        if (result !== 20'hFF801) begin
            $display("ERROR: inf_minus_inf expected 0xFF801 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: inf_minus_inf");
        end

        // Test: NaN + x = NaN
        a = 20'hFF801;   // NaN
        b = 20'h3F000;
        #1;
        if (result !== 20'hFF801) begin
            $display("ERROR: nan_plus_x expected 0xFF801 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: nan_plus_x");
        end

        // Test: -inf + finite = -inf
        a = 20'hFF800;   // -inf
        b = 20'h20000;
        #1;
        if (result !== 20'hFF800) begin
            $display("ERROR: minus_inf_plus_finite expected 0xFF800 got 0x%05X", result);
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_plus_finite");
        end

        // Summary
        $display("");
        $display("=== GF20 Add Test Summary (phi_dist: 0.035) ===");
        $display("Total: 4, Passed: %0d, Failed: %0d", 4 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule