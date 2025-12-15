"""Test examples startup directly (without packed gallery).

This module tests that all examples in the examples/ directory can be launched
and run without errors. It uses subprocess to run each example and checks for:
1. Successful process start
2. No immediate crashes
3. Clean exit on termination

Usage:
    pytest tests/test_examples_startup.py -v

    # Or run directly:
    python tests/test_examples_startup.py

This is a simpler alternative to CDP-based testing that doesn't require
the packed gallery to be running.
"""

from __future__ import annotations

import os
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import pytest

# Add project root to path
PROJECT_ROOT = Path(__file__).parent.parent
EXAMPLES_DIR = PROJECT_ROOT / "examples"
sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(PROJECT_ROOT / "python"))


# ─────────────────────────────────────────────────────────────────────────────
# Test Configuration
# ─────────────────────────────────────────────────────────────────────────────

STARTUP_TIMEOUT = 5  # seconds to wait for example to start
RUN_DURATION = 2  # seconds to let example run
GRACEFUL_SHUTDOWN_TIMEOUT = 3  # seconds to wait for graceful shutdown


@dataclass
class ExampleInfo:
    """Information about an example."""

    id: str
    title: str
    file_path: Path
    requires_qt: bool = False
    requires_dcc: bool = False
    has_gui: bool = True
    tags: list = field(default_factory=list)


@dataclass
class TestResult:
    """Result of testing an example."""

    example: ExampleInfo
    success: bool
    startup_time_ms: float = 0
    error: Optional[str] = None
    stdout: str = ""
    stderr: str = ""
    exit_code: Optional[int] = None


# ─────────────────────────────────────────────────────────────────────────────
# Example Discovery
# ─────────────────────────────────────────────────────────────────────────────

# Examples that require special environments
QT_EXAMPLES = {"maya_qt_echo_demo", "qt_style_tool", "qt_custom_menu_demo"}
DCC_EXAMPLES = {"dcc_integration_example", "maya_qt_echo_demo"}

# Examples that don't have GUI (run and exit)
NON_GUI_EXAMPLES = set()

# Examples with known API issues (need to be fixed)
# These use APIs that don't exist in current WebView implementation
KNOWN_BROKEN_EXAMPLES = {
    "dcc_integration_example": "Requires DCC environment",
}


def discover_examples() -> list[ExampleInfo]:
    """Discover all testable examples."""
    examples = []

    for py_file in sorted(EXAMPLES_DIR.glob("*.py")):
        if py_file.name.startswith("__"):
            continue

        example_id = py_file.stem

        # Remove common suffixes for title
        title = example_id
        for suffix in ["_demo", "_example", "_test"]:
            title = title.replace(suffix, "")
        title = title.replace("_", " ").title()

        # Determine requirements
        requires_qt = example_id in QT_EXAMPLES
        requires_dcc = example_id in DCC_EXAMPLES
        has_gui = example_id not in NON_GUI_EXAMPLES

        examples.append(
            ExampleInfo(
                id=example_id,
                title=title,
                file_path=py_file,
                requires_qt=requires_qt,
                requires_dcc=requires_dcc,
                has_gui=has_gui,
            )
        )

    return examples


def get_testable_examples(include_broken: bool = False) -> list[ExampleInfo]:
    """Get examples that can be tested in current environment.

    Args:
        include_broken: If True, include examples with known issues
    """
    examples = discover_examples()

    # Filter out examples that require special environments
    testable = []
    for ex in examples:
        if ex.requires_dcc:
            continue  # Skip DCC-specific examples
        if ex.requires_qt:
            # Check if Qt is available
            try:
                import PySide6  # noqa: F401
            except ImportError:
                try:
                    import PySide2  # noqa: F401
                except ImportError:
                    continue  # Skip Qt examples if Qt not available

        # Skip known broken examples unless explicitly included
        if not include_broken and ex.id in KNOWN_BROKEN_EXAMPLES:
            continue

        testable.append(ex)

    return testable


def get_broken_examples() -> list[tuple[str, str]]:
    """Get list of examples with known issues."""
    return list(KNOWN_BROKEN_EXAMPLES.items())


# ─────────────────────────────────────────────────────────────────────────────
# Example Testing
# ─────────────────────────────────────────────────────────────────────────────


