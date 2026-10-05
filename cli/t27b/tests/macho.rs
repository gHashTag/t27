//! The relocatable object, linked by the system linker and run: functions that
//! address read-only data (`__const`, through ADRP + ADD relocations) and
//! frame slots must return what the interpreter computes.
//!
//! Needs Apple clang and an arm64 Mac, like the encoder test.
#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use std::process::Command;
use t27b::codegen::{self, TrapStyle};
use t27b::eval::Interp;
use t27b::ir::*;
use t27b::macho;

fn konst(ty: Ty, v: i128) -> Expr {
    Expr { ty, kind: ExprKind::Const(v) }
}

fn var(ty: Ty, id: u32) -> Expr {
    Expr { ty, kind: ExprKind::Var(id) }
}

fn data(k: u32) -> Expr {
    Expr { ty: Ty::Ptr, kind: ExprKind::Data(k) }
}

fn slot(k: u32) -> Expr {
    Expr { ty: Ty::Ptr, kind: ExprKind::Slot(k) }
}

fn load(ty: Ty, addr: Expr, off: u32) -> Expr {
    Expr { ty, kind: ExprKind::Load { addr: Box::new(addr), off } }
}

fn offset(base: Expr, idx: Expr, scale: u32) -> Expr {
    Expr { ty: Ty::Ptr, kind: ExprKind::Offset { base: Box::new(base), idx: Box::new(idx), scale } }
}

fn arith(ty: Ty, op: ArithOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr { ty, kind: ExprKind::Arith { op, lhs: Box::new(lhs), rhs: Box::new(rhs), site: 0 } }
}

fn func(name: &str, params: &[Ty], ret: Ty, body: Vec<Stmt>, slots: Vec<SlotInfo>) -> Func {
    Func {
        name: name.into(),
        nparams: params.len(),
        ret: Some(ret),
        vars: params.iter().enumerate().map(|(i, &t)| Var { name: format!("p{}", i), ty: t }).collect(),
        body,
        line: 1,
        is_test: false,
        is_invariant: false,
        noreturn_site: 1,
        slots,
    }
}

