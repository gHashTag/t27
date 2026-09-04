"""Regression suite for measure.py's counting logic.

measure.py's own docstrings document nine-plus wrong-direction miscounts
found only by manually opening a generated file -- never by a test on the
script itself. Each test here pins one of those already-documented failure
shapes with a tiny, hand-verified fixture, so a future change to the
counting logic cannot silently reintroduce one.

Run: python3 gen/zig/test_measure.py
"""
import os
import shutil
import tempfile
import unittest

import measure


class TempCorpus:
    """Points measure.ROOT/measure.GEN at a scratch directory for the
    duration of a test, then restores them. silent()/truncated_bodies()/
    parse_zig_test_output() all read ROOT/GEN as module globals at call
    time, so reassigning the attributes on the imported module is enough --
    no monkeypatching library needed for this one substitution.
    """

    def __enter__(self):
        self.tmpdir = tempfile.mkdtemp(prefix="measure_test_")
        self.orig_root, self.orig_gen = measure.ROOT, measure.GEN
        measure.ROOT = self.tmpdir
        measure.GEN = os.path.join(self.tmpdir, "gen", "zig")
        os.makedirs(os.path.join(self.tmpdir, "specs"), exist_ok=True)
        os.makedirs(measure.GEN, exist_ok=True)
        return self

    def __exit__(self, *exc):
        measure.ROOT, measure.GEN = self.orig_root, self.orig_gen
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def write_spec(self, rel_path, content):
        p = os.path.join(self.tmpdir, "specs", rel_path)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "w") as f:
            f.write(content)
        return p

    def write_gen(self, rel_path, content):
        p = os.path.join(measure.GEN, rel_path)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, "w") as f:
            f.write(content)
        return p


class TestSilent(unittest.TestCase):
    """silent(): call sites where the emitter deleted a method call's
    receiver, e.g. `x.abs()` -> `abs()`."""

    def test_intact_chained_call_is_not_counted(self):
        with TempCorpus() as c:
            c.write_spec("a.t27", "let y = remainder.abs().compare_abs(b);")
            c.write_gen("a.zig", "const y = remainder.abs().compare_abs(b);")
            tot, files = measure.silent()
            self.assertEqual(tot, 0)
            self.assertEqual(files, {})

    def test_deleted_receiver_is_counted(self):
        """The detector's own pattern is `\\).name(` -- it only sees a
        deleted receiver when the receiver is itself a call result (the
        docstring's own example: `remainder.abs().compare_abs(b)` ->
        `compare_abs(b)`). A plain identifier receiver (`x.abs()`) is
        outside what this specific regex claims to catch -- confirmed by
        writing that case first and watching it correctly report 0."""
        with TempCorpus() as c:
            c.write_spec("a.t27", "let y = remainder.abs().compare_abs(b);")
            c.write_gen("a.zig", "const y = compare_abs(b);")  # .abs() chain link dropped
            tot, files = measure.silent()
            self.assertEqual(tot, 1)

    def test_declaration_line_is_not_mistaken_for_a_bare_call(self):
        """Regression: `pub fn to_i64(` looks exactly like a bare call to
        `to_i64` and once turned a true count of 14 into 1 when declarations
        weren't excluded."""
        with TempCorpus() as c:
            c.write_spec("a.t27", "let y = x.to_i64();")
            c.write_gen(
                "a.zig",
                "pub fn to_i64(x: i64) i64 { return x; }\n"
                "const y = x.to_i64();\n",
            )
            tot, files = measure.silent()
            self.assertEqual(tot, 0)

    def test_math_shim_bare_calls_are_not_contamination(self):
        """Regression: abs/round/pow are math shims the specs legitimately
        call as free functions (9, 15, 1 times in three real files). Once
        uncorrected every one of those was counted as a deleted receiver,
        reporting 16 where the truth was 5. Here `abs` both (a) appears in a
        genuine chain (so it enters the detector's candidate set at all) and
        (b) is separately called bare as a shim, intact in both spec and
        gen -- the legitimate bare use must not itself register as damage."""
        with TempCorpus() as c:
            c.write_spec(
                "a.t27",
                "let y = remainder.abs().compare_abs(b); let w = abs(z);",
            )
            c.write_gen(
                "a.zig",
                "const y = remainder.abs().compare_abs(b);\nconst w = abs(z);\n",
            )
            tot, files = measure.silent()
            self.assertEqual(tot, 0)

    def test_spaced_dot_call_is_not_mistaken_for_deleted(self):
        """Regression: capture_to_semicolon joins tokens with single spaces,
        so an INTACT call renders as `( a - b ) . abs ( )`. A plain
        not-preceded-by-a-dot test flags this as a deleted receiver even
        though the receiver is right there, just spaced."""
        with TempCorpus() as c:
            c.write_spec("a.t27", "let y = (a - b).abs();")
            c.write_gen("a.zig", "const y = ( a - b ) . abs ( ) ;")
            tot, files = measure.silent()
            self.assertEqual(tot, 0)


