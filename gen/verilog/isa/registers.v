// Auto-generated from specs/isa/registers.t27
// DO NOT EDIT -- regenerate with: tri gen specs/isa/registers.t27
// phi^2 + phi^-2 = 3 | TRINITY
// Ring: 28 | Module: ISARegisters
// Synthesizable Verilog for 27-register file with Coptic encoding
// TernaryWord: 27 trits packed in REG_WIDTH*2 bits (2 bits per trit)

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */
/* verilator lint_off MULTIDRIVEN */


module isa_registers #(
    parameter NUM_REGS  = 27,
    parameter REG_WIDTH = 27,
    parameter TRIT_BITS = 2,
    parameter WORD_BITS = REG_WIDTH * TRIT_BITS   // 54 bits per register
)(
    input  wire                     clk,
    input  wire                     rst_n,

    // --- Read port A ---
    input  wire [4:0]               rd_addr_a,
    output reg  [WORD_BITS-1:0]     rd_data_a,

    // --- Read port B ---
    input  wire [4:0]               rd_addr_b,
    output reg  [WORD_BITS-1:0]     rd_data_b,

    // --- Write port ---
    input  wire [4:0]               wr_addr,
    input  wire [WORD_BITS-1:0]     wr_data,
    input  wire                     wr_en,
    output wire                     wr_ack,

    // --- Status register flags (directly exposed) ---
    output wire                     flag_zero,
    output wire                     flag_neg,
    output wire                     flag_carry,
    output wire                     flag_overflow,
    output wire                     flag_trap,
    output wire                     flag_interrupt,

    // --- Status flag write ---
    input  wire [2:0]               status_flag_addr,
    input  wire                     status_flag_data,
    input  wire                     status_flag_wr_en,

    // --- Stack pointer (R17) direct access ---
    output wire [WORD_BITS-1:0]     sp_value,

    // --- Coptic encoding lookup ---
    input  wire [4:0]               coptic_lookup_reg,
    output reg  [15:0]              coptic_codepoint
);

    // =================================================================
    // Register file storage (27 registers x WORD_BITS bits)
    // =================================================================
    reg [WORD_BITS-1:0] regfile [0:NUM_REGS-1];

    // R0 is hardwired to zero
    integer i;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            for (i = 0; i < NUM_REGS; i = i + 1) begin
                regfile[i] <= {WORD_BITS{1'b0}};
            end
        end else if (wr_en && wr_addr != 5'd0 && wr_addr < NUM_REGS) begin
            regfile[wr_addr] <= wr_data;
        end
    end

    // Write acknowledge: succeeds unless writing to R0 or invalid
    assign wr_ack = wr_en && (wr_addr != 5'd0) && (wr_addr < NUM_REGS);

    // =================================================================
    // Read ports (R0 always reads as 0)
    // =================================================================
    always @(*) begin
        if (rd_addr_a == 5'd0 || rd_addr_a >= NUM_REGS)
            rd_data_a = {WORD_BITS{1'b0}};
        else
            rd_data_a = regfile[rd_addr_a];
    end

    always @(*) begin
        if (rd_addr_b == 5'd0 || rd_addr_b >= NUM_REGS)
            rd_data_b = {WORD_BITS{1'b0}};
        else
            rd_data_b = regfile[rd_addr_b];
    end

    // =================================================================
    // Status register (R20) flag extraction
    // =================================================================
    wire [WORD_BITS-1:0] status_reg = regfile[20];

    assign flag_zero      = status_reg[0];
    assign flag_neg       = status_reg[1];
    assign flag_carry     = status_reg[2];
    assign flag_overflow  = status_reg[3];
    assign flag_trap      = status_reg[4];
    assign flag_interrupt = status_reg[5];

    // Status flag individual write
    always @(posedge clk) begin
        if (status_flag_wr_en && status_flag_addr <= 3'd5) begin
            if (status_flag_data)
                regfile[20][status_flag_addr] <= 1'b1;
            else
                regfile[20][status_flag_addr] <= 1'b0;
        end
    end

    // =================================================================
    // Stack pointer direct access
    // =================================================================
    assign sp_value = regfile[17];

    // =================================================================
    // Coptic alphabet ROM
    // =================================================================
    always @(*) begin
        case (coptic_lookup_reg)
            5'd0:  coptic_codepoint = 16'h03B1;  // alpha
            5'd1:  coptic_codepoint = 16'h03B2;  // bita
            5'd2:  coptic_codepoint = 16'h03B3;  // gamma
            5'd3:  coptic_codepoint = 16'h03B4;  // dalda
            5'd4:  coptic_codepoint = 16'h03B5;  // ei
            5'd5:  coptic_codepoint = 16'h03C6;  // sima
            5'd6:  coptic_codepoint = 16'h03B6;  // zata
            5'd7:  coptic_codepoint = 16'h03B7;  // ita
            5'd8:  coptic_codepoint = 16'h03B8;  // thita
            5'd9:  coptic_codepoint = 16'h03B9;  // iota
            5'd10: coptic_codepoint = 16'h03BA;  // kappa
            5'd11: coptic_codepoint = 16'h03BB;  // lauda
            5'd12: coptic_codepoint = 16'h03BC;  // mi
            5'd13: coptic_codepoint = 16'h03BD;  // ni
            5'd14: coptic_codepoint = 16'h03BE;  // ksi
            5'd15: coptic_codepoint = 16'h03C0;  // pi
            5'd16: coptic_codepoint = 16'h03C1;  // ro
            5'd17: coptic_codepoint = 16'h03C3;  // sigma
            5'd18: coptic_codepoint = 16'h03C4;  // tau
            5'd19: coptic_codepoint = 16'h03C5;  // upsilon
            5'd20: coptic_codepoint = 16'h03C6;  // fi
            5'd21: coptic_codepoint = 16'h03C7;  // khi
            5'd22: coptic_codepoint = 16'h03C8;  // psi
            5'd23: coptic_codepoint = 16'h03C9;  // ou
            5'd24: coptic_codepoint = 16'h0417;  // sampi
            5'd25: coptic_codepoint = 16'h0418;  // koppa
            5'd26: coptic_codepoint = 16'h0419;  // shei
            default: coptic_codepoint = 16'h0000;
        endcase
    end

endmodule
