"""
Maya Integration Package for AuroraView Outliner

This package provides Maya-specific integration code.
Renamed from 'maya' to 'maya_integration' to avoid namespace conflicts
with Maya's core 'maya' package.
"""

from .maya_outliner import MayaOutliner, main

__all__ = ["MayaOutliner", "main"]
