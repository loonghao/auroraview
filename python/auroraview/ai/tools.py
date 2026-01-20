# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AI Agent tool system for function calling."""

from __future__ import annotations

import inspect
import logging
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Callable, Dict, List, Optional, get_type_hints

if TYPE_CHECKING:
    from ..core.webview import WebView

logger = logging.getLogger(__name__)


@dataclass
class Tool:
    """Definition of a tool that AI can call.

    Attributes:
        name: Tool identifier (e.g., "api.export_scene")
        description: Human-readable description for AI
        parameters: JSON Schema for parameters
        handler: Python callable to execute
        returns_schema: Optional JSON Schema for return value
    """

    name: str
    description: str
    parameters: Dict[str, Any]
    handler: Optional[Callable[..., Any]] = None
    returns_schema: Optional[Dict[str, Any]] = None

    def to_openai_format(self) -> Dict[str, Any]:
        """Convert to OpenAI function calling format."""
        return {
            "type": "function",
            "function": {
                "name": self.name.replace(".", "_"),  # OpenAI doesn't allow dots
                "description": self.description,
                "parameters": self.parameters,
            },
        }

    def to_anthropic_format(self) -> Dict[str, Any]:
        """Convert to Anthropic tool format."""
        return {
            "name": self.name.replace(".", "_"),
            "description": self.description,
            "input_schema": self.parameters,
        }