def run_example(example: ExampleInfo, timeout: float = STARTUP_TIMEOUT) -> TestResult:
    """Run an example and check if it starts successfully.

    For GUI examples, we start the process, wait a bit, then terminate it.
    For non-GUI examples, we wait for them to complete.
    """
    start_time = time.time()

    try:
        # Start the example process
        env = os.environ.copy()
        env["PYTHONPATH"] = str(PROJECT_ROOT / "python") + os.pathsep + env.get("PYTHONPATH", "")

        proc = subprocess.Popen(
            [sys.executable, str(example.file_path)],
            cwd=str(EXAMPLES_DIR),
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

        startup_time = (time.time() - start_time) * 1000

        if example.has_gui:
            # For GUI apps, wait a bit then check if still running
            time.sleep(RUN_DURATION)

            if proc.poll() is not None:
                # Process exited - check if it was an error
                stdout, stderr = proc.communicate(timeout=1)
                exit_code = proc.returncode

                if exit_code != 0:
                    return TestResult(
                        example=example,
                        success=False,
                        startup_time_ms=startup_time,
                        error=f"Process exited with code {exit_code}",
                        stdout=stdout,
                        stderr=stderr,
                        exit_code=exit_code,
                    )

            # Process is still running - success! Now terminate it
            proc.terminate()
            try:
                stdout, stderr = proc.communicate(timeout=GRACEFUL_SHUTDOWN_TIMEOUT)
            except subprocess.TimeoutExpired:
                proc.kill()
                stdout, stderr = proc.communicate(timeout=1)

            return TestResult(
                example=example,
                success=True,
                startup_time_ms=startup_time,
                stdout=stdout,
                stderr=stderr,
                exit_code=proc.returncode,
            )
        else:
            # For non-GUI apps, wait for completion
            try:
                stdout, stderr = proc.communicate(timeout=timeout)
                exit_code = proc.returncode

                return TestResult(
                    example=example,
                    success=exit_code == 0,
                    startup_time_ms=startup_time,
                    error=f"Exit code: {exit_code}" if exit_code != 0 else None,
                    stdout=stdout,
                    stderr=stderr,
                    exit_code=exit_code,
                )
            except subprocess.TimeoutExpired:
                proc.kill()
                stdout, stderr = proc.communicate(timeout=1)
                return TestResult(
                    example=example,
                    success=False,
                    startup_time_ms=startup_time,
                    error="Timeout waiting for completion",
                    stdout=stdout,
                    stderr=stderr,
                )

    except Exception as e:
        return TestResult(
            example=example,
            success=False,
            startup_time_ms=(time.time() - start_time) * 1000,
            error=str(e),
        )


# ─────────────────────────────────────────────────────────────────────────────
# Pytest Fixtures and Tests
# ─────────────────────────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def all_examples() -> list[ExampleInfo]:
    """Get all discovered examples."""
    return discover_examples()


@pytest.fixture(scope="module")
def testable_examples() -> list[ExampleInfo]:
    """Get testable examples for current environment."""
    return get_testable_examples()


# Generate test IDs from example IDs
def get_example_ids():
    """Get list of testable example IDs for parametrize."""
    return [ex.id for ex in get_testable_examples()]


class TestExamplesStartup:
    """Test that examples can start successfully."""

    @pytest.mark.parametrize("example_id", get_example_ids())
    def test_example_starts(self, example_id: str):
        """Test that an example can start without immediate errors."""
        examples = {ex.id: ex for ex in get_testable_examples()}
        example = examples.get(example_id)

        if not example:
            pytest.skip(f"Example {example_id} not found or not testable")

        result = run_example(example)

        # Print output for debugging
        if result.stderr:
            print(f"\nStderr:\n{result.stderr[:500]}")

        assert result.success, (
            f"Example '{example.title}' failed to start:\n"
            f"  Error: {result.error}\n"
            f"  Exit code: {result.exit_code}\n"
            f"  Stderr: {result.stderr[:200] if result.stderr else 'None'}"
        )

        print(f"✓ {example.title} started successfully ({result.startup_time_ms:.0f}ms)")

    def test_discover_examples(self, all_examples):
        """Test that we can discover examples."""
        assert len(all_examples) > 0, "No examples found"
        print(f"\nDiscovered {len(all_examples)} examples:")
        for ex in all_examples:
            flags = []
            if ex.requires_qt:
                flags.append("qt")
            if ex.requires_dcc:
                flags.append("dcc")
            flag_str = f" [{', '.join(flags)}]" if flags else ""
            print(f"  - {ex.id}: {ex.title}{flag_str}")


# ─────────────────────────────────────────────────────────────────────────────
# Standalone Runner
# ─────────────────────────────────────────────────────────────────────────────


def run_all_tests(include_broken: bool = False):
    """Run all example tests and print a summary.

    Args:
        include_broken: If True, also test examples with known issues
    """
    print("=" * 60)
    print("AuroraView Examples Startup Test Suite")
    print("=" * 60)

    # Show skipped examples
    broken = get_broken_examples()
    if broken and not include_broken:
        print(f"\nSkipping {len(broken)} examples with known issues:")
        for ex_id, reason in broken:
            print(f"  ⊘ {ex_id}: {reason}")

    examples = get_testable_examples(include_broken=include_broken)
    print(f"\nTesting {len(examples)} examples\n")

    results: list[TestResult] = []

    for example in examples:
        print(f"Testing: {example.id}...", end=" ", flush=True)

        result = run_example(example)
        results.append(result)

        if result.success:
            print(f"✓ OK ({result.startup_time_ms:.0f}ms)")
        else:
            print(f"✗ FAILED: {result.error}")
            if result.stderr:
                # Show first few lines of stderr
                lines = result.stderr.strip().split("\n")[:3]
                for line in lines:
                    print(f"    {line[:80]}")

    # Print summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)

    passed = sum(1 for r in results if r.success)
    failed = sum(1 for r in results if not r.success)
    skipped = len(broken) if not include_broken else 0

    print(f"Total: {len(results)}")
    print(f"Passed: {passed}")
    print(f"Failed: {failed}")
    if skipped:
        print(f"Skipped: {skipped} (known issues)")

    if failed > 0:
        print("\nFailed examples:")
        for r in results:
            if not r.success:
                print(f"  - {r.example.title}: {r.error}")

    # Calculate average startup time for successful tests
    successful_times = [r.startup_time_ms for r in results if r.success]
    if successful_times:
        avg_time = sum(successful_times) / len(successful_times)
        print(f"\nAverage startup time: {avg_time:.0f}ms")

    return 0 if failed == 0 else 1


def generate_report():
    """Generate a detailed report of all examples."""
    print("=" * 60)
    print("AuroraView Examples Status Report")
    print("=" * 60)

    all_examples = discover_examples()
    testable = get_testable_examples(include_broken=False)
    broken = get_broken_examples()

    print(f"\nTotal examples: {len(all_examples)}")
    print(f"Testable: {len(testable)}")
    print(f"Known issues: {len(broken)}")

    # Test all testable examples
    print("\n" + "-" * 60)
    print("Testing Stable Examples")
    print("-" * 60)

    results = []
    for example in testable:
        print(f"  {example.id}...", end=" ", flush=True)
        result = run_example(example)
        results.append(result)

        if result.success:
            print(f"✓ ({result.startup_time_ms:.0f}ms)")
        else:
            print(f"✗ {result.error[:50]}")

    # Summary
    passed = sum(1 for r in results if r.success)

    print("\n" + "-" * 60)
    print("Summary")
    print("-" * 60)
    print(f"Stable examples passing: {passed}/{len(testable)}")

    if broken:
        print(f"\nExamples needing fixes ({len(broken)}):")
        for ex_id, reason in broken:
            print(f"  - {ex_id}: {reason}")

    # List all examples by category
    print("\n" + "-" * 60)
    print("All Examples by Status")
    print("-" * 60)

    working = [r.example.id for r in results if r.success]
    failing = [r.example.id for r in results if not r.success]
    broken_ids = [ex_id for ex_id, _ in broken]

    print(f"\n✓ Working ({len(working)}):")
    for ex_id in working:
        print(f"    {ex_id}")

    if failing:
        print(f"\n✗ Failing ({len(failing)}):")
        for ex_id in failing:
            print(f"    {ex_id}")

    print(f"\n⊘ Known Issues ({len(broken_ids)}):")
    for ex_id in broken_ids:
        print(f"    {ex_id}")

    return 0


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Test AuroraView examples startup")
    parser.add_argument("--all", action="store_true", help="Include examples with known issues")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")
    args = parser.parse_args()

    if args.report:
        sys.exit(generate_report())
    else:
        sys.exit(run_all_tests(include_broken=args.all))
