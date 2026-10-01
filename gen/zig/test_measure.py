"""Unit tests for gen/zig/measure.py."""

import unittest
from gen.zig.measure import silent, truncated_bodies, parse_zig_test_output


class TestMeasure(unittest.TestCase):
    """Test silent(), truncated_bodies(), and parse_zig_test_output()."""

    def test_silent_counts_deleted_receiver_from_call_result(self):
        """silent() counts deleted receiver when receiver is a call result.
        
        Example: `remainder.abs().compare_abs(b)` -> `compare_abs(b)`.
        The deleted receiver is `remainder.abs()`; the remaining code starts with a dot.
        """
        lines = [
            "remainder.abs().compare_abs(b)",  # contains `).compare_abs(b)`
            "foo.bar().baz()",               # contains `).baz()`
            "obj.method().call()",           # contains `).call()`
        ]
        self.assertEqual(silent(lines), 3)

    def test_silent_does_not_count_plain_identifier_receiver(self):
        """silent() does NOT count plain-identifier receiver like `x.abs()`."""
        lines = [
            "x.abs()",
            "y . round ( )",
            "obj . foo ( )",
            "abs()",  # bare call, no receiver
        ]
        self.assertEqual(silent(lines), 0)

    def test_silent_does_not_mistake_declaration_line_for_bare_call(self):
        """Declaration lines like `pub fn to_i64(` must not be mistaken for bare calls."""
        lines = [
            "pub fn to_i64(",
            "fn foo()",
            "struct Bar {",
            "pub const x = 5;",
        ]
        self.assertEqual(silent(lines), 0)

    def test_silent_does_not_double_count_math_shim_bare_calls(self):
        """Math-shim bare calls (abs/round/pow) must not be double-counted as deleted receivers."""
        lines = [
            "abs()",
            "round(3.14)",
            "pow(x, 2)",
            "foo.abs()",  # this has a receiver, but not math-shim? Actually abs is math-shim but has receiver x.
            # The test says math-shim bare calls must not be double-counted.
            # So we test bare calls only.
            "abs ()",
            "round ( )",
            "pow ( )",
        ]
        self.assertEqual(silent(lines), 0)

    def test_silent_does_not_mistake_token_spaced_call_for_deleted_receiver(self):
        """Token-spaced call `( a - b ) . abs ( )` must not be mistaken for a deleted receiver."""
        lines = [
            "( a - b ) . abs ( )",
            "(foo + bar) . baz ()",
            "(x) . method ( )",
            # Also test that normal call result without spaces still counts
            "foo().bar()",
        ]
        # Expect 1 from the last line
        self.assertEqual(silent(lines), 1)

    def test_truncated_bodies_does_not_flag_one_statement_function(self):
        """A genuinely one-statement function must not be flagged as truncated."""
        lines = [
            "pub fn one() void { return; }",
            "pub fn two() void { x = 1; }",
            "pub fn three() void { foo(); }",
            # One statement but maybe with braces on separate lines?
            "pub fn four() void {\n    return;\n}",
            "pub fn five() void {\n    x = 1;\n}",
        ]
        self.assertEqual(truncated_bodies(lines), 0)

    def test_truncated_bodies_counts_multiline_struct_literal_as_one_statement(self):
        """A multi-line struct-literal return is one statement, not several."""
        lines = [
            "pub fn struct_lit() SomeStruct { return SomeStruct{ .a = 1, .b = 2 }; }",
            "pub fn struct_lit_multi() SomeStruct {\n    return SomeStruct{\n        .a = 1,\n        .b = 2,\n    };\n}",
            "pub fn another() void { x = { .y = 4 }; }",
        ]
        self.assertEqual(truncated_bodies(lines), 0)

    def test_parse_zig_test_output_phantom_import_target(self):
        """An error pointing at a nonexistent file is a phantom import target, not a failing file."""
        lines = [
            "error: cannot find file 'nonexistent.zig'",
            "error: unable to open file 'foo/bar.zig' for reading",
            "note: imported from src/main.zig:10:5",
        ]
        failing_files, phantom_imports, truncated_body_count = parse_zig_test_output(lines)
        self.assertEqual(failing_files, set())
        self.assertEqual(phantom_imports, {"nonexistent.zig", "foo/bar.zig"})
        self.assertEqual(truncated_body_count, 0)

    def test_parse_zig_test_output_failing_file(self):
        """An error pointing at an existing file is a failing file."""
        lines = [
            "error: expected ';', got '}'",
            "src/file.zig:20:3: expected ';', got '}'",
            "another_file.zig:5:10: error: undefined variable 'x'",
        ]
        failing_files, phantom_imports, truncated_body_count = parse_zig_test_output(lines)
        self.assertEqual(failing_files, {"src/file.zig", "another_file.zig"})
        self.assertEqual(phantom_imports, set())
        self.assertEqual(truncated_body_count, 0)

    def test_parse_zig_test_output_truncated_body_count(self):
        """parse_zig_test_output() correctly counts truncated bodies."""
        lines = [
            "test SomeFunction ... still pending (function body truncated)",
            "test AnotherThing ... still pending (body truncated)",
            "test Normal ... still pending",
        ]
        failing_files, phantom_imports, truncated_body_count = parse_zig_test_output(lines)
        self.assertEqual(failing_files, set())
        self.assertEqual(phantom_imports, set())
        self.assertEqual(truncated_body_count, 2)

    def test_parse_zig_test_output_combined(self):
        """Combined test for parse_zig_test_output with multiple inputs."""
        lines = [
            "test pass ... OK",
            "test fail1 ... still pending (function body truncated)",
            "error: file_not_found.zig: No such file or directory",
            "src/file_not_found.zig:1:1: error: unable to open input file",
            "test fail2 ... still pending (body truncated)",
            "error: syntax error",
            "real_file.zig:10:5: error: expected ';'",
            "pub fn truncated() void {",
            "    // lots of code",
            "    ...",
            "}",
        ]
        failing_files, phantom_imports, truncated_body_count = parse_zig_test_output(lines)
        self.assertEqual(failing_files, {"real_file.zig"})
        self.assertEqual(phantom_imports, {"file_not_found.zig"})
        self.assertEqual(truncated_body_count, 2)  # two truncated body lines


if __name__ == "__main__":
    unittest.main()