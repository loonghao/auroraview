# -*- coding: utf-8 -*-
"""Snapshot testing support for AuroraView.

This module provides utilities for snapshot testing - comparing current output
against previously saved "golden" snapshots to detect regressions.

Snapshot testing is useful for:
- UI regression testing (HTML structure, rendered output)
- API response validation
- Configuration file changes
- Any output that should remain stable

Example:
    >>> from auroraview.testing.snapshot import SnapshotTest
    >>>
    >>> def test_html_output(snapshot):
    ...     html = render_component()
    ...     snapshot.assert_match(html, "component_output.html")
    >>>
    >>> def test_api_response(snapshot):
    ...     response = api.get_user(1)
    ...     snapshot.assert_match_json(response, "user_response.json")

Requirements:
    pip install pytest-snapshot (optional, for pytest integration)
"""

from __future__ import annotations

import difflib
import hashlib
import json
import os
from pathlib import Path
from typing import Any, Callable, Optional, Union


class SnapshotMismatchError(AssertionError):
    """Raised when snapshot doesn't match current output."""

    def __init__(
        self,
        message: str,
        expected: str,
        actual: str,
        diff: Optional[str] = None,
    ):
        super().__init__(message)
        self.expected = expected
        self.actual = actual
        self.diff = diff


class SnapshotTest:
    """Snapshot testing utility.

    Manages snapshot files and comparisons for regression testing.

    Args:
        snapshot_dir: Directory to store snapshots
        update_snapshots: If True, update snapshots instead of comparing

    Example:
        >>> snapshot = SnapshotTest("tests/snapshots")
        >>> snapshot.assert_match("<div>Hello</div>", "hello.html")
    """

    def __init__(
        self,
        snapshot_dir: Union[str, Path] = "snapshots",
        update_snapshots: bool = False,
    ):
        self.snapshot_dir = Path(snapshot_dir)
        self.update_snapshots = update_snapshots or os.environ.get(
            "UPDATE_SNAPSHOTS", ""
        ).lower() in ("1", "true", "yes")
        self._ensure_dir()

    def _ensure_dir(self) -> None:
        """Ensure snapshot directory exists."""
        self.snapshot_dir.mkdir(parents=True, exist_ok=True)

    def _get_snapshot_path(self, name: str) -> Path:
        """Get full path for a snapshot file."""
        return self.snapshot_dir / name

    def _read_snapshot(self, name: str) -> Optional[str]:
        """Read existing snapshot content."""
        path = self._get_snapshot_path(name)
        if path.exists():
            return path.read_text(encoding="utf-8")
        return None

    def _write_snapshot(self, name: str, content: str) -> None:
        """Write snapshot content to file."""
        path = self._get_snapshot_path(name)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def _generate_diff(self, expected: str, actual: str) -> str:
        """Generate unified diff between expected and actual."""
        expected_lines = expected.splitlines(keepends=True)
        actual_lines = actual.splitlines(keepends=True)

        diff = difflib.unified_diff(
            expected_lines,
            actual_lines,
            fromfile="expected (snapshot)",
            tofile="actual (current)",
            lineterm="",
        )
        return "".join(diff)

    def assert_match(
        self,
        content: str,
        name: str,
        normalize: Optional[Callable[[str], str]] = None,
    ) -> None:
        """Assert that content matches snapshot.

        Args:
            content: Current content to compare
            name: Snapshot file name
            normalize: Optional function to normalize content before comparison

        Raises:
            SnapshotMismatchError: If content doesn't match snapshot
        """
        if normalize:
            content = normalize(content)

        existing = self._read_snapshot(name)

        if existing is None or self.update_snapshots:
            # Create or update snapshot
            self._write_snapshot(name, content)
            if existing is None:
                print(f"Created new snapshot: {name}")
            else:
                print(f"Updated snapshot: {name}")
            return

        if normalize:
            existing = normalize(existing)

        if content != existing:
            diff = self._generate_diff(existing, content)
            raise SnapshotMismatchError(
                f"Snapshot mismatch for '{name}'.\n"
                f"Run with UPDATE_SNAPSHOTS=1 to update.\n\n"
                f"Diff:\n{diff}",
                expected=existing,
                actual=content,
                diff=diff,
            )

    def assert_match_json(
        self,
        data: Any,
        name: str,
        indent: int = 2,
        sort_keys: bool = True,
    ) -> None:
        """Assert that JSON data matches snapshot.

        Args:
            data: JSON-serializable data
            name: Snapshot file name
            indent: JSON indentation
            sort_keys: Sort dictionary keys

        Raises:
            SnapshotMismatchError: If data doesn't match snapshot
        """
        content = json.dumps(data, indent=indent, sort_keys=sort_keys, ensure_ascii=False)
        self.assert_match(content, name)

    def assert_match_html(
        self,
        html: str,
        name: str,
        normalize_whitespace: bool = True,
    ) -> None:
        """Assert that HTML matches snapshot.

        Args:
            html: HTML content
            name: Snapshot file name
            normalize_whitespace: Normalize whitespace before comparison

        Raises:
            SnapshotMismatchError: If HTML doesn't match snapshot
        """

        def normalize(content: str) -> str:
            if normalize_whitespace:
                # Normalize whitespace while preserving structure
                lines = content.splitlines()
                lines = [line.strip() for line in lines if line.strip()]
                return "\n".join(lines)
            return content

        self.assert_match(html, name, normalize=normalize)

    def hash(self, content: str) -> str:
        """Generate hash of content for quick comparison.

        Args:
            content: Content to hash

        Returns:
            SHA256 hash string
        """
        return hashlib.sha256(content.encode("utf-8")).hexdigest()

    def assert_hash_match(
        self,
        content: str,
        name: str,
    ) -> None:
        """Assert that content hash matches snapshot hash.

        Useful for large content where storing full snapshot is impractical.

        Args:
            content: Content to compare
            name: Snapshot file name (will store hash)

        Raises:
            SnapshotMismatchError: If hash doesn't match
        """
        current_hash = self.hash(content)
        self.assert_match(current_hash, f"{name}.hash")


