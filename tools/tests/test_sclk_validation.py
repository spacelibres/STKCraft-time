"""Fail-closed checks for the lossless SCLK validation protocol."""
import importlib.util
import argparse
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("sclk_validation", Path(__file__).resolve().parents[1]/"validate_sclk.py")
VALIDATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATOR)

class SclkValidationTests(unittest.TestCase):
    def test_protocol_preserves_rounding_counterexample(self):
        expected = -8387601.368572062
        actual = VALIDATOR.parse_results(f"159438732146.9826,{expected!r}\n", [159438732146.9826])
        self.assertEqual(actual, [expected])

    def test_protocol_rejects_missing_extra_duplicate_and_nonfinite(self):
        for text in ["", "1,2\n1,2\n", "2,3\n", "1,nan\n", "1,inf\n", "nan,2\n", "1,2,3\n", "junk\n"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                VALIDATOR.parse_results(text, [1.])

    def test_tolerances_are_finite_and_nonnegative(self):
        for token in ["nan", "inf", "-inf", "-1"]:
            with self.subTest(token=token), self.assertRaises(argparse.ArgumentTypeError):
                VALIDATOR.finite_nonnegative(token)
        self.assertEqual(VALIDATOR.finite_nonnegative("0"), 0.)
        self.assertEqual(VALIDATOR.finite_nonnegative("1e-9"), 1e-9)

if __name__ == "__main__":
    unittest.main()
