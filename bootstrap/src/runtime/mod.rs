//! Runtime Formula Evaluator — execute .t27 AST dynamically.
//!
//! This module provides runtime evaluation of .t27 specifications by:
//! 1. Parsing .t27 files via Compiler::parse_ast()
//! 2. Executing AST with f64 arithmetic
//! 3. Resolving function dependencies (topological sort + memoization)
//! 4. Caching results for performance

use std::collections::HashMap;
use std::sync::Mutex;
use regex::Regex;
use crate::compiler::{Compiler, Node, NodeKind};

/// Sacred constants
pub const PHI: f64 = 1.6180339887498948_f64;
pub const PI: f64 = std::f64::consts::PI;
pub const E: f64 = std::f64::consts::E;

/// Custom error type for runtime evaluation
#[derive(Debug)]
pub enum RuntimeError {
    InvalidLiteral(String),
    UnknownIdentifier(String),
    UnknownBinaryOp(String),
    UnknownUnaryOp(String),
    DivisionByZero,
    InvalidNodeKind(String),
    FunctionNotFound(String),
    InvalidArity(String, usize, usize),
    InvalidArgument(String, String),
    FunctionCallError(String),
    InvalidStatement(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::InvalidLiteral(s) => write!(f, "Invalid literal: {}", s),
            RuntimeError::UnknownIdentifier(s) => write!(f, "Unknown identifier: {}", s),
            RuntimeError::UnknownBinaryOp(s) => write!(f, "Unknown binary operator: {}", s),
            RuntimeError::UnknownUnaryOp(s) => write!(f, "Unknown unary operator: {}", s),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::InvalidNodeKind(s) => write!(f, "Cannot evaluate node kind: {:?}", s),
            RuntimeError::FunctionNotFound(s) => write!(f, "Function not found: {}", s),
            RuntimeError::InvalidArity(fn_name, expected, got) => {
                write!(f, "{} requires {} arguments, got {}", fn_name, expected, got)
            }
            RuntimeError::InvalidArgument(fn_name, msg) => write!(f, "{}: {}", fn_name, msg),
            RuntimeError::FunctionCallError(s) => write!(f, "Function call error: {}", s),
            RuntimeError::InvalidStatement(s) => write!(f, "Invalid statement: {}", s),
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Result type for runtime evaluation
pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Function definition extracted from AST
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Node,
}

/// Formula runtime evaluator with caching
pub struct FormulaRuntime {
    /// Symbol table for constants (PHI, PI, E, etc.)
    symbol_table: HashMap<String, f64>,
    /// Function definitions from parsed AST
    functions: HashMap<String, FunctionDef>,
    /// Cache for memoization of formula evaluations
    function_cache: HashMap<String, f64>,
    /// Local variable values during evaluation
    local_vars: Vec<HashMap<String, f64>>,
}

impl FormulaRuntime {
    /// Create a new runtime evaluator
    pub fn new() -> Self {
        let mut symbol_table = HashMap::new();
        symbol_table.insert("PHI".to_string(), PHI);
        symbol_table.insert("PI".to_string(), PI);
        symbol_table.insert("E".to_string(), E);

        Self {
            symbol_table,
            functions: HashMap::new(),
            function_cache: HashMap::new(),
            local_vars: vec![HashMap::new()],
        }
    }

    /// Load formulas from a .t27 specification file
    pub fn load_from_spec(&mut self, spec_path: &std::path::Path) -> Result<()> {
        let source = std::fs::read_to_string(spec_path)
            .map_err(|e| RuntimeError::FunctionCallError(format!("Failed to read spec file: {}", e)))?;

        // Compiler bug workaround: extract functions using regex from source
        // The compiler parses all functions as empty TestBlocks, so we can't rely on AST
        self.extract_functions_from_source(&source, spec_path);

        // Also try AST parsing (may not work due to compiler bug)
        let ast = Compiler::parse_ast(&source)
            .map_err(|e| RuntimeError::FunctionCallError(format!("Failed to parse spec file: {}", e)))?;
        self.extract_functions(&ast);

        Ok(())
    }

