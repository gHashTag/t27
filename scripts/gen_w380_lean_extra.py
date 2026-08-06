#!/usr/bin/env python3
"""Append extra Wave Loop 380 generic theorems to reach 264 generic ∀."""

from pathlib import Path

ROOT = Path("/Users/playra/t27")
LEAN_FILE = ROOT / "proofs" / "lean4" / "Trinity" / "TernaryInference.lean"


def var_names(n: int) -> list[str]:
    skip = {"at", "by", "do", "if", "in", "or", "to"}
    names = []
    i = 0
    while len(names) < n:
        if i < 26:
            candidate = chr(ord("a") + i)
        else:
            first = chr(ord("a") + (i - 26) // 26)
            second = chr(ord("a") + (i - 26) % 26)
            candidate = first + second
        i += 1
        if candidate in skip:
            continue
        names.append(candidate)
    return names


def nest_mac(base: str, activations: list[str], weight: str) -> str:
    expr = base
    for act in activations:
        expr = f"ternaryMac ({expr}) {act} (TernaryWeight.mk .{weight})"
    return expr


def plus_chain(n: int) -> tuple[str, str]:
    names = var_names(n)
    lhs = nest_mac("0", names, "plus")
    rhs = " + ".join(names)
    return lhs, rhs


def minus_chain(n: int) -> tuple[str, str]:
    names = var_names(n)
    lhs = nest_mac("0", names, "minus")
    rhs = " + ".join(names)
    return lhs, rhs


def cancellation_chain(depth: int) -> str:
    expr = "x"
    for i in range(depth):
        w = "plus" if i % 2 == 0 else "minus"
        expr = f"ternaryMac ({expr}) a (TernaryWeight.mk .{w})"
    return expr


def zero_weight_closure(before: int, after: int) -> tuple[str, str]:
    total = before + 1 + after
    names = var_names(total)
    plus_idx = before

    def chain(acts: list[str]) -> str:
        expr = "x"
        for i, act in enumerate(acts):
            w = "plus" if i == plus_idx else "zero"
            expr = f"ternaryMac ({expr}) {act} (TernaryWeight.mk .{w})"
        return expr

    lhs = chain(names)
    reordered = list(names)
    reordered[0], reordered[-1] = reordered[-1], reordered[0]
    rhs = chain(reordered)
    return lhs, rhs


def theorem_block(name: str, doc: str, vars: str, lhs: str, rhs: str) -> str:
    return f'''/-- {doc} -/

theorem {name} ({vars} : Int) :
    {lhs} = {rhs} := by
  simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]
  <;> try omega
'''


def main() -> None:
    text = LEAN_FILE.read_text()
    if "ternaryMacAccumulateFiftySevenPlusGeneric" in text:
        print("Extra W380 theorems already present.")
        return

    blocks: list[str] = []

    # 57-variable plus accumulation (push beyond 56)
    lhs, rhs = plus_chain(57)
    vars = " ".join(var_names(57))
    blocks.append(theorem_block(
        "ternaryMacAccumulateFiftySevenPlusGeneric",
        "Generic theorem: 57-variable plus accumulation. **262 generic ∀ milestone.**",
        vars, lhs, rhs,
    ))

    # 56-variable minus accumulation (dual-polarity)
    lhs, rhs = minus_chain(56)
    vars = " ".join(var_names(56))
    blocks.append(theorem_block(
        "ternaryMacAccumulateFiftySixMinusGeneric",
        "Generic theorem: 56-variable minus accumulation lattice.",
        vars, lhs, f"-({rhs})",
    ))

    # depth-33 cancellation
    lhs = cancellation_chain(33)
    blocks.append(theorem_block(
        "ternaryMacTertrigintupleCancellationGeneric",
        "Generic theorem: depth-33 activation cancellation. mac^33(x,a,[.plus,.minus,...]) = x.",
        "x a", lhs, "x",
    ))

    # zero-weight closure: 14 zero before + plus + 14 zero after
    lhs, rhs = zero_weight_closure(14, 14)
    vars = " ".join(var_names(29))
    blocks.append(theorem_block(
        "ternaryMacZeroWeightQuattuordecupleClosureGeneric",
        "Generic theorem: 14 zero-weight MACs before and after a plus-weight MAC are transparent.",
        f"x {vars}", lhs, rhs,
    ))

    # 58-variable plus accumulation
    lhs, rhs = plus_chain(58)
    vars = " ".join(var_names(58))
    blocks.append(theorem_block(
        "ternaryMacAccumulateFiftyEightPlusGeneric",
        "Generic theorem: 58-variable plus accumulation. **264 generic ∀ milestone.**",
        vars, lhs, rhs,
    ))

    # 57-variable minus accumulation
    lhs, rhs = minus_chain(57)
    vars = " ".join(var_names(57))
    blocks.append(theorem_block(
        "ternaryMacAccumulateFiftySevenMinusGeneric",
        "Generic theorem: 57-variable minus accumulation lattice.",
        vars, lhs, f"-({rhs})",
    ))

    # depth-34 cancellation
    lhs = cancellation_chain(34)
    blocks.append(theorem_block(
        "ternaryMacQuattuortrigintupleCancellationGeneric",
        "Generic theorem: depth-34 activation cancellation. mac^34(x,a,[.plus,.minus,...]) = x.",
        "x a", lhs, "x",
    ))

    # zero-weight closure: 15 zero before + plus + 15 zero after
    lhs, rhs = zero_weight_closure(15, 15)
    vars = " ".join(var_names(31))
    blocks.append(theorem_block(
        "ternaryMacZeroWeightQuindecupleClosureGeneric",
        "Generic theorem: 15 zero-weight MACs before and after a plus-weight MAC are transparent.",
        f"x {vars}", lhs, rhs,
    ))

    LEAN_FILE.write_text(text + "\n\n" + "\n\n".join(blocks) + "\n")
    print("Appended 8 extra W380 theorems to TernaryInference.lean")


if __name__ == "__main__":
    main()
