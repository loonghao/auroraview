"""Tests for auroraview_ai.tools module."""

from __future__ import annotations

from typing import Any

import pytest

from auroraview_ai.tools import (
    DCCTool,
    DCCToolCategory,
    _infer_schema_from_function,
    _python_type_to_json_schema,
    dcc_tool,
)


class TestDCCToolCategory:
    """Tests for DCCToolCategory enum."""

    def test_all_categories_exist(self) -> None:
        expected = {"scene", "object", "material", "render", "animation", "asset", "ui", "utility", "custom"}
        actual = {c.value for c in DCCToolCategory}
        assert expected.issubset(actual)


class TestDCCTool:
    """Tests for DCCTool dataclass."""

    def test_minimal_construction(self) -> None:
        tool = DCCTool(name="test_tool", description="A test tool")
        assert tool.name == "test_tool"
        assert tool.description == "A test tool"
        assert tool.category == DCCToolCategory.UTILITY
        assert tool.handler is None
        assert tool.confirm is False
        assert tool.dangerous is False

    def test_get_schema_no_handler(self) -> None:
        tool = DCCTool(name="no_handler", description="No handler")
        schema = tool.get_schema()
        assert schema == {"type": "object", "properties": {}}

    def test_get_schema_with_handler(self) -> None:
        def my_func(format: str, quality: int) -> dict:
            return {}

        tool = DCCTool(name="export", description="Export", handler=my_func)
        schema = tool.get_schema()
        assert "format" in schema["properties"]
        assert "quality" in schema["properties"]
        assert "format" in schema["required"]
        assert "quality" in schema["required"]

    def test_get_schema_with_default_params(self) -> None:
        def my_func(format: str = "fbx", quality: int = 100) -> dict:
            return {}

        tool = DCCTool(name="export", description="Export", handler=my_func)
        schema = tool.get_schema()
        assert schema["required"] == []

    def test_dangerous_flag(self) -> None:
        tool = DCCTool(name="delete_all", description="Delete everything", dangerous=True)
        assert tool.dangerous is True

    def test_confirm_flag(self) -> None:
        tool = DCCTool(name="export", description="Export", confirm=True)
        assert tool.confirm is True


class TestDCCToolDecorator:
    """Tests for dcc_tool decorator."""

    def test_basic_decorator(self) -> None:
        @dcc_tool()
        def export_scene() -> dict:
            """Export the current scene."""
            return {}

        assert hasattr(export_scene, "_dcc_tool")
        assert export_scene._dcc_tool is True
        assert hasattr(export_scene, "_dcc_tool_info")

    def test_decorator_uses_function_name(self) -> None:
        @dcc_tool()
        def my_tool() -> None:
            pass

        info = my_tool._dcc_tool_info
        assert info.name == "my_tool"

    def test_decorator_uses_custom_name(self) -> None:
        @dcc_tool(name="custom_name")
        def my_func() -> None:
            pass

        info = my_func._dcc_tool_info
        assert info.name == "custom_name"

    def test_decorator_uses_function_docstring(self) -> None:
        @dcc_tool()
        def export_scene() -> dict:
            """Export the current scene to a file."""
            return {}

        info = export_scene._dcc_tool_info
        assert info.description == "Export the current scene to a file."

    def test_decorator_uses_custom_description(self) -> None:
        @dcc_tool(description="Custom description")
        def my_func() -> None:
            pass

        info = my_func._dcc_tool_info
        assert info.description == "Custom description"

    def test_decorator_empty_description(self) -> None:
        @dcc_tool()
        def my_func() -> None:
            pass

        info = my_func._dcc_tool_info
        assert info.description == ""

    def test_decorator_category(self) -> None:
        @dcc_tool(category=DCCToolCategory.SCENE)
        def export_scene() -> dict:
            """Export."""
            return {}

        info = export_scene._dcc_tool_info
        assert info.category == DCCToolCategory.SCENE

    def test_decorator_confirm(self) -> None:
        @dcc_tool(confirm=True)
        def delete_all() -> None:
            pass

        info = delete_all._dcc_tool_info
        assert info.confirm is True

    def test_decorator_dangerous(self) -> None:
        @dcc_tool(dangerous=True)
        def reset_scene() -> None:
            pass

        info = reset_scene._dcc_tool_info
        assert info.dangerous is True

    def test_decorated_function_still_callable(self) -> None:
        @dcc_tool()
        def add(a: int, b: int) -> int:
            return a + b

        assert add(2, 3) == 5

    def test_handler_stored_in_info(self) -> None:
        @dcc_tool()
        def my_func() -> None:
            pass

        info = my_func._dcc_tool_info
        assert info.handler is my_func