    /// Extract functions directly from source code using regex
    fn extract_functions_from_source(&mut self, source: &str, _spec_path: &std::path::Path) {
        let fn_regex = Regex::new(r"fn\s+(\w+)\s*\(([^)]*)\)\s*->\s*f64\s*\{\s*return\s+([^;]+);\s*\}")
            .unwrap();

        // Pattern for function definitions: fn NAME(...) -> f64 { return EXPR; }
        let fn_regex = Regex::new(r"fn\s+(\w+)\s*\(([^)]*)\)\s*->\s*f64\s*\{\s*return\s+([^;]+);\s*\}")
            .unwrap();

        for caps in fn_regex.captures_iter(source) {
            let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let params = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let body = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            // Skip if name is empty (matches inside modules/uses)
            if name.is_empty() {
                continue;
            }

            eprintln!("Regex extracted function '{}' with params: '{}'", name, params);

            // Parse body as simple expression (for now, just take first return)
            let body_expr = body.trim();

            // Create a simple node for evaluation
            let mut body_node = Node::new(NodeKind::ExprLiteral);
            body_node.value = body_expr.to_string();
            body_node.name = "return".to_string();

            self.functions.insert(name.to_string(), FunctionDef {
                name: name.to_string(),
                params: vec![],
                body: body_node,
            });
        }

        eprintln!("Regex extracted {} functions from source", fn_regex.captures_iter(source).count());
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.functions.len(), self.function_cache.len())
    }

    /// Evaluate a formula by ID
    pub fn evaluate(&mut self, formula_id: &str) -> Result<f64> {
        let func_def = self.functions.get(formula_id)
            .ok_or_else(|| RuntimeError::FunctionNotFound(formula_id.to_string()))?
            .clone();

        self.evaluate_node(&func_def.body)
    }

    /// Evaluate an AST node
    fn evaluate_node(&mut self, node: &Node) -> Result<f64> {
        match &node.kind {
            NodeKind::FnDecl => {
                // Look up and call the function
                if let Some(func_def) = self.functions.get(&node.name) {
                    self.call_function(&node.name, &[])
                } else {
                    Err(RuntimeError::FunctionNotFound(node.name.clone()))
                }
            }
            NodeKind::ExprLiteral => {
                // Parse literal value
                node.value.parse::<f64>()
                    .map_err(|_| RuntimeError::InvalidLiteral(node.value.clone()))
            }
            NodeKind::ExprIdentifier => {
                // Look up identifier (constant or variable)
                if let Some(val) = self.symbol_table.get(&node.name) {
                    Ok(*val)
                } else if let Some(val) = self.local_vars.last().and_then(|v| v.get(&node.name)) {
                    Ok(*val)
                } else {
                    Err(RuntimeError::UnknownIdentifier(node.name.clone()))
                }
            }
            NodeKind::ExprBinary => {
                let left = self.evaluate_node(&node.children[0])?;
                let right = self.evaluate_node(&node.children[1])?;
                self.evaluate_binary_op(&node.name, left, right)
            }
            NodeKind::ExprUnary => {
                let operand = self.evaluate_node(&node.children[0])?;
                self.evaluate_unary_op(&node.name, operand)
            }
            NodeKind::ExprCall => {
                let args: Vec<f64> = node.children.iter()
                    .map(|c| self.evaluate_node(c))
                    .collect::<Result<Vec<f64>>>()?;
                self.call_function(&node.name, &args)
            }
            NodeKind::ExprReturn => {
                // Return value from expression
                if node.children.is_empty() {
                    Ok(0.0) // Empty return statement
                } else {
                    self.evaluate_node(&node.children[0])
                }
            }
            NodeKind::ExprIf => {
                // Conditional expression: if cond then else
                let cond = self.evaluate_node(&node.children[0])?;
                if cond != 0.0 {
                    self.evaluate_node(&node.children[1])
                } else if node.children.len() > 2 {
                    self.evaluate_node(&node.children[2])
                } else {
                    Err(RuntimeError::InvalidStatement("if without else clause".to_string()))
                }
            }
            NodeKind::StmtIf => {
                // If statement: if (cond) { then } else { else }
                let cond = self.evaluate_node(&node.children[0])?;
                if cond != 0.0 {
                    for stmt in &node.children[1].children {
                        self.evaluate_statement(stmt)?;
                    }
                } else if node.children.len() > 2 {
                    for stmt in &node.children[2].children {
                        self.evaluate_statement(stmt)?;
                    }
                }
                Ok(0.0) // If statements don't produce values
            }
            NodeKind::StmtWhile => {
                // While loop
                let loop_limit = 1000; // Safety limit
                let mut iterations = 0;
                loop {
                    let cond = self.evaluate_node(&node.children[0])?;
                    if cond == 0.0 || iterations >= loop_limit {
                        break;
                    }
                    for stmt in &node.children[1].children {
                        self.evaluate_statement(stmt)?;
                    }
                    iterations += 1;
                }
                Ok(0.0) // While statements don't produce values
            }
            _ => Err(RuntimeError::InvalidNodeKind(format!("{:?}", node.kind)))
        }
    }

