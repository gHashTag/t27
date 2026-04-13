#!/usr/bin/env python3
"""
Targeted W/Z Mass Search (SIMPLIFIED, NO COMPLEX STRINGS)
Fixed: Removed variable name conflicts
"""

import math
import random
import time

PHI = 1.6180339887498948
PI = math.pi
E = math.e

W_target = 80.377
Z_target = 91.1876

found_w = []
found_z = []
threshold = 0.05

print("=" * 70)
print("  Targeted W/Z Mass Search")
print("  Threshold: {}%".format(threshold))
print("  Searching...")

count = 0
start = time.time()

# Simple loops without complex f-string formatting
for coeff in range(1, 101):
    for i in range(-8, 9):
        for j in range(-8, 9):
            for k in range(-8, 9):
                val = coeff * PHI**i * PI**j * E**k

                # Check W
                err_w = abs(val - W_target) / W_target * 100
                if err_w < threshold:
                    formula = str(coeff) + "*phi^" + str(i) + "*pi^" + str(j) + "*e^" + str(k)
                    found_w.append({
                        "expr": formula,
                        "value": val,
                        "error": err_w,
                        "method": "wz_simple"
                    })
                    count += 1

                # Check Z
                err_z = abs(val - Z_target) / Z_target * 100
                if err_z < threshold:
                    formula = str(coeff) + "*phi^" + str(i) + "*pi^" + str(j) + "*e^" + str(k)
                    found_z.append({
                        "expr": formula,
                        "value": val,
                        "error": err_z,
                        "method": "wz_simple"
                    })
                    count += 1

    # Progress every 1000 samples
    if count % 1000 == 0 and count > 0:
        print("  Progress: {} formulas found...".format(count))

elapsed = time.time() - start
print("\\nTotal: {} formulas found".format(count))
print("Elapsed: {:.1f} seconds".format(elapsed))

# Save results
timestamp = time.strftime("%Y%m%d_%H%M%S")
output = "/tmp/wz_simple_{}.txt".format(timestamp)

with open(output, "w") as f:
    f.write("# Targeted W/Z Mass Search\\n")
    f.write("# Generated: {}\\n".format(timestamp))
    f.write("# Threshold: {}%\\n".format(threshold))
    f.write("# Total found: {}\\n".format(count))

    # W results
    f.write("\\n## Top 50 W Masses\\n")
    for r in found_w[:50]:
        f.write("{}\\n".format(r["expr"]))

    f.write("\\n## Top 50 Z Masses\\n")
    for r in found_z[:50]:
        f.write("{}\\n".format(r["expr"]))

print("Results saved to: " + output)