class TestTruncatedBodies(unittest.TestCase):
    """truncated_bodies(): a function body reduced to parameter discards
    plus a single `return`, the signature left behind by a parse error's
    recovery discarding everything else."""

    def test_genuinely_truncated_body_is_counted(self):
        with TempCorpus() as c:
            c.write_spec(
                "a.t27",
                "fn foo(x: i64) i64 {\n"
                "    let a = x + 1;\n"
                "    let b = a * 2;\n"
                "    return b;\n"
                "}\n",
            )
            c.write_gen(
                "a.zig",
                "pub fn foo(x: i64) i64 {\n"
                "    _ = &x;\n"
                "    return x;\n"
                "}\n",
            )
            hits = measure.truncated_bodies()
            self.assertEqual(hits, ["a.zig"])

    def test_a_real_one_statement_function_is_not_flagged(self):
        """Regression: a spec function that really is one statement produces
        the same shape as a truncated one. Counted without confirming
        against the spec, this reported 49; with the check, 13; the true
        number for a genuinely one-line function is 0."""
        with TempCorpus() as c:
            c.write_spec("a.t27", "fn identity(x: i64) i64 {\n    return x;\n}\n")
            c.write_gen(
                "a.zig",
                "pub fn identity(x: i64) i64 {\n"
                "    _ = &x;\n"
                "    return x;\n"
                "}\n",
            )
            hits = measure.truncated_bodies()
            self.assertEqual(hits, [])

    def test_multiline_struct_literal_return_is_one_statement_not_several(self):
        """Regression: counting STATEMENT LINES instead of statement
        TERMINATORS made a six-line struct-literal return look like six
        statements, reporting 13 truncated bodies where the truth is 0."""
        with TempCorpus() as c:
            c.write_spec(
                "a.t27",
                "fn make() Result {\n"
                "    return Result{\n"
                "        .a = 1,\n"
                "        .b = 2,\n"
                "        .c = 3,\n"
                "    };\n"
                "}\n",
            )
            c.write_gen(
                "a.zig",
                "pub fn make() Result {\n"
                "    return Result{\n"
                "        .a = 1,\n"
                "        .b = 2,\n"
                "        .c = 3,\n"
                "    };\n"
                "}\n",
            )
            hits = measure.truncated_bodies()
            self.assertEqual(hits, [])


class TestParseZigTestOutput(unittest.TestCase):
    """The pure parsing step of measure(): splits real failing files from
    phantom import targets and tallies error-message classes."""

    def test_error_in_an_existing_file_counts_as_a_failing_file(self):
        with TempCorpus() as c:
            c.write_gen("a.zig", "const x: i64 = undeclared_name;\n")
            output = "gen/zig/a.zig:1:16: error: use of undeclared identifier 'undeclared_name'\n"
            files, classes, phantom = measure.parse_zig_test_output(output, measure.GEN)
            self.assertEqual(files, {"a.zig"})
            self.assertEqual(phantom, set())

    def test_error_in_a_nonexistent_file_is_phantom_not_failing(self):
        """Regression: a path that does not exist on disk is an import
        target some spec declares that the corpus never provides -- the
        emitter is faithful, the defect is in the spec. Counting it as a
        failing FILE inflated the headline; two were once attributed to a
        patch before this was caught."""
        with TempCorpus() as c:
            # base/benchmarking.zig is never written -- deliberately absent.
            output = "gen/zig/base/benchmarking.zig:1:1: error: unable to find 'base/benchmarking.zig'\n"
            files, classes, phantom = measure.parse_zig_test_output(output, measure.GEN)
            self.assertEqual(files, set())
            self.assertEqual(phantom, {"base/benchmarking.zig"})

    def test_error_classes_are_tallied_by_message_prefix(self):
        with TempCorpus() as c:
            c.write_gen("a.zig", "")
            c.write_gen("b.zig", "")
            output = (
                "gen/zig/a.zig:1:1: error: use of undeclared identifier 'x'\n"
                "gen/zig/b.zig:2:1: error: use of undeclared identifier 'y'\n"
            )
            files, classes, phantom = measure.parse_zig_test_output(output, measure.GEN)
            self.assertEqual(files, {"a.zig", "b.zig"})
            self.assertEqual(classes["use of undeclared identifier"], 2)


if __name__ == "__main__":
    unittest.main(verbosity=2)
