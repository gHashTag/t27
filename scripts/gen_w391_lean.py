#!/usr/bin/env python3
"""Append Wave Loop 391 generic theorems to Trinity.TernaryInference.lean."""

from pathlib import Path

# The repository root, derived from where this file sits, not from the
# machine it was written on. This was an absolute path into one
# developer's home directory, so it could run for exactly one person --
# and the secret-scan gate rejects that path, which is how it was found.
# (The old literal is deliberately not quoted here: a comment naming it
# trips the same gate.)
ROOT = Path(__file__).resolve().parent.parent
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


def cancellation_chain(depth: int) -> tuple[str, str]:
    expr = "x"
    for i in range(depth):
        w = "plus" if i % 2 == 0 else "minus"
        expr = f"ternaryMac ({expr}) a (TernaryWeight.mk .{w})"
    rhs = "x" if depth % 2 == 0 else "ternaryMac x a (TernaryWeight.mk .plus)"
    return expr, rhs


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
    if "ternaryMacZeroWeightTwentySixPairClosureGeneric" in text:
        print("W391 Lean theorems already present.")
        return

    blocks: list[str] = []

    # 69-variable plus accumulation
    lhs, rhs = plus_chain(69)
    vars = " ".join(var_names(69))
    blocks.append(theorem_block(
        "ternaryMacAccumulateSixtyNinePlusGeneric",
        "Generic theorem: 69-variable plus accumulation. **305 generic ∀ milestone.**",
        vars, lhs, rhs,
    ))

    # 68-variable minus accumulation
    lhs, rhs = minus_chain(68)
    vars = " ".join(var_names(68))
    blocks.append(theorem_block(
        "ternaryMacAccumulateSixtyEightMinusGeneric",
        "Generic theorem: 68-variable minus accumulation lattice.",
        vars, lhs, rhs,
    ))

    # depth-51 residual cancellation (odd depth returns mac(x,a,.plus))
    lhs, rhs = cancellation_chain(51)
    blocks.append(theorem_block(
        "ternaryMacQuinquagintupleUnoCancellationGeneric",
        "Generic theorem: depth-51 activation cancellation. mac^51(x,a,[.plus,.minus,...]) = mac(x,a,.plus).",
        "x a", lhs, rhs,
    ))

    # zero-weight closure: 26 zero before + plus + 26 zero after
    lhs, rhs = zero_weight_closure(26, 26)
    vars = " ".join(var_names(53))
    blocks.append(theorem_block(
        "ternaryMacZeroWeightTwentySixPairClosureGeneric",
        "Generic theorem: 26 zero-weight MACs before and after a plus-weight MAC are transparent. **308 generic ∀ milestone.**",
        f"x {vars}", lhs, rhs,
    ))

    LEAN_FILE.write_text(text + "\n\n" + "\n\n".join(blocks) + "\n")
    print("Appended W391 theorems to TernaryInference.lean")


if __name__ == "__main__":
    main()
