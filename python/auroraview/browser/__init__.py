# -*- coding: utf-8 -*-
"""AuroraView Browser Module.

This module provides high-level APIs for building multi-tab browsers
and tabbed interfaces.

Example:
    >>> from auroraview.browser import Browser
    >>> browser = Browser(title="My Browser")
    >>> browser.new_tab("https://google.com")
    >>> browser.run()
"""

from __future__ import annotations

from .tab_container import TabContainer, TabState
from .browser import Browser

__all__ = [
    "Browser",
    "TabContainer",
    "TabState",
]
