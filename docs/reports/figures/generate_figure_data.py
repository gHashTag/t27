#!/usr/bin/env python3
"""
Trinity S^3AI -- Figure Data Generator
Generates placeholder datasets for PRL manuscript figures.
Usage: python3 generate_figure_data.py
Outputs: figure1_600cell_projection.csv, figure2_heat_kernel.csv
"""

import csv
import math

PHI = (1.0 + math.sqrt(5.0)) / 2.0


def generate_600cell_vertices():
    """
    Generate the 120 vertices of the 600-cell as unit quaternions.
    Returns list of dicts with keys: q0,q1,q2,q3, generation, orbit_index.
    """
    vertices = []

    # Set A: 24 permutations of (+-1, +-1, +-1, +-1)/2
    from itertools import product, permutations
    seen = set()
    for signs in product([-1, 1], repeat=4):
        tup = tuple(sorted([(abs(x), s) for x, s in zip([1, 1, 1, 1], signs)]))
        if tup not in seen:
            seen.add(tup)
            q = [s * 0.5 for s in signs]
            vertices.append(q)

    # Set B: 64 even permutations of (+-phi, +-1, +-1/phi, 0)/2
    # Simplified: generate representative subset and scale
    coords = [PHI, 1.0, 1.0 / PHI, 0.0]
    for perm in permutations(coords):
        for signs in product([-1, 1], repeat=4):
            q = [perm[i] * signs[i] * 0.5 for i in range(4)]
            norm = math.sqrt(sum(x * x for x in q))
            if abs(norm - 1.0) < 1e-9:
                qn = tuple(round(x, 12) for x in q)
                if qn not in seen:
                    seen.add(qn)
                    vertices.append(list(qn))

    # Set C: 32 permutations of (+-phi^2, +-1/phi^2, 0, 0)/2
    coords_c = [PHI ** 2, 1.0 / PHI ** 2, 0.0, 0.0]
    for perm in permutations(coords_c):
        for signs in product([-1, 1], repeat=4):
            q = [perm[i] * signs[i] * 0.5 for i in range(4)]
            norm = math.sqrt(sum(x * x for x in q))
            if abs(norm - 1.0) < 1e-9:
                qn = tuple(round(x, 12) for x in q)
                if qn not in seen:
                    seen.add(qn)
                    vertices.append(list(qn))

    # Assign generation by vertex index modulo 3 (simulating 53-cycle orbit)
    result = []
    for idx, v in enumerate(vertices[:120]):
        gen = (idx % 3) + 1
        result.append(
            {
                "q0": v[0],
                "q1": v[1],
                "q2": v[2],
                "q3": v[3],
                "generation": gen,
                "orbit_index": idx // 3,
            }
        )
    return result


def stereographic_projection_3d(q):
    """Project 4D unit quaternion to 3D via stereographic projection."""
    denom = 1.0 - q[3] + 1e-12
    x = q[0] / denom
    y = q[1] / denom
    z = q[2] / denom
    return x, y, z


def generate_figure1_csv():
    """Generate CSV for Figure 1: 600-cell 3D projection colored by generation."""
    verts = generate_600cell_vertices()
    rows = []
    for v in verts:
        x, y, z = stereographic_projection_3d([v["q0"], v["q1"], v["q2"], v["q3"]])
        rows.append(
            {
                "x": round(x, 6),
                "y": round(y, 6),
                "z": round(z, 6),
                "generation": v["generation"],
                "orbit_index": v["orbit_index"],
            }
        )
    with open("figure1_600cell_projection.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["x", "y", "z", "generation", "orbit_index"])
        writer.writeheader()
        writer.writerows(rows)
    print(f"Wrote {len(rows)} vertices to figure1_600cell_projection.csv")


def heat_kernel_trace(n_eigen, Lambda, max_order=4):
    """
    Compute spectral action coefficients via heat-kernel expansion.
    Simplified: a_0 = N/16pi^2, a_2 = sum(eigenvalues^2)/48pi^2, etc.
    """
    import random
    random.seed(42)

    # Simulate 480 eigenvalues symmetric about zero with phi-spacing
    eigenvalues = []
    for i in range(n_eigen // 2):
        lam = PHI * (i + 1) * 0.1 + random.gauss(0, 0.02)
        eigenvalues.append(lam)
        eigenvalues.append(-lam)

    a0 = n_eigen / (16.0 * math.pi ** 2)
    a2 = sum(lam ** 2 for lam in eigenvalues) / (48.0 * math.pi ** 2)
    a4 = sum(lam ** 4 for lam in eigenvalues) / (360.0 * math.pi ** 2)

    action_vals = []
    for cutoff in range(1, max_order + 1):
        Lambda_local = Lambda * cutoff
        s = sum(math.exp(-(lam ** 2) / (Lambda_local ** 2)) for lam in eigenvalues)
        action_vals.append({"cutoff_order": cutoff, "Lambda": round(Lambda_local, 4), "action": round(s, 4)})
    return action_vals, {"a0": round(a0, 6), "a2": round(a2, 6), "a4": round(a4, 6)}


def generate_figure2_csv():
    """Generate CSV for Figure 2: Heat-kernel convergence of spectral action."""
    vals, coeffs = heat_kernel_trace(n_eigen=480, Lambda=1.0, max_order=8)
    with open("figure2_heat_kernel.csv", "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=["cutoff_order", "Lambda", "action"])
        writer.writeheader()
        writer.writerows(vals)
    print(f"Wrote {len(vals)} heat-kernel points to figure2_heat_kernel.csv")
    print(f"Spectral coefficients: a0={coeffs['a0']}, a2={coeffs['a2']}, a4={coeffs['a4']}")


if __name__ == "__main__":
    generate_figure1_csv()
    generate_figure2_csv()
