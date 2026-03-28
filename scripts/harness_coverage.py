"""Collect and unify coverage + mutation data into a single JSON report.

Gathers:
- Rust coverage (cargo-llvm-cov JSON summary)
- Python coverage (coverage.xml or .coverage JSON)
- Mutation scores (mutants.out/ directory)
- Existing test summary from harness_summary.py

Outputs a single JSON document for agent and CI consumption.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import xml.etree.ElementTree as ET
from typing import Any, Dict, List, Optional


def _parse_rust_llvm_cov_json(search_dir: str) -> Optional[Dict[str, Any]]:
    """Parse cargo-llvm-cov JSON summary output."""
    candidates = [
        os.path.join(search_dir, "target", "llvm-cov", "llvm-cov-summary.json"),
        os.path.join(search_dir, "llvm-cov-summary.json"),
    ]
    for path in candidates:
        if not os.path.isfile(path):
            continue
        try:
            with open(path, "r", encoding="utf-8") as fh:
                data = json.load(fh)
        except (json.JSONDecodeError, OSError):
            continue
        totals = data.get("data", [{}])[0].get("totals", {})
        lines = totals.get("lines", {})
        return {
            "lines_covered": lines.get("covered", 0),
            "lines_total": lines.get("count", 0),
            "lines_percent": round(
                100.0 * lines.get("covered", 0) / max(lines.get("count", 1), 1), 2
            ),
            "source": path,
        }
    return None


def _parse_python_coverage_xml(search_dir: str) -> Optional[Dict[str, Any]]:
    """Parse Python coverage.xml (Cobertura format)."""
    candidates = [
        os.path.join(search_dir, "coverage.xml"),
        os.path.join(search_dir, "htmlcov", "coverage.xml"),
    ]
    for path in candidates:
        if not os.path.isfile(path):
            continue
        try:
            tree = ET.parse(path)
        except ET.ParseError:
            continue
        root = tree.getroot()
        line_rate = root.get("line-rate")
        if line_rate is not None:
            try:
                percent = round(float(line_rate) * 100, 2)
            except ValueError:
                continue
            lines_valid = int(root.get("lines-valid", "0"))
            lines_covered = int(root.get("lines-covered", "0"))
            return {
                "lines_covered": lines_covered,
                "lines_total": lines_valid,
                "lines_percent": percent,
                "source": path,
            }
    return None


def _parse_mutants_out(search_dir: str) -> Optional[Dict[str, Any]]:
    """Parse mutants.out/ directory from cargo-mutants."""
    mutants_dir = os.path.join(search_dir, "mutants.out")
    if not os.path.isdir(mutants_dir):
        return None

    def _count_file(name: str) -> int:
        path = os.path.join(mutants_dir, name)
        if not os.path.isfile(path):
            return 0
        with open(path, "r", encoding="utf-8") as fh:
            return sum(1 for line in fh if line.strip())

    caught = _count_file("caught.txt")
    missed = _count_file("missed.txt")
    timeout = _count_file("timeout.txt")
    unviable = _count_file("unviable.txt")
    total_testable = caught + missed
    score = round(100.0 * caught / max(total_testable, 1), 2) if total_testable > 0 else None

    # Collect missed mutant details (up to 20)
    missed_details = []  # type: List[str]
    missed_path = os.path.join(mutants_dir, "missed.txt")
    if os.path.isfile(missed_path):
        with open(missed_path, "r", encoding="utf-8") as fh:
            for i, line in enumerate(fh):
                if i >= 20:
                    break
                missed_details.append(line.strip())

    return {
        "caught": caught,
        "missed": missed,
        "timeout": timeout,
        "unviable": unviable,
        "score": score,
        "missed_details": missed_details,
    }


def collect_coverage(search_dir: str = ".") -> Dict[str, Any]:
    """Collect all coverage and mutation data into a unified report."""
    report = {
        "ok": True,
        "rust_coverage": None,
        "python_coverage": None,
        "mutations": None,
        "source": "harness_coverage",
    }  # type: Dict[str, Any]

    report["rust_coverage"] = _parse_rust_llvm_cov_json(search_dir)
    report["python_coverage"] = _parse_python_coverage_xml(search_dir)
    report["mutations"] = _parse_mutants_out(search_dir)

    # Determine overall status
    if report["mutations"] and report["mutations"].get("missed", 0) > 0:
        report["ok"] = False

    return report


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Collect unified coverage and mutation testing report."
    )
    parser.add_argument(
        "--dir",
        default=".",
        help="Directory to search for coverage/mutation artifacts (default: current dir).",
    )
    parser.add_argument(
        "--output",
        default=None,
        help="Write report to a file instead of stdout.",
    )
    args = parser.parse_args(argv)

    report = collect_coverage(args.dir)
    text = json.dumps(report, indent=2, ensure_ascii=False)

    if args.output:
        with open(args.output, "w", encoding="utf-8") as fh:
            fh.write(text)
            fh.write("\n")
        print("Coverage report written to {0}".format(args.output))
    else:
        print(text)

    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
