//! Runtime Formula Evaluator — Phase 1 Implementation
//!
//! This module provides runtime evaluation of formula expressions from .t27 specs.
//! It parses AST, resolves function dependencies, and evaluates with f64 arithmetic.

use std::collections::HashMap;
use std::path::Path;
use crate::compiler::{Compiler, Node, NodeKind};

/// Sacred constants
pub const PHI: f64 = 1.6180339887498948_f64;
pub const PI: f64 = std::f64::consts::PI;
pub const E: f64 = std::f64::consts::E;

/// Custom error type for runtime evaluation
#[derive(Debug)]
pub enum RuntimeError {
    InvalidExpression(String),
    UnknownIdentifier(String),
    UnknownOperator(String),
    FunctionNotFound(String),
    InvalidArgumentCount(String, usize, usize),
    CircularDependency(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::InvalidExpression(s) => write!(f, "Invalid expression: {}", s),
            RuntimeError::UnknownIdentifier(s) => write!(f, "Unknown identifier: {}", s),
            RuntimeError::UnknownOperator(s) => write!(f, "Unknown operator: {}", s),
            RuntimeError::FunctionNotFound(s) => write!(f, "Function not found: {}", s),
            RuntimeError::InvalidArgumentCount(name, expected, actual) => {
                write!(f, "{}: expected {} args, got {}", name, expected, actual)
            }
            RuntimeError::CircularDependency(s) => {
                write!(f, "Circular dependency detected: {}", s)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Result type for runtime evaluation
pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Function definition extracted from AST
#[derive(Debug, Clone)]
struct FunctionDef {
    name: String,
    return_type: String,
    params: Vec<String>,
    body: Vec<Node>,
    dependencies: Vec<String>,
}

/// Runtime formula evaluator with memoization
pub struct FormulaRuntime {
    /// Symbol table for constants (PHI, PI, E, GA)
    symbol_table: HashMap<String, f64>,

    /// Function definitions extracted from source
    functions: HashMap<String, FunctionDef>,

    /// Cache for memoization
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
        symbol_table.insert("GA".to_string(), 360.0 / (PHI * PHI)); // Golden angle

        Self {
            symbol_table,
            functions: HashMap::new(),
            function_cache: HashMap::new(),
            local_vars: vec![HashMap::new()],
        }
    }

    /// Load formulas from a .t27 specification file
    pub fn load_from_spec(&mut self, spec_path: &Path) -> Result<usize> {
        let source = std::fs::read_to_string(spec_path)
            .map_err(|e| RuntimeError::InvalidExpression(format!("Failed to read spec: {}", e)))?;

        let ast = Compiler::parse_ast(&source)
            .map_err(|e| RuntimeError::InvalidExpression(format!("Failed to parse spec: {}", e)))?;

        let mut count = 0;

        // Extract all function declarations
        self.extract_functions(&ast, &mut count);

        // Resolve dependencies for all functions
        self.resolve_all_dependencies()?;

        Ok(count)
    }

    /// Extract all function definitions from AST
    fn extract_functions(&mut self, node: &Node, count: &mut usize) {
        if node.kind == NodeKind::FnDecl {
            let name = node.name.clone();
            let return_type = node.extra_return_type.clone();
            let params: Vec<String> = node.params.iter().map(|(p, _)| p.clone()).collect();

            // Extract dependencies by finding function calls
            let dependencies = self.extract_dependencies(&node.children);

            self.functions.insert(
                name.clone(),
                FunctionDef {
                    name: name.clone(),
                    return_type,
                    params,
                    body: node.children.clone(),
                    dependencies,
                },
            );
            *count += 1;
        }

        for child in &node.children {
            self.extract_functions(child, count);
        }
    }

    /// Extract function call dependencies from AST nodes
    fn extract_dependencies(&self, nodes: &[Node]) -> Vec<String> {
        let mut deps = Vec::new();
        for node in nodes {
            self.find_deps_in_node(node, &mut deps);
        }
        deps.sort();
        deps.dedup();
        deps
    }

    /// Recursively find function calls in a node
    fn find_deps_in_node(&self, node: &Node, deps: &mut Vec<String>) {
        if node.kind == NodeKind::ExprCall {
            let func_name = node.name.clone();
            // Only add if it's a user-defined function, not a builtin
            if self.functions.contains_key(&func_name) || deps.contains(&func_name) {
                deps.push(func_name);
            }
        }
        for child in &node.children {
            self.find_deps_in_node(child, deps);
        }
    }

    /// Resolve dependencies for all loaded functions
    fn resolve_all_dependencies(&mut self) -> Result<()> {
        for name in self.functions.keys().cloned().collect::<Vec<_>>() {
            if !self.dependencies_sorted(&name)? {
                self.update_dependencies_sorted(&name, true);
            }
        }
        Ok(())
    }

    /// Check if dependencies are sorted (topologically ordered)
    fn dependencies_sorted(&self, name: &str) -> Result<bool> {
        if let Some(func) = self.functions.iter().find(|f| f.0 == name).map(|f| f.1.clone()) {
            let mut seen = std::collections::HashSet::new();
            for dep in &func.dependencies {
                if !seen.insert(dep) {
                    if let Some(dep_func) = self.functions.get(&dep.to_string()) {
                        // Check if this dependency also depends on us (circular)
                        if dep_func.dependencies.iter().any(|x| x == name) {
                            return Err(RuntimeError::CircularDependency(format!(
                                "{} <-> {}", name, dep
                            )));
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    /// Update the sorted flag for a function
    fn update_dependencies_sorted(&mut self, _name: &str, _sorted: bool) {
        // In this implementation, we just track dependencies
        // The actual topological sort happens during evaluation
    }

    /// Evaluate a formula by ID (function name)
    pub fn evaluate(&mut self, formula_id: &str) -> Result<f64> {
        println!("DEBUG: evaluate() called with formula_id='{}'", formula_id);

        // Check cache first
        if let Some(&cached) = self.function_cache.get(formula_id) {
            println!("DEBUG: Found in cache: {}", cached);
            return Ok(cached);
        }

        // Find the function and clone its data to avoid borrow checker issues
        let func_def = self.functions.get(&formula_id.to_string())
            .ok_or_else(|| RuntimeError::FunctionNotFound(formula_id.to_string()))?
            .clone();

        println!("DEBUG: Found function: {} (return_type: {})", formula_id, func_def.return_type);

        // Check return type is f64
        if func_def.return_type != "f64" {
            return Err(RuntimeError::InvalidExpression(format!(
                "Function {} returns {}, expected f64", formula_id, func_def.return_type
            )));
        }

        // Save current local scope
        let saved_vars = self.local_vars.last().cloned().unwrap_or_default();
        self.local_vars.push(HashMap::new());

        // Evaluate dependencies first (if any)
        println!("DEBUG: Dependencies: {:?}", func_def.dependencies);
        for dep in &func_def.dependencies {
            println!("DEBUG: Evaluating dependency: {}", dep);
            self.evaluate(dep)?;
        }

        // Evaluate the function body
        let mut result: Option<f64> = None;
        for stmt in &func_def.body {
            println!("DEBUG: Evaluating statement: {:?}", stmt.kind);
            if let Some(val) = self.evaluate_stmt(stmt)? {
                result = Some(val);
            }
        }

        // Restore local scope
        self.local_vars.pop();
        self.local_vars.push(saved_vars);

        let value = result.ok_or_else(|| {
            RuntimeError::InvalidExpression(format!("Function {} did not return a value", formula_id))
        })?;

        // Cache the result
        self.function_cache.insert(formula_id.to_string(), value);
        Ok(value)
    }

    /// Evaluate a statement node
    fn evaluate_stmt(&mut self, node: &Node) -> Result<Option<f64>> {
        match node.kind {
            NodeKind::StmtLocal => {
                // Local variable declaration: let x = expr;
                if node.children.len() >= 2 {
                    let var_name = node.children[0].name.clone();
                    let value = self.evaluate_expr(&node.children[1])?;
                    if let Some(scope) = self.local_vars.last_mut() {
                        scope.insert(var_name, value);
                    }
                }
                Ok(None)
            }
            NodeKind::StmtAssign => {
                // Assignment: x = expr;
                if node.children.len() >= 2 {
                    let var_name = node.children[0].name.clone();
                    let value = self.evaluate_expr(&node.children[1])?;
                    for scope in self.local_vars.iter_mut().rev() {
                        if scope.contains_key(&var_name.clone()) {
                            scope.insert(var_name, value);
                            return Ok(None);
                        }
                    }
                }
                Ok(None)
            }
            NodeKind::ExprReturn => {
                // Return statement: return expr;
                if !node.children.is_empty() {
                    Ok(Some(self.evaluate_expr(&node.children[0])?))
                } else {
                    Ok(None)
                }
            }
            NodeKind::StmtExpr => {
                // Expression statement: func(a, b);
                if !node.children.is_empty() {
                    let val = self.evaluate_expr(&node.children[0])?;
                    Ok(Some(val))
                } else {
                    Ok(None)
                }
            }
            _ => {
                // Other statement types (if, while, for) - evaluate as expression
                self.evaluate_expr(node).map(Some)
            }
        }
    }

    /// Evaluate an expression node
    fn evaluate_expr(&mut self, node: &Node) -> Result<f64> {
        match &node.kind {
            NodeKind::ExprLiteral => {
                // Number literal
                let val_str = node.value.trim();
                let is_negative = val_str.starts_with('-');
                let abs_str = if is_negative { &val_str[1..] } else { val_str };
                abs_str.parse::<f64>()
                    .map(|v| if is_negative { -v } else { v })
                    .map_err(|_| RuntimeError::InvalidExpression(format!("Invalid number: {}", val_str)))
            }
            NodeKind::ExprIdentifier => {
                // Variable or constant lookup
                let name = node.name.trim();

                // DEBUG: Show what we're looking up
                println!("DEBUG: Looking up identifier: '{}' (is_function: {}, in_symbol_table: {}, in_local_vars: {})",
                    name,
                    self.functions.contains_key(name),
                    self.symbol_table.contains_key(name),
                    self.local_vars.last().map_or(false, |v| v.contains_key(name))
                );

                // Check if it's a user-defined function FIRST
                if self.functions.contains_key(name) {
                    // User-defined function - delegate to evaluate()
                    return self.evaluate(name);
                }

                // Check local variables
                for scope in self.local_vars.iter().rev() {
                    if let Some(&val) = scope.get(name) {
                        return Ok(val);
                    }
                }

                // Check symbol table (constants)
                if let Some(&val) = self.symbol_table.get(name) {
                    return Ok(val);
                }

                Err(RuntimeError::UnknownIdentifier(name.to_string()))
            }
            NodeKind::ExprCall => {
                // Function call: func(arg1, arg2, ...)
                self.evaluate_function_call(node)
            }
            NodeKind::ExprBinary => {
                // Binary expression: left op right
                self.evaluate_binary(node)
            }
            NodeKind::ExprUnary => {
                // Unary expression: -expr, +expr
                self.evaluate_unary(node)
            }
            _ => {
                // For now, skip other node types or return error
                Err(RuntimeError::InvalidExpression(format!(
                    "Unsupported expression type: {:?}", node.kind
                )))
            }
        }
    }

    /// Evaluate a function call node
    fn evaluate_function_call(&mut self, node: &Node) -> Result<f64> {
        if node.children.is_empty() {
            return Err(RuntimeError::InvalidExpression("Empty function call".to_string()));
        }

        let func_name = node.name.trim().to_string();
        let args: Vec<f64> = node.children.iter()
            .map(|arg| self.evaluate_expr(arg))
            .collect::<Result<_>>()?;

        // Handle built-in mathematical functions
        match func_name.as_str() {
            "pow" => {
                if args.len() != 2 {
                    return Err(RuntimeError::InvalidArgumentCount(func_name, 2, args.len()));
                }
                Ok(args[0].powf(args[1]))
            }
            "ln" => {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArgumentCount(func_name, 1, args.len()));
                }
                if args[0] <= 0.0 {
                    return Err(RuntimeError::InvalidExpression(
                        "ln() requires positive argument".to_string()
                    ));
                }
                Ok(args[0].ln())
            }
            "exp" => {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArgumentCount(func_name, 1, args.len()));
                }
                Ok(args[0].exp())
            }
            "sin" => {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArgumentCount(func_name, 1, args.len()));
                }
                Ok(args[0].sin())
            }
            "cos" => {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArgumentCount(func_name, 1, args.len()));
                }
                Ok(args[0].cos())
            }
            "tan" => {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArgumentCount(func_name, 1, args.len()));
                }
                Ok(args[0].tan())
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArgumentCount(func_name, 1, args.len()));
                }
                if args[0] < 0.0 {
                    return Err(RuntimeError::InvalidExpression(
                        "sqrt() requires non-negative argument".to_string()
                    ));
                }
                Ok(args[0].sqrt())
            }
            "abs" => {
                if args.len() != 1 {
                    return Err(RuntimeError::InvalidArgumentCount(func_name, 1, args.len()));
                }
                Ok(args[0].abs())
            }
            _ => {
                // User-defined function - delegate to evaluate()
                // But only if it's actually a function (not a constant)
                if self.functions.contains_key(&func_name) {
                    return self.evaluate(&func_name);
                }
                // Check symbol table (constants)
                if let Some(&val) = self.symbol_table.get(&func_name) {
                    return Ok(val);
                }
                // Check local variables
                for scope in self.local_vars.iter().rev() {
                    if let Some(&val) = scope.get(&func_name) {
                        return Ok(val);
                    }
                }
                // Unknown identifier
                Err(RuntimeError::UnknownIdentifier(func_name))
            }
        }
    }

    /// Evaluate a binary expression node
    fn evaluate_binary(&mut self, node: &Node) -> Result<f64> {
        if node.children.len() < 2 {
            return Err(RuntimeError::InvalidExpression(
                "Binary expression missing operands".to_string()
            ));
        }

        let op = node.extra_op.trim();
        let left = self.evaluate_expr(&node.children[0])?;
        let right = self.evaluate_expr(&node.children[1])?;

        match op {
            "+" => Ok(left + right),
            "-" => Ok(left - right),
            "*" | "·" => Ok(left * right),
            "/" | "÷" => {
                if right.abs() < 1e-15 {
                    return Err(RuntimeError::InvalidExpression("Division by zero".to_string()));
                }
                Ok(left / right)
            }
            "%" => {
                if right.abs() < 1e-15 {
                    return Err(RuntimeError::InvalidExpression("Modulo by zero".to_string()));
                }
                Ok(left % right)
            }
            "^" | "**" => Ok(left.powf(right)),
            "<" => Ok(if left < right { 1.0 } else { 0.0 }),
            ">" => Ok(if left > right { 1.0 } else { 0.0 }),
            "<=" => Ok(if left <= right { 1.0 } else { 0.0 }),
            ">=" => Ok(if left >= right { 1.0 } else { 0.0 }),
            "==" => Ok(if (left - right).abs() < 1e-12 { 1.0 } else { 0.0 }),
            "!=" => Ok(if (left - right).abs() >= 1e-12 { 1.0 } else { 0.0 }),
            "&&" => Ok(if left != 0.0 && right != 0.0 { 1.0 } else { 0.0 }),
            "||" => Ok(if left != 0.0 || right != 0.0 { 1.0 } else { 0.0 }),
            _ => Err(RuntimeError::UnknownOperator(op.to_string())),
        }
    }

    /// Evaluate a unary expression node
    fn evaluate_unary(&mut self, node: &Node) -> Result<f64> {
        if node.children.is_empty() {
            return Err(RuntimeError::InvalidExpression(
                "Unary expression missing operand".to_string()
            ));
        }

        let op = node.extra_op.trim();
        let operand = self.evaluate_expr(&node.children[0])?;

        match op {
            "-" => Ok(-operand),
            "+" => Ok(operand),
            "!" => Ok(if operand == 0.0 { 1.0 } else { 0.0 }),
            _ => Err(RuntimeError::UnknownOperator(op.to_string())),
        }
    }

    /// Get list of all loaded function names
    pub fn get_function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get information about a loaded function
    pub(crate) fn get_function_info<'a>(&'a self, name: &str) -> Option<&'a FunctionDef> {
        self.functions.get(name)
    }

    /// Clear the memoization cache
    pub fn clear_cache(&mut self) {
        self.function_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.function_cache.len(), self.functions.len())
    }
}
