// Auto-generated from compiler/ast.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/ast.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Token Types
// ============================================================================

pub const TokenType = enum(u8) {
    // Punctuation
    eof = 0,
    newline = 1,
    dot = 2,
    colon = 3,
    semicolon = 4,
    comma = 5,
    hash = 6,
    l_paren = 7,
    r_paren = 8,
    l_bracket = 9,
    r_bracket = 10,
    plus = 11,
    minus = 12,
    star = 13,
    slash = 14,
    percent = 15,
    and_ = 16,
    or_ = 17,
    xor = 18,
    tilde = 19,
    lt = 20,
    gt = 21,
    eq = 22,
    excl = 23,

    // Keywords
    use = 24,
    const_ = 25,
    data = 26,
    code = 27,
    dword = 28,
    dspace = 29,
    dtrit = 30,

    // TDD-Inside-Spec sections
    test_ = 31,
    invariant = 32,
    bench = 33,
    verify = 34,
    expected = 35,
    setup = 36,
    rationale = 37,
    measure = 38,
    target = 39,

    // Literals and identifiers
    integer = 40,
    float_ = 41,
    string = 42,
    identifier = 43,
    reg = 44,
    label = 45,

    // Opcodes (60+)
    mov = 60,
    jz = 61,
    jnz = 62,
    jmp = 63,
    jge = 64,
    jgt = 65,
    jle = 66,
    jlt = 67,
    jeq = 68,
    jne = 69,
    call = 70,
    ret = 71,
    mul = 72,
    add = 73,
    sub = 74,
    div = 75,
    bind = 76,
    bundle = 77,
    halt = 78,
    push = 79,
    pop = 80,
    load = 81,
    store = 82,
    shl = 83,
    shr = 84,
    and_op = 85,
    or_op = 86,
    xor_op = 87,
    not_ = 88,
    neg = 89,
    sqrt = 90,
    tanh = 91,
    trap = 92,

    // TDD-Inside-Spec high-level keywords (spec-style)
    spec = 93,
    rule = 94,
    given = 95,
    when_ = 96,
    then_ = 97,
    assert_ = 98,
    and_kw = 99,
    expect = 100,
};

// ============================================================================
// Node Types
// ============================================================================

pub const NodeType = enum(u8) {
    // Program structure
    program = 0,
    data_section = 1,
    code_section = 2,

    // Constants
    const_def = 10,

    // Data declarations
    d_word = 20,
    d_space = 21,
    d_trit = 22,

    // Instructions
    mov = 30,
    jz = 31,
    jnz = 32,
    jmp = 33,
    mul_node = 34,
    add_node = 35,
    sub_node = 36,
    bind_node = 37,
    bundle_node = 38,
    halt_node = 39,

    // Operands
    reg_operand = 40,
    imm_operand = 41,
    label_operand = 42,
    mem_operand = 43,

    // TDD-Inside-Spec blocks (assembly-style)
    test_block = 50,
    invariant_block = 51,
    bench_block = 52,
    test_case = 53,
    invariant_decl = 54,
    bench_decl = 55,

    // TDD-Inside-Spec high-level structures (spec-style)
    spec_decl = 60,
    rule_block = 61,
    given_clause = 62,
    when_clause = 63,
    then_clause = 64,
    and_clause = 65,
    expect_clause = 66,
    assert_stmt = 67,
    test_block_hl = 68,
    rule_decl = 69,
};

// ============================================================================
// Opcode Enum
// ============================================================================

pub const Opcode = enum(u8) {
    mov = 0,
    jz = 1,
    jnz = 2,
    jmp = 3,
    mul = 4,
    add = 5,
    sub = 6,
    bind = 7,
    bundle = 8,
    halt = 9,
};

// ============================================================================
// Operand Type
// ============================================================================

pub const OperandType = enum(u8) {
    register = 0,
    immediate = 1,
    label_ref = 2,
    memory = 3,
};

// ============================================================================
// Compilation Phase
// ============================================================================

pub const CompilationPhase = enum(u8) {
    parsing = 0,
    semantic_analysis = 1,
    code_generation = 2,
    optimization = 3,
};

// ============================================================================
// AST Node -- base structure for all nodes
// ============================================================================

pub const ASTNode = struct {
    node_type: NodeType,
    line: u32,
    column: u32,
    source_file: []const u8,
};

// ============================================================================
// Program -- root node
// ============================================================================

pub const Program = struct {
    node: ASTNode,
    constants: []const ConstDef,
    data_section: ?*DataSection,
    code_section: ?*CodeSection,
    test_section: ?*TestSection,
    invariant_section: ?*InvariantSection,
    bench_section: ?*BenchSection,
    exports: []const []const u8,
    imports: []const []const u8,
};

// ============================================================================
// Section Structures
// ============================================================================

pub const TestSection = struct {
    node: ASTNode,
    test_cases: []const TestCase,
};

pub const InvariantSection = struct {
    node: ASTNode,
    invariants: []const InvariantDecl,
};

pub const BenchSection = struct {
    node: ASTNode,
    benchmarks: []const BenchDecl,
};

// ============================================================================
// Constants and Data
// ============================================================================

pub const ConstDef = struct {
    node: ASTNode,
    name: []const u8,
    value: i64,
};

pub const DataSection = struct {
    node: ASTNode,
    declarations: []const DataDecl,
};

pub const DataDecl = struct {
    node: ASTNode,
    size: u8,
    initial_value: i64,
    label: []const u8,
};

// ============================================================================
// Code and Instructions
// ============================================================================

pub const CodeSection = struct {
    node: ASTNode,
    instructions: []const Instruction,
};

pub const Instruction = struct {
    node: ASTNode,
    opcode: Opcode,
    operands: []const Operand,
};

pub const Operand = struct {
    node: ASTNode,
    operand_type: OperandType,
};

pub const RegOperand = struct {
    node: ASTNode,
    reg_num: u8,
};

pub const ImmOperand = struct {
    node: ASTNode,
    value: i64,
};

pub const LabelOperand = struct {
    node: ASTNode,
    label_name: []const u8,
};

pub const MemOperand = struct {
    node: ASTNode,
    base_reg: u8,
    offset: i16,
};

// ============================================================================
// TDD-Inside-Spec Structures
// ============================================================================

pub const TestCase = struct {
    node: ASTNode,
    name: []const u8,
    verify_description: []const u8,
    expected_outcome: []const u8,
    setup_description: []const u8,
    rationale_text: ?[]const u8,
};

pub const InvariantDecl = struct {
    node: ASTNode,
    name: []const u8,
    formal_statement: []const u8,
    rationale_text: []const u8,
};

pub const BenchDecl = struct {
    node: ASTNode,
    name: []const u8,
    measure_description: []const u8,
    target_value: ?[]const u8,
    units: []const u8,
};

// ============================================================================
// Type Information
// ============================================================================

pub const TypeInfo = struct {
    name: []const u8,
    size_bits: u8,
    is_signed: bool,
};

// ============================================================================
// Symbol Table
// ============================================================================

pub const Symbol = struct {
    name: []const u8,
    node: ASTNode,
    scope: []const u8,
    is_exported: bool,
    is_defined: bool,
};

// ============================================================================
// Compiler Context
// ============================================================================

pub const CompilerError = struct {
    message: []const u8,
    line: u32,
    column: u32,
    source_file: []const u8,
    phase: CompilationPhase,
};

pub const CompilerWarning = struct {
    message: []const u8,
    line: u32,
    column: u32,
    source_file: []const u8,
};

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "token_type_eof_value" {
    try std.testing.expect(@intFromEnum(TokenType.eof) == 0);
}

test "token_type_newline_value" {
    try std.testing.expect(@intFromEnum(TokenType.newline) == 1);
}

test "node_type_program_value" {
    try std.testing.expect(@intFromEnum(NodeType.program) == 0);
}

test "node_type_test_case_value" {
    try std.testing.expect(@intFromEnum(NodeType.test_case) == 53);
}

test "opcode_mov_value" {
    try std.testing.expect(@intFromEnum(Opcode.mov) == 0);
}

test "opcode_halt_value" {
    try std.testing.expect(@intFromEnum(Opcode.halt) == 9);
}

test "compilation_phase_parsing_value" {
    try std.testing.expect(@intFromEnum(CompilationPhase.parsing) == 0);
}

test "compilation_phase_code_generation_value" {
    try std.testing.expect(@intFromEnum(CompilationPhase.code_generation) == 2);
}

test "ast_node_creation" {
    const node = ASTNode{
        .node_type = .program,
        .line = 1,
        .column = 1,
        .source_file = "test.t27",
    };
    try std.testing.expect(node.line == 1);
    try std.testing.expect(node.source_file.len > 0);
}

test "compilation_phases_sequential" {
    try std.testing.expect(@intFromEnum(CompilationPhase.parsing) < @intFromEnum(CompilationPhase.semantic_analysis));
    try std.testing.expect(@intFromEnum(CompilationPhase.semantic_analysis) < @intFromEnum(CompilationPhase.code_generation));
    try std.testing.expect(@intFromEnum(CompilationPhase.code_generation) < @intFromEnum(CompilationPhase.optimization));
}
