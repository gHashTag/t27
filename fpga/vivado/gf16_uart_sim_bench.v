module gf16_uart_sim_bench;
    reg clk;
    reg rst_n;
    reg uart_rx;
    wire uart_tx;
    wire led_r23;
    wire led_t23;

    localparam CLK_HZ = 323_310_000;
    localparam CLK_PERIOD = 1_000_000_000_000 / CLK_HZ;
    localparam UART_BAUD = 115200;
    localparam CLK_DIV = CLK_HZ / (UART_BAUD * 16);

    reg [15:0] test_a [0:3];
    reg [15:0] test_b [0:3];
    reg [15:0] expected [0:3];
    integer pass_count;
    integer fail_count;
    integer total_tests;

    gf16_uart_sim_top uut (
        .clk(clk),
        .rst_n(rst_n),
        .uart_rx(uart_rx),
        .uart_tx(uart_tx),
        .led_r23(led_r23),
        .led_t23(led_t23)
    );

    always #(CLK_PERIOD/2) clk = ~clk;

    task uart_send_byte;
        input [7:0] data;
        integer i;
        begin
            uart_rx = 0;
            #(1_000_000_000_000 / UART_BAUD);
            for (i = 0; i < 8; i = i + 1) begin
                uart_rx = data[i];
                #(1_000_000_000_000 / UART_BAUD);
            end
            uart_rx = 1;
            #(1_000_000_000_000 / UART_BAUD);
        end
    endtask

    task uart_recv_byte;
        output [7:0] data;
        integer i;
        begin
            @(negedge uart_tx);
            #(1_000_000_000_000 / UART_BAUD / 2);
            for (i = 0; i < 8; i = i + 1) begin
                #(1_000_000_000_000 / UART_BAUD);
                data[i] = uart_tx;
            end
            #(1_000_000_000_000 / UART_BAUD);
        end
    endtask

    task send_gf16;
        input [15:0] val;
        begin
            uart_send_byte(val[7:0]);
            uart_send_byte(val[15:8]);
        end
    endtask

    task recv_gf16;
        output [15:0] val;
        reg [7:0] lo, hi;
        begin
            uart_recv_byte(lo);
            uart_recv_byte(hi);
            val = {hi, lo};
        end
    endtask

    task run_matmul_test;
        input [15:0] a0, a1, a2, a3;
        input [15:0] b0, b1, b2, b3;
        input [15:0] exp_c00, exp_c01, exp_c02, exp_c03;
        reg [15:0] r00, r01, r02, r03;
        begin
            send_gf16(a0); send_gf16(a1); send_gf16(a2); send_gf16(a3);
            send_gf16(b0); send_gf16(b1); send_gf16(b2); send_gf16(b3);
            send_gf16(16'h0000); send_gf16(16'h0000); send_gf16(16'h0000); send_gf16(16'h0000);
            send_gf16(16'h0000); send_gf16(16'h0000); send_gf16(16'h0000); send_gf16(16'h0000);

            recv_gf16(r00); recv_gf16(r01); recv_gf16(r02); recv_gf16(r03);

            total_tests = total_tests + 1;
            if (r00 == exp_c00 && r01 == exp_c01 && r02 == exp_c02 && r03 == exp_c03) begin
                pass_count = pass_count + 1;
            end else begin
                fail_count = fail_count + 1;
                $display("FAIL test %0d: got [%04X %04X %04X %04X] exp [%04X %04X %04X %04X]",
                    total_tests, r00, r01, r02, r03, exp_c00, exp_c01, exp_c02, exp_c03);
            end
        end
    endtask

    integer i, n_iter;
    real t_start, t_end, elapsed_ns, matmuls_per_sec;
    reg [15:0] dummy_r;

    initial begin
        clk = 0;
        rst_n = 0;
        uart_rx = 1;
        pass_count = 0;
        fail_count = 0;
        total_tests = 0;

        #(CLK_PERIOD * 100);
        rst_n = 1;
        #(CLK_PERIOD * 100);

        $display("=== GF16 Matmul4x4 UART Simulation Benchmark ===");
        $display("Clock: %0d Hz (%0d MHz)", CLK_HZ, CLK_HZ/1000000);
        $display("UART: %0d baud", UART_BAUD);
        $display("");

        $display("--- Correctness Tests ---");

        run_matmul_test(
            16'h3E00, 16'h0000, 16'h0000, 16'h0000,
            16'h3E00, 16'h0000, 16'h0000, 16'h0000,
            16'h3E00, 16'h0000, 16'h0000, 16'h0000
        );

        run_matmul_test(
            16'h3E00, 16'h3E00, 16'h3E00, 16'h3E00,
            16'h3E00, 16'h3E00, 16'h3E00, 16'h3E00,
            16'h4200, 16'h4200, 16'h4200, 16'h4200
        );

        $display("");
        $display("--- Performance Benchmark ---");

        n_iter = 100;
        t_start = $realtime;

        for (i = 0; i < n_iter; i = i + 1) begin
            send_gf16(16'h3E00); send_gf16(16'h4000); send_gf16(16'h4100); send_gf16(16'h4200);
            send_gf16(16'h3E00); send_gf16(16'h0000); send_gf16(16'h0000); send_gf16(16'h0000);
            send_gf16(16'h0000); send_gf16(16'h3E00); send_gf16(16'h0000); send_gf16(16'h0000);
            send_gf16(16'h0000); send_gf16(16'h0000); send_gf16(16'h3E00); send_gf16(16'h0000);
            send_gf16(16'h0000); send_gf16(16'h0000); send_gf16(16'h0000); send_gf16(16'h3E00);
            recv_gf16(dummy_r); recv_gf16(dummy_r); recv_gf16(dummy_r); recv_gf16(dummy_r);
        end

        t_end = $realtime;
        elapsed_ns = (t_end - t_start);
        matmuls_per_sec = n_iter * 1.0e9 / elapsed_ns;

        $display("");
        $display("=== Results ===");
        $display("Correctness: %0d/%0d passed, %0d failed", pass_count, total_tests, fail_count);
        $display("Iterations: %0d", n_iter);
        $display("Total sim time: %.0f ns", elapsed_ns);
        $display("Matmul4x4 throughput: %.2f per sec (simulated)", matmuls_per_sec);
        $display("Compute-only throughput: %0d MHz x 1 matmul/cycle = %.2f M matmul/sec",
            CLK_HZ/1000000, CLK_HZ/1000000.0);
        $display("Compute GOPS: %.2f", 7.0 * 16 * CLK_HZ / 1.0e9);
        $display("");

        $finish;
    end

endmodule
