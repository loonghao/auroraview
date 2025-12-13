#!/usr/bin/env python
"""Gallery Test Loop - Automated testing and demo discovery.

This script provides a continuous testing loop for the Gallery:
1. Discovers all demos in examples/
2. Runs contract tests to verify API compatibility
3. Runs E2E tests with Playwright
4. Reports issues and suggests fixes

Usage:
    python scripts/test_gallery_loop.py
    python scripts/test_gallery_loop.py --watch  # Watch mode
    python scripts/test_gallery_loop.py --fix    # Auto-fix issues

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import argparse
import ast
import json
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

PROJECT_ROOT = Path(__file__).parent.parent
GALLERY_DIR = PROJECT_ROOT / "gallery"
EXAMPLES_DIR = PROJECT_ROOT / "examples"
TESTS_DIR = PROJECT_ROOT / "tests" / "python" / "integration"


@dataclass
class DemoInfo:
    """Information about a demo file."""

    id: str
    path: Path
    title: str
    docstring: str
    has_docstring: bool
    category: str
    issues: list = field(default_factory=list)


@dataclass
class TestResult:
    """Result of a test run."""

    name: str
    passed: bool
    duration: float
    error: Optional[str] = None
    output: Optional[str] = None


def discover_demos() -> list[DemoInfo]:
    """Discover all demo files in examples/."""
    demos = []

    for py_file in sorted(EXAMPLES_DIR.glob("*.py")):
        if py_file.name.startswith("__"):
            continue

        demo_id = py_file.stem
        for suffix in ["_demo", "_example", "_test"]:
            demo_id = demo_id.replace(suffix, "")

        # Parse file
        try:
            source = py_file.read_text(encoding="utf-8")
            tree = ast.parse(source)
            docstring = ast.get_docstring(tree) or ""
        except Exception as e:
            docstring = ""
            print(f"Warning: Could not parse {py_file}: {e}")

        # Extract title from docstring
        title = demo_id.replace("_", " ").title()
        if docstring:
            first_line = docstring.split("\n")[0].strip()
            if " - " in first_line:
                title = first_line.split(" - ")[0].strip()
            elif first_line:
                title = first_line.rstrip(".")

        # Infer category
        text = (py_file.name + " " + docstring).lower()
        category = "getting_started"
        category_keywords = {
            "dcc": "dcc_integration",
            "maya": "dcc_integration",
            "qt": "dcc_integration",
            "desktop": "desktop_features",
            "file": "desktop_features",
            "event": "api_patterns",
            "window": "window_features",
            "floating": "window_features",
        }
        for keyword, cat in category_keywords.items():
            if keyword in text:
                category = cat
                break

        demo = DemoInfo(
            id=demo_id,
            path=py_file,
            title=title,
            docstring=docstring,
            has_docstring=bool(docstring),
            category=category,
        )

        # Check for issues
        if not docstring:
            demo.issues.append("Missing docstring")
        elif len(docstring) < 50:
            demo.issues.append("Docstring too short")

        demos.append(demo)

    return demos


def run_pytest(test_pattern: str, verbose: bool = False) -> TestResult:
    """Run pytest with given pattern."""
    start = time.time()

    cmd = [
        sys.executable, "-m", "pytest",
        test_pattern,
        "-v" if verbose else "-q",
        "--tb=short",
        "-x",
    ]

    try:
        result = subprocess.run(
            cmd,
            cwd=str(PROJECT_ROOT),
            capture_output=True,
            text=True,
            timeout=120,
        )
        duration = time.time() - start

        passed = result.returncode == 0
        output = result.stdout + result.stderr

        return TestResult(
            name=test_pattern,
            passed=passed,
            duration=duration,
            error=None if passed else "Tests failed",
            output=output,
        )
    except subprocess.TimeoutExpired:
        return TestResult(
            name=test_pattern,
            passed=False,
            duration=120,
            error="Timeout",
        )
    except Exception as e:
        return TestResult(
            name=test_pattern,
            passed=False,
            duration=time.time() - start,
            error=str(e),
        )


def run_contract_tests() -> TestResult:
    """Run API contract tests."""
    return run_pytest("tests/python/integration/test_gallery_contract.py")


def run_e2e_tests() -> TestResult:
    """Run E2E tests."""
    return run_pytest("tests/python/integration/test_gallery_e2e.py")


def run_real_e2e_tests() -> TestResult:
    """Run real E2E tests with Gallery frontend."""
    return run_pytest("tests/python/integration/test_gallery_real_e2e.py")


def check_gallery_build() -> bool:
    """Check if Gallery is built."""
    index_html = GALLERY_DIR / "dist" / "index.html"
    return index_html.exists()


def build_gallery() -> bool:
    """Build Gallery frontend."""
    print("Building Gallery frontend...")
    try:
        subprocess.run(
            ["npm", "run", "build"],
            cwd=str(GALLERY_DIR),
            check=True,
            capture_output=True,
        )
        return True
    except subprocess.CalledProcessError as e:
        print(f"Build failed: {e}")
        return False


def generate_report(demos: list[DemoInfo], results: list[TestResult]) -> str:
    """Generate a test report."""
    lines = [
        "=" * 60,
        "Gallery Test Report",
        "=" * 60,
        "",
        f"Demos discovered: {len(demos)}",
        f"Demos with issues: {sum(1 for d in demos if d.issues)}",
        "",
    ]

    # Demo issues
    demos_with_issues = [d for d in demos if d.issues]
    if demos_with_issues:
        lines.append("Demo Issues:")
        for demo in demos_with_issues:
            lines.append(f"  - {demo.id}: {', '.join(demo.issues)}")
        lines.append("")

    # Test results
    lines.append("Test Results:")
    for result in results:
        status = "PASS" if result.passed else "FAIL"
        lines.append(f"  [{status}] {result.name} ({result.duration:.1f}s)")
        if result.error:
            lines.append(f"         Error: {result.error}")

    lines.append("")
    lines.append("=" * 60)

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Gallery Test Loop")
    parser.add_argument("--watch", action="store_true", help="Watch mode")
    parser.add_argument("--fix", action="store_true", help="Auto-fix issues")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose output")
    parser.add_argument("--skip-build", action="store_true", help="Skip Gallery build")
    args = parser.parse_args()

    print("Gallery Test Loop")
    print("=" * 40)

    # Check/build Gallery
    if not args.skip_build and not check_gallery_build():
        print("Gallery not built, building...")
        if not build_gallery():
            print("Failed to build Gallery")
            sys.exit(1)

    while True:
        # Discover demos
        print("\n1. Discovering demos...")
        demos = discover_demos()
        print(f"   Found {len(demos)} demos")

        for demo in demos:
            if demo.issues:
                print(f"   - {demo.id}: {', '.join(demo.issues)}")

        # Run tests
        print("\n2. Running tests...")
        results = []

        print("   - Contract tests...")
        results.append(run_contract_tests())

        print("   - E2E tests...")
        results.append(run_e2e_tests())

        if check_gallery_build():
            print("   - Real E2E tests...")
            results.append(run_real_e2e_tests())

        # Generate report
        report = generate_report(demos, results)
        print("\n" + report)

        # Summary
        all_passed = all(r.passed for r in results)
        if all_passed:
            print("\nAll tests passed!")
        else:
            print("\nSome tests failed. See above for details.")
            if args.fix:
                print("Auto-fix not yet implemented.")

        if not args.watch:
            break

        print("\nWatching for changes... (Ctrl+C to stop)")
        try:
            time.sleep(5)
        except KeyboardInterrupt:
            print("\nStopped.")
            break

    sys.exit(0 if all_passed else 1)


if __name__ == "__main__":
    main()
