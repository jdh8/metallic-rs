#!/usr/bin/env python3
"""Check historical rendering without trusting mismatched archived sources."""

import unittest
from unittest.mock import patch

import benchmark_report as report


class HistoricalReportTest(unittest.TestCase):
    def test_archive_renders_but_mixed_sources_are_rejected(self):
        directory = report.ROOT / "benchmarks/2026-09-05"
        self.assertEqual(report.render(directory),
                         (report.ROOT / "BENCHMARKS.md").read_text())
        load = report.load_snapshot

        def mismatched_source(directory, precision, expected):
            data = load(directory, precision, expected)
            if precision == "f128":
                data["source"]["source_files_sha256"]["src/lib.rs"] = "0" * 64
            return data

        with patch.object(report, "load_snapshot", mismatched_source):
            with self.assertRaisesRegex(ValueError, "different library or harness source"):
                report.render(directory)


if __name__ == "__main__":
    unittest.main()
