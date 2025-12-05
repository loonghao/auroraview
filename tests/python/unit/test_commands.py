"""Unit tests for the CommandRegistry class."""

from __future__ import annotations

import pytest

from auroraview.core.commands import CommandRegistry


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
        
        result = registry._handle_invoke({
            "id": "test_1",
            "command": "multiply",
            "args": {"a": 3, "b": 4}
        })

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

        result = registry._handle_invoke({
            "id": "test_3",
            "command": "unknown",
            "args": {}
        })

        assert "error" in result
        assert result["error"]["code"] == "COMMAND_NOT_FOUND"
        assert "unknown" in result["error"]["message"]

    def test_handle_invoke_invalid_args(self):
        """Test invocation with invalid arguments."""
        registry = CommandRegistry()

        @registry.register
        def needs_args(x: int) -> int:
            return x

        result = registry._handle_invoke({
            "id": "test_4",
            "command": "needs_args",
            "args": {}  # Missing required 'x'
        })

        assert "error" in result
        assert result["error"]["code"] == "INVALID_ARGUMENTS"

