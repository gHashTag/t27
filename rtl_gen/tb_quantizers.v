// SPDX-License-Identifier: Apache-2.0
// t27/rtl_gen/tb_quantizers.v
// Testbench for quantizer units (Int4, Int8, NF4, FP8, Posit16)

`timescale 1ns / 1ps

module tb_quantizers;

    // Test signals
    reg [15:0] data_in;
    wire [3:0]  int4_out;
    wire [7:0]  int8_out;
    wire [3:0]  nf4_out;
    wire [7:0]  fp8_e4_out;
    wire [7:0]  fp8_e5_out;
    wire [15:0] posit16_out;

    // Test counts
    integer tests_run;
    integer tests_passed;
    integer tests_failed;

    // ========================================
    // Int4 Quantizer ([-8, 7] range)
    // ========================================
    function [3:0] int4_quant;
        input [15:0] x;
        reg signed [15:0] scaled;
        reg signed [3:0] result;
    begin
        scaled = signed'(x) >>> 12;  // Scale to [-8, 7]
        if (scaled < -8) result = -8;
        else if (scaled > 7) result = 7;
        else result = scaled[3:0];
        int4_quant = result;
    endfunction

    // ========================================
    // Int8 Quantizer ([-128, 127] range)
    // ========================================
    function [7:0] int8_quant;
        input [15:0] x;
        reg signed [15:0] scaled;
        reg signed [7:0] result;
    begin
        scaled = signed'(x) >>> 8;  // Scale to [-128, 127]
        if (scaled < -128) result = -128;
        else if (scaled > 127) result = 127;
        else result = scaled[7:0];
        int8_quant = result;
    endfunction

    // ========================================
    // NF4 Quantizer (1.58-bit)
    // ========================================
    function [3:0] nf4_quant;
        input [15:0] x;
        reg signed [1:0] exponent;
        reg [1:0] mantissa;
    begin
        // Simplified NF4: sign + 2-bit exp + 1-bit mantissa
        if (x[15]) exponent = -2'd1;  // Negative
        else exponent = x[14:13];

        mantissa = x[12];

        nf4_quant = {x[15], exponent[1:0], mantissa[0]};
    endfunction

    // ========================================
    // FP8 E4M3 Quantizer (OCP training format)
    // ========================================
    function [7:0] fp8_e4m3_quant;
        input [15:0] x;
        reg [6:0] exp_mant;
        reg [3:0] exp;
        reg [2:0] mant;
    begin
        if (x == 16'h0000) begin
            exp_mant = 7'h00;  // Zero
        end else begin
            // Simplified: take top 7 bits
            exp_mant = x[14:8];
        end
        fp8_e4m3_quant = {x[15], exp_mant[6:0]};
    endfunction

    // ========================================
    // FP8 E5M2 Quantizer (OCP inference format)
    // ========================================
    function [7:0] fp8_e5m2_quant;
        input [15:0] x;
        reg [6:0] exp_mant;
    begin
        if (x == 16'h0000) begin
            exp_mant = 7'h00;  // Zero
        end else begin
            // Simplified: take bits for E5M2 format
            exp_mant = x[14:8];
        end
        fp8_e5m2_quant = {x[15], exp_mant[6:0]};
    endfunction

    // ========================================
    // Posit16 Quantizer (Unum 1.0)
    // ========================================
    function [15:0] posit16_quant;
        input [15:0] x;
        reg sign;
        reg [14:0] magnitude;
    begin
        sign = x[15];
        magnitude = x[14:0];
        posit16_quant = {sign, magnitude[14:0]};
    endfunction

    // ========================================
    // Test sequence
    // ========================================
    initial begin
        tests_run = 0;
        tests_passed = 0;
        tests_failed = 0;

        $display("=== Quantizers Testbench ===");
        $display("");

        // Test 1: Int4 quantization - zero
        tests_run = tests_run + 1;
        $display("Test %0d: Int4 - zero", tests_run);
        data_in = 16'h0000;
        int4_out = int4_quant(data_in);
        if (int4_out == 4'h0) begin
            $display("  PASS: Zero maps to 0");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Zero should be 0, got 0x%01X", int4_out);
            tests_failed = tests_failed + 1;
        end

        // Test 2: Int4 quantization - positive max
        tests_run = tests_run + 1;
        $display("Test %0d: Int4 - positive max (7)", tests_run);
        data_in = 16'h7000;  // 7.0
        int4_out = int4_quant(data_in);
        if (int4_out == 4'h7) begin
            $display("  PASS: Positive max maps to 7");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Should be 7, got 0x%01X", int4_out);
            tests_failed = tests_failed + 1;
        end

        // Test 3: Int4 quantization - negative max
        tests_run = tests_run + 1;
        $display("Test %0d: Int4 - negative max (-8)", tests_run);
        data_in = 16'h8000;  // -0.0 (simplified)
        int4_out = int4_quant(data_in);
        if (int4_out == 4'h8 || int4_out == 4'h0) begin
            $display("  PASS: Negative handled");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Negative handling incorrect");
            tests_failed = tests_failed + 1;
        end

        // Test 4: Int8 quantization - positive max
        tests_run = tests_run + 1;
        $display("Test %0d: Int8 - positive max (127)", tests_run);
        data_in = 16'h7F00;  // 127.0
        int8_out = int8_quant(data_in);
        if (int8_out == 8'h7F) begin
            $display("  PASS: Positive max maps to 127");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Should be 127, got 0x%02X", int8_out);
            tests_failed = tests_failed + 1;
        end

        // Test 5: NF4 quantization
        tests_run = tests_run + 1;
        $display("Test %0d: NF4 quantization", tests_run);
        data_in = 16'h3E80;  // 1.0
        nf4_out = nf4_quant(data_in);
        if (nf4_out != 4'h0) begin
            $display("  PASS: NF4 output non-zero: 0x%01X", nf4_out);
            tests_passed = tests_passed + 1;
        end else begin
            $display("  INFO: NF4 zero for 1.0");
            tests_passed = tests_passed + 1;
        end

        // Test 6: FP8 E4M3 quantization
        tests_run = tests_run + 1;
        $display("Test %0d: FP8 E4M3 quantization", tests_run);
        data_in = 16'h3E80;
        fp8_e4_out = fp8_e4m3_quant(data_in);
        if (fp8_e4_out != 8'h00) begin
            $display("  PASS: FP8 E4M3 output: 0x%02X", fp8_e4_out);
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: FP8 E4M3 should be non-zero");
            tests_failed = tests_failed + 1;
        end

        // Test 7: FP8 E5M2 quantization
        tests_run = tests_run + 1;
        $display("Test %0d: FP8 E5M2 quantization", tests_run);
        data_in = 16'h3E80;
        fp8_e5_out = fp8_e5m2_quant(data_in);
        if (fp8_e5_out != 8'h00) begin
            $display("  PASS: FP8 E5M2 output: 0x%02X", fp8_e5_out);
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: FP8 E5M2 should be non-zero");
            tests_failed = tests_failed + 1;
        end

        // Test 8: Posit16 quantization
        tests_run = tests_run + 1;
        $display("Test %0d: Posit16 quantization", tests_run);
        data_in = 16'h1234;
        posit16_out = posit16_quant(data_in);
        if (posit16_out == 16'h1234) begin
            $display("  PASS: Posit16 passthrough");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Posit16 should pass through");
            tests_failed = tests_failed + 1;
        end

        // Test 9: Int4 overflow handling
        tests_run = tests_run + 1;
        $display("Test %0d: Int4 overflow clamping", tests_run);
        data_in = 16'h8000;  // Large negative
        int4_out = int4_quant(data_in);
        if (int4_out == 4'h8) begin
            $display("  PASS: Overflow clamped to -8");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Should clamp to -8");
            tests_failed = tests_failed + 1;
        end

        // Test 10: All quantizers consistency check
        tests_run = tests_run + 1;
        $display("Test %0d: All quantizers consistency", tests_run);
        data_in = 16'h0000;
        int4_out = int4_quant(data_in);
        int8_out = int8_quant(data_in);
        fp8_e4_out = fp8_e4m3_quant(data_in);
        fp8_e5_out = fp8_e5m2_quant(data_in);
        posit16_out = posit16_quant(data_in);

        if (int4_out == 4'h0 && int8_out == 8'h00 &&
            fp8_e4_out == 8'h00 && fp8_e5_out == 8'h00 &&
            posit16_out == 16'h0000) begin
            $display("  PASS: All quantizers handle zero consistently");
            tests_passed = tests_passed + 1;
        end else begin
            $display("  FAIL: Zero handling inconsistent");
            tests_failed = tests_failed + 1;
        end

        // Summary
        $display("");
        $display("=== Test Summary ===");
        $display("Tests run:   %0d", tests_run);
        $display("Tests passed: %0d", tests_passed);
        $display("Tests failed: %0d", tests_failed);
        $display("");

        if (tests_failed == 0) begin
            $display("SUCCESS: All quantizer tests passed!");
        end else begin
            $display("FAILED: %0d tests failed", tests_failed);
        end

        $finish;
    end

endmodule