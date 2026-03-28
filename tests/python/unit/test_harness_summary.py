"""Tests for scripts/harness_summary.py."""

from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "scripts"))
from harness_summary import _parse_junit_xml, _parse_pytest_json, collect_summary


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


JUNIT_ALL_PASS = """\
<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="tests" tests="3" failures="0" errors="0">
    <testcase name="test_a" classname="mod::tests"/>
    <testcase name="test_b" classname="mod::tests"/>
    <testcase name="test_c" classname="mod::tests"/>
  </testsuite>
</testsuites>
"""

JUNIT_WITH_FAILURE = """\
<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="tests" tests="2" failures="1" errors="0">
  <testcase name="test_ok" classname="mod::tests"/>
  <testcase name="test_fail" classname="mod::tests">
    <failure message="assertion failed">expected true, got false</failure>
  </testcase>
</testsuite>
"""

PYTEST_JSON_PASS = {
    "summary": {"total": 5, "passed": 5, "failed": 0},
    "duration": 1.5,
    "tests": [
        {"nodeid": "test_a", "outcome": "passed", "duration": 0.1},
        {"nodeid": "test_b", "outcome": "passed", "duration": 0.2},
    ],
}

PYTEST_JSON_FAIL = {
    "summary": {"total": 3, "passed": 2, "failed": 1},
    "duration": 2.0,
    "tests": [
        {"nodeid": "test_a", "outcome": "passed", "duration": 0.1},
        {
            "nodeid": "test_fail",
            "outcome": "failed",
            "duration": 0.5,
            "call": {"crash": {"message": "AssertionError: 1 != 2"}},
        },
    ],
}


class TestParseJunitXml:
    def test_all_pass(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "junit.xml")
        _write(path, JUNIT_ALL_PASS)
        result = _parse_junit_xml(path)
        assert result is not None
        assert result["total"] == 3
        assert result["passed"] == 3
        assert result["failed"] == 0
        assert result["failures"] == []

    def test_with_failure(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "junit.xml")
        _write(path, JUNIT_WITH_FAILURE)
        result = _parse_junit_xml(path)
        assert result is not None
        assert result["total"] == 2
        assert result["passed"] == 1
        assert result["failed"] == 1
        assert len(result["failures"]) == 1
        assert result["failures"][0]["name"] == "test_fail"

    def test_missing_file(self):
        # type: () -> None
        result = _parse_junit_xml("/nonexistent/path.xml")
        assert result is None

    def test_invalid_xml(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "bad.xml")
        _write(path, "not xml at all")
        result = _parse_junit_xml(path)
        assert result is None


class TestParsePytestJson:
    def test_all_pass(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "report.json")
        _write(path, json.dumps(PYTEST_JSON_PASS))
        result = _parse_pytest_json(path)
        assert result is not None
        assert result["total"] == 5
        assert result["passed"] == 5
        assert result["failed"] == 0
        assert result["failures"] == []

    def test_with_failure(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "report.json")
        _write(path, json.dumps(PYTEST_JSON_FAIL))
        result = _parse_pytest_json(path)
        assert result is not None
        assert result["total"] == 3
        assert result["failed"] == 1
        assert len(result["failures"]) == 1
        assert result["failures"][0]["nodeid"] == "test_fail"

    def test_missing_file(self):
        # type: () -> None
        result = _parse_pytest_json("/nonexistent/path.json")
        assert result is None

    def test_invalid_json(self, tmp_dir):
        # type: (str) -> None
        path = os.path.join(tmp_dir, "bad.json")
        _write(path, "{broken json")
        result = _parse_pytest_json(path)
        assert result is None


class TestCollectSummary:
    def test_empty_dir(self, tmp_dir):
        # type: (str) -> None
        result = collect_summary(tmp_dir)
        assert result["ok"] is True
        assert result["rust"] is None
        assert result["python"] is None

    def test_rust_and_python(self, tmp_dir):
        # type: (str) -> None
        # Create nextest JUnit
        junit_dir = os.path.join(tmp_dir, "target", "nextest", "ci")
        os.makedirs(junit_dir)
        _write(os.path.join(junit_dir, "junit.xml"), JUNIT_ALL_PASS)
        # Create pytest JSON report
        _write(
            os.path.join(tmp_dir, "test-report.json"),
            json.dumps(PYTEST_JSON_PASS),
        )
        result = collect_summary(tmp_dir)
        assert result["ok"] is True
        assert result["rust"] is not None
        assert result["python"] is not None
        assert result["rust"]["passed"] == 3
        assert result["python"]["passed"] == 5

    def test_failure_sets_ok_false(self, tmp_dir):
        # type: (str) -> None
        _write(
            os.path.join(tmp_dir, "test-report.json"),
            json.dumps(PYTEST_JSON_FAIL),
        )
        result = collect_summary(tmp_dir)
        assert result["ok"] is False
