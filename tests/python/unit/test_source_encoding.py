# -*- coding: utf-8 -*-
"""Tests for Python source file encoding declarations.

Python 3.7 requires explicit encoding declarations (PEP 263) when source
files contain non-ASCII characters. This test ensures all Python files
in the package either:
1. Contain only ASCII characters, or
2. Have a proper UTF-8 encoding declaration in the first two lines

This prevents SyntaxError on Python 3.7:
'Non-UTF-8 code starting with... but no encoding declared'
"""

from __future__ import annotations

import re
import tokenize
from pathlib import Path
from typing import List, Tuple

import pytest

# Root directory of the Python package
PYTHON_SRC_DIR = Path(__file__).parent.parent.parent.parent / "python"

# Pattern to match encoding declaration (PEP 263)
# Examples: # -*- coding: utf-8 -*-  or  # coding=utf-8  or  # coding: utf-8
ENCODING_PATTERN = re.compile(
    r"^[ \t\f]*#.*?coding[:=][ \t]*([-\w.]+)", re.ASCII
)


def has_non_ascii(content: str) -> bool:
    """Check if content contains non-ASCII characters."""
    return any(ord(char) > 127 for char in content)


def has_encoding_declaration(lines: List[str]) -> bool:
    """Check if the first two lines contain an encoding declaration.

    Per PEP 263, the encoding declaration must be on line 1 or 2.
    If line 1 is a shebang (#!), the encoding can be on line 2.
    """
    for line in lines[:2]:
        if ENCODING_PATTERN.match(line):
            return True
    return False


def get_python_files(directory: Path) -> List[Path]:
    """Get all Python files in directory recursively."""
    return list(directory.rglob("*.py"))


def check_file_encoding(filepath: Path) -> Tuple[bool, str]:
    """Check if a Python file has proper encoding declaration.

    Returns:
        Tuple of (is_valid, message)
    """
    try:
        content = filepath.read_text(encoding="utf-8")
    except UnicodeDecodeError as e:
        return False, f"Cannot read as UTF-8: {e}"

    lines = content.splitlines()
    if not lines:
        return True, "Empty file"

    if not has_non_ascii(content):
        return True, "ASCII only"

    if has_encoding_declaration(lines):
        return True, "Has encoding declaration"

    # Find first non-ASCII character for error message
    for line_num, line in enumerate(lines, 1):
        for col, char in enumerate(line):
            if ord(char) > 127:
                return False, (
                    f"Non-ASCII character {repr(char)} (U+{ord(char):04X}) "
                    f"at line {line_num}, column {col + 1}, "
                    f"but no encoding declaration found"
                )

    return False, "Contains non-ASCII but no encoding declaration"


class TestSourceEncoding:
    """Test suite for Python source file encoding compliance."""

    def test_python_src_dir_exists(self):
        """Verify the Python source directory exists."""
        assert PYTHON_SRC_DIR.exists(), f"Python source dir not found: {PYTHON_SRC_DIR}"
        assert PYTHON_SRC_DIR.is_dir()

    def test_all_files_have_proper_encoding(self):
        """Test that all Python files have proper encoding declarations.

        This is the main test that ensures Python 3.7 compatibility.
        """
        python_files = get_python_files(PYTHON_SRC_DIR)
        assert python_files, "No Python files found"

        errors = []
        for filepath in python_files:
            is_valid, message = check_file_encoding(filepath)
            if not is_valid:
                rel_path = filepath.relative_to(PYTHON_SRC_DIR)
                errors.append(f"{rel_path}: {message}")

        if errors:
            error_msg = (
                "The following files contain non-ASCII characters "
                "but lack UTF-8 encoding declaration:\n"
                + "\n".join(f"  - {e}" for e in errors)
                + "\n\nFix: Add '# -*- coding: utf-8 -*-' as the first line"
            )
            pytest.fail(error_msg)

    def test_encoding_declaration_format(self):
        """Test that encoding declarations follow PEP 263 format."""
        python_files = get_python_files(PYTHON_SRC_DIR)

        for filepath in python_files:
            content = filepath.read_text(encoding="utf-8")
            lines = content.splitlines()

            if not lines:
                continue

            # Check first two lines for encoding declaration
            for i, line in enumerate(lines[:2]):
                if "coding" in line.lower() and "#" in line:
                    # Verify it matches PEP 263 pattern
                    if not ENCODING_PATTERN.match(line):
                        rel_path = filepath.relative_to(PYTHON_SRC_DIR)
                        pytest.fail(
                            f"{rel_path}: Line {i + 1} contains 'coding' "
                            f"but doesn't match PEP 263 format: {line!r}"
                        )

    def test_tokenize_all_files(self):
        """Test that all Python files can be tokenized.

        This catches encoding issues that would cause SyntaxError.
        """
        python_files = get_python_files(PYTHON_SRC_DIR)
        errors = []

        for filepath in python_files:
            try:
                with tokenize.open(filepath) as f:
                    # Read all tokens to verify file is valid
                    list(tokenize.generate_tokens(f.readline))
            except (SyntaxError, tokenize.TokenizeError) as e:
                rel_path = filepath.relative_to(PYTHON_SRC_DIR)
                errors.append(f"{rel_path}: {e}")

        if errors:
            pytest.fail(
                "The following files have tokenization errors:\n"
                + "\n".join(f"  - {e}" for e in errors)
            )


@pytest.mark.parametrize(
    "line,expected",
    [
        ("# -*- coding: utf-8 -*-", True),
        ("# coding: utf-8", True),
        ("# coding=utf-8", True),
        ("# vim: set fileencoding=utf-8 :", True),
        ("#coding:utf-8", True),
        ("# -*- coding: latin-1 -*-", True),  # Any encoding is valid
        ("# no encoding here", False),
        ("print('hello')", False),
        ("", False),
    ],
)
def test_encoding_pattern_matching(line: str, expected: bool):
    """Test the encoding pattern regex against various inputs."""
    result = bool(ENCODING_PATTERN.match(line))
    assert result == expected, f"Pattern match for {line!r}: expected {expected}, got {result}"