class ScreenshotSnapshot(SnapshotTest):
    """Screenshot-based snapshot testing.

    Extends SnapshotTest with image comparison capabilities.

    Example:
        >>> snapshot = ScreenshotSnapshot("tests/screenshots")
        >>> snapshot.assert_screenshot_match(page, "homepage.png")
    """

    def __init__(
        self,
        snapshot_dir: Union[str, Path] = "screenshots",
        update_snapshots: bool = False,
        threshold: float = 0.01,
    ):
        """Initialize screenshot snapshot.

        Args:
            snapshot_dir: Directory to store screenshots
            update_snapshots: If True, update snapshots
            threshold: Maximum allowed difference ratio (0-1)
        """
        super().__init__(snapshot_dir, update_snapshots)
        self.threshold = threshold

    def assert_screenshot_match(
        self,
        screenshot_data: bytes,
        name: str,
    ) -> None:
        """Assert that screenshot matches snapshot.

        Args:
            screenshot_data: PNG image data
            name: Snapshot file name

        Raises:
            SnapshotMismatchError: If screenshot doesn't match
        """
        path = self._get_snapshot_path(name)

        if not path.exists() or self.update_snapshots:
            path.write_bytes(screenshot_data)
            if not path.exists():
                print(f"Created new screenshot: {name}")
            else:
                print(f"Updated screenshot: {name}")
            return

        existing_data = path.read_bytes()

        # Simple byte comparison first
        if screenshot_data == existing_data:
            return

        # Try pixel-level comparison if PIL is available
        try:
            import io

            from PIL import Image

            existing_img = Image.open(io.BytesIO(existing_data))
            current_img = Image.open(io.BytesIO(screenshot_data))

            if existing_img.size != current_img.size:
                raise SnapshotMismatchError(
                    f"Screenshot size mismatch for '{name}': "
                    f"expected {existing_img.size}, got {current_img.size}",
                    expected=str(existing_img.size),
                    actual=str(current_img.size),
                )

            # Calculate pixel difference
            diff_count = 0
            total_pixels = existing_img.width * existing_img.height

            existing_pixels = list(existing_img.getdata())
            current_pixels = list(current_img.getdata())

            for ep, cp in zip(existing_pixels, current_pixels):
                if ep != cp:
                    diff_count += 1

            diff_ratio = diff_count / total_pixels

            if diff_ratio > self.threshold:
                raise SnapshotMismatchError(
                    f"Screenshot mismatch for '{name}': "
                    f"{diff_ratio:.2%} pixels differ (threshold: {self.threshold:.2%})",
                    expected=f"{total_pixels - diff_count} matching pixels",
                    actual=f"{diff_count} different pixels",
                )

        except ImportError as e:
            # PIL not available, fall back to byte comparison
            raise SnapshotMismatchError(
                f"Screenshot mismatch for '{name}' (byte comparison). "
                f"Install Pillow for pixel-level comparison.",
                expected="<binary>",
                actual="<binary>",
            ) from e


