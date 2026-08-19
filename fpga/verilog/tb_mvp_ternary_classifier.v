`timescale 1ns / 1ps

// tb_mvp_ternary_classifier.v -- checks the on-silicon self-check.
//
// The self-check in mvp_ternary_classifier_check.v is what turns "a bitstream
// loaded" into "the silicon computed the right class".  That makes the harness
// itself load-bearing, so it gets the treatment every other check in this
// project gets: it must be shown capable of FAILING before a PASS from it means
// anything.  Two hundred and sixty-five Icarus baselines in this repo recorded
// a harness that could not fail (WAVE_LOOP_656 section 7); this one is proved
// otherwise here.
//
// Phase 1  correct DUT           -> led_t23 stays DARK, led_r23 toggles
// Phase 2  result forced wrong   -> led_t23 LIGHTS
// Phase 3  force released        -> led_t23 STAYS LIT (the verdict is sticky)
//
// Phase 3 is the one that matters most: a verdict that recovers when the fault
// goes away would blink happily through an intermittent wrong answer.
//
// Refs #1959

module tb_mvp_ternary_classifier;

    reg clk = 1'b0;
    always #5 clk = ~clk;          // 100 MHz

    wire led_r23, led_t23;

    // PRESCALE_BITS = 2 -> one vector every 4 clocks, so ten vectors take 40.
    mvp_ternary_classifier_check #(
        .PRESCALE_BITS(2)
    ) uut (
        .clk(clk),
        .led_r23(led_r23),
        .led_t23(led_t23)
    );

    integer fails = 0;
    integer r23_edges = 0;
    reg     r23_prev = 1'b0;
    integer i;

    // Count led_r23 transitions during phase 1 only.
    reg counting = 1'b0;
    always @(posedge clk) begin
        if (counting && (led_r23 !== r23_prev)) r23_edges = r23_edges + 1;
        r23_prev <= led_r23;
    end

    task check_dark;
        input [8*32-1:0] label;
        begin
            if (led_t23 !== 1'b0) begin
                $display("FAILED: %0s -- led_t23 lit, expected dark", label);
                fails = fails + 1;
            end
        end
    endtask

    task check_lit;
        input [8*32-1:0] label;
        begin
            if (led_t23 !== 1'b1) begin
                $display("FAILED: %0s -- led_t23 dark, expected lit", label);
                fails = fails + 1;
            end
        end
    endtask

    initial begin
        // ---- Phase 1: the real network over the FULL input space.
        // ---- The sweep advances one input per clock, so 600 clocks covers
        // ---- all 256 values more than twice -- both the ten reference
        // ---- comparisons and the range invariant on the other 246.
        counting = 1'b1;
        for (i = 0; i < 600; i = i + 1) begin
            @(posedge clk);
            check_dark("phase1 correct network");
        end
        counting = 1'b0;

        if (r23_edges < 4) begin
            $display("FAILED: phase1 -- led_r23 made %0d transitions, expected >= 4",
                     r23_edges);
            fails = fails + 1;
        end

        // ---- Phase 2: force a wrong class.  8'd7 is not a valid class at all,
        // ---- so it mismatches whichever vector is current when a step lands.
        force uut.result = 8'd7;
        for (i = 0; i < 40; i = i + 1) @(posedge clk);
        check_lit("phase2 forced wrong result");

        // ---- Phase 3: remove the fault.  The verdict must NOT recover. ----
        release uut.result;
        for (i = 0; i < 200; i = i + 1) begin
            @(posedge clk);
            check_lit("phase3 verdict must stay latched");
        end

        if (led_r23 !== 1'b0) begin
            $display("FAILED: phase3 -- led_r23 still blinking after a mismatch");
            fails = fails + 1;
        end

        if (fails == 0)
            $display("PASSED: self-check blinks on a correct network, latches on a wrong one");
        else
            $display("FAILED: %0d check(s)", fails);
        $finish;
    end

endmodule
