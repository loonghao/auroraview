"""File protocol utilities for AuroraView.

This module provides utilities for working with file:// protocol URLs
and preparing HTML content with local asset paths.
"""

from __future__ import annotations

import os
import re
from pathlib import Path
from typing import Dict, Optional, Union

# Protocol patterns
_FILE_URL_PATTERN = re.compile(r"^file:/{2,3}")  # file:// or file:///
_AURORAVIEW_BASE_URL = "https://auroraview.localhost"


def _normalize_path(path: Union[str, Path]) -> str:
    """Normalize a file path to forward slashes and absolute form.

    Args:
        path: Local file path (can be relative or absolute)

    Returns:
        Normalized absolute path with forward slashes
    """
    abs_path = Path(path).resolve()
    return str(abs_path).replace(os.sep, "/")


def _extract_path_from_file_url(file_url: str) -> Optional[str]:
    """Extract the file path from a file:// URL.

    Args:
        file_url: A file:// protocol URL

    Returns:
        Extracted path, or None if not a valid file:// URL
    """
    match = _FILE_URL_PATTERN.match(file_url)
    if not match:
        return None
    return file_url[match.end() :]


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
    path_str = _normalize_path(path)

    # Ensure proper file:/// prefix (3 slashes for absolute paths)
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


def path_to_auroraview_url(path: Union[str, Path]) -> str:
    """Convert local file path to auroraview protocol URL.

    This function converts a local file path to an AuroraView-compatible URL
    that can be loaded in the WebView without triggering file:// security
    restrictions.

    The returned URL uses the format:
    - https://auroraview.localhost/file/C:/path/to/file.ext (Windows)
    - https://auroraview.localhost/file/path/to/file.ext (Unix)

    Args:
        path: Local file path (can be relative or absolute)

    Returns:
        AuroraView protocol URL string

    Examples:
        >>> path_to_auroraview_url("C:/icons/maya.svg")
        'https://auroraview.localhost/file/C:/icons/maya.svg'
        >>> path_to_auroraview_url("/home/user/icons/maya.svg")
        'https://auroraview.localhost/file/home/user/icons/maya.svg'
    """
    path_str = _normalize_path(path).lstrip("/")
    return f"{_AURORAVIEW_BASE_URL}/file/{path_str}"


def get_auroraview_entry_url(entry_path: str = "index.html") -> str:
    """Get the AuroraView protocol URL for an entry page.

    This function generates a URL for loading entry pages (like index.html)
    relative to the asset_root configured in the WebView.

    Args:
        entry_path: Path relative to asset_root (default: "index.html")

    Returns:
        AuroraView protocol URL string

    Examples:
        >>> get_auroraview_entry_url()
        'https://auroraview.localhost/index.html'
        >>> get_auroraview_entry_url("settings.html")
        'https://auroraview.localhost/settings.html'
    """
    # Remove leading slash if present
    entry_path = entry_path.lstrip("/")
    return f"{_AURORAVIEW_BASE_URL}/{entry_path}"


def file_url_to_auroraview_url(file_url: str) -> str:
    """Convert file:// URL to auroraview protocol URL.

    This is a convenience function that converts an existing file:// URL
    to the AuroraView protocol format. Useful for transforming URLs in
    existing HTML content.

    Args:
        file_url: A file:// protocol URL

    Returns:
        AuroraView protocol URL string

    Examples:
        >>> file_url_to_auroraview_url("file:///C:/icons/maya.svg")
        'https://auroraview.localhost/file/C:/icons/maya.svg'
        >>> file_url_to_auroraview_url("file:///home/user/icons/maya.svg")
        'https://auroraview.localhost/file/home/user/icons/maya.svg'
    """
    extracted_path = _extract_path_from_file_url(file_url)
    if extracted_path is None:
        # Not a file:// URL, return as-is
        return file_url

    path_str = extracted_path.lstrip("/")
    return f"{_AURORAVIEW_BASE_URL}/file/{path_str}"


__all__ = [
    "path_to_file_url",
    "path_to_auroraview_url",
    "file_url_to_auroraview_url",
    "get_auroraview_entry_url",
    "prepare_html_with_local_assets",
]
