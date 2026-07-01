`default_nettype none

// tb_ternary_mac.v — simple self-checking testbench for ternary_mac_top
// Run with: iverilog -o tb_ternary_mac.vvp tb_ternary_mac.v ternary_mac_synth.v && vvp tb_ternary_mac.vvp

module tb_ternary_mac;
    reg clk = 0;
    reg rst_n = 0;
    reg en = 1;
    reg signed [7:0] a;
    reg [1:0] w_code;
    reg signed [31:0] acc_in;
    wire signed [31:0] acc_out;

    ternary_mac_top dut (
        .clk(clk),
        .rst_n(rst_n),
        .en(en),
        .a(a),
        .w_code(w_code),
        .acc_in(acc_in),
        .acc_out(acc_out)
    );

    always #5 clk = ~clk;

    integer errors = 0;

    task check;
        input [7:0] a_val;
        input [1:0] w_val;
        input signed [31:0] acc_val;
        input signed [31:0] expected;
        begin
            @(negedge clk);
            a = a_val;
            w_code = w_val;
            acc_in = acc_val;
            @(posedge clk); #1;
            if (acc_out !== expected) begin
                $display("FAIL: a=%0d w=%0b acc_in=%0d expected=%0d got=%0d", a_val, w_val, acc_val, expected, acc_out);
                errors = errors + 1;
            end else begin
                $display("PASS: a=%0d w=%0b acc_in=%0d out=%0d", a_val, w_val, acc_val, acc_out);
            end
        end
    endtask

    initial begin
        $display("=== ternary_mac_top self-check ===");
        #12 rst_n = 1;

        // weight +1 (2'b01)
        check(3, 2'b01, 10, 13);
        // weight -1 (2'b10)
        check(5, 2'b10, 20, 15);
        // weight 0 (2'b00)
        check(7, 2'b00, 30, 30);
        // weight 0 (2'b11)
        check(9, 2'b11, 40, 40);
        // negative activation, plus weight
        check(-4, 2'b01, 100, 96);
        // negative activation, minus weight
        check(-6, 2'b10, 50, 56);

        #20;
        if (errors == 0)
            $display("=== ALL TESTS PASSED ===");
        else
            $display("=== %0d TESTS FAILED ===", errors);
        $finish;
    end
endmodule
