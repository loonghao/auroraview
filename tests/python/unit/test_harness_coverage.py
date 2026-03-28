"""Tests for scripts/harness_coverage.py."""

from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "scripts"))
from harness_coverage import (
    _parse_mutants_out,
    _parse_python_coverage_xml,
    _parse_rust_llvm_cov_json,
    collect_coverage,
)


@pytest.fixture()
def tmp_dir():
    """Create a temporary directory for test artifacts."""
    with tempfile.TemporaryDirectory() as d:
        yield d


def _write(path, content):
    # type: (str, str) -> None
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(content)


LLVM_COV_SUMMARY = {
    "data": [
        {
            "totals": {
                "lines": {"covered": 800, "count": 1000},
                "functions": {"covered": 50, "count": 100},
            }
        }
    ]
}

COVERAGE_XML = """\
<?xml version="1.0" ?>
<coverage version="7.0" timestamp="1234567890"
    lines-valid="500" lines-covered="350" line-rate="0.70"
    branches-valid="100" branches-covered="60" branch-rate="0.60">
  <packages>
    <package name="auroraview" line-rate="0.70">
      <classes/>
    </package>
  </packages>
</coverage>
"""


class TestParseRustLlvmCovJson:
    def test_valid_summary(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "llvm-cov-summary.json")
        _write(path, json.dumps(LLVM_COV_SUMMARY))
        result = _parse_rust_llvm_cov_json(tmp_dir)
        assert result is not None
        assert result["lines_covered"] == 800
        assert result["lines_total"] == 1000
        assert result["lines_percent"] == 80.0

    def test_nested_path(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "target", "llvm-cov", "llvm-cov-summary.json")
        _write(path, json.dumps(LLVM_COV_SUMMARY))
        result = _parse_rust_llvm_cov_json(tmp_dir)
        assert result is not None
        assert result["lines_percent"] == 80.0

    def test_missing_file(self, tmp_dir):
        # type: (str) -> None
        result = _parse_rust_llvm_cov_json(tmp_dir)
        assert result is None

    def test_invalid_json(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "llvm-cov-summary.json")
        _write(path, "{broken json")
        result = _parse_rust_llvm_cov_json(tmp_dir)
        assert result is None


class TestParsePythonCoverageXml:
    def test_valid_coverage(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "coverage.xml")
        _write(path, COVERAGE_XML)
        result = _parse_python_coverage_xml(tmp_dir)
        assert result is not None
        assert result["lines_covered"] == 350
        assert result["lines_total"] == 500
        assert result["lines_percent"] == 70.0

    def test_missing_file(self, tmp_dir):
        # type: (str) -> None
        result = _parse_python_coverage_xml(tmp_dir)
        assert result is None

    def test_invalid_xml(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "coverage.xml")
        _write(path, "not xml")
        result = _parse_python_coverage_xml(tmp_dir)
        assert result is None


class TestParseMutantsOut:
    def test_all_caught(self, tmp_dir):
        # type: (str) -> None
        mdir = os.path.join(tmp_dir, "mutants.out")
        os.makedirs(mdir)
        _write(os.path.join(mdir, "caught.txt"), "mutant_1\nmutant_2\nmutant_3\n")
        _write(os.path.join(mdir, "missed.txt"), "")
        _write(os.path.join(mdir, "timeout.txt"), "")
        _write(os.path.join(mdir, "unviable.txt"), "")
        result = _parse_mutants_out(tmp_dir)
        assert result is not None
        assert result["caught"] == 3
        assert result["missed"] == 0
        assert result["score"] == 100.0

    def test_with_missed(self, tmp_dir):
        # type: (str) -> None
        mdir = os.path.join(tmp_dir, "mutants.out")
        os.makedirs(mdir)
        _write(os.path.join(mdir, "caught.txt"), "m1\nm2\n")
        _write(os.path.join(mdir, "missed.txt"), "m3\n")
        _write(os.path.join(mdir, "timeout.txt"), "")
        _write(os.path.join(mdir, "unviable.txt"), "")
        result = _parse_mutants_out(tmp_dir)
        assert result is not None
        assert result["caught"] == 2
        assert result["missed"] == 1
        assert result["score"] == pytest.approx(66.67, rel=0.01)
        assert result["missed_details"] == ["m3"]

    def test_missing_dir(self, tmp_dir):
        # type: (str) -> None
        result = _parse_mutants_out(tmp_dir)
        assert result is None

    def test_empty_results(self, tmp_dir):
        # type: (str) -> None
        mdir = os.path.join(tmp_dir, "mutants.out")
        os.makedirs(mdir)
        _write(os.path.join(mdir, "caught.txt"), "")
        _write(os.path.join(mdir, "missed.txt"), "")
        result = _parse_mutants_out(tmp_dir)
        assert result is not None
        assert result["caught"] == 0
        assert result["missed"] == 0
        assert result["score"] is None


class TestCollectCoverage:
    def test_empty_dir(self, tmp_dir):
        # type: (str) -> None
        result = collect_coverage(tmp_dir)
        assert result["ok"] is True
        assert result["rust_coverage"] is None
        assert result["python_coverage"] is None
        assert result["mutations"] is None

    def test_full_report(self, tmp_dir):
        # type: (str) -> None
        # Rust coverage
        _write(
            os.path.join(tmp_dir, "llvm-cov-summary.json"),
            json.dumps(LLVM_COV_SUMMARY),
        )
        # Python coverage
        _write(os.path.join(tmp_dir, "coverage.xml"), COVERAGE_XML)
        # Mutations (all caught)
        mdir = os.path.join(tmp_dir, "mutants.out")
        os.makedirs(mdir)
        _write(os.path.join(mdir, "caught.txt"), "m1\nm2\n")
        _write(os.path.join(mdir, "missed.txt"), "")

        result = collect_coverage(tmp_dir)
        assert result["ok"] is True
        assert result["rust_coverage"]["lines_percent"] == 80.0
        assert result["python_coverage"]["lines_percent"] == 70.0
        assert result["mutations"]["score"] == 100.0

    def test_missed_mutants_sets_ok_false(self, tmp_dir):
        # type: (str) -> None
        mdir = os.path.join(tmp_dir, "mutants.out")
        os.makedirs(mdir)
        _write(os.path.join(mdir, "caught.txt"), "m1\n")
        _write(os.path.join(mdir, "missed.txt"), "m2\n")

        result = collect_coverage(tmp_dir)
        assert result["ok"] is False
        assert result["mutations"]["missed"] == 1
