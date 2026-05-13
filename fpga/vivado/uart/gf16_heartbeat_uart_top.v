`default_nettype none

// gf16_heartbeat_uart_top — heartbeat bitstream with UART telemetry
//
// Adds 115200-baud UART TX output to the existing gf16_heartbeat_top design.
// Every layer-transition (~0.82 Hz at 66 MHz CFGMCLK) emits a one-line ASCII
// frame on uart_tx so the host can measure live throughput (tok/s) and
// observe the GF(2^4) dot4 result over time.
//
// Frame format (12 bytes incl. CR/LF):
//   "T:HH R:HHHH\r\n"
//   T  = temporal_layer (0..3, hex)
//   R  = dot4_result (16-bit, hex)
//
// At 0.82 layer/s × 12 bytes × 10 bits/byte = ~98 baud average traffic,
// well within 115200.

module gf16_heartbeat_uart_top (
    output wire led_d5,
    output wire led_d6,
    output wire led_j26,
    output wire uart_tx
);

    // ───────────────────────────────────────────────────────────────────────
    // STARTUPE2 — primary clock source on Wukong V1 (no external osc on
    // QMTech core board; CFGMCLK is ~66 MHz internal config oscillator).
    // ───────────────────────────────────────────────────────────────────────
    wire cfgmclk;
    STARTUPE2 #(
        .PROG_USR("FALSE"),
        .SIM_CCLK_FREQ(10.0)
    ) startup (
        .CFGCLK(),
        .CFGMCLK(cfgmclk),
        .EOS(),
        .PREQ(),
        .CLK(1'b0),
        .GSR(1'b0),
        .GTS(1'b0),
        .KEYCLEARB(1'b0),
        .PACK(1'b0),
        .USRCCLKO(1'b0),
        .USRCCLKTS(1'b0),
        .USRDONEO(1'b1),
        .USRDONETS(1'b1)
    );

    // ───────────────────────────────────────────────────────────────────────
    // φ-cycle and temporal layer counter (unchanged from original heartbeat)
    // ───────────────────────────────────────────────────────────────────────
    localparam PHI_CYCLE       = 27'd80_901_699;
    localparam GF16_ONE        = 16'h3E00;
    localparam GF16_PHI_FRAC   = 16'h3D9E;
    localparam GF16_HALF       = 16'h3C00;
    localparam GF16_QUARTER    = 16'h3A00;

    reg [26:0] phi_counter = 0;
    reg [1:0]  temporal_layer = 0;
    reg [24:0] blink_counter = 0;
    reg        layer_changed = 1'b0;

    always @(posedge cfgmclk) begin
        blink_counter <= blink_counter + 1'b1;
        phi_counter   <= phi_counter + 1'b1;
        layer_changed <= 1'b0;
        if (phi_counter >= PHI_CYCLE) begin
            phi_counter    <= 0;
            temporal_layer <= temporal_layer + 1'b1;
            layer_changed  <= 1'b1;          // one-cycle pulse for UART trigger
        end
    end

    // ───────────────────────────────────────────────────────────────────────
    // GF(2^4) dot4 (unchanged)
    // ───────────────────────────────────────────────────────────────────────
    reg [15:0] vec_a0, vec_a1, vec_a2, vec_a3;
    reg [15:0] vec_b0, vec_b1, vec_b2, vec_b3;

    always @(posedge cfgmclk) begin
        vec_a0 <= GF16_ONE;
        vec_a1 <= GF16_PHI_FRAC;
        vec_a2 <= GF16_HALF;
        vec_a3 <= GF16_QUARTER;
        case (temporal_layer)
            2'd0: begin
                vec_b0 <= GF16_ONE;
                vec_b1 <= GF16_ONE;
                vec_b2 <= GF16_ONE;
                vec_b3 <= GF16_ONE;
            end
            2'd1: begin
                vec_b0 <= GF16_PHI_FRAC;
                vec_b1 <= GF16_PHI_FRAC;
                vec_b2 <= GF16_PHI_FRAC;
                vec_b3 <= GF16_PHI_FRAC;
            end
            2'd2: begin
                vec_b0 <= GF16_HALF;
                vec_b1 <= GF16_HALF;
                vec_b2 <= GF16_HALF;
                vec_b3 <= GF16_HALF;
            end
            default: begin
                vec_b0 <= GF16_QUARTER;
                vec_b1 <= GF16_QUARTER;
                vec_b2 <= GF16_QUARTER;
                vec_b3 <= GF16_QUARTER;
            end
        endcase
    end

    wire [15:0] dot4_result;
    gf16_dot4 dot4 (
        .a0(vec_a0), .a1(vec_a1), .a2(vec_a2), .a3(vec_a3),
        .b0(vec_b0), .b1(vec_b1), .b2(vec_b2), .b3(vec_b3),
        .result(dot4_result)
    );

    wire dot4_is_positive = ~dot4_result[15];
    wire [5:0] dot4_exp = dot4_result[14:9];
    wire dot4_is_nonzero = (dot4_exp != 6'd0);

    // ───────────────────────────────────────────────────────────────────────
    // LED outputs (unchanged from heartbeat)
    // ───────────────────────────────────────────────────────────────────────
    reg d5_out, d6_out, j26_out;
    always @(*) begin
        d5_out  = 1'b0;
        d6_out  = 1'b0;
        j26_out = 1'b0;
        case (temporal_layer)
            2'd0: begin
                d5_out  = blink_counter[24];
                d6_out  = blink_counter[24];
                j26_out = dot4_is_nonzero ? blink_counter[20] : 1'b0;
            end
            2'd1: begin
                d5_out  = 1'b1;
                d6_out  = 1'b1;
                j26_out = dot4_is_positive ? blink_counter[19] : 1'b0;
            end
            2'd2: begin
                d5_out  = blink_counter[22];
                d6_out  = blink_counter[22];
                j26_out = dot4_is_positive ? blink_counter[18] : 1'b0;
            end
            default: begin
                d5_out  = 1'b0;
                d6_out  = 1'b0;
                j26_out = 1'b0;
            end
        endcase
    end

    assign led_d5  = d5_out;
    assign led_d6  = d6_out;
    assign led_j26 = j26_out;

    // ───────────────────────────────────────────────────────────────────────
    // UART telemetry — emit "T:HH R:HHHH\r\n" on every layer transition
    // ───────────────────────────────────────────────────────────────────────
    //
    // Baud divider:
    //   CFGMCLK is nominally ~65 MHz on -1 speed grade XC7A100T.
    //   For 115200 baud: divisor = 65_000_000 / 115_200 ≈ 564.
    //   We use 564 — within ±3% of true rate, well within UART tolerance (±2.5%).
    //
    localparam integer BAUD_DIVISOR = 564;

    // ── nibble-to-ASCII-hex helper ──
    function [7:0] hex_nibble;
        input [3:0] nib;
        begin
            hex_nibble = (nib < 4'd10) ? (8'h30 + {4'b0, nib}) : (8'h41 + {4'b0, nib} - 8'd10);
        end
    endfunction

    // ── TX byte stream ROM (12 bytes per frame) ──
    reg  [7:0] tx_byte;
    reg        tx_start;
    wire       tx_busy;

    reg  [3:0] tx_idx = 4'd0;   // 0..11 = frame bytes, 12 = idle
    reg [15:0] latched_dot4 = 16'd0;
    reg  [1:0] latched_layer = 2'd0;

    // Frame builder
    always @(posedge cfgmclk) begin
        tx_start <= 1'b0;
        if (layer_changed && tx_idx == 4'd12) begin
            latched_dot4  <= dot4_result;
            latched_layer <= temporal_layer;
            tx_idx        <= 4'd0;
            tx_byte       <= 8'h54;             // 'T'
            tx_start      <= 1'b1;
        end else if (!tx_busy && tx_idx != 4'd12 && !tx_start) begin
            // Advance to next byte after previous transmit finished
            case (tx_idx)
                4'd0:  begin tx_byte <= 8'h3A;                                         tx_start <= 1'b1; tx_idx <= 4'd1;  end // ':'
                4'd1:  begin tx_byte <= hex_nibble({2'b00, latched_layer});            tx_start <= 1'b1; tx_idx <= 4'd2;  end // layer hi (zero)
                4'd2:  begin tx_byte <= hex_nibble({2'b00, latched_layer});            tx_start <= 1'b1; tx_idx <= 4'd3;  end // layer lo
                4'd3:  begin tx_byte <= 8'h20;                                         tx_start <= 1'b1; tx_idx <= 4'd4;  end // ' '
                4'd4:  begin tx_byte <= 8'h52;                                         tx_start <= 1'b1; tx_idx <= 4'd5;  end // 'R'
                4'd5:  begin tx_byte <= 8'h3A;                                         tx_start <= 1'b1; tx_idx <= 4'd6;  end // ':'
                4'd6:  begin tx_byte <= hex_nibble(latched_dot4[15:12]);               tx_start <= 1'b1; tx_idx <= 4'd7;  end
                4'd7:  begin tx_byte <= hex_nibble(latched_dot4[11:8]);                tx_start <= 1'b1; tx_idx <= 4'd8;  end
                4'd8:  begin tx_byte <= hex_nibble(latched_dot4[7:4]);                 tx_start <= 1'b1; tx_idx <= 4'd9;  end
                4'd9:  begin tx_byte <= hex_nibble(latched_dot4[3:0]);                 tx_start <= 1'b1; tx_idx <= 4'd10; end
                4'd10: begin tx_byte <= 8'h0D;                                         tx_start <= 1'b1; tx_idx <= 4'd11; end // CR
                4'd11: begin tx_byte <= 8'h0A;                                         tx_start <= 1'b1; tx_idx <= 4'd12; end // LF
                default: ;
            endcase
        end
    end

    // ── UART transmitter (115200 8N1) ──
    uart_tx_8n1 #(
        .CLK_HZ(65_000_000),
        .BAUD(115_200)
    ) tx (
        .clk     (cfgmclk),
        .data    (tx_byte),
        .start   (tx_start),
        .tx      (uart_tx),
        .busy    (tx_busy)
    );

endmodule

// ──────────────────────────────────────────────────────────────────────────
// Standard 8N1 UART transmitter
// ──────────────────────────────────────────────────────────────────────────
module uart_tx_8n1 #(
    parameter integer CLK_HZ = 65_000_000,
    parameter integer BAUD   = 115_200
) (
    input  wire       clk,
    input  wire [7:0] data,
    input  wire       start,
    output reg        tx,
    output wire       busy
);
    localparam integer DIV = CLK_HZ / BAUD;

    reg [15:0] tick_cnt = 16'd0;
    reg [3:0]  bit_idx  = 4'd0;
    reg [9:0]  shifter  = 10'b1111111111;   // idle high
    reg        active   = 1'b0;

    assign busy = active;

    initial tx = 1'b1;

    always @(posedge clk) begin
        if (!active) begin
            tx <= 1'b1;
            if (start) begin
                // 10-bit frame: start(0) + data[0..7] + stop(1)
                shifter <= {1'b1, data, 1'b0};
                bit_idx <= 4'd0;
                tick_cnt <= 16'd0;
                active   <= 1'b1;
                tx       <= 1'b0;   // start bit immediately
            end
        end else begin
            if (tick_cnt + 1 >= DIV) begin
                tick_cnt <= 16'd0;
                shifter  <= {1'b1, shifter[9:1]};
                tx       <= shifter[1];
                if (bit_idx == 4'd9) begin
                    active <= 1'b0;
                end else begin
                    bit_idx <= bit_idx + 4'd1;
                end
            end else begin
                tick_cnt <= tick_cnt + 16'd1;
            end
        end
    end
endmodule
