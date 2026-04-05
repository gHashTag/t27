// Auto-generated from specs/compiler/parser.t27
// DO NOT EDIT -- regenerate with: tri gen specs/compiler/parser.t27
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


// ============================================================================
// Token Kind Encoding (6-bit)
// ============================================================================

`define TOK_EOF         6'd0
`define TOK_IDENT       6'd1
`define TOK_LITERAL     6'd2
`define TOK_PUB         6'd3
`define TOK_CONST       6'd4
`define TOK_EXTERN      6'd5
`define TOK_STRUCT      6'd6
`define TOK_ENUM        6'd7
`define TOK_FN          6'd8
`define TOK_RETURN      6'd9
`define TOK_IF          6'd10
`define TOK_ELSE        6'd11
`define TOK_USING       6'd12
`define TOK_TEST        6'd13
`define TOK_BENCH       6'd14
`define TOK_INVARIANT   6'd15
`define TOK_MODULE      6'd16
`define TOK_TYPE        6'd17
`define TOK_PLUS        6'd18
`define TOK_MINUS       6'd19
`define TOK_STAR        6'd20
`define TOK_SLASH       6'd21
`define TOK_PERCENT     6'd22
`define TOK_EQ          6'd23
`define TOK_NE          6'd24
`define TOK_LT          6'd25
`define TOK_LE          6'd26
`define TOK_GT          6'd27
`define TOK_GE          6'd28
`define TOK_AND         6'd29
`define TOK_OR          6'd30
`define TOK_XOR         6'd31
`define TOK_NOT         6'd32
`define TOK_ASSIGN      6'd33
`define TOK_ARROW       6'd34
`define TOK_L_PAREN     6'd35
`define TOK_R_PAREN     6'd36
`define TOK_L_BRACE     6'd37
`define TOK_R_BRACE     6'd38
`define TOK_L_BRACKET   6'd39
`define TOK_R_BRACKET   6'd40
`define TOK_COMMA       6'd41
`define TOK_COLON       6'd42
`define TOK_DOT         6'd43
`define TOK_SEMICOLON   6'd44
`define TOK_QUESTION    6'd45

// ============================================================================
// AST Node Kind Encoding (4-bit)
// ============================================================================

`define AST_MODULE       4'd0
`define AST_CONST_DECL   4'd1
`define AST_ENUM_DECL    4'd2
`define AST_STRUCT_DECL  4'd3
`define AST_FN_DECL      4'd4
`define AST_INVARIANT    4'd5
`define AST_TEST_DECL    4'd6
`define AST_BENCH_DECL   4'd7

// ============================================================================
// Tokenizer FSM States
// ============================================================================

`define ST_IDLE         4'd0
`define ST_IDENT        4'd1
`define ST_NUMBER       4'd2
`define ST_STRING       4'd3
`define ST_COMMENT_SC   4'd4
`define ST_COMMENT_SL   4'd5
`define ST_MINUS        4'd6
`define ST_ASSIGN       4'd7
`define ST_BANG         4'd8
`define ST_LESS         4'd9
`define ST_GREATER      4'd10
`define ST_DONE         4'd11

// ============================================================================
// T27Parser -- Hardware tokenizer FSM
// ============================================================================
//
// This module implements the tokenizer portion of the T27 parser as a
// synchronous FSM. It reads one byte per clock from a source buffer and
// emits token kind + start/end positions on the output port.
//
// The full parser (AST construction) is inherently sequential and is
// best implemented in software. The tokenizer is the compute-intensive
// inner loop suitable for hardware acceleration.
// ============================================================================