#[test]
fn object_with_const_data_links_and_runs() {
    // Blob 0 spans two pages, so blobs 1 and 2 sit on a later page than the
    // code: the ADRP page delta and the ADD page offset are both non-zero.
    let d0: Vec<u8> = (0..5000u32).map(|i| (i * 7 + 3) as u8).collect();
    let d1: Vec<u8> = [-5i64, 1 << 40, i64::MIN, 77, -1].iter().flat_map(|v| v.to_le_bytes()).collect();
    let d2: Vec<u8> = (0..100u32).map(|i| (i * 37 + 200) as u8).collect();
    let w = |r: Expr| vec![Stmt::Return(Some(r))];
    let funcs = vec![
        // f0(i) = d0[(i & 4095) + 900]
        func(
            "f0",
            &[Ty::U64],
            Ty::U8,
            w(load(
                Ty::U8,
                offset(data(0), arith(Ty::U64, ArithOp::And, var(Ty::U64, 0), konst(Ty::U64, 4095)), 1),
                900,
            )),
            vec![],
        ),
        // f1(i) = ((i64*)d1)[(i & 3) + 1]
        func(
            "f1",
            &[Ty::U64],
            Ty::I64,
            w(load(Ty::I64, offset(data(1), arith(Ty::U64, ArithOp::And, var(Ty::U64, 0), konst(Ty::U64, 3)), 8), 8)),
            vec![],
        ),
        // f2(x): s0 = d2; s0[4..8] = x; s1 = s0; s1[4..8] +% s1[99]
        func(
            "f2",
            &[Ty::U32],
            Ty::U32,
            vec![
                Stmt::Copy { dst: slot(0), src: data(2), size: 100 },
                Stmt::Store { addr: slot(0), off: 4, value: var(Ty::U32, 0) },
                Stmt::Copy { dst: slot(1), src: slot(0), size: 100 },
                Stmt::Return(Some(arith(
                    Ty::U32,
                    ArithOp::AddW,
                    load(Ty::U32, slot(1), 4),
                    Expr { ty: Ty::U32, kind: ExprKind::Widen(Box::new(load(Ty::U8, slot(1), 99))) },
                ))),
            ],
            vec![SlotInfo { size: 100, align: 8 }, SlotInfo { size: 100, align: 4 }],
        ),
        // f3() = sign-extended halfword of d2 at 50
        func("f3", &[], Ty::I16, w(load(Ty::I16, data(2), 50)), vec![]),
        // f4(x) = f2(x) +% d2[0..4]
        func(
            "f4",
            &[Ty::U32],
            Ty::U32,
            w(arith(
                Ty::U32,
                ArithOp::AddW,
                Expr { ty: Ty::U32, kind: ExprKind::Call { func: 2, args: vec![var(Ty::U32, 0)] } },
                load(Ty::U32, data(2), 0),
            )),
            vec![],
        ),
    ];
    let mk = |kind, ty| Site { kind, line: 1, what: String::new(), ty };
    let prog = Program {
        module: "obj".into(),
        funcs,
        sites: vec![mk(TrapKind::Overflow, Ty::U8), mk(TrapKind::NoReturn, Ty::U32)],
        mode: OverflowMode::Trap,
        unchecked: Vec::new(),
        data: vec![d0, d1, d2],
        globals: Vec::new(),
        internal_abi: Vec::new(),
    };

    let calls: Vec<(usize, Vec<i128>)> = vec![
        (0, vec![0]),
        (0, vec![4095]),
        (0, vec![4096 + 17]),
        (0, vec![u64::MAX as i128]),
        (1, vec![0]),
        (1, vec![1]),
        (1, vec![2]),
        (1, vec![7]),
        (2, vec![0]),
        (2, vec![0xdead_beef]),
        (3, vec![]),
        (4, vec![123_456_789]),
    ];
    let c_types = ["uint8_t", "int64_t", "uint32_t", "int16_t", "uint32_t"];
    let c_params = ["uint64_t", "uint64_t", "uint32_t", "void", "uint32_t"];
    let mut want = String::new();
    let mut c = String::from("#include <stdio.h>\n#include <stdint.h>\n");
    for (i, f) in prog.funcs.iter().enumerate() {
        c.push_str(&format!("extern {} {}({});\n", c_types[i], f.name, c_params[i]));
    }
    c.push_str("int main(void) {\n");
    for (fi, args) in &calls {
        let v = Interp::new(&prog).call(*fi, args).expect("interpreter").expect("value");
        want.push_str(&format!("{}\n", v));
        let ty = prog.funcs[*fi].ret.unwrap();
        let (fmt, cast) = if ty.signed() { ("%lld", "long long") } else { ("%llu", "unsigned long long") };
        let a: Vec<String> = args.iter().map(|a| format!("{}ull", *a as u64)).collect();
        c.push_str(&format!(
            "    printf(\"{}\\n\", ({}){}({}));\n",
            fmt,
            cast,
            prog.funcs[*fi].name,
            a.join(", ")
        ));
    }
    c.push_str("    return 0;\n}\n");

    let code = codegen::compile(&prog, TrapStyle::Brk, false).expect("codegen");
    let linked = codegen::link(Vec::new(), None, &code, prog.funcs.len()).expect("link");
    assert!(linked.data_refs.len() >= 5, "every data access is a relocated ADRP + ADD pair");
    let names: Vec<String> = prog.funcs.iter().map(|f| f.name.clone()).collect();
    let obj = macho::object(&linked, &names, &prog.data);

    let dir = std::env::temp_dir().join(format!("t27b-macho-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (o, h, exe) = (dir.join("m.o"), dir.join("main.c"), dir.join("main"));
    std::fs::write(&o, obj).unwrap();
    std::fs::write(&h, c).unwrap();
    let st = Command::new("clang").arg("-o").arg(&exe).arg(&h).arg(&o).output().expect("run clang");
    assert!(st.status.success(), "link failed:\n{}", String::from_utf8_lossy(&st.stderr));
    let run = Command::new(&exe).output().expect("run the linked program");
    assert!(run.status.success(), "linked program failed: {:?}", run.status);
    let got = String::from_utf8_lossy(&run.stdout).to_string();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(got, want, "native object against the interpreter");
}
