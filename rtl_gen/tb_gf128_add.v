// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_gf128_add.v
// Testbench for GF128 Addition Unit (phi_dist = 0.010)

`timescale 1ns / 1ps

module tb_gf128_add;
    reg [127:0] a;
    reg [127:0] b;
    wire [127:0] result;

    gf128_add dut (
        .a(a),
        .b(b),
        .result(result)
    );

    integer i;
    integer errors;

    initial begin
        $dumpfile("tb_gf128_add.vcd");
        $dumpvars(0, tb_gf128_add);

        a = 128'h0;
        b = 128'h0;
        errors = 0;

        // Test: 0 + 0 = 0
        a = 128'h00000000000000000000000000000000;
        b = 128'h00000000000000000000000000000000;
        #1;
        if (result !== 128'h00000000000000000000000000000000) begin
            $display("ERROR: zero_plus_zero expected 0x0");
            errors = errors + 1;
        end else begin
            $display("PASS: zero_plus_zero");
        end

        // Test: +inf + -inf = NaN
        a = 128'h7FFFFFFFFFFFFFFFF800;
        b = 128'hFFFFFFFFFFFFFFFFFFFFF800;
        #1;
        if (result !== 128'hFFFFFFFFFFFFFFFFFFFFF801) begin
            $display("ERROR: inf_minus_inf expected NaN");
            errors = errors + 1;
        end else begin
            $display("PASS: inf_minus_inf");
        end

        // Test: NaN + x = NaN
        a = 128'hFFFFFFFFFFFFFFFFFFFFF801;
        b = 128'h3FFFFFFFFFFFFFE000000000000000;
        #1;
        if (result !== 128'hFFFFFFFFFFFFFFFFFFFFF801) begin
            $display("ERROR: nan_plus_x expected NaN");
            errors = errors + 1;
        end else begin
            $display("PASS: nan_plus_x");
        end

        // Test: -inf + finite = -inf
        a = 128'hFFFFFFFFFFFFFFFFFFFFF800;
        b = 128'h20000000000000000000000000000000;
        #1;
        if (result !== 128'hFFFFFFFFFFFFFFFFFFFFF800) begin
            $display("ERROR: minus_inf_plus_finite expected -inf");
            errors = errors + 1;
        end else begin
            $display("PASS: minus_inf_plus_finite");
        end

        // Summary
        $display("");
        $display("=== GF128 Add Test Summary (phi_dist: 0.010) ===");
        $display("Total: 4, Passed: %0d, Failed: %0d", 4 - errors, errors);

        if (errors == 0) begin
            $display("SUCCESS: All tests passed!");
        end else begin
            $display("FAILURE: Some tests failed!");
        end

        $finish;
    end

endmodule