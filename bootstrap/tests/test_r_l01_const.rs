// R-L01 Test: lower_expr for Const
//
// Test that lower_to_tri_ir correctly handles literal expressions:
// - Integer: 42 -> Const(42)
// - Trit: Pos -> Trit(1)
// - Bool: true -> Const(1)

// Use aliased imports to avoid chrono::format::Item conflicts
use t27c::compiler::ast::Expr as AstExpr;
use t27c::compiler::ast::Literal as AstLiteral;
use t27c::compiler::ast::TritValue as AstTritValue;
use t27c::compiler::lower::tri_ir::lower_to_tri_ir;
use t27c::compiler::lower::tri_ir::TriIr;

#[test]
fn test_lower_const_integer() {
    let expr = AstExpr::Literal(AstLiteral::Integer(42));
    let result = lower_to_tri_ir(&expr).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        TriIr::Const(n) => {
            assert_eq!(*n, 42);
        }
        _ => panic!("Expected Const(42), got {:?}", result[0]),
    }
}

#[test]
fn test_lower_const_trit_pos() {
    let expr = AstExpr::Literal(AstLiteral::Trit(AstTritValue::Pos));
    let result = lower_to_tri_ir(&expr).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        TriIr::Trit(t) => {
            assert_eq!(*t, 1i8);
        }
        _ => panic!("Expected Trit(1), got {:?}", result[0]),
    }
}

#[test]
fn test_lower_const_trit_neg() {
    let expr = AstExpr::Literal(AstLiteral::Trit(AstTritValue::Neg));
    let result = lower_to_tri_ir(&expr).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        TriIr::Trit(t) => {
            assert_eq!(*t, -1i8);
        }
        _ => panic!("Expected Trit(-1), got {:?}", result[0]),
    }
}

#[test]
fn test_lower_const_bool_true() {
    let expr = AstExpr::Literal(AstLiteral::Bool(true));
    let result = lower_to_tri_ir(&expr).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        TriIr::Const(n) => {
            assert_eq!(*n, 1);
        }
        _ => panic!("Expected Const(1), got {:?}", result[0]),
    }
}

#[test]
fn test_lower_const_bool_false() {
    let expr = AstExpr::Literal(AstLiteral::Bool(false));
    let result = lower_to_tri_ir(&expr).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        TriIr::Const(n) => {
            assert_eq!(*n, 0);
        }
        _ => panic!("Expected Const(0), got {:?}", result[0]),
    }
}

#[test]
fn test_lower_const_var() {
    let expr = AstExpr::Var("x".to_string());
    let result = lower_to_tri_ir(&expr).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        TriIr::Load(name) => {
            assert_eq!(name, "x");
        }
        _ => panic!("Expected Load(\"x\"), got {:?}", result[0]),
    }
}

#[test]
fn test_lower_const_float() {
    let expr = AstExpr::Literal(AstLiteral::Float(3.14));
    let result = lower_to_tri_ir(&expr).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        TriIr::GF32(_) => {
            // OK - GF32 representation
        }
        _ => panic!("Expected GF32, got {:?}", result[0]),
    }
}

#[test]
fn test_lower_const_trit_zero() {
    let expr = AstExpr::Literal(AstLiteral::Trit(AstTritValue::Zero));
    let result = lower_to_tri_ir(&expr).unwrap();

    assert_eq!(result.len(), 1);
    match &result[0] {
        TriIr::Trit(t) => {
            assert_eq!(*t, 0i8);
        }
        _ => panic!("Expected Trit(0), got {:?}", result[0]),
    }
}
