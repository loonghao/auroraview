"""File protocol utilities for AuroraView.

This module provides utilities for working with file:// protocol URLs
and preparing HTML content with local asset paths.
"""

from __future__ import annotations

import os
from pathlib import Path
from typing import Dict, Optional, Union


def path_to_file_url(path: Union[str, Path]) -> str:
    """Convert local file path to file:/// URL.

    Args:
        path: Local file path (can be relative or absolute)

    Returns:
        file:/// URL string

    Examples:
        >>> path_to_file_url("/tmp/test.txt")
        'file:///tmp/test.txt'
        >>> path_to_file_url("C:\\Users\\test.txt")  # On Windows
        'file:///C:/Users/test.txt'
    """
    # Convert to absolute path
    abs_path = Path(path).resolve()

    # Convert to file:/// URL format
    # On Windows: file:///C:/path/to/file
    # On Unix: file:///path/to/file
    path_str = str(abs_path).replace(os.sep, "/")

    # Ensure proper file:/// prefix
    if not path_str.startswith("/"):
        path_str = "/" + path_str

    return f"file://{path_str}"


def prepare_html_with_local_assets(
    html: str,
    asset_paths: Optional[Dict[str, Union[str, Path]]] = None,
    manifest_path: Optional[Union[str, Path]] = None,
) -> str:
    """Prepare HTML content by replacing placeholders with file:// URLs.

    This function replaces template placeholders (e.g., {{IMAGE_PATH}}) with
    file:/// URLs pointing to local files. It also handles the special
    {{MANIFEST_PATH}} placeholder if manifest_path is provided.

    Args:
        html: HTML content with placeholders
        asset_paths: Dictionary mapping placeholder names to file paths
        manifest_path: Optional path to manifest file (replaces {{MANIFEST_PATH}})

    Returns:
        HTML content with placeholders replaced by file:/// URLs

    Examples:
        >>> html = '<img src="{{IMAGE_PATH}}">'
        >>> result = prepare_html_with_local_assets(html, {"IMAGE_PATH": "test.png"})
        >>> "file://" in result
        True

        >>> html = '<iframe src="{{MANIFEST_PATH}}"></iframe>'
        >>> result = prepare_html_with_local_assets(html, manifest_path="index.html")
        >>> "file://" in result
        True
    """
    result = html

    # Replace asset paths
    if asset_paths:
        for placeholder, path in asset_paths.items():
            file_url = path_to_file_url(path)
            result = result.replace(f"{{{{{placeholder}}}}}", file_url)

    # Replace manifest path
    if manifest_path:
        file_url = path_to_file_url(manifest_path)
        result = result.replace("{{MANIFEST_PATH}}", file_url)

    return result


__all__ = [
    "path_to_file_url",
    "prepare_html_with_local_assets",
]
