// Auto-generated from compiler/parser/lexer.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/parser/lexer.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// Token Type Encoding (7-bit) for Lexer
// ============================================================================

`define LTOK_EOF        7'd0
`define LTOK_NEWLINE    7'd1
`define LTOK_DOT        7'd2
`define LTOK_COLON      7'd3
`define LTOK_SEMICOLON  7'd4
`define LTOK_COMMA      7'd5
`define LTOK_HASH       7'd6
`define LTOK_LPAREN     7'd7
`define LTOK_RPAREN     7'd8
`define LTOK_LBRACKET   7'd9
`define LTOK_RBRACKET   7'd10
`define LTOK_PLUS       7'd11
`define LTOK_MINUS      7'd12
`define LTOK_STAR       7'd13
`define LTOK_SLASH      7'd14
`define LTOK_PERCENT    7'd15
`define LTOK_AND        7'd16
`define LTOK_OR         7'd17
`define LTOK_XOR        7'd18
`define LTOK_TILDE      7'd19
`define LTOK_LT         7'd20
`define LTOK_GT         7'd21
`define LTOK_EQ         7'd22
`define LTOK_EXCL       7'd23
`define LTOK_IDENTIFIER 7'd51
`define LTOK_INTEGER    7'd48
`define LTOK_STRING     7'd50

// ============================================================================
// Hardware Lexer FSM
// ============================================================================

module t27_lexer_fsm (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        byte_valid,
    input  wire [7:0]  byte_in,
    output reg         token_valid,
    output reg  [6:0]  token_kind,
    output reg  [15:0] token_start,
    output reg  [15:0] token_end,
    output reg  [15:0] line_num,
    output reg  [15:0] col_num
);

    // FSM states
    localparam S_IDLE    = 3'd0;
    localparam S_IDENT   = 3'd1;
    localparam S_NUMBER  = 3'd2;
    localparam S_STRING  = 3'd3;
    localparam S_COMMENT = 3'd4;
    localparam S_EMIT    = 3'd5;

    reg [2:0]  state;
    reg [15:0] pos;
    reg [15:0] tok_start;

    wire is_alpha   = (byte_in >= 8'd65 && byte_in <= 8'd90) ||
                      (byte_in >= 8'd97 && byte_in <= 8'd122) ||
                      (byte_in == 8'd95);
    wire is_digit_w = (byte_in >= 8'd48 && byte_in <= 8'd57);
    wire is_ws      = (byte_in == 8'd32) || (byte_in == 8'd9) || (byte_in == 8'd13);
    wire is_nl      = (byte_in == 8'd10);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state       <= S_IDLE;
            token_valid <= 1'b0;
            token_kind  <= `LTOK_EOF;
            token_start <= 16'd0;
            token_end   <= 16'd0;
            line_num    <= 16'd1;
            col_num     <= 16'd1;
            pos         <= 16'd0;
            tok_start   <= 16'd0;
        end else if (byte_valid) begin
            token_valid <= 1'b0;
            pos         <= pos + 16'd1;

            if (is_nl) begin
                line_num <= line_num + 16'd1;
                col_num  <= 16'd1;
            end else begin
                col_num <= col_num + 16'd1;
            end

            case (state)
                S_IDLE: begin
                    if (is_ws) begin
                        // skip whitespace
                    end else if (is_nl) begin
                        token_valid <= 1'b1;
                        token_kind  <= `LTOK_NEWLINE;
                        token_start <= pos;
                        token_end   <= pos + 16'd1;
                    end else if (is_alpha) begin
                        state     <= S_IDENT;
                        tok_start <= pos;
                    end else if (is_digit_w) begin
                        state     <= S_NUMBER;
                        tok_start <= pos;
                    end else if (byte_in == 8'd34) begin // '"'
                        state     <= S_STRING;
                        tok_start <= pos;
                    end else begin
                        // Single char token
                        token_valid <= 1'b1;
                        token_start <= pos;
                        token_end   <= pos + 16'd1;
                        case (byte_in)
                            8'd46:  token_kind <= `LTOK_DOT;
                            8'd58:  token_kind <= `LTOK_COLON;
                            8'd59:  token_kind <= `LTOK_SEMICOLON;
                            8'd44:  token_kind <= `LTOK_COMMA;
                            8'd43:  token_kind <= `LTOK_PLUS;
                            8'd45:  token_kind <= `LTOK_MINUS;
                            8'd42:  token_kind <= `LTOK_STAR;
                            8'd47:  token_kind <= `LTOK_SLASH;
                            8'd40:  token_kind <= `LTOK_LPAREN;
                            8'd41:  token_kind <= `LTOK_RPAREN;
                            8'd91:  token_kind <= `LTOK_LBRACKET;
                            8'd93:  token_kind <= `LTOK_RBRACKET;
                            8'd61:  token_kind <= `LTOK_EQ;
                            8'd60:  token_kind <= `LTOK_LT;
                            8'd62:  token_kind <= `LTOK_GT;
                            default: token_kind <= `LTOK_EOF;
                        endcase
                    end
                end

                S_IDENT: begin
                    if (!is_alpha && !is_digit_w && byte_in != 8'd95 && byte_in != 8'd45) begin
                        token_valid <= 1'b1;
                        token_kind  <= `LTOK_IDENTIFIER;
                        token_start <= tok_start;
                        token_end   <= pos;
                        state       <= S_IDLE;
                    end
                end

                S_NUMBER: begin
                    if (!is_digit_w) begin
                        token_valid <= 1'b1;
                        token_kind  <= `LTOK_INTEGER;
                        token_start <= tok_start;
                        token_end   <= pos;
                        state       <= S_IDLE;
                    end
                end

                S_STRING: begin
                    if (byte_in == 8'd34) begin // closing '"'
                        token_valid <= 1'b1;
                        token_kind  <= `LTOK_STRING;
                        token_start <= tok_start;
                        token_end   <= pos + 16'd1;
                        state       <= S_IDLE;
                    end
                end

                default: state <= S_IDLE;
            endcase
        end
    end

endmodule
