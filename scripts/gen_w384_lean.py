#!/usr/bin/env python3
"""Append Wave Loop 384 generic theorems to Trinity.TernaryInference.lean."""

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
    return lhs, f"-({rhs})"


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
    return f"""/-- {doc} -/

theorem {name} ({vars} : Int) :
    {lhs} = {rhs} := by
  simp [ternaryMac_eq_acc_plus_mul, ternaryMul, ternaryDecode]
  <;> try omega
"""


def main() -> None:
    text = LEAN_FILE.read_text()
    if "ternaryMacZeroWeightNineteenPairClosureGeneric" in text:
        print("W384 Lean theorems already present.")
        return

    blocks: list[str] = []

    # 62-variable plus accumulation
    lhs, rhs = plus_chain(62)
    vars = " ".join(var_names(62))
    blocks.append(theorem_block(
        "ternaryMacAccumulateSixtyTwoPlusGeneric",
        "Generic theorem: 62-variable plus accumulation. **277 generic ∀ milestone.**",
        vars, lhs, rhs,
    ))

    # 61-variable minus accumulation
    lhs, rhs = minus_chain(61)
    vars = " ".join(var_names(61))
    blocks.append(theorem_block(
        "ternaryMacAccumulateSixtyOneMinusGeneric",
        "Generic theorem: 61-variable minus accumulation lattice.",
        vars, lhs, rhs,
    ))

    # depth-44 cancellation
    lhs = cancellation_chain(44)
    blocks.append(theorem_block(
        "ternaryMacQuadragintupleQuattuorCancellationGeneric",
        "Generic theorem: depth-44 activation cancellation. mac^44(x,a,[.plus,.minus,...]) = x.",
        "x a", lhs, "x",
    ))

    # zero-weight closure: 19 zero before + plus + 19 zero after
    lhs, rhs = zero_weight_closure(19, 19)
    vars = " ".join(var_names(39))
    blocks.append(theorem_block(
        "ternaryMacZeroWeightNineteenPairClosureGeneric",
        "Generic theorem: 19 zero-weight MACs before and after a plus-weight MAC are transparent. **280 generic ∀ milestone.**",
        f"x {vars}", lhs, rhs,
    ))

    LEAN_FILE.write_text(text + "\n\n" + "\n\n".join(blocks) + "\n")
    print("Appended W384 theorems to TernaryInference.lean")


if __name__ == "__main__":
    main()
