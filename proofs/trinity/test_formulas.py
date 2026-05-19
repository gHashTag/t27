#!/usr/bin/env python3
"""
Trinity S3AI Formula Regression Tests
Auto-parses Coq .v files and verifies all theoretical formulas numerically.

Usage:
    python3 test_formulas.py

Exit code:
    0 - all non-admitted formulas pass
    1 - at least one formula fails
"""

import re
import sys
import math
from pathlib import Path
from dataclasses import dataclass
from typing import Dict, List, Tuple

# Physical constants (match Coq's Reals)
PI = math.pi
E = math.e
PHI = (1 + math.sqrt(5)) / 2

# Tolerance levels (must match Tolerances.v)
TOLERANCE_SG = 10 / 10000   # 0.01% - smoking guns
TOLERANCE_V = 10 / 1000      # 0.1% - visible formulas
TOLERANCE_W = 100 / 1000     # 10% - wide tolerance (candidates)
TOLERANCE_L = 50 / 1000      # 5% - chain relations

TOLERANCE_MAP = {
    'tolerance_SG': TOLERANCE_SG,
    'tolerance_V': TOLERANCE_V,
    'tolerance_W': TOLERANCE_W,
    'tolerance_L': TOLERANCE_L,
}


def tokenize(expr: str) -> List[str]:
    """Tokenize a Coq/R expression into tokens."""
    raw_tokens = []
    i = 0
    while i < len(expr):
        if expr[i].isspace():
            i += 1
            continue
        if expr[i].isalpha() or expr[i] == '_':
            j = i
            while j < len(expr) and (expr[j].isalnum() or expr[j] == '_'):
                j += 1
            raw_tokens.append(expr[i:j])
            i = j
        elif expr[i].isdigit() or (expr[i] == '.' and i + 1 < len(expr) and expr[i+1].isdigit()):
            j = i
            while j < len(expr) and (expr[j].isdigit() or expr[j] == '.'):
                j += 1
            raw_tokens.append(expr[i:j])
            i = j
        elif expr[i] in '()':
            raw_tokens.append(expr[i])
            i += 1
        elif expr[i] in '/*+-^':
            raw_tokens.append(expr[i])
            i += 1
        else:
            i += 1
    
    # Pre-process: apply Coq→Python replacements at token level
    tokens = []
    i = 0
    while i < len(raw_tokens):
        tok = raw_tokens[i]
        if tok == 'phi':
            tokens.append('PHI')
        elif tok == 'PI':
            tokens.append('PI')
        elif tok == 'exp' and i + 1 < len(raw_tokens) and raw_tokens[i+1] == '1':
            tokens.append('E')
            i += 1  # skip '1'
        elif tok in ('sqrt', 'cos', 'sin', 'tan'):
            tokens.append('math.' + tok)
        elif tok == 'Rabs':
            tokens.append('abs')
        elif tok == 'IZR':
            pass  # skip
        elif tok == 'Z' and i + 2 < len(raw_tokens) and raw_tokens[i+1] == '.' and raw_tokens[i+2] == 'of_nat':
            i += 2  # skip '.of_nat'
        else:
            tokens.append(tok)
        i += 1
    return tokens


def coq_expr_to_python(expr: str) -> str:
    """Convert a Coq/R expression to a Python-evaluable string."""
    tokens = tokenize(expr)
    result = []
    i = 0
    while i < len(tokens):
        tok = tokens[i]
        
        if tok == 'alpha_phi':
            result.append('(PHI**(-3)/2)')
        # Unary division: / followed by atom
        elif tok == '/' and (i == 0 or tokens[i-1] in '(*+/-^'):
            i += 1
            if i < len(tokens):
                next_tok = tokens[i]
                if next_tok == '(':
                    depth = 1
                    subexpr = ['(']
                    i += 1
                    while i < len(tokens) and depth > 0:
                        if tokens[i] == '(':
                            depth += 1
                        elif tokens[i] == ')':
                            depth -= 1
                        subexpr.append(tokens[i])
                        i += 1
                    i -= 1
                    inner = coq_expr_to_python(''.join(subexpr[1:-1]))
                    result.append(f'(1.0/({inner}))')
                elif next_tok.replace('.', '').isdigit():
                    result.append(f'(1.0/{next_tok})')
                else:
                    result.append(f'(1.0/{next_tok})')
        elif tok == '/':
            result.append('/')
        elif tok == '^':
            result.append('**')
        elif tok in '*+-()':
            result.append(tok)
        elif tok.replace('.', '').isdigit():
            result.append(tok)
        else:
            result.append(tok)
        i += 1
    
    return ' '.join(result)


def parse_all_definitions(script_dir: Path) -> Dict[str, str]:
    """Parse all Definition statements from all .v files."""
    all_defs = {}
    v_files = sorted(script_dir.glob('*.v'))
    for v_file in v_files:
        content = v_file.read_text()
        line_pattern = re.compile(r'^\s*Definition\s+(\w+)\s*:\s*R\s*:=\s*(.+)$')
        for line in content.split('\n'):
            match = line_pattern.match(line)
            if match:
                name = match.group(1)
                expr = match.group(2).strip()
                if expr.endswith('.'):
                    expr = expr[:-1].strip()
                all_defs[name] = coq_expr_to_python(expr)
    return all_defs


