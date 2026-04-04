/* AUTO-GENERATED from specs/ar/ternary_logic.t27 — DO NOT EDIT */
/* Ring: 18 | Module: TernaryLogic | phi^2 + 1/phi^2 = 3 */

#ifndef TERNARY_LOGIC_H
#define TERNARY_LOGIC_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* Kleene K3 Truth Values — Isomorphism: Trit ≅ Kleene K3 */
typedef int8_t Trit;

#define K_FALSE  ((Trit)-1)
#define K_UNKNOWN ((Trit)0)
#define K_TRUE   ((Trit)1)

/* Horn clause rule */
typedef struct {
    Trit antecedent;
    Trit consequent;
} Rule;

/* Kleene K3 Logical Operations */
Trit k3_and(Trit a, Trit b);
Trit k3_or(Trit a, Trit b);
Trit k3_not(Trit a);
Trit k3_implies(Trit a, Trit b);
Trit k3_equiv(Trit a, Trit b);

/* Automated Reasoning Primitives */
Trit forward_chain(Rule rule, Trit fact);
Trit backward_chain(Trit goal, const Rule *rules, size_t count);
void resolve(const Trit *clause_a, const Trit *clause_b, size_t len, Trit *result);

/* Restraint and Bounded Rationality */
bool is_restraint(Trit t);
void apply_restraint(const Trit *values, size_t len, Trit *result);

#endif /* TERNARY_LOGIC_H */