module T27Parser (
    input  wire        clk,
    input  wire        rst_n,

    // Source input interface
    input  wire        src_valid,      // byte available
    input  wire [7:0]  src_char,       // current source byte
    input  wire        src_last,       // last byte of source
    output reg         src_ready,      // ready to accept byte

    // Token output interface
    output reg         tok_valid,      // token emitted this cycle
    output reg  [5:0]  tok_kind,       // token kind (see defines)
    output reg  [15:0] tok_start,      // start position in source
    output reg  [15:0] tok_end,        // end position in source (exclusive)
    output reg  [15:0] tok_line,       // line number (1-based)
    output reg  [15:0] tok_col,        // column number (1-based)
    output reg         tok_eof         // EOF token emitted (parsing done)
);

    // ========================================================================
    // Internal State
    // ========================================================================

    reg [3:0]  state;
    reg [3:0]  next_state;
    reg [15:0] pos;            // current position in source
    reg [15:0] line_num;       // current line (1-based)
    reg [15:0] col_num;        // current column (1-based)
    reg [15:0] token_start;    // start of current token
    reg [15:0] token_line;     // line at token start
    reg [15:0] token_col;      // column at token start
    reg        neg_number;     // tracking negative number

    // ========================================================================
    // Character Classification (combinational)
    // ========================================================================

    wire is_alpha    = (src_char >= 8'd65 && src_char <= 8'd90)  ||  // A-Z
                       (src_char >= 8'd97 && src_char <= 8'd122) ||  // a-z
                       (src_char == 8'd95);                          // _
    wire is_digit    = (src_char >= 8'd48 && src_char <= 8'd57);     // 0-9
    wire is_alnum    = is_alpha || is_digit || (src_char == 8'd45);  // includes -
    wire is_ws       = (src_char == 8'd32)  ||  // space
                       (src_char == 8'd9)   ||  // tab
                       (src_char == 8'd10)  ||  // LF
                       (src_char == 8'd13);     // CR
    wire is_newline  = (src_char == 8'd10);
    wire is_quote    = (src_char == 8'd34);     // "
    wire is_semicol  = (src_char == 8'd59);     // ;
    wire is_slash    = (src_char == 8'd47);     // /
    wire is_minus    = (src_char == 8'd45);     // -
    wire is_eq       = (src_char == 8'd61);     // =
    wire is_bang     = (src_char == 8'd33);     // !
    wire is_lt       = (src_char == 8'd60);     // <
    wire is_gt       = (src_char == 8'd62);     // >

    // ========================================================================
    // Single-char token kind lookup (combinational)
    // ========================================================================

    reg [5:0] single_kind;
    always @(*) begin
        case (src_char)
            8'd43:  single_kind = `TOK_PLUS;       // +
            8'd42:  single_kind = `TOK_STAR;       // *
            8'd47:  single_kind = `TOK_SLASH;      // /
            8'd37:  single_kind = `TOK_PERCENT;    // %
            8'd61:  single_kind = `TOK_ASSIGN;     // =
            8'd60:  single_kind = `TOK_LT;         // <
            8'd62:  single_kind = `TOK_GT;         // >
            8'd33:  single_kind = `TOK_NOT;        // !
            8'd38:  single_kind = `TOK_AND;        // &
            8'd124: single_kind = `TOK_OR;         // |
            8'd94:  single_kind = `TOK_XOR;        // ^
            8'd40:  single_kind = `TOK_L_PAREN;    // (
            8'd41:  single_kind = `TOK_R_PAREN;    // )
            8'd123: single_kind = `TOK_L_BRACE;    // {
            8'd125: single_kind = `TOK_R_BRACE;    // }
            8'd91:  single_kind = `TOK_L_BRACKET;  // [
            8'd93:  single_kind = `TOK_R_BRACKET;  // ]
            8'd44:  single_kind = `TOK_COMMA;      // ,
            8'd58:  single_kind = `TOK_COLON;      // :
            8'd46:  single_kind = `TOK_DOT;        // .
            8'd59:  single_kind = `TOK_SEMICOLON;  // ;
            8'd63:  single_kind = `TOK_QUESTION;   // ?
            default: single_kind = `TOK_EOF;
        endcase
    end

    // ========================================================================
    // Emit helper: registered token output
    // ========================================================================

    reg        emit_en;
    reg [5:0]  emit_kind;
    reg [15:0] emit_start;
    reg [15:0] emit_end;
    reg [15:0] emit_line;
    reg [15:0] emit_col;

    // ========================================================================
    // Main FSM (sequential)
    // ========================================================================

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state       <= `ST_IDLE;
            pos         <= 16'd0;
            line_num    <= 16'd1;
            col_num     <= 16'd1;
            token_start <= 16'd0;
            token_line  <= 16'd1;
            token_col   <= 16'd1;
            neg_number  <= 1'b0;
            tok_valid   <= 1'b0;
            tok_kind    <= `TOK_EOF;
            tok_start   <= 16'd0;
            tok_end     <= 16'd0;
            tok_line    <= 16'd1;
            tok_col     <= 16'd1;
            tok_eof     <= 1'b0;
            src_ready   <= 1'b1;
            emit_en     <= 1'b0;
        end else begin
            // Default: clear token output
            tok_valid <= 1'b0;

            // Apply pending emit
            if (emit_en) begin
                tok_valid <= 1'b1;
                tok_kind  <= emit_kind;
                tok_start <= emit_start;
                tok_end   <= emit_end;
                tok_line  <= emit_line;
                tok_col   <= emit_col;
                emit_en   <= 1'b0;
            end

            if (src_valid && src_ready) begin
                case (state)
                    `ST_IDLE: begin
                        if (src_last && src_char == 8'd0) begin
                            // EOF
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_EOF;
                            emit_start <= pos;
                            emit_end   <= pos;
                            emit_line  <= line_num;
                            emit_col   <= col_num;
                            tok_eof    <= 1'b1;
                            src_ready  <= 1'b0;
                            state      <= `ST_DONE;
                        end else if (is_ws) begin
                            if (is_newline) begin
                                line_num <= line_num + 16'd1;
                                col_num  <= 16'd1;
                            end else begin
                                col_num <= col_num + 16'd1;
                            end
                            pos <= pos + 16'd1;
                        end else if (is_semicol) begin
                            state <= `ST_COMMENT_SC;
                            pos   <= pos + 16'd1;
                        end else if (is_slash) begin
                            // Could be // comment or / operator
                            token_start <= pos;
                            token_line  <= line_num;
                            token_col   <= col_num;
                            state       <= `ST_COMMENT_SL;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else if (is_alpha) begin
                            token_start <= pos;
                            token_line  <= line_num;
                            token_col   <= col_num;
                            state       <= `ST_IDENT;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else if (is_digit) begin
                            token_start <= pos;
                            token_line  <= line_num;
                            token_col   <= col_num;
                            neg_number  <= 1'b0;
                            state       <= `ST_NUMBER;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else if (is_quote) begin
                            token_start <= pos + 16'd1;
                            token_line  <= line_num;
                            token_col   <= col_num + 16'd1;
                            state       <= `ST_STRING;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else if (is_minus) begin
                            token_start <= pos;
                            token_line  <= line_num;
                            token_col   <= col_num;
                            state       <= `ST_MINUS;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else if (is_eq) begin
                            token_start <= pos;
                            token_line  <= line_num;
                            token_col   <= col_num;
                            state       <= `ST_ASSIGN;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else if (is_bang) begin
                            token_start <= pos;
                            token_line  <= line_num;
                            token_col   <= col_num;
                            state       <= `ST_BANG;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else if (is_lt) begin
                            token_start <= pos;
                            token_line  <= line_num;
                            token_col   <= col_num;
                            state       <= `ST_LESS;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else if (is_gt) begin
                            token_start <= pos;
                            token_line  <= line_num;
                            token_col   <= col_num;
                            state       <= `ST_GREATER;
                            pos         <= pos + 16'd1;
                            col_num     <= col_num + 16'd1;
                        end else begin
                            // Single-char token
                            if (single_kind != `TOK_EOF) begin
                                emit_en    <= 1'b1;
                                emit_kind  <= single_kind;
                                emit_start <= pos;
                                emit_end   <= pos + 16'd1;
                                emit_line  <= line_num;
                                emit_col   <= col_num;
                            end
                            pos     <= pos + 16'd1;
                            col_num <= col_num + 16'd1;
                        end
                    end

                    `ST_IDENT: begin
                        if (is_alnum) begin
                            pos     <= pos + 16'd1;
                            col_num <= col_num + 16'd1;
                        end else begin
                            // Emit identifier/keyword token
                            // Keyword detection is done downstream in software
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_IDENT;
                            emit_start <= token_start;
                            emit_end   <= pos;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            state      <= `ST_IDLE;
                            // Do not consume current char
                        end
                    end

                    `ST_NUMBER: begin
                        if (is_digit) begin
                            pos     <= pos + 16'd1;
                            col_num <= col_num + 16'd1;
                        end else begin
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_LITERAL;
                            emit_start <= token_start;
                            emit_end   <= pos;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            state      <= `ST_IDLE;
                        end
                    end

                    `ST_STRING: begin
                        if (is_quote) begin
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_LITERAL;
                            emit_start <= token_start;
                            emit_end   <= pos;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            pos        <= pos + 16'd1;
                            col_num    <= col_num + 16'd1;
                            state      <= `ST_IDLE;
                        end else begin
                            pos     <= pos + 16'd1;
                            col_num <= col_num + 16'd1;
                        end
                    end

                    `ST_COMMENT_SC: begin
                        if (is_newline) begin
                            line_num <= line_num + 16'd1;
                            col_num  <= 16'd1;
                            pos      <= pos + 16'd1;
                            state    <= `ST_IDLE;
                        end else begin
                            pos     <= pos + 16'd1;
                            col_num <= col_num + 16'd1;
                        end
                    end

                    `ST_COMMENT_SL: begin
                        if (is_slash) begin
                            // Confirmed // comment
                            pos   <= pos + 16'd1;
                            state <= `ST_COMMENT_SC; // reuse semicol comment state
                        end else begin
                            // Was just a / operator
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_SLASH;
                            emit_start <= token_start;
                            emit_end   <= token_start + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            state      <= `ST_IDLE;
                        end
                    end

                    `ST_MINUS: begin
                        if (is_gt) begin
                            // Arrow ->
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_ARROW;
                            emit_start <= token_start;
                            emit_end   <= pos + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            pos        <= pos + 16'd1;
                            col_num    <= col_num + 16'd1;
                            state      <= `ST_IDLE;
                        end else if (is_digit) begin
                            // Negative number
                            neg_number <= 1'b1;
                            state      <= `ST_NUMBER;
                            pos        <= pos + 16'd1;
                            col_num    <= col_num + 16'd1;
                        end else begin
                            // Bare minus
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_MINUS;
                            emit_start <= token_start;
                            emit_end   <= token_start + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            state      <= `ST_IDLE;
                        end
                    end

                    `ST_ASSIGN: begin
                        if (is_eq) begin
                            // ==
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_EQ;
                            emit_start <= token_start;
                            emit_end   <= pos + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            pos        <= pos + 16'd1;
                            col_num    <= col_num + 16'd1;
                            state      <= `ST_IDLE;
                        end else begin
                            // Single =
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_ASSIGN;
                            emit_start <= token_start;
                            emit_end   <= token_start + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            state      <= `ST_IDLE;
                        end
                    end

                    `ST_BANG: begin
                        if (is_eq) begin
                            // !=
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_NE;
                            emit_start <= token_start;
                            emit_end   <= pos + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            pos        <= pos + 16'd1;
                            col_num    <= col_num + 16'd1;
                            state      <= `ST_IDLE;
                        end else begin
                            // Single !
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_NOT;
                            emit_start <= token_start;
                            emit_end   <= token_start + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            state      <= `ST_IDLE;
                        end
                    end

                    `ST_LESS: begin
                        if (is_eq) begin
                            // <=
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_LE;
                            emit_start <= token_start;
                            emit_end   <= pos + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            pos        <= pos + 16'd1;
                            col_num    <= col_num + 16'd1;
                            state      <= `ST_IDLE;
                        end else begin
                            // Single <
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_LT;
                            emit_start <= token_start;
                            emit_end   <= token_start + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            state      <= `ST_IDLE;
                        end
                    end

                    `ST_GREATER: begin
                        if (is_eq) begin
                            // >=
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_GE;
                            emit_start <= token_start;
                            emit_end   <= pos + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            pos        <= pos + 16'd1;
                            col_num    <= col_num + 16'd1;
                            state      <= `ST_IDLE;
                        end else begin
                            // Single >
                            emit_en    <= 1'b1;
                            emit_kind  <= `TOK_GT;
                            emit_start <= token_start;
                            emit_end   <= token_start + 16'd1;
                            emit_line  <= token_line;
                            emit_col   <= token_col;
                            state      <= `ST_IDLE;
                        end
                    end

                    `ST_DONE: begin
                        // Stay idle after EOF
                        src_ready <= 1'b0;
                    end

                    default: begin
                        state <= `ST_IDLE;
                    end
                endcase

                // Handle last byte: emit EOF if idle after processing
                if (src_last && state == `ST_IDLE && !tok_eof) begin
                    // Will be handled next cycle when no more input
                end
            end
        end
    end

endmodule
