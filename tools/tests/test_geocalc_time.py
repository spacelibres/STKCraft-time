"""Fail-closed checks for the independent Geocalc integration validator."""
import copy
import hashlib
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "validate_geocalc_time.py"


class ValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location("geocalc_validation", SCRIPT)
        cls.validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.validator)

    def setUp(self):
        self.key = (0, 399, 0, "same_et")
        self.row = dict(et=64.184, sec=43232, nanos=0,
                        state=[1., 2., 3., 4., 5., 6.])
        self.expected = {self.key: copy.deepcopy(self.row)}

    def check_rows(self, rows):
        return self.validator.compare_rows(rows, self.expected, 1e-6, 1e-10, 1e-6)

    def test_equal_values_pass_and_report_component_errors(self):
        result = self.check_rows({self.key: self.row})
        self.assertEqual(result[0]["max_position_error_km"], 0.)

    def test_missing_extra_and_empty_rows_fail(self):
        for rows in ({}, {**self.expected, (99, 399, 0, "same_et"): self.row}):
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                self.check_rows(rows)
        with self.assertRaises(ValueError):
            self.validator.compare_rows({}, {}, 1e-6, 1e-10, 1e-6)

    def test_nan_infinity_wrong_width_and_bad_epoch_fail(self):
        for field, value in (("et", float("nan")), ("state", [float("inf")]*6),
                             ("state", [1.]*5), ("nanos", 1000000000),
                             ("sec", 43233)):
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                self.check_rows({self.key: {**self.row, field: value}})

    def test_component_and_et_tolerances_enforced(self):
        for field, value in (("et", 64.185), ("state", [1.00001, 2., 3., 4., 5., 6.]),
                             ("state", [1., 2., 3., 4.000000001, 5., 6.])):
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.check_rows({self.key: {**self.row, field: value}})

    def test_reference_nan_also_fails(self):
        self.expected[self.key]["et"] = float("nan")
        with self.assertRaises(ValueError):
            self.check_rows({self.key: self.row})

    def test_positive_finite_options(self):
        for value in ("0", "-1", "nan", "inf", "-inf"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.validator.positive_finite(value)
        self.assertEqual(self.validator.positive_finite("1e-10"), 1e-10)

    def test_protocol_duplicate_and_malformed_rows_fail(self):
        line = "0\t399\t0\tsame_et\t64.184\t43232\t0\t1\t2\t3\t4\t5\t6\n"
        self.assertEqual(self.validator.parse_rows(line)[self.key], self.row)
        for text in (line + line, line + "junk\n", line.replace("43232", "43232.0")):
            with self.subTest(text=text), self.assertRaises(ValueError):
                self.validator.parse_rows(text)

    def test_kernel_manifest_uses_real_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "kernel.bsp"
            path.write_bytes(b"known kernel bytes")
            record = self.validator.file_record(path)
            self.assertEqual(record["sha256"], hashlib.sha256(path.read_bytes()).hexdigest())
            self.assertEqual(record["path"], str(path.resolve()))
            path.write_bytes(b"changed")
            self.assertNotEqual(record["sha256"], self.validator.file_record(path)["sha256"])

    def test_epoch_oracle_routes_supplied_numeric_tai_separately(self):
        """Synthetic oracle only tests routing, never scientific accuracy."""
        class OracleRoutingProbe:
            def __init__(self):
                self.inputs = []

            def unitim(self, value, source, target):
                self.inputs.append((value, source, target))
                return 99.5

            def spkez(self, target, et, frame, correction, observer):
                return [et] * 6, 0.

        oracle = OracleRoutingProbe()
        rows = self.validator.reference_rows(
            [dict(index=0, sec=765914437, nanos=123456789, et=61.)], oracle)
        self.assertEqual(oracle.inputs, [(765914437.1234568-43200., "TAI", "ET")])
        self.assertEqual(rows[(0, 399, 0, "utc")]["et"], 61.)
        self.assertEqual(rows[(0, 399, 0, "same_et")]["state"], [61.] * 6)
        self.assertEqual(rows[(0, 399, 0, "epoch")]["et"], 99.5)
        self.assertEqual(rows[(0, 399, 0, "epoch")]["state"], [99.5] * 6)

    def test_actual_client_with_explicit_official_kernel(self):
        """Opt in with STKCRAFT_VALIDATION_SPK; needs Cargo and spiceypy."""
        spk = os.environ.get("STKCRAFT_VALIDATION_SPK")
        if not spk:
            self.skipTest("set STKCRAFT_VALIDATION_SPK to an official local SPK")
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            self.assertEqual(self.validator.main(["--spk", spk, "--report", str(report)]), 0)
            import json
            result = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(result["status"], "passed")
            self.assertEqual(len(result["results"]), len(self.validator.DEFAULT_UTC) * 6)


if __name__ == "__main__":
    unittest.main()