# ─────────────────────────────────────────────────────────────────────────────
# Pytest Integration
# ─────────────────────────────────────────────────────────────────────────────


def pytest_snapshot_fixture(request) -> SnapshotTest:
    """Create snapshot fixture for pytest.

    Usage in conftest.py:
        >>> import pytest
        >>> from auroraview.testing.snapshot import pytest_snapshot_fixture
        >>>
        >>> @pytest.fixture
        >>> def snapshot(request):
        ...     return pytest_snapshot_fixture(request)

    Then in tests:
        >>> def test_output(snapshot):
        ...     snapshot.assert_match(output, "output.txt")
    """
    # Get test file directory
    test_dir = Path(request.fspath).parent
    snapshot_dir = test_dir / "snapshots"

    # Check for update flag
    update = request.config.getoption("--update-snapshots", default=False)

    return SnapshotTest(snapshot_dir, update_snapshots=update)


def pytest_screenshot_fixture(request) -> ScreenshotSnapshot:
    """Create screenshot snapshot fixture for pytest.

    Usage in conftest.py:
        >>> import pytest
        >>> from auroraview.testing.snapshot import pytest_screenshot_fixture
        >>>
        >>> @pytest.fixture
        >>> def screenshot_snapshot(request):
        ...     return pytest_screenshot_fixture(request)
    """
    test_dir = Path(request.fspath).parent
    snapshot_dir = test_dir / "screenshots"
    update = request.config.getoption("--update-snapshots", default=False)

    return ScreenshotSnapshot(snapshot_dir, update_snapshots=update)


# ─────────────────────────────────────────────────────────────────────────────
# Utility Functions
# ─────────────────────────────────────────────────────────────────────────────


def normalize_html(html: str) -> str:
    """Normalize HTML for comparison.

    - Removes extra whitespace
    - Normalizes attribute order
    - Removes comments

    Args:
        html: HTML string

    Returns:
        Normalized HTML string
    """
    import re

    # Remove HTML comments
    html = re.sub(r"<!--.*?-->", "", html, flags=re.DOTALL)

    # Normalize whitespace
    html = re.sub(r"\s+", " ", html)

    # Remove whitespace around tags
    html = re.sub(r">\s+<", "><", html)

    # Trim
    return html.strip()


def normalize_json(data: Any) -> str:
    """Normalize JSON data for comparison.

    Args:
        data: JSON-serializable data

    Returns:
        Normalized JSON string
    """
    return json.dumps(data, indent=2, sort_keys=True, ensure_ascii=False)


__all__ = [
    "SnapshotTest",
    "ScreenshotSnapshot",
    "SnapshotMismatchError",
    "pytest_snapshot_fixture",
    "pytest_screenshot_fixture",
    "normalize_html",
    "normalize_json",
]
