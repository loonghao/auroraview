#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Test script to debug gallery white screen issue in development mode."""

from __future__ import annotations

import sys
from pathlib import Path

from auroraview import WebView
from auroraview.utils.file_protocol import path_to_file_url

PROJECT_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(PROJECT_ROOT))
sys.path.insert(0, str(PROJECT_ROOT / "python"))

DIST_DIR = Path(__file__).parent / "dist"


def main():
    """Test minimal gallery setup."""
    print("[Test] Starting test...", file=sys.stderr)

    index_html = DIST_DIR / "index.html"
    if not index_html.exists():
        print(f"[Test] Error: {index_html} not found", file=sys.stderr)
        sys.exit(1)

    url = path_to_file_url(index_html)
    print(f"[Test] Loading URL: {url}", file=sys.stderr)

    view = WebView(
        title="Gallery Test",
        url=url,
        width=1200,
        height=800,
        debug=True,
    )

    @view.bind_call("api.get_samples")
    def get_samples() -> list:
        """Get all samples."""
        print("[Test] api.get_samples called", file=sys.stderr)
        return [
            {
                "id": "test_sample",
                "title": "Test Sample",
                "category": "getting_started",
                "description": "A test sample",
                "icon": "code",
                "source_file": "test.py",
                "tags": ["test"],
            }
        ]

    @view.bind_call("api.get_categories")
    def get_categories() -> dict:
        """Get all categories."""
        print("[Test] api.get_categories called", file=sys.stderr)
        return {
            "getting_started": {
                "title": "Getting Started",
                "icon": "rocket",
                "description": "Quick start examples",
            }
        }

    print("[Test] Starting WebView...", file=sys.stderr)
    view.show()
    print("[Test] Done", file=sys.stderr)


if __name__ == "__main__":
    main()
