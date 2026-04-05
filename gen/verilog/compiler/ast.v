// Auto-generated from compiler/ast.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/ast.t27
// phi^2 + phi^-2 = 3 | TRINITY

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */
/* verilator lint_off MULTITOP */


// ============================================================================
// Token Type Encoding (7-bit)
// ============================================================================

`define TOK_EOF         7'd0
`define TOK_NEWLINE     7'd1
`define TOK_DOT         7'd2
`define TOK_COLON       7'd3
`define TOK_SEMICOLON   7'd4
`define TOK_COMMA       7'd5
`define TOK_HASH        7'd6
`define TOK_LPAREN      7'd7
`define TOK_RPAREN      7'd8
`define TOK_LBRACKET    7'd9
`define TOK_RBRACKET    7'd10
`define TOK_PLUS        7'd11
`define TOK_MINUS       7'd12
`define TOK_STAR        7'd13
`define TOK_SLASH       7'd14
`define TOK_PERCENT     7'd15
`define TOK_AND         7'd16
`define TOK_OR          7'd17
`define TOK_XOR         7'd18
`define TOK_TILDE       7'd19
`define TOK_LT          7'd20
`define TOK_GT          7'd21
`define TOK_EQ          7'd22
`define TOK_EXCL        7'd23
`define TOK_USE         7'd24
`define TOK_CONST       7'd25
`define TOK_DATA        7'd26
`define TOK_CODE        7'd27
`define TOK_DWORD       7'd28
`define TOK_DSPACE      7'd29
`define TOK_DTRIT       7'd30
`define TOK_TEST        7'd31
`define TOK_INVARIANT   7'd32
`define TOK_BENCH       7'd33
`define TOK_INTEGER     7'd40
`define TOK_FLOAT       7'd41
`define TOK_STRING      7'd42
`define TOK_IDENTIFIER  7'd43
`define TOK_REG         7'd44
`define TOK_LABEL       7'd45
`define TOK_MOV         7'd60
`define TOK_JZ          7'd61
`define TOK_JNZ         7'd62
`define TOK_JMP         7'd63
`define TOK_HALT        7'd78

// ============================================================================
// Node Type Encoding (7-bit)
// ============================================================================

`define NODE_PROGRAM       7'd0
`define NODE_DATA_SECTION  7'd1
`define NODE_CODE_SECTION  7'd2
`define NODE_CONST_DEF     7'd10
`define NODE_DWORD         7'd20
`define NODE_MOV           7'd30
`define NODE_JZ            7'd31
`define NODE_JNZ           7'd32
`define NODE_JMP           7'd33
`define NODE_HALT          7'd39
`define NODE_TEST          7'd50
`define NODE_INVARIANT     7'd51
`define NODE_SPEC_DECL     7'd60

// ============================================================================
// Opcode Encoding (4-bit)
// ============================================================================

`define OP_MOV    4'd0
`define OP_JZ     4'd1
`define OP_JNZ    4'd2
`define OP_JMP    4'd3
`define OP_MUL    4'd4
`define OP_ADD    4'd5
`define OP_SUB    4'd6
`define OP_BIND   4'd7
`define OP_BUNDLE 4'd8
`define OP_HALT   4'd9

// ============================================================================
// AST Node Register (Hardware representation)
// ============================================================================

module t27_ast_node_reg (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        wr_en,
    input  wire [6:0]  node_type_in,
    input  wire [15:0] line_in,
    input  wire [15:0] column_in,
    output reg  [6:0]  node_type_out,
    output reg  [15:0] line_out,
    output reg  [15:0] column_out
);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            node_type_out <= `NODE_PROGRAM;
            line_out      <= 16'd0;
            column_out    <= 16'd0;
        end else if (wr_en) begin
            node_type_out <= node_type_in;
            line_out      <= line_in;
            column_out    <= column_in;
        end
    end

endmodule

// ============================================================================
// Opcode Decoder
// ============================================================================

module t27_opcode_decoder (
    input  wire [3:0] opcode,
    output reg        is_jump,
    output reg        is_alu,
    output reg        is_mem,
    output reg        is_halt
);

    always @(*) begin
        is_jump = 1'b0;
        is_alu  = 1'b0;
        is_mem  = 1'b0;
        is_halt = 1'b0;

        case (opcode)
            `OP_MOV:    is_mem  = 1'b1;
            `OP_JZ:     is_jump = 1'b1;
            `OP_JNZ:    is_jump = 1'b1;
            `OP_JMP:    is_jump = 1'b1;
            `OP_MUL:    is_alu  = 1'b1;
            `OP_ADD:    is_alu  = 1'b1;
            `OP_SUB:    is_alu  = 1'b1;
            `OP_BIND:   is_alu  = 1'b1;
            `OP_BUNDLE: is_alu  = 1'b1;
            `OP_HALT:   is_halt = 1'b1;
            default: begin
                is_jump = 1'b0;
                is_alu  = 1'b0;
            end
        endcase
    end

endmodule

// ============================================================================
// Instruction ROM (stores opcode + operand count per PC address)
// ============================================================================

module t27_instruction_rom #(
    parameter ADDR_WIDTH = 8,
    parameter DATA_WIDTH = 8
) (
    input  wire [ADDR_WIDTH-1:0] addr,
    output reg  [DATA_WIDTH-1:0] data
);

    // Placeholder ROM -- populated from AST code_section during codegen
    always @(*) begin
        case (addr)
            default: data = {DATA_WIDTH{1'b0}};
        endcase
    end

endmodule
