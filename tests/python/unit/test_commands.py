"""Unit tests for the CommandRegistry class."""

from __future__ import annotations

import pytest

from auroraview.core.commands import CommandError, CommandErrorCode, CommandRegistry


class TestCommandErrorCode:
    """Test CommandErrorCode enum."""

    def test_error_codes_exist(self):
        """Test all error codes are defined."""
        assert CommandErrorCode.UNKNOWN.value == "UNKNOWN"
        assert CommandErrorCode.INTERNAL.value == "INTERNAL"
        assert CommandErrorCode.INVALID_DATA.value == "INVALID_DATA"
        assert CommandErrorCode.MISSING_COMMAND.value == "MISSING_COMMAND"
        assert CommandErrorCode.COMMAND_NOT_FOUND.value == "COMMAND_NOT_FOUND"
        assert CommandErrorCode.INVALID_ARGUMENTS.value == "INVALID_ARGUMENTS"
        assert CommandErrorCode.MISSING_ARGUMENT.value == "MISSING_ARGUMENT"
        assert CommandErrorCode.TYPE_ERROR.value == "TYPE_ERROR"
        assert CommandErrorCode.EXECUTION_ERROR.value == "EXECUTION_ERROR"
        assert CommandErrorCode.TIMEOUT.value == "TIMEOUT"
        assert CommandErrorCode.CANCELLED.value == "CANCELLED"
        assert CommandErrorCode.PERMISSION_DENIED.value == "PERMISSION_DENIED"


class TestCommandError:
    """Test CommandError exception class."""

    def test_basic_error(self):
        """Test basic CommandError creation."""
        error = CommandError(CommandErrorCode.UNKNOWN, "Test error")
        assert error.code == CommandErrorCode.UNKNOWN
        assert error.message == "Test error"
        assert error.details == {}

    def test_error_with_details(self):
        """Test CommandError with details."""
        error = CommandError(
            CommandErrorCode.COMMAND_NOT_FOUND,
            "Command not found",
            {"command": "test_cmd", "available": ["cmd1", "cmd2"]},
        )
        assert error.code == CommandErrorCode.COMMAND_NOT_FOUND
        assert error.details["command"] == "test_cmd"
        assert error.details["available"] == ["cmd1", "cmd2"]

    def test_to_dict(self):
        """Test CommandError.to_dict()."""
        error = CommandError(CommandErrorCode.INVALID_ARGUMENTS, "Bad args")
        result = error.to_dict()

        assert result["code"] == "INVALID_ARGUMENTS"
        assert result["message"] == "Bad args"
        assert "details" not in result  # Empty details excluded

    def test_to_dict_with_details(self):
        """Test CommandError.to_dict() with details."""
        error = CommandError(
            CommandErrorCode.TYPE_ERROR, "Type mismatch", {"expected": "int", "got": "str"}
        )
        result = error.to_dict()

        assert result["code"] == "TYPE_ERROR"
        assert result["details"]["expected"] == "int"

    def test_repr(self):
        """Test CommandError repr."""
        error = CommandError(CommandErrorCode.EXECUTION_ERROR, "Failed")
        assert "EXECUTION_ERROR" in repr(error)
        assert "Failed" in repr(error)

    def test_error_is_exception(self):
        """Test CommandError can be raised and caught."""
        with pytest.raises(CommandError) as exc_info:
            raise CommandError(CommandErrorCode.TIMEOUT, "Operation timed out")

        assert exc_info.value.code == CommandErrorCode.TIMEOUT


class TestCommandRegistryBasic:
    """Test basic CommandRegistry functionality."""

    def test_init_empty(self):
        """Test CommandRegistry initialization."""
        registry = CommandRegistry()
        assert len(registry) == 0
        assert registry.list_commands() == []

    def test_register_decorator(self):
        """Test registering command with decorator."""
        registry = CommandRegistry()

        @registry.register
        def greet(name: str) -> str:
            return f"Hello, {name}!"

        assert "greet" in registry
        assert registry.has_command("greet")

    def test_register_with_custom_name(self):
        """Test registering command with custom name."""
        registry = CommandRegistry()

        @registry.register("custom_greet")
        def greet(name: str) -> str:
            return f"Hello, {name}!"

        assert "custom_greet" in registry
        assert "greet" not in registry

    def test_register_with_parens(self):
        """Test registering command with empty parens."""
        registry = CommandRegistry()

        @registry.register()
        def my_command():
            return "result"

        assert "my_command" in registry

    def test_invoke_command(self):
        """Test invoking a registered command."""
        registry = CommandRegistry()

        @registry.register
        def add(x: int, y: int) -> int:
            return x + y

        result = registry.invoke("add", x=1, y=2)
        assert result == 3

    def test_invoke_unknown_command(self):
        """Test invoking unknown command raises error."""
        registry = CommandRegistry()

        with pytest.raises(KeyError, match="Unknown command"):
            registry.invoke("nonexistent")

    def test_unregister(self):
        """Test unregistering a command."""
        registry = CommandRegistry()

        @registry.register
        def temp_command():
            pass

        assert "temp_command" in registry
        assert registry.unregister("temp_command") is True
        assert "temp_command" not in registry
        assert registry.unregister("temp_command") is False

    def test_list_commands(self):
        """Test listing all commands."""
        registry = CommandRegistry()

        @registry.register
        def cmd1():
            pass

        @registry.register
        def cmd2():
            pass

        commands = registry.list_commands()
        assert set(commands) == {"cmd1", "cmd2"}

    def test_len(self):
        """Test length of registry."""
        registry = CommandRegistry()
        assert len(registry) == 0

        @registry.register
        def cmd():
            pass

        assert len(registry) == 1

    def test_contains(self):
        """Test 'in' operator."""
        registry = CommandRegistry()

        @registry.register
        def exists():
            pass

        assert "exists" in registry
        assert "missing" not in registry

    def test_repr(self):
        """Test string representation."""
        registry = CommandRegistry()

        @registry.register
        def my_cmd():
            pass

        assert "CommandRegistry" in repr(registry)
        assert "my_cmd" in repr(registry)


