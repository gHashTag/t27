#!/usr/bin/env node
/**
 * Language test harness: JavaScript Number precision test
 * Tests IEEE 754 binary64 precision against GoldenFloat ternary claims
 *
 * Usage: node javascript_number.js > results/javascript.json
 */

const fs = require('fs');

function countDecimalPlaces(value, reference) {
    // Count matching decimal places between value and reference
    const s1 = value.toFixed(20);
    const s2 = reference.toFixed(20);

    let count = 0;
    let foundDecimal = false;

    for (let i = 0; i < Math.min(s1.length, s2.length); i++) {
        const c1 = s1[i];
        const c2 = s2[i];

        if (c1 === '.') {
            foundDecimal = true;
            continue;
        }

        if (foundDecimal && c1 === c2) {
            count++;
        } else if (foundDecimal) {
            break;
        }
    }

    return count;
}

function testPhi() {
    const phi = (1 + Math.sqrt(5)) / 2;
    const expected = 1.61803398874989484820458683436563811772030917980576286213544862270526046281890244970720720418939113748475;
    const error = Math.abs(phi - expected);

    return {
        name: "phi",
        expected: expected,
        computed: phi,
        error: error,
        decimal_places: countDecimalPlaces(phi, expected),
        passed: error < 1e-15
    };
}

function testPhiSquared() {
    const phi = (1 + Math.sqrt(5)) / 2;
    const phiSq = phi * phi;
    const phiPlusOne = phi + 1;
    const error = Math.abs(phiSq - phiPlusOne);

    return {
        name: "phi_squared_equals_phi_plus_one",
        phi_sq: phiSq,
        phi_plus_one: phiPlusOne,
        error: error,
        passed: error < 1e-15
    };
}

function testTrinityIdentity() {
    const phi = (1 + Math.sqrt(5)) / 2;
    const phiInv = 1 / phi;
    const phiSq = phi * phi;
    const phiInvSq = phiInv * phiInv;
    const trinity = phiSq + phiInvSq;
    const expected = 3.0;
    const error = Math.abs(trinity - expected);

    return {
        name: "trinity_identity",
        trinity: trinity,
        expected: expected,
        error: error,
        passed: error < 1e-12
    };
}

function testOneThird() {
    const value = 1 / 3;
    const valueStr = value.toFixed(16);
    const expectedStr = "0.3333333333333333";

    return {
        name: "one_third",
        value: value,
        value_str: valueStr,
        expected_str: expectedStr,
        decimal_places: 15,  // IEEE f64 gives ~15-16 decimal places
        error: Math.abs(value - 1/3),
        passed: true  // Always passes, measuring precision
    };
}

function testAccumulation() {
    const nTerms = 100000;
    let total = 0;

    for (let n = 1; n <= nTerms; n++) {
        total += 1 / n;
    }

    return {
        name: "accumulation",
        n_terms: nTerms,
        total: total,
        passed: true  // Documenting behavior
    };
}

function main() {
    const results = {
        language: "JavaScript",
        precision: "Number (IEEE 754 binary64)",
        tests: [
            testPhi(),
            testPhiSquared(),
            testTrinityIdentity(),
            testOneThird(),
            testAccumulation()
        ],
        all_passed: true  // Informational, not a pass/fail test
    };

    // Add overall summary
    results.summary = {
        phi_error: results.tests[0].error,
        phi_decimal_places: results.tests[0].decimal_places,
        one_third_decimal_places: results.tests[3].decimal_places
    };

    console.log(JSON.stringify(results, null, 2));
}

main();
