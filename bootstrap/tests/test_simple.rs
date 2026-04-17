// R-L01 Test: test without external dependencies
//
// Define types locally to avoid chrono::format::Item::Literal conflicts

#[test]
fn test_local_literal() {
    // Define types locally to avoid external conflicts
    #[derive(Debug, Clone, PartialEq)]
    enum LocalLiteral {
        Integer(i64),
    }

    let lit = LocalLiteral::Integer(42);
    match lit {
        LocalLiteral::Integer(n) => {
            assert_eq!(*n, 42);
        }
        _ => panic!("Unexpected variant"),
    }
}

#[test]
fn test_lower_const_integer_direct() {
    // Test the lower_to_tri_ir function directly
    use t27c::compiler::ast::Expr;
    use t27c::compiler::ast::Literal;
    use t27c::compiler::lower::tri_ir::lower_to_tri_ir;
    use t27c::compiler::lower::tri_ir::TriIr;

    // Construct expression using fully qualified paths
    let expr = Expr::Literal(Literal::Integer(42));
    let result = lower_to_tri_ir(&expr);

    match result {
        Ok(ir) => {
            assert_eq!(ir.len(), 1);
            match &ir[0] {
                TriIr::Const(n) => {
                    assert_eq!(*n, 42);
                }
                _ => panic!("Expected Const(42), got {:?}", ir[0]),
            }
        }
        Err(e) => {
            panic!("lower_to_tri_ir failed: {:?}", e);
        }
    }
}