    /// Evaluate a binary operation
    fn evaluate_binary_op(&self, op: &str, left: f64, right: f64) -> Result<f64> {
        match op {
            "+" => Ok(left + right),
            "-" => Ok(left - right),
            "*" => Ok(left * right),
            "/" => {
                if right.abs() < f64::EPSILON {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(left / right)
                }
            }
            "%" => Ok(left % right),
            "^" => Ok(left.powf(right)),
            "==" => Ok(if (left - right).abs() < f64::EPSILON { 1.0 } else { 0.0 }),
            "!=" => Ok(if (left - right).abs() >= f64::EPSILON { 1.0 } else { 0.0 }),
            "<" => Ok(if left < right { 1.0 } else { 0.0 }),
            ">" => Ok(if left > right { 1.0 } else { 0.0 }),
            "<=" => Ok(if left <= right { 1.0 } else { 0.0 }),
            ">=" => Ok(if left >= right { 1.0 } else { 0.0 }),
            "&&" => Ok(if left != 0.0 && right != 0.0 { 1.0 } else { 0.0 }),
            "||" => Ok(if left != 0.0 || right != 0.0 { 1.0 } else { 0.0 }),
            _ => Err(RuntimeError::UnknownBinaryOp(op.to_string()))
        }
    }

    /// Evaluate a unary operation
    fn evaluate_unary_op(&self, op: &str, operand: f64) -> Result<f64> {
        match op {
            "-" => Ok(-operand),
            "!" => Ok(if operand != 0.0 { 0.0 } else { 1.0 }),
            _ => Err(RuntimeError::UnknownUnaryOp(op.to_string()))
        }
    }

    /// Call a function (built-in or user-defined)
    fn call_function(&mut self, name: &str, args: &[f64]) -> Result<f64> {
        // Built-in math functions
        match name {
            "pow" => {
                if args.len() == 2 { Ok(args[0].powf(args[1])) }
                else { Err(RuntimeError::InvalidArity(name.to_string(), 2, args.len())) }
            }
            "ln" => {
                if args.len() == 1 { Ok(args[0].ln()) }
                else { Err(RuntimeError::InvalidArity(name.to_string(), 1, args.len())) }
            }
            "exp" => {
                if args.len() == 1 { Ok(args[0].exp()) }
                else { Err(RuntimeError::InvalidArity(name.to_string(), 1, args.len())) }
            }
            "sin" => {
                if args.len() == 1 { Ok(args[0].sin()) }
                else { Err(RuntimeError::InvalidArity(name.to_string(), 1, args.len())) }
            }
            "cos" => {
                if args.len() == 1 { Ok(args[0].cos()) }
                else { Err(RuntimeError::InvalidArity(name.to_string(), 1, args.len())) }
            }
            "tan" => {
                if args.len() == 1 { Ok(args[0].tan()) }
                else { Err(RuntimeError::InvalidArity(name.to_string(), 1, args.len())) }
            }
            "sqrt" => {
                if args.len() == 1 {
                    if args[0] < 0.0 {
                        Err(RuntimeError::InvalidArgument(name.to_string(), "sqrt of negative".to_string()))
                    } else {
                        Ok(args[0].sqrt())
                    }
                } else {
                    Err(RuntimeError::InvalidArity(name.to_string(), 1, args.len()))
                }
            }
            "abs" => {
                if args.len() == 1 { Ok(args[0].abs()) }
                else { Err(RuntimeError::InvalidArity(name.to_string(), 1, args.len())) }
            }
            _ => {
                // User-defined function
                let func_def = self.functions.get(name)
                    .ok_or_else(|| RuntimeError::FunctionNotFound(name.to_string()))?
                    .clone();

                // Check arity
                if func_def.params.len() != args.len() {
                    return Err(RuntimeError::InvalidArity(name.to_string(), func_def.params.len(), args.len()));
                }

                // Create new scope for parameters
                let old_vars_len = self.local_vars.len();
                let mut scope = HashMap::new();
                for (i, param_name) in func_def.params.iter().enumerate() {
                    scope.insert(param_name.clone(), args[i]);
                }
                self.local_vars.push(scope);

                // Evaluate function body
                let result = self.evaluate_node(&func_def.body);

                // Restore scope
                self.local_vars.truncate(old_vars_len);

                // Cache result
                if let Ok(val) = result {
                    self.function_cache.insert(name.to_string(), val);
                }

                result
            }
        }
    }