class ToolRegistry:
    """Registry for AI-callable tools.

    Manages the collection of tools available to the AI agent,
    including auto-discovery from WebView bound APIs.
    """

    def __init__(self):
        self._tools: Dict[str, Tool] = {}

    def register(self, tool: Tool) -> None:
        """Register a tool."""
        self._tools[tool.name] = tool
        logger.debug("Registered tool: %s", tool.name)

    def unregister(self, name: str) -> bool:
        """Unregister a tool by name."""
        if name in self._tools:
            del self._tools[name]
            return True
        return False

    def get(self, name: str) -> Optional[Tool]:
        """Get a tool by name."""
        return self._tools.get(name)

    def all(self) -> List[Tool]:
        """Get all registered tools."""
        return list(self._tools.values())

    def names(self) -> List[str]:
        """Get all tool names."""
        return list(self._tools.keys())

    def to_openai_tools(self) -> List[Dict[str, Any]]:
        """Convert all tools to OpenAI format."""
        return [t.to_openai_format() for t in self._tools.values()]

    def to_anthropic_tools(self) -> List[Dict[str, Any]]:
        """Convert all tools to Anthropic format."""
        return [t.to_anthropic_format() for t in self._tools.values()]

    async def execute(self, name: str, arguments: Dict[str, Any]) -> Any:
        """Execute a tool by name with arguments."""
        tool = self._tools.get(name)
        if not tool:
            # Try with underscores (OpenAI format)
            tool = self._tools.get(name.replace("_", "."))

        if not tool:
            raise ValueError(f"Tool not found: {name}")

        if not tool.handler:
            raise ValueError(f"Tool has no handler: {name}")

        # Execute handler
        if inspect.iscoroutinefunction(tool.handler):
            return await tool.handler(**arguments)
        else:
            return tool.handler(**arguments)

    def discover_from_webview(self, webview: "WebView") -> int:
        """Discover and register tools from WebView bound APIs.

        This method inspects all methods bound via webview.bind_call() and
        automatically creates Tool definitions with inferred JSON schemas.

        Args:
            webview: WebView instance to discover APIs from

        Returns:
            Number of tools discovered and registered
        """
        count = 0
        bound_methods = webview.get_bound_methods()

        for method_name in bound_methods:
            # Get the actual function from the registry
            func = webview._bound_functions.get(method_name)
            if not func:
                continue

            # Create tool from function
            tool = self._create_tool_from_function(method_name, func)
            if tool:
                self.register(tool)
                count += 1

        logger.info("Discovered %d tools from WebView", count)
        return count

    def _create_tool_from_function(self, name: str, func: Callable[..., Any]) -> Optional[Tool]:
        """Create a Tool from a Python function.

        Extracts:
        - Description from docstring
        - Parameter schema from type hints
        """
        # Get description from docstring
        description = func.__doc__ or f"Call {name}"
        # Clean up docstring
        description = description.strip().split("\n")[0]  # First line only

        # Infer parameters schema from type hints
        parameters = self._infer_parameters_schema(func)

        return Tool(
            name=name,
            description=description,
            parameters=parameters,
            handler=func,
        )

    def _infer_parameters_schema(self, func: Callable[..., Any]) -> Dict[str, Any]:
        """Infer JSON Schema from function signature and type hints.

        Args:
            func: Python callable to analyze

        Returns:
            JSON Schema dict for function parameters
        """
        schema: Dict[str, Any] = {
            "type": "object",
            "properties": {},
            "required": [],
        }

        try:
            hints = get_type_hints(func)
        except Exception:
            hints = {}

        sig = inspect.signature(func)

        for param_name, param in sig.parameters.items():
            # Skip self, cls, *args, **kwargs
            if param_name in ("self", "cls"):
                continue
            if param.kind in (
                inspect.Parameter.VAR_POSITIONAL,
                inspect.Parameter.VAR_KEYWORD,
            ):
                continue

            # Get type hint
            type_hint = hints.get(param_name, Any)
            prop_schema = self._python_type_to_json_schema(type_hint)

            # Add description from param annotation if available
            schema["properties"][param_name] = prop_schema

            # Check if required (no default value)
            if param.default is inspect.Parameter.empty:
                schema["required"].append(param_name)

        return schema

    def _python_type_to_json_schema(self, type_hint: Any) -> Dict[str, Any]:
        """Convert Python type hint to JSON Schema."""
        # Handle None type
        if type_hint is type(None):
            return {"type": "null"}

        # Handle basic types
        type_mapping = {
            str: {"type": "string"},
            int: {"type": "integer"},
            float: {"type": "number"},
            bool: {"type": "boolean"},
            list: {"type": "array"},
            dict: {"type": "object"},
            Any: {},  # Any type - no constraint
        }

        if type_hint in type_mapping:
            return type_mapping[type_hint]

        # Handle typing module types
        origin = getattr(type_hint, "__origin__", None)

        if origin is list:
            args = getattr(type_hint, "__args__", ())
            if args:
                return {
                    "type": "array",
                    "items": self._python_type_to_json_schema(args[0]),
                }
            return {"type": "array"}

        if origin is dict:
            return {"type": "object"}

        # Handle Optional (Union with None)
        if origin is type(None):
            return {"type": "null"}

        # Handle Union types
        try:
            from typing import Union, get_args, get_origin

            if get_origin(type_hint) is Union:
                args = get_args(type_hint)
                # Check for Optional (Union[X, None])
                non_none_args = [a for a in args if a is not type(None)]
                if len(non_none_args) == 1:
                    return self._python_type_to_json_schema(non_none_args[0])
                # Multiple types - use anyOf
                return {"anyOf": [self._python_type_to_json_schema(a) for a in args]}
        except ImportError:
            pass

        # Default: no constraint
        return {}


def tool(
    name: Optional[str] = None,
    description: Optional[str] = None,
) -> Callable[[Callable], Callable]:
    """Decorator to mark a function as an AI tool.

    Usage:
        @tool(description="Export the scene to a file")
        def export_scene(format: str = "fbx") -> dict:
            return {"status": "ok"}
    """

    def decorator(func: Callable) -> Callable:
        # Store metadata on the function
        func._ai_tool = True  # type: ignore
        func._ai_tool_name = name or func.__name__  # type: ignore
        func._ai_tool_description = description or func.__doc__ or ""  # type: ignore
        return func

    return decorator