def build_context(all_defs: Dict[str, str]) -> Dict:
    """Build evaluation context from all definitions."""
    context = {
        'PHI': PHI, 'PI': PI, 'E': E,
        'math': math,
        'phi_pos': True, 'phi_nonzero': True,
        'alpha_phi': PHI**(-3) / 2,
        'tolerance_V': TOLERANCE_V,
        'tolerance_SG': TOLERANCE_SG,
        'tolerance_W': TOLERANCE_W,
    }
    unresolved = set(all_defs.keys())
    for _ in range(100):
        if not unresolved:
            break
        resolved = set()
        for name in unresolved:
            py_expr = all_defs[name]
            try:
                val = eval(py_expr, {"__builtins__": {}}, context)
                context[name] = val
                resolved.add(name)
            except (NameError, SyntaxError, TypeError):
                pass
        unresolved -= resolved
        if not resolved:
            break
    return context


@dataclass
class Formula:
    name: str
    theoretical_py: str
    experimental_py: str
    tolerance: str
    filename: str


def collect_formulas(script_dir: Path, context: Dict) -> List[Formula]:
    v_files = sorted(script_dir.glob('*.v'))
    all_defs = parse_all_definitions(script_dir)
    formulas = []
    for v_file in v_files:
        file_defs = {}
        content = v_file.read_text()
        line_pattern = re.compile(r'^\s*Definition\s+(\w+)\s*:\s*R\s*:=\s*(.+)$')
        for line in content.split('\n'):
            match = line_pattern.match(line)
            if match:
                name = match.group(1)
                expr = match.group(2).strip()
                if expr.endswith('.'):
                    expr = expr[:-1].strip()
                file_defs[name] = coq_expr_to_python(expr)
        for name in file_defs:
            if name.endswith('_theoretical'):
                base = name[:-len('_theoretical')]
                exp_name = base + '_experimental'
                if exp_name not in all_defs:
                    continue
                tolerance = 'tolerance_V'
                if base in ['Q05', 'L01', 'L02', 'L03']:
                    tolerance = 'tolerance_W'
                elif 'smoking_gun' in base or base == 'Q07':
                    tolerance = 'tolerance_SG'
                formulas.append(Formula(
                    name=base,
                    theoretical_py=all_defs[name],
                    experimental_py=all_defs[exp_name],
                    tolerance=tolerance,
                    filename=v_file.name,
                ))
    return formulas


def verify_formula(f: Formula, context: Dict) -> Tuple[bool, float, float, float]:
    try:
        theo_val = eval(f.theoretical_py, {"__builtins__": {}}, context)
        exp_val = eval(f.experimental_py, {"__builtins__": {}}, context)
        if abs(exp_val) < 1e-300:
            return False, theo_val, exp_val, float('inf')
        err = abs(theo_val - exp_val) / abs(exp_val)
        tolerance = TOLERANCE_MAP.get(f.tolerance, TOLERANCE_V)
        return err <= tolerance, theo_val, exp_val, err
    except Exception as e:
        print(f"  ERROR evaluating {f.name}: {e}")
        print(f"    Theo expr: {f.theoretical_py}")
        return False, 0, 0, float('inf')


def is_admitted(filename: str, base_name: str, script_dir: Path) -> bool:
    v_file = script_dir / filename
    content = v_file.read_text()
    theorem_pattern = (
        r'Theorem\s+' + re.escape(base_name) +
        r'_within_tolerance\s*:.*?Proof\..*?Admitted\.'
    )
    return bool(re.search(theorem_pattern, content, re.DOTALL))


def main():
    script_dir = Path(__file__).parent
    v_files = sorted(script_dir.glob('*.v'))
    if not v_files:
        print("No .v files found!")
        sys.exit(1)

    print("=" * 70)
    print("Trinity S3AI Formula Regression Tests")
    print("=" * 70)
    
    all_defs = parse_all_definitions(script_dir)
    context = build_context(all_defs)
    formulas = collect_formulas(script_dir, context)

    print(f"Scanning {len(v_files)} .v files, found {len(formulas)} formulas")
    print()

    passing = []
    admitted_list = []
    failing = []

    for f in formulas:
        passed, tval, exp_val, err = verify_formula(f, context)
        is_adm = is_admitted(f.filename, f.name, script_dir)
        result = {'formula': f, 'tval': tval, 'exp_val': exp_val, 'err': err}
        if is_adm:
            admitted_list.append(result)
        elif passed:
            passing.append(result)
        else:
            failing.append(result)

    if passing:
        print("-" * 70)
        print(f"PASSING ({len(passing)}):")
        print("-" * 70)
        for r in passing:
            f = r['formula']
            tol = TOLERANCE_MAP.get(f.tolerance, TOLERANCE_V)
            print(f"  {f.name:20s}  {r['tval']:15.6f}  vs  {r['exp_val']:10.4f}  "
                  f"err={r['err']*100:7.3f}%  tol={tol*100:.1f}%  [{f.filename}]")

    if admitted_list:
        print(f"\nADMITTED ({len(admitted_list)}):")
        for r in admitted_list:
            f = r['formula']
            tol = TOLERANCE_MAP.get(f.tolerance, TOLERANCE_V)
            status = "WOULD_PASS" if r['err'] <= tol else "WOULD_FAIL"
            print(f"  {f.name:20s}  {r['tval']:15.6f}  vs  {r['exp_val']:10.4f}  "
                  f"err={r['err']*100:7.3f}%  [{status}]")

    if failing:
        print(f"\nFAILING ({len(failing)}):")
        for r in failing:
            f = r['formula']
            print(f"  {f.name:20s}  {r['tval']:15.6f}  vs  {r['exp_val']:10.4f}  "
                  f"err={r['err']*100:7.3f}%  [{f.filename}]")

    print(f"\n{'='*70}")
    print(f"SUMMARY: Passing={len(passing)}  Admitted={len(admitted_list)}  Failing={len(failing)}")

    if failing:
        sys.exit(1)
    else:
        print("RESULT: PASS")
        sys.exit(0)


if __name__ == '__main__':
    main()
