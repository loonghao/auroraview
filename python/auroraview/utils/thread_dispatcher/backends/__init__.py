# -*- coding: utf-8 -*-
"""DCC thread dispatcher backends.

This package contains backends for various DCC applications.
"""

from __future__ import annotations

from .blender import BlenderDispatcherBackend
from .fallback import FallbackDispatcherBackend
from .houdini import HoudiniDispatcherBackend
from .maya import MayaDispatcherBackend
from .max import MaxDispatcherBackend
from .nuke import NukeDispatcherBackend
from .qt import QtDispatcherBackend
from .unreal import UnrealDispatcherBackend

__all__ = [
    "BlenderDispatcherBackend",
    "FallbackDispatcherBackend",
    "HoudiniDispatcherBackend",
    "MayaDispatcherBackend",
    "MaxDispatcherBackend",
    "NukeDispatcherBackend",
    "QtDispatcherBackend",
    "UnrealDispatcherBackend",
]
