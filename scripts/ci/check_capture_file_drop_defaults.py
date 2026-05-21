#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""RFC 0017 §5: static guard for the Python ``capture_file_drop`` tri-state.

Refuses to let the passthrough chain collapse ``Optional[bool]`` into
plain ``bool`` before the Rust PyO3 boundary. Detects three forbidden
patterns inside ``python/auroraview/`` (and any future passthrough layer):

    1. ``setdefault('capture_file_drop', ...)`` — silent middle-layer default.
    2. ``get|pop('capture_file_drop', True|False)`` — flatten via dict default.
    3. ``capture_file_drop or True|False`` — flatten via boolean fallback.

The ONLY permitted flatten point is in Rust (``unwrap_or(false)`` in
``src/bindings/desktop_runner.rs``); that file is in Rust code and not
scanned by this script.

Cross-platform Python implementation chosen so CI on both Windows and
Linux runs the same logic without depending on shell-specific features.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCAN_ROOT = REPO_ROOT / "python" / "auroraview"


# (description, compiled regex)
FORBIDDEN_PATTERNS = [
    (
        "setdefault on capture_file_drop",
        re.compile(r"""setdefault\s*\(\s*['"]capture_file_drop['"]"""),
    ),
    (
        "dict.get/pop with bool default for capture_file_drop",
        re.compile(
            r"""(?:get|pop)\s*\(\s*['"]capture_file_drop['"]\s*,\s*(?:True|False)\b"""
        ),
    ),
    (
        "boolean fallback (or True/False) on capture_file_drop",
        re.compile(r"""capture_file_drop\s+or\s+(?:True|False)\b"""),
    ),
]


def main() -> int:
    if not SCAN_ROOT.is_dir():
        print(f"[check] scan root does not exist: {SCAN_ROOT}", file=sys.stderr)
        return 2

    failures: list[tuple[str, Path, int, str]] = []
    py_files = list(SCAN_ROOT.rglob("*.py"))

    for path in py_files:
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as exc:
            print(f"[check] cannot read {path}: {exc}", file=sys.stderr)
            return 2

        for line_no, raw_line in enumerate(text.splitlines(), start=1):
            line = raw_line.strip()
            if not line or line.startswith("#"):
                continue
            for description, pattern in FORBIDDEN_PATTERNS:
                if pattern.search(raw_line):
                    failures.append((description, path, line_no, raw_line.rstrip()))

    if failures:
        print("ERROR: capture_file_drop tri-state contract violated (RFC 0017 §3.3).")
        print("       Tri-state Optional[bool] must reach the Rust PyO3 binding intact;")
        print("       Rust applies unwrap_or(false). Do not flatten None to False in Python.")
        print()
        for description, path, line_no, line in failures:
            rel = path.relative_to(REPO_ROOT)
            print(f"  {rel}:{line_no}: {description}")
            print(f"      {line}")
        print()
        print(f"Scanned {len(py_files)} files; {len(failures)} violation(s).")
        return 1

    print(
        f"OK: capture_file_drop passthrough rules satisfied ({len(py_files)} files scanned)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
