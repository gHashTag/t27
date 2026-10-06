//! The interpreter's memory model, which is the oracle for every load, store
//! and copy the JIT runs: what it accepts, and what it refuses as an IR defect
//! (`Stop::Fault`) rather than letting a wrong program pass.

use t27b::eval::{Interp, Stop};
use t27b::ir::*;

fn konst(ty: Ty, v: i128) -> Expr {
    Expr { ty, kind: ExprKind::Const(v) }
}

fn slot(k: u32) -> Expr {
    Expr { ty: Ty::Ptr, kind: ExprKind::Slot(k) }
}

fn load(ty: Ty, addr: Expr, off: u32) -> Expr {
    Expr { ty, kind: ExprKind::Load { addr: Box::new(addr), off } }
}

fn store(addr: Expr, off: u32, value: Expr) -> Stmt {
    Stmt::Store { addr, off, value }
}

fn func(ret: Option<Ty>, body: Vec<Stmt>, slots: &[u32]) -> Func {
    Func {
        name: "f".into(),
        nparams: 0,
        ret,
        vars: vec![],
        body,
        line: 1,
        is_test: false,
        is_invariant: false,
        noreturn_site: 1,
        slots: slots.iter().map(|&size| SlotInfo { size, align: 8 }).collect(),
    }
}

fn run(funcs: Vec<Func>, data: Vec<Vec<u8>>) -> Result<Option<i128>, Stop> {
    let mk = |kind| Site { kind, line: 1, what: String::new(), ty: Ty::U64 };
    let prog = Program {
        module: "m".into(),
        funcs,
        sites: vec![mk(TrapKind::Overflow), mk(TrapKind::NoReturn), mk(TrapKind::Bounds)],
        mode: OverflowMode::Trap,
        unchecked: Vec::new(),
        data,
        globals: Vec::new(),
        internal_abi: Vec::new(),
    };
    Interp::new(&prog).call(0, &[])
}

fn fault(r: Result<Option<i128>, Stop>) -> String {
    match r {
        Err(Stop::Fault(m)) => m,
        other => panic!("expected a fault, got {:?}", other),
    }
}

#[test]
fn stores_and_loads_round_trip_little_endian() {
    let body = vec![
        store(slot(0), 0, konst(Ty::U32, 0x8899_aabb)),
        store(slot(0), 4, konst(Ty::I16, -2)),
        store(slot(0), 6, konst(Ty::Bool, 1)),
        store(slot(0), 7, konst(Ty::U8, 0x7f)),
        Stmt::Return(Some(load(Ty::U64, slot(0), 0))),
    ];
    assert_eq!(run(vec![func(Some(Ty::U64), body, &[8])], vec![]), Ok(Some(0x7f01_fffe_8899_aabb)));
    // A signed narrow load sign-extends.
    let body = vec![store(slot(0), 0, konst(Ty::U8, 0xf0)), Stmt::Return(Some(load(Ty::I8, slot(0), 0)))];
    assert_eq!(run(vec![func(Some(Ty::I8), body, &[1])], vec![]), Ok(Some(-16)));
}

#[test]
fn refuses_reads_of_unwritten_bytes() {
    let body = vec![store(slot(0), 0, konst(Ty::U8, 1)), Stmt::Return(Some(load(Ty::U16, slot(0), 0)))];
    let m = fault(run(vec![func(Some(Ty::U16), body, &[4])], vec![]));
    assert!(m.contains("never written"), "{}", m);
}

#[test]
fn refuses_accesses_outside_the_slot() {
    // One byte past the end, even though the frame has room after it.
    let body = vec![store(slot(0), 3, konst(Ty::U16, 1)), Stmt::Return(Some(konst(Ty::U8, 0)))];
    let m = fault(run(vec![func(Some(Ty::U8), body, &[4, 16])], vec![]));
    assert!(m.contains("outside every live slot"), "{}", m);
}

#[test]
fn refuses_stores_into_data_and_reads_past_a_blob() {
    let d = Expr { ty: Ty::Ptr, kind: ExprKind::Data(0) };
    let body = vec![store(d.clone(), 0, konst(Ty::U8, 1)), Stmt::Return(Some(konst(Ty::U8, 0)))];
    let m = fault(run(vec![func(Some(Ty::U8), body, &[])], vec![vec![1, 2, 3]]));
    assert!(m.contains("read-only"), "{}", m);
    let body = vec![Stmt::Return(Some(load(Ty::U32, d.clone(), 0)))];
    let m = fault(run(vec![func(Some(Ty::U32), body, &[])], vec![vec![1, 2, 3]]));
    assert!(m.contains("outside every blob"), "{}", m);
    let body = vec![Stmt::Return(Some(load(Ty::U16, d, 1)))];
    assert_eq!(run(vec![func(Some(Ty::U16), body, &[])], vec![vec![1, 2, 3]]), Ok(Some(0x0302)));
}

#[test]
fn refuses_a_bool_load_of_another_byte() {
    let body = vec![store(slot(0), 0, konst(Ty::U8, 2)), Stmt::Return(Some(load(Ty::Bool, slot(0), 0)))];
    let m = fault(run(vec![func(Some(Ty::Bool), body, &[1])], vec![]));
    assert!(m.contains("bool load of 2"), "{}", m);
}

#[test]
fn copy_carries_written_bytes_only() {
    // s1 = s0 where only s0[0] was written: s1[0] reads back, s1[1] faults.
    let mk = |off| {
        vec![
            store(slot(0), 0, konst(Ty::U8, 9)),
            Stmt::Copy { dst: slot(1), src: slot(0), size: 2 },
            Stmt::Return(Some(load(Ty::U8, slot(1), off))),
        ]
    };
    assert_eq!(run(vec![func(Some(Ty::U8), mk(0), &[2, 2])], vec![]), Ok(Some(9)));
    let m = fault(run(vec![func(Some(Ty::U8), mk(1), &[2, 2])], vec![]));
    assert!(m.contains("never written"), "{}", m);
}

#[test]
fn a_slot_dies_with_its_call() {
    // f1 returns the address of its own slot; loading through it faults.
    let callee = func(
        Some(Ty::Ptr),
        vec![store(slot(0), 0, konst(Ty::U64, 5)), Stmt::Return(Some(slot(0)))],
        &[8],
    );
    let call = Expr { ty: Ty::Ptr, kind: ExprKind::Call { func: 1, args: vec![] } };
    let caller = func(Some(Ty::U64), vec![Stmt::Return(Some(load(Ty::U64, call, 0)))], &[]);
    let m = fault(run(vec![caller, callee], vec![]));
    assert!(m.contains("outside every live slot"), "{}", m);
}

#[test]
fn bounds_traps_with_index_and_length() {
    let chk = |i| Expr {
        ty: Ty::U64,
        kind: ExprKind::Bounds { idx: Box::new(konst(Ty::U64, i)), len: Box::new(konst(Ty::U64, 4)), site: 2 },
    };
    assert_eq!(run(vec![func(Some(Ty::U64), vec![Stmt::Return(Some(chk(3)))], &[])], vec![]), Ok(Some(3)));
    assert_eq!(
        run(vec![func(Some(Ty::U64), vec![Stmt::Return(Some(chk(4)))], &[])], vec![]),
        Err(Stop::Trap { site: 2, a: 4, b: 4 })
    );
}