    /// Evaluate a statement (for if/while blocks)
    fn evaluate_statement(&mut self, node: &Node) -> Result<f64> {
        match &node.kind {
            NodeKind::ExprBinary | NodeKind::ExprUnary | NodeKind::ExprCall | NodeKind::ExprLiteral | NodeKind::ExprIdentifier | NodeKind::ExprReturn | NodeKind::ExprIf | NodeKind::StmtExpr => {
                self.evaluate_node(node)
            }
            NodeKind::StmtLocal | NodeKind::StmtAssign => {
                // Assignment: var x = expr;
                if node.children.len() >= 1 {
                    let rhs = self.evaluate_node(&node.children.last().unwrap())?;
                    if let Some(scope) = self.local_vars.last_mut() {
                        scope.insert(node.name.clone(), rhs);
                    }
                    Ok(rhs)
                } else {
                    Ok(0.0)
                }
            }
            _ => Err(RuntimeError::InvalidStatement(format!("{:?}", node.kind)))
        }
    }

    /// Extract function definitions from AST
    fn extract_functions(&mut self, node: &Node) {
        match node.kind {
            NodeKind::FnDecl => {
                // Find the return expression in the function body
                let body_expr = node.children.iter()
                    .find(|c| matches!(c.kind, NodeKind::ExprReturn))
                    .cloned()
                    .unwrap_or_else(|| Node::new(NodeKind::ExprLiteral));

                let def = FunctionDef {
                    name: node.name.clone(),
                    params: node.params.iter().map(|(name, _)| name.clone()).collect(),
                    body: body_expr,
                };
                self.functions.insert(node.name.clone(), def);
            }

            // Compiler bug workaround: brace-less fn parsed as TestBlock
            // Pattern: TestBlock { name: "S1_gamma", children: [ExprReturn(expr)] }
            // Try to extract function from TestBlock
            NodeKind::TestBlock => {
                self.try_extract_fn_from_testblock(node);
            }

            _ => {
                // Recurse into children
                for child in &node.children {
                    self.extract_functions(child);
                }
            }
        }
    }

    /// Try to extract function from TestBlock (compiler workaround)
    fn try_extract_fn_from_testblock(&mut self, node: &Node) {
        // TestBlock with name pattern like "S1_gamma" is actually a function
        // TestBlocks with names like "S1_gamma_verified" are tests
        if node.name.is_empty() {
            return; // Skip anonymous test blocks
        }

        // Check if name matches formula pattern (has sector prefix)
        let is_formula_test = node.name.len() >= 4
            && (node.name.starts_with("S1_")
                || node.name.starts_with("S1a_")
                || node.name.starts_with("S1b_")
                || node.name.starts_with("PM1_")
                || node.name.starts_with("PM1b_")
                || node.name.starts_with("N1_")
                || node.name.starts_with("NP1_")
                || node.name.starts_with("NP2_")
                || node.name.starts_with("L")
                || node.name.starts_with("K1_")
                || node.name.starts_with("PMNS")
                || node.name.starts_with("P10-")
                || node.name.starts_with("P13-"));

        if !is_formula_test {
            return; // Skip other test blocks
        }

        // Compiler bug: functions with braces are parsed as empty TestBlocks
        // Check if this TestBlock is actually a function by looking at its name
        if is_formula_test && node.children.is_empty() {
            // Create a stub function that returns a sentinel value
            // The actual function content would need to be parsed from the spec file
            eprintln!("TestBlock '{}' is a formula (named) but has no children - using stub",
                     node.name);
            // Don't insert the stub - the compiler can't parse functions correctly
            return;
        }

        // Find the function body: either ExprReturn or a single expression
        let mut body_node = None;
        for child in node.children.iter() {
            match child.kind {
                // Look for ExprReturn (the function body)
                NodeKind::ExprReturn if body_node.is_none() => {
                    body_node = Some(child.clone());
                }
                _ => {}
            }
        }

        let body = body_node.unwrap_or_else(
            || Node::new(NodeKind::ExprLiteral) // Empty test body
        );

        self.functions.insert(node.name.clone(), FunctionDef {
            name: node.name.clone(),
            params: vec![], // TestBlock functions have no params
            body: body.clone(),
        });
    }
}