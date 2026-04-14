/**
 * T27 (TRI-27) Grammar for tree-sitter
 * Trinity S³AI ternary language parser
 */

module.exports = grammar({
  name: 't27',

  extras: $ => [
    /\s/,
    $.line_comment,
  ],

  rules: {
    source_file: $ => repeat(choice(
      $.module_declaration,
      $.const_declaration,
      $.struct_declaration,
      $.enum_declaration,
      $.function_declaration,
      $.test_declaration,
      $.invariant_declaration,
      $.bench_declaration,
      $.expression_statement,
    )),

    // Comments
    line_comment: $ => token(seq(
      choice(';', '//'),
      /.*/
    )),

    // Module
    module_declaration: $ => seq(
      'module',
      field('name', $.identifier),
      optional($.module_body)
    ),

    module_body: $ => seq('{', repeat($.item), '}'),

    // Items
    item: $ => choice(
      $.const_declaration,
      $.struct_declaration,
      $.enum_declaration,
      $.function_declaration,
    ),

    // Declarations
    const_declaration: $ => seq(
      optional('pub'),
      'const',
      field('name', $.identifier),
      optional(seq(':', field('type', $.type))),
      '=',
      field('value', $.expression),
      optional(';')
    ),

    struct_declaration: $ => seq(
      optional('pub'),
      'struct',
      field('name', $.identifier),
      optional($.generic_params),
      '{',
      optional(sep(',', $.field_decl)),
      optional(','),
      '}'
    ),

    field_decl: $ => seq(
      field('name', $.identifier),
      ':',
      field('type', $.type)
    ),

    enum_declaration: $ => seq(
      optional('pub'),
      'enum',
      optional(seq('(', field('tag_type', $.type), ')')),
      field('name', $.identifier),
      '{',
      repeat(seq(
        field('variant', $.enum_variant),
        ','
      )),
      field('variant', $.enum_variant),
      optional(','),
      '}'
    ),

    enum_variant: $ => seq(
      field('name', $.identifier),
      optional(seq('=', field('value', $.expression)))
    ),

    // Functions
    function_declaration: $ => seq(
      optional('pub'),
      'fn',
      field('name', $.identifier),
      field('parameters', $.parameters),
      optional(seq(':', field('return_type', $.type))),
      field('body', $.block)
    ),

    parameters: $ => seq(
      '(',
      optional(sep(',', $.parameter)),
      ')'
    ),

    parameter: $ => seq(
      field('name', $.identifier),
      optional(seq(':', field('type', $.type)))
    ),

    block: $ => seq(
      '{',
      repeat(choice(
        $.statement,
        $.expression_statement,
      )),
      '}'
    ),

    // Test/Invariant/Bench
    test_declaration: $ => seq(
      'test',
      $.string_literal,
      field('body', $.block)
    ),

    invariant_declaration: $ => seq(
      'invariant',
      $.string_literal,
      field('body', $.block)
    ),

    bench_declaration: $ => seq(
      'bench',
      $.string_literal,
      field('body', $.block)
    ),

    // Statements
    statement: $ => choice(
      $.var_declaration,
      $.return_statement,
      $.if_statement,
      $.switch_statement,
      $.for_statement,
      $.try_statement,
      $.expression_statement,
    ),

    var_declaration: $ => seq(
      'var',
      field('name', $.identifier),
      ':',
      field('type', $.type),
      '=',
      field('value', $.expression),
      ';'
    ),

    return_statement: $ => seq(
      'return',
      optional($.expression),
      ';'
    ),

    if_statement: $ => seq(
      'if',
      '(',
      field('condition', $.expression),
      ')',
      field('consequence', $.block),
      optional(seq('else', field('alternative', choice($.block, $.if_statement)))
    ),

    switch_statement: $ => seq(
      'switch',
      '(',
      field('value', $.expression),
      ')',
      '{',
      repeat($.switch_case),
      '}'
    ),

    switch_case: $ => seq(
      field('pattern', $.switch_pattern),
      '=>',
      field('result', $.expression),
      ','
    ),

    switch_pattern: $ => seq('.', $.identifier),

    for_statement: $ => seq(
      'for',
      '(',
      field('range', $.range_expression),
      ')',
      field('body', $.block)
    ),

    try_statement: $ => seq(
      'try',
      field('try_block', $.block),
      optional($.catch_clause)
    ),

    catch_clause: $ => seq(
      'catch',
      field('error', $.identifier),
      field('catch_block', $.block)
    ),

    // Expressions
    expression_statement: $ => seq(
      $.expression,
      ';'
    ),

    expression: $ => choice(
      $.unary_expression,
      $.binary_expression,
      $.call_expression,
      $.array_access,
      $.field_access,
      $.primary_expression,
    ),

    unary_expression: $ => prec(1, seq(
      field('operator', choice('!', '-', '~', '&', '*')),
      field('operand', $.expression)
    )),

    binary_expression: $ => choice(
      prec.left(10, binaryOp('*', '/', '%', $.expression, $.expression)),
      prec.left(9, binaryOp('+', '-', $.expression, $.expression)),
      prec.left(8, binaryOp('<<', '>>', $.expression, $.expression)),
      prec.left(7, binaryOp('<', '<=', '>', '>=', $.expression, $.expression)),
      prec.left(6, binaryOp('==', '!=', $.expression, $.expression)),
      prec.left(5, binaryOp('&', $.expression, $.expression)),
      prec.left(4, binaryOp('^', $.expression, $.expression)),
      prec.left(3, binaryOp('|', $.expression, $.expression)),
      prec.left(2, binaryOp('&&', $.expression, $.expression)),
      prec.left(1, binaryOp('||', $.expression, $.expression)),
      prec.left(0, binaryOp('=', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=', '<<=', '>>=', $.expression, $.expression)),
    ),

    call_expression: $ => prec(11, seq(
      field('function', $.primary_expression),
      field('arguments', $.arguments)
    )),

    arguments: $ => seq(
      '(',
      optional(sep(',', $.expression)),
      ')'
    ),

    array_access: $ => prec(11, seq(
      field('array', $.primary_expression),
      '[',
      field('index', $.expression),
      ']'
    )),

    field_access: $ => prec(11, seq(
      field('object', $.primary_expression),
      '.',
      field('field', $.identifier)
    )),

    primary_expression: $ => choice(
      $.identifier,
      $.number,
      $.string_literal,
      $.boolean,
      $.array_literal,
      $.parenthesized_expression,
      $.builtin_call,
      $.enum_literal,
    ),

    parenthesized_expression: $ => seq('(', $.expression, ')'),

    array_literal: $ => seq(
      $.type,
      '{',
      optional(sep(',', $.expression)),
      optional(','),
      '}'
    ),

    builtin_call: $ => seq(
      '@',
      $.builtin_name,
      '(',
      optional(sep(',', $.expression)),
      ')'
    ),

    builtin_name: $ => choice(
      'as', 'intCast', 'intFromEnum', 'enumFromInt',
      'compileAssert', 'setEvalBranchQuota', 'import',
      'embedFile', 'typeName', 'sizeOf', 'alignOf', 'offsetOf',
      'hasDecl', 'hasField', 'cDefine', 'cInclude', 'cUndef'
    ),

    enum_literal: $ => seq(
      '.',
      $.identifier
    ),

    // Types
    type: $ => choice(
      $.primitive_type,
      $.ternary_type,
      $.array_type,
      $.identifier,
    ),

    primitive_type: $ => choice(
      'i8', 'u8', 'i16', 'u16', 'i32', 'u32', 'i64', 'u64',
      'f16', 'f32', 'f64', 'bool', 'void', 'usize', 'isize'
    ),

    ternary_type: $ => choice(
      'Trit', 'PackedTrit', 'TernaryWord', 'Ternary', 'Triformat'
    ),

    array_type: $ => seq(
      '[',
      choice($.number, '_'),
      ']',
      $.type
    ),

    // Literals
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    number: $ => choice(
      $.hex_number,
      $.float_number,
      $.integer,
    ),

    hex_number: $ => /0x[0-9a-fA-F]+/,

    float_number: $ => /\d+\.\d+([eE][+-]?\d+)?/,

    integer: $ => /\d+/,

    string_literal: $ => seq(
      '"',
      /[^"]*/,
      '"'
    ),

    boolean: $ => choice('true', 'false'),

    // Range
    range_expression: $ => seq(
      $.integer,
      '..',
      $.integer
    ),

    // Constants
    constant: $ => choice(
      $.phi_constant,
      $.boolean,
    ),

    phi_constant: $ => choice('PHI', 'PHI_INV', 'PHI_SQ', 'TRINITY', 'GOLDEN_RATIO'),
  },

  word: $ => $.identifier,
});

function binaryOp(...operators) {
  return {
    type: 'binary_expression',
    operator: prec(1, choice(...operators)),
    left: $ => $.expression,
    right: $ => $.expression,
  };
}

function sep(sep, rule) {
  return optional(seq(rule, repeat(seq(sep, rule))));
}
