`default_nettype none

// tb_ternary_mac_demo_v2.v -- self-checking testbench for ternary_mac_demo_top_v2.
//
// Proves the three properties the old demo could not demonstrate on silicon:
//   1. The accumulator actually accumulates (acc_in <- acc_out feedback works).
//   2. All four weight encodings are driven, so the +1 / -1 / zero decode
//      branches are all live rather than constant-folded.
//   3. The LED outputs are functions of accumulator state that change at a
//      rate a human can see, so a flashed board has a checkable signature.
//
// Run with:
//   iverilog -g2005 -o tb_ternary_mac_demo_v2.vvp \
//       tb_ternary_mac_demo_v2.v ternary_mac_demo_core.v ternary_mac_synth.v
//   vvp tb_ternary_mac_demo_v2.vvp

module tb_ternary_mac_demo_v2;

    // 2^4 = 16 model clocks per datapath step keeps the run short.
    localparam integer TB_PRESCALE_BITS = 4;
    localparam integer STEP_CLOCKS      = (1 << TB_PRESCALE_BITS);

    wire led_r23;
    wire led_t23;

    // The board wrapper adds only STARTUPE2; the core carries all behaviour,
    // so the testbench drives the clock itself and needs no primitive stub.
    reg clk = 1'b0;
    always #5 clk = ~clk;

    ternary_mac_demo_core #(
        .PRESCALE_BITS(TB_PRESCALE_BITS)
    ) dut (
        .clk(clk),
        .led_r23(led_r23),
        .led_t23(led_t23)
    );

    integer errors = 0;

    // Mirror of the LED encoding: LEDs are active-low.
    wire acc_nonzero = ~led_r23;
    wire acc_negative = ~led_t23;

    task check_acc;
        input signed [31:0] expected;
        input [359:0] label;
        begin
            if (dut.acc_out !== expected) begin
                $display("FAIL %0s: expected acc=%0d got acc=%0d", label, expected, dut.acc_out);
                errors = errors + 1;
            end else begin
                $display("PASS %0s: acc=%0d  led_r23=%b led_t23=%b", label, dut.acc_out, led_r23, led_t23);
            end
        end
    endtask

    task check_flag;
        input actual;
        input expected;
        input [359:0] label;
        begin
            if (actual !== expected) begin
                $display("FAIL %0s: expected %b got %b", label, expected, actual);
                errors = errors + 1;
            end else begin
                $display("PASS %0s: %b", label, actual);
            end
        end
    endtask

    // Track which weight encodings were actually applied to the MAC.
    reg saw_plus = 0, saw_minus = 0, saw_zero_a = 0, saw_zero_b = 0;
    always @(posedge clk) begin
        if (dut.rst_n && dut.step) begin
            case (dut.w_code)
                2'b01: saw_plus   <= 1;
                2'b10: saw_minus  <= 1;
                2'b00: saw_zero_a <= 1;
                2'b11: saw_zero_b <= 1;
            endcase
        end
    end

    // Count LED transitions to prove the output is not stuck.
    integer led_r23_edges = 0;
    reg led_r23_prev;
    initial led_r23_prev = 1'bx;
    always @(posedge clk) begin
        if (led_r23 !== led_r23_prev) begin
            if (led_r23_prev !== 1'bx)
                led_r23_edges = led_r23_edges + 1;
            led_r23_prev <= led_r23;
        end
    end

    // Advance exactly n datapath steps, sampling just after each step fires.
    task advance_steps;
        input integer n;
        integer k;
        begin
            for (k = 0; k < n; k = k + 1) begin
                @(posedge dut.step);
                @(posedge clk);
                #1;
            end
        end
    endtask

    initial begin
        $display("=== ternary_mac_demo_top_v2 self-check ===");

        // Power-on reset consumes the first three steps (por 00 -> 11).
        advance_steps(3);
        check_acc(32'sd0, "after power-on reset");

        // Weight sequence is {+1, 0, -1, 0} against a = +1, accumulating.
        // Reset releases with phase back at 0, so the walk is:
        //   +1 -> +1 -> +1 -> 0 -> 0 -> 0 -> +1 ...
        advance_steps(1);
        check_acc(32'sd1, "step 1: w=+1 accumulates to +1");

        advance_steps(1);
        check_acc(32'sd1, "step 2: w=0 holds at +1");

        advance_steps(1);
        check_acc(32'sd0, "step 3: w=-1 returns to 0");

        advance_steps(1);
        check_acc(32'sd0, "step 4: w=0 holds at 0");

        advance_steps(1);
        check_acc(32'sd1, "step 5: sequence repeats, +1 again");

        // Run a few more full cycles, then confirm every branch was live.
        advance_steps(12);

        check_flag(saw_plus,   1'b1, "weight +1 (2'b01) was applied");
        check_flag(saw_minus,  1'b1, "weight -1 (2'b10) was applied");
        check_flag(saw_zero_a, 1'b1, "weight 0 encoding 2'b00 was applied");
        check_flag(saw_zero_b, 1'b1, "weight 0 encoding 2'b11 was applied");

        // The accumulator never goes negative in this sequence, so the sign
        // LED must stay dark -- a lit led_t23 means the minus path is wrong.
        check_flag(acc_negative, 1'b0, "acc never negative -> led_t23 dark");

        // The activity LED must have toggled: a stuck LED is the exact failure
        // mode the v1 demo could not detect.
        if (led_r23_edges < 4) begin
            $display("FAIL led_r23 is effectively stuck (%0d edges)", led_r23_edges);
            errors = errors + 1;
        end else begin
            $display("PASS led_r23 toggles with accumulator state (%0d edges)", led_r23_edges);
        end

        $display("--- steps are %0d clocks apart in this TB (2^%0d) ---",
                 STEP_CLOCKS, TB_PRESCALE_BITS);

        if (errors == 0)
            $display("=== ALL TESTS PASSED ===");
        else
            $display("=== %0d TESTS FAILED ===", errors);
        $finish;
    end

    // Watchdog: never let a broken step signal hang the run.
    initial begin
        #500000;
        $display("=== TIMEOUT: step signal never fired ===");
        $finish;
    end
endmodule