class TestCommandInvocation:
    """Test command invocation via _handle_invoke."""

    def test_handle_invoke_success(self):
        """Test successful command invocation."""
        registry = CommandRegistry()

        @registry.register
        def multiply(a: int, b: int) -> int:
            return a * b

        result = registry._handle_invoke(
            {"id": "test_1", "command": "multiply", "args": {"a": 3, "b": 4}}
        )

        assert result == {"id": "test_1", "result": 12}

    def test_handle_invoke_missing_command(self):
        """Test invocation with missing command name."""
        registry = CommandRegistry()

        result = registry._handle_invoke({"id": "test_2", "args": {}})
        assert "error" in result
        assert result["error"]["code"] == "MISSING_COMMAND"
        assert "Missing command" in result["error"]["message"]

    def test_handle_invoke_unknown_command(self):
        """Test invocation of unknown command."""
        registry = CommandRegistry()

        result = registry._handle_invoke({"id": "test_3", "command": "unknown", "args": {}})

        assert "error" in result
        assert result["error"]["code"] == "COMMAND_NOT_FOUND"
        assert "unknown" in result["error"]["message"]

    def test_handle_invoke_invalid_args(self):
        """Test invocation with invalid arguments."""
        registry = CommandRegistry()

        @registry.register
        def needs_args(x: int) -> int:
            return x

        result = registry._handle_invoke(
            {
                "id": "test_4",
                "command": "needs_args",
                "args": {},  # Missing required 'x'
            }
        )

        assert "error" in result
        assert result["error"]["code"] == "INVALID_ARGUMENTS"

    def test_handle_invoke_invalid_data_type(self):
        """Test invocation with non-dict data."""
        registry = CommandRegistry()

        result = registry._handle_invoke("not a dict")

        assert "error" in result
        assert result["error"]["code"] == "INVALID_DATA"

    def test_handle_invoke_execution_error(self):
        """Test invocation when command raises exception."""
        registry = CommandRegistry()

        @registry.register
        def failing_cmd():
            raise ValueError("Something went wrong")

        result = registry._handle_invoke({"id": "test_5", "command": "failing_cmd", "args": {}})

        assert "error" in result
        assert result["error"]["code"] == "EXECUTION_ERROR"
        assert "Something went wrong" in result["error"]["message"]

    def test_handle_invoke_command_error(self):
        """Test invocation when command raises CommandError."""
        registry = CommandRegistry()

        @registry.register
        def cmd_with_error():
            raise CommandError(CommandErrorCode.PERMISSION_DENIED, "Access denied")

        result = registry._handle_invoke({"id": "test_6", "command": "cmd_with_error", "args": {}})

        assert "error" in result
        assert result["error"]["code"] == "PERMISSION_DENIED"
        assert "Access denied" in result["error"]["message"]

    def test_handle_invoke_no_id(self):
        """Test invocation without id field."""
        registry = CommandRegistry()

        @registry.register
        def simple():
            return "ok"

        result = registry._handle_invoke({"command": "simple", "args": {}})

        assert result["id"] == ""
        assert result["result"] == "ok"

    def test_handle_invoke_default_args(self):
        """Test invocation uses empty dict for missing args."""
        registry = CommandRegistry()

        @registry.register
        def no_args():
            return "success"

        result = registry._handle_invoke({"id": "test_7", "command": "no_args"})

        assert result["result"] == "success"


class TestCommandRegistryAsync:
    """Test async command handling."""

    def test_register_async_command(self):
        """Test registering async command."""
        registry = CommandRegistry()

        @registry.register
        async def async_cmd(x: int) -> int:
            return x * 2

        assert "async_cmd" in registry

    def test_invoke_async_command_no_loop(self):
        """Test invoking async command without running loop."""
        registry = CommandRegistry()

        @registry.register
        async def async_add(a: int, b: int) -> int:
            return a + b

        result = registry._handle_invoke(
            {"id": "async_1", "command": "async_add", "args": {"a": 1, "b": 2}}
        )

        assert result["result"] == 3
