"""AuroraView Gallery Backend Module.

This module provides the backend functionality for the AuroraView Gallery,
organized into logical submodules for better maintainability.

Submodules:
- config: Constants, categories, and mapping configurations
- samples: Example scanning and management
- process_api: Process management API handlers
- extension_api: Browser extension bridge API handlers
- webview_extension_api: WebView2 extension management API handlers
- child_manager: Child window management for running examples as sub-windows
"""

from __future__ import annotations

from .config import (
    CATEGORIES,
    CATEGORY_MAPPING,
    EXAMPLES_DIR,
    GALLERY_DIR,
    ICON_MAPPING,
    PROJECT_ROOT,
    TAG_MAPPING,
)
from .samples import (
    extract_docstring,
    filename_to_id,
    filename_to_title,
    get_sample_by_id,
    get_samples_list,
    get_source_code,
    infer_category,
    infer_icon,
    infer_tags,
    parse_docstring,
    scan_examples_from,
)
from .child_manager import (
    ChildWindow,
    ChildWindowManager,
    get_manager as get_child_manager,
    init_manager as init_child_manager,
    cleanup_manager as cleanup_child_manager,
)

__all__ = [
    # Config
    "PROJECT_ROOT",
    "GALLERY_DIR",
    "EXAMPLES_DIR",
    "CATEGORIES",
    "ICON_MAPPING",
    "CATEGORY_MAPPING",
    "TAG_MAPPING",
    # Samples
    "extract_docstring",
    "parse_docstring",
    "infer_category",
    "infer_icon",
    "infer_tags",
    "filename_to_title",
    "filename_to_id",
    "get_samples_list",
    "scan_examples_from",
    "get_sample_by_id",
    "get_source_code",
    # Child Manager
    "ChildWindow",
    "ChildWindowManager",
    "get_child_manager",
    "init_child_manager",
    "cleanup_child_manager",
]
