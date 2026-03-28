"""Generate a structured JSON summary from test artifacts.

Collects results from:
- pytest JSON report (test-report.json)
- pytest JUnit XML (coverage.xml / junit-python.xml)
- nextest JUnit XML (junit.xml / agent-junit.xml)

Outputs a single machine-readable JSON summary to stdout,
designed for agent consumption and automated decision-making.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import xml.etree.ElementTree as ET
from typing import Any, Dict, List, Optional


def _parse_junit_xml(path: str) -> Optional[Dict[str, Any]]:
    """Parse a JUnit XML file into a summary dict."""
    if not os.path.isfile(path):
        return None
    try:
        tree = ET.parse(path)
    except ET.ParseError:
        return None
    root = tree.getroot()
    # Handle both <testsuites> and <testsuite> root elements
    if root.tag == "testsuites":
        suites = list(root)
    elif root.tag == "testsuite":
        suites = [root]
    else:
        return None

    total = 0
    passed = 0
    failed = 0
    skipped = 0
    errors = 0
    failures = []  # type: List[Dict[str, str]]

    for suite in suites:
        for tc in suite.iter("testcase"):
            total += 1
            fail_el = tc.find("failure")
            err_el = tc.find("error")
            skip_el = tc.find("skipped")
            if fail_el is not None:
                failed += 1
                failures.append(
                    {
                        "name": tc.get("name", ""),
                        "classname": tc.get("classname", ""),
                        "message": (fail_el.get("message") or "")[:500],
                    }
                )
            elif err_el is not None:
                errors += 1
                failures.append(
                    {
                        "name": tc.get("name", ""),
                        "classname": tc.get("classname", ""),
                        "message": (err_el.get("message") or "")[:500],
                    }
                )
            elif skip_el is not None:
                skipped += 1
            else:
                passed += 1

    return {
        "total": total,
        "passed": passed,
        "failed": failed,
        "errors": errors,
        "skipped": skipped,
        "failures": failures[:20],  # Limit to 20 failures
    }


def _parse_pytest_json(path: str) -> Optional[Dict[str, Any]]:
    """Parse a pytest-json-report file into a summary dict."""
    if not os.path.isfile(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as fh:
            data = json.load(fh)
    except (json.JSONDecodeError, OSError):
        return None

    summary = data.get("summary", {})
    failures = []  # type: List[Dict[str, str]]
    for test in data.get("tests", []):
        if test.get("outcome") in ("failed", "error"):
            call = test.get("call", {})
            failures.append(
                {
                    "nodeid": test.get("nodeid", ""),
                    "outcome": test.get("outcome", ""),
                    "message": (call.get("crash", {}).get("message") or "")[:500],
                    "duration": test.get("duration", 0),
                }
            )

    return {
        "total": summary.get("total", 0),
        "passed": summary.get("passed", 0),
        "failed": summary.get("failed", 0),
        "errors": summary.get("error", 0),
        "skipped": summary.get("deselected", 0) + summary.get("xfailed", 0),
        "duration": data.get("duration", 0),
        "failures": failures[:20],
    }


def collect_summary(search_dir: str = ".") -> Dict[str, Any]:
    """Collect all available test results into a unified summary."""
    result = {
        "ok": True,
        "rust": None,
        "python": None,
        "source": "harness_summary",
    }  # type: Dict[str, Any]

    # Try nextest JUnit artifacts
    for name in ("agent-junit.xml", "junit.xml"):
        path = os.path.join(search_dir, "target", "nextest", "ci", name)
        if not os.path.isfile(path):
            path = os.path.join(search_dir, name)
        parsed = _parse_junit_xml(path)
        if parsed is not None:
            result["rust"] = parsed
            if parsed["failed"] > 0 or parsed["errors"] > 0:
                result["ok"] = False
            break

    # Try pytest JSON report
    for name in ("test-report.json",):
        path = os.path.join(search_dir, name)
        parsed = _parse_pytest_json(path)
        if parsed is not None:
            result["python"] = parsed
            if parsed["failed"] > 0 or parsed["errors"] > 0:
                result["ok"] = False
            break

    # Fall back to pytest JUnit XML
    if result["python"] is None:
        for name in ("junit-python.xml", "coverage.xml"):
            path = os.path.join(search_dir, name)
            parsed = _parse_junit_xml(path)
            if parsed is not None:
                result["python"] = parsed
                if parsed["failed"] > 0 or parsed["errors"] > 0:
                    result["ok"] = False
                break

    return result


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Generate a structured JSON summary from test artifacts."
    )
    parser.add_argument(
        "--dir",
        default=".",
        help="Directory to search for test artifacts (default: current dir).",
    )
    parser.add_argument(
        "--output",
        default=None,
        help="Write summary to a file instead of stdout.",
    )
    args = parser.parse_args(argv)

    summary = collect_summary(args.dir)
    text = json.dumps(summary, indent=2, ensure_ascii=False)

    if args.output:
        with open(args.output, "w", encoding="utf-8") as fh:
            fh.write(text)
            fh.write("\n")
        print("Summary written to {0}".format(args.output))
    else:
        print(text)

    return 0 if summary["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
