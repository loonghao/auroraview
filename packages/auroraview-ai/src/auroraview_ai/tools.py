# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""DCC Tool definitions and decorators for AuroraView AI."""

from __future__ import annotations

import inspect
from collections.abc import Callable
from dataclasses import dataclass
from enum import Enum
from typing import Any


class DCCToolCategory(Enum):
    """Categories for DCC tools."""

    SCENE = "scene"
    OBJECT = "object"
    MATERIAL = "material"
    RENDER = "render"
    ANIMATION = "animation"
    ASSET = "asset"
    UI = "ui"
    UTILITY = "utility"
    CUSTOM = "custom"


@dataclass
class DCCTool:
    """Definition of a DCC tool for AI agent.

    Attributes:
        name: Tool identifier
        description: Human-readable description
        category: DCC tool category
        handler: Python callable
        confirm: Require user confirmation before execution
        dangerous: Mark as potentially destructive operation
    """

    name: str
    description: str
    category: DCCToolCategory = DCCToolCategory.UTILITY
    handler: Callable[..., Any] | None = None
    confirm: bool = False
    dangerous: bool = False

    def get_schema(self) -> dict[str, Any]:
        """Get JSON Schema for this tool's parameters."""
        if not self.handler:
            return {"type": "object", "properties": {}}

        return _infer_schema_from_function(self.handler)


def dcc_tool(
    *,
    name: str | None = None,
    description: str | None = None,
    category: DCCToolCategory = DCCToolCategory.UTILITY,
    confirm: bool = False,
    dangerous: bool = False,
) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
    """Decorator to mark a function as a DCC tool.

    Example:
        @dcc_tool(category=DCCToolCategory.SCENE, confirm=True)
        def export_scene(format: str = "fbx") -> dict:
            '''Export the current scene to a file.'''
            return {"status": "ok"}
    """

    def decorator(func: Callable[..., Any]) -> Callable[..., Any]:
        func._dcc_tool = True  # type: ignore[attr-defined]
        func._dcc_tool_info = DCCTool(  # type: ignore[attr-defined]
            name=name or func.__name__,
            description=description or func.__doc__ or "",
            category=category,
            handler=func,
            confirm=confirm,
            dangerous=dangerous,
        )
        return func

    return decorator


def _infer_schema_from_function(func: Callable[..., Any]) -> dict[str, Any]:
    """Infer JSON Schema from function signature."""
    schema: dict[str, Any] = {
        "type": "object",
        "properties": {},
        "required": [],
    }

    try:
        from typing import get_type_hints
        hints = get_type_hints(func)
    except Exception:
        hints = {}

    sig = inspect.signature(func)

    for param_name, param in sig.parameters.items():
        if param_name in ("self", "cls", "ctx"):
            continue
        if param.kind in (
            inspect.Parameter.VAR_POSITIONAL,
            inspect.Parameter.VAR_KEYWORD,
        ):
            continue

        type_hint = hints.get(param_name, Any)
        prop_schema = _python_type_to_json_schema(type_hint)
        schema["properties"][param_name] = prop_schema

        if param.default is inspect.Parameter.empty:
            schema["required"].append(param_name)

    return schema


def _python_type_to_json_schema(type_hint: Any) -> dict[str, Any]:
    """Convert Python type hint to JSON Schema."""
    type_mapping: dict[type, dict[str, str]] = {
        str: {"type": "string"},
        int: {"type": "integer"},
        float: {"type": "number"},
        bool: {"type": "boolean"},
        list: {"type": "array"},
        dict: {"type": "object"},
    }

    if type_hint in type_mapping:
        return type_mapping[type_hint]

    origin = getattr(type_hint, "__origin__", None)
    if origin is list:
        args = getattr(type_hint, "__args__", ())
        if args:
            return {"type": "array", "items": _python_type_to_json_schema(args[0])}
        return {"type": "array"}

    if origin is dict:
        return {"type": "object"}

    return {}