class TestInferSchemaFromFunction:
    """Tests for _infer_schema_from_function."""

    def test_no_params(self) -> None:
        def func() -> None:
            pass

        schema = _infer_schema_from_function(func)
        assert schema["properties"] == {}
        assert schema["required"] == []

    def test_self_cls_ctx_skipped(self) -> None:
        def func(self: Any, cls: Any, ctx: Any, value: str) -> None:
            pass

        schema = _infer_schema_from_function(func)
        assert "self" not in schema["properties"]
        assert "cls" not in schema["properties"]
        assert "ctx" not in schema["properties"]
        assert "value" in schema["properties"]

    def test_var_args_skipped(self) -> None:
        def func(*args: Any, **kwargs: Any) -> None:
            pass

        schema = _infer_schema_from_function(func)
        assert schema["properties"] == {}

    def test_required_params(self) -> None:
        def func(name: str, count: int) -> None:
            pass

        schema = _infer_schema_from_function(func)
        assert "name" in schema["required"]
        assert "count" in schema["required"]

    def test_optional_params_not_required(self) -> None:
        def func(name: str = "default") -> None:
            pass

        schema = _infer_schema_from_function(func)
        assert "name" not in schema["required"]

    def test_mixed_params(self) -> None:
        def func(required: str, optional: int = 0) -> None:
            pass

        schema = _infer_schema_from_function(func)
        assert "required" in schema["required"]
        assert "optional" not in schema["required"]


class TestPythonTypeToJsonSchema:
    """Tests for _python_type_to_json_schema."""

    def test_str(self) -> None:
        assert _python_type_to_json_schema(str) == {"type": "string"}

    def test_int(self) -> None:
        assert _python_type_to_json_schema(int) == {"type": "integer"}

    def test_float(self) -> None:
        assert _python_type_to_json_schema(float) == {"type": "number"}

    def test_bool(self) -> None:
        assert _python_type_to_json_schema(bool) == {"type": "boolean"}

    def test_list(self) -> None:
        assert _python_type_to_json_schema(list) == {"type": "array"}

    def test_dict(self) -> None:
        assert _python_type_to_json_schema(dict) == {"type": "object"}

    def test_list_of_str(self) -> None:
        from typing import List
        result = _python_type_to_json_schema(List[str])
        assert result == {"type": "array", "items": {"type": "string"}}

    def test_list_of_int(self) -> None:
        from typing import List
        result = _python_type_to_json_schema(List[int])
        assert result == {"type": "array", "items": {"type": "integer"}}

    def test_dict_generic(self) -> None:
        from typing import Dict
        result = _python_type_to_json_schema(Dict[str, Any])
        assert result == {"type": "object"}

    def test_unknown_type_returns_empty(self) -> None:
        class MyClass:
            pass
        result = _python_type_to_json_schema(MyClass)
        assert result == {}

    def test_any_returns_empty(self) -> None:
        result = _python_type_to_json_schema(Any)
        assert result == {}

    def test_list_without_args(self) -> None:
        from typing import List
        result = _python_type_to_json_schema(List)
        assert result == {"type": "array"}


class TestInferSchemaGetTypeHintsException:
    """Tests for _infer_schema_from_function when get_type_hints raises."""

    def test_get_type_hints_exception_falls_back(self) -> None:
        """When get_type_hints raises, fall back to empty hints dict."""
        from unittest.mock import patch

        def func(value: str, count: int) -> None:
            pass

        with patch("typing.get_type_hints", side_effect=Exception("hints error")):
            schema = _infer_schema_from_function(func)
            # Should still have the params in properties, but no type info
            assert "value" in schema["properties"]
            assert "count" in schema["properties"]
            # With no type hints, types will be Any → empty schema
            assert schema["properties"]["value"] == {}
            assert schema["properties"]["count"] == {}
