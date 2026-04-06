; T27 Syntax Highlighting Queries for tree-sitter

; Comments
(line_comment) @comment

; Keywords
["module" "pub" "const" "var" "fn" "struct" "enum" "test" "invariant" "bench"] @keyword.declaration
["if" "else" "switch" "for" "return" "try" "catch"] @keyword.control

; Types
["i8" "u8" "i16" "u16" "i32" "u32" "i64" "u64" "f16" "f32" "f64" "bool" "void" "usize" "isize"] @type.builtin
["Trit" "PackedTrit" "TernaryWord" "Ternary" "Triformat"] @type.ternary

; Constants
["PHI" "PHI_INV" "PHI_SQ" "TRINITY" "GOLDEN_RATIO"] @constant.phi
["true" "false"] @constant.boolean

; Functions
(function_declaration
  name: (identifier) @function)

(call_expression
  function: (identifier) @function.call)

; Builtins
(builtin_call
  (builtin_name) @function.builtin)

; Parameters
(parameter
  name: (identifier) @variable.parameter)

; Variables
(var_declaration
  name: (identifier) @variable)

(identifier) @variable

; Numbers
(integer) @number
(float_number) @number.float
(hex_number) @number

; Strings
(string_literal) @string

; Operators
"=" @operator
"==" @operator
"!=" @operator
"<" @operator
">" @operator
"<=" @operator
">=" @operator
"+" @operator
"-" @operator
"*" @operator
"/" @operator
"%" @operator
"&&" @operator
"||" @operator
"!" @operator
"&" @operator
"|" @operator
"^" @operator
"~" @operator
"<<" @operator
">>" @operator
"+=" @operator
"-=" @operator
"*=" @operator
"/=" @operator
".." @operator
"**" @operator
"=>" @operator
"." @operator
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket

; Enum literals
(enum_literal
  (identifier) @constant.enum_member)

; Switch cases
(switch_pattern) @label

; Arrays
(array_type) @type.array
(array_literal) @constructor

; Generic parameters
(generic_params) @type.generic
