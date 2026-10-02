#!/usr/bin/env python3
"""Regression tests for gen/zig/measure.py.

These tests pin already-documented failure shapes with tiny, hand-verified
fixtures. Run with: python3 -m unittest gen/zig/test_measure.py
"""

import unittest
import sys
import os

# Ensure we can import measure from the same directory
sys.path.insert(0, os.path.dirname(__file__))
import measure


class TestSilent(unittest.TestCase):
    """Tests for the silent() function."""

    def test_declaration_line_not_mistaken_for_bare_call(self):
        """Declaration lines like `pub fn to_i64(` must not be mistaken for bare calls."""
        # Fixture: a line that is a declaration, should not increment silent count.
        output = "pub fn to_i64(a: i32, b: i32) i32 {\n"
        self.assertEqual(measure.silent(output), 0)

    def test_math_shim_bare_calls_not_double_counted(self):
        """Math-shim bare calls (abs/round/pow) must not be double-counted as deleted receivers."""
        # Fixture: a line with a math shim call like `abs(x)`; should not be counted as deleted receiver.
        output = "const result = abs(-5);\n"
        self.assertEqual(measure.silent(output), 0)

    def test_token_spaced_call_not_mistaken_for_deleted_receiver(self):
        """A token-spaced call like `( a - b ) . abs ( )` must not be mistaken for a deleted receiver."""
        # From `capture_to_semicolon`: spaces around operators and dots.
        output = "const val = ( a - b ) . abs ( );\n"
        self.assertEqual(measure.silent(output), 0)

    def test_plain_identifier_receiver_not_detected(self):
        """silent()'s detector only matches `).name(` -- plain-identifier receiver (`x.abs()` -> `abs()`) is not detected.
        
        This is a real scope boundary in the existing regex, documented here so it
        isn't rediscovered by surprise later.
        """
        # Fixture: plain-identifier receiver call.
        output = "const result = x.abs();\n"
        # We expect 0 because the regex only matches `).name(`.
        self.assertEqual(measure.silent(output), 0)

    def test_deleted_receiver_call_result_is_counted(self):
        """Deleted receiver when the receiver is a call result should be counted.
        
        Example: `remainder.abs().compare_abs(b)` -> `compare_abs(b)`.
        """
        # Fixture: a line that looks like a call on a call result, e.g., `).compare_abs(`.
        output = "const result = remainder.abs().compare_abs(b);\n"
        # We expect 1 because the pattern `).name(` matches.
        self.assertEqual(measure.silent(output), 1)


class TestTruncatedBodies(unittest.TestCase):
    """Tests for the truncated_bodies() function."""

    def test_genuinely_one_statement_function_not_flagged(self):
        """A genuinely one-statement function must not be flagged as truncated."""
        output = "fn one_stmt() void { return; }\n"
        self.assertEqual(measure.truncated_bodies(output), 0)

    def test_multi_line_struct_literal_return_is_one_statement(self):
        """A multi-line struct-literal return is one statement, not several."""
        output = """fn struct_literal() MyStruct {
        return MyStruct{
            .field1 = 1,
            .field2 = 2,
        };
    }\n"""
        self.assertEqual(measure.truncated_bodies(output), 0)

    def test_truncated_body_detected(self):
        """A function body that is truncated in the error output should be counted."""
        # Fixture: a line indicating a truncated body.
        output = "test.zig:10: error: function body truncated\n"
        self.assertEqual(measure.truncated_bodies(output), 1)


class TestParseZigTestOutput(unittest.TestCase):
    """Tests for parse_zig_test_output()."""

    def test_error_pointing_at_nonexistent_file_is_phantom_import(self):
        """An error pointing at a nonexistent file is a phantom import target, not a failing file."""
        output = "error: file not found: 'nonexistent.zig'\n"
        failing_files, phantom_imports, truncated = measure.parse_zig_test_output(output)
        self.assertEqual(failing_files, set())
        self.assertIn('nonexistent.zig', phantom_imports)

    def test_real_failing_file_is_captured(self):
        """A real failing file should be captured in failing_files."""
        output = "test.zig:10:5: error: something\n"
        failing_files, phantom_imports, truncated = measure.parse_zig_test_output(output)
        self.assertIn('test.zig', failing_files)
        self.assertEqual(phantom_imports, set())

    def test_mixed_real_and_phantom(self):
        """Mixed real and phantom files should be separated."""
        output = """real.zig:5: error: real error
error: file not found: 'missing.zig'
"""
        failing_files, phantom_imports, truncated = measure.parse_zig_test_output(output)
        self.assertIn('real.zig', failing_files)
        self.assertIn('missing.zig', phantom_imports)


if __name__ == '__main__':
    unittest.main()