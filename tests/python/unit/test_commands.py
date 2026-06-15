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


class TestCliRegistration:
    """Test RFC 0018 CLI decorator metadata (cli / help / args_help)."""

    def test_cli_false_default_not_exposed(self):
        """A command without `cli` is not CLI-exposed."""
        registry = CommandRegistry()

        @registry.register
        def plain(x: int) -> int:
            return x

        assert registry.cli_meta("plain") is None
        assert registry.list_cli_commands() == []

    def test_cli_true_no_alias(self):
        """cli=True exposes the command with no aliases."""
        registry = CommandRegistry()

        @registry.register("sync", cli=True)
        def sync_cmd() -> None:
            """Sync everything."""

        meta = registry.cli_meta("sync")
        assert meta is not None
        assert meta.aliases == []
        assert meta.help == "Sync everything."

    def test_cli_str_alias(self):
        """cli='exi' exposes the command with a single alias."""
        registry = CommandRegistry()

        @registry.register("export-image", cli="exi", help="Export image")
        def export_image(path: str, dpi: int = 300) -> dict:
            return {"path": path, "dpi": dpi}

        meta = registry.cli_meta("export-image")
        assert meta.aliases == ["exi"]
        assert meta.help == "Export image"

    def test_cli_list_aliases(self):
        """cli=['val','v'] exposes multiple aliases."""
        registry = CommandRegistry()

        @registry.register("validate", cli=["val", "v"])
        def validate() -> bool:
            return True

        assert registry.cli_meta("validate").aliases == ["val", "v"]

    def test_help_falls_back_to_docstring(self):
        """Help text defaults to the docstring first non-empty line."""
        registry = CommandRegistry()

        @registry.register("doc", cli=True)
        def doc_cmd() -> None:
            """First line of help.

            More detail here.
            """

        assert registry.cli_meta("doc").help == "First line of help."

    def test_help_empty_when_no_doc_no_help(self):
        """Help is empty string when neither help= nor docstring present."""
        registry = CommandRegistry()

        @registry.register("nodoc", cli=True)
        def nodoc_cmd() -> None:
            pass

        assert registry.cli_meta("nodoc").help == ""

    def test_invalid_cli_value_raises(self):
        """A non-bool/str/list cli value raises ValueError."""
        registry = CommandRegistry()

        with pytest.raises(ValueError):

            @registry.register("bad", cli=123)
            def bad_cmd() -> None:
                pass

    def test_empty_alias_raises(self):
        """A blank alias string raises ValueError."""
        registry = CommandRegistry()

        with pytest.raises(ValueError):

            @registry.register("blank", cli="   ")
            def blank_cmd() -> None:
                pass


class TestCliParamIntrospection:
    """Test CliCommandMeta parameter introspection (§13.2)."""

    def test_params_types_defaults_required(self):
        registry = CommandRegistry()

        @registry.register(
            "exi",
            cli=True,
            args_help={"path": "output dir", "dpi": "resolution"},
        )
        def exi(path: str, dpi: int = 300) -> dict:
            return {}

        meta = registry.cli_meta("exi")
        params = meta.params(registry._commands["exi"])
        assert params == [
            {"name": "path", "type": "str", "required": True, "default": None, "help": "output dir"},
            {"name": "dpi", "type": "int", "required": False, "default": 300, "help": "resolution"},
        ]

    def test_params_unannotated_is_any(self):
        registry = CommandRegistry()

        @registry.register("anyp", cli=True)
        def anyp(x) -> None:
            pass

        params = registry.cli_meta("anyp").params(registry._commands["anyp"])
        assert params[0]["type"] == "any"

    def test_to_dict_structure(self):
        registry = CommandRegistry()

        @registry.register("export-image", cli="exi", help="Export image")
        def export_image(path: str, dpi: int = 300) -> dict:
            return {}

        meta = registry.cli_meta("export-image")
        d = meta.to_dict(registry._commands["export-image"])
        assert d["name"] == "export-image"
        assert d["aliases"] == ["exi"]
        assert d["help"] == "Export image"
        assert len(d["params"]) == 2


class TestCliAliasConflicts:
    """Test fail-fast alias conflict detection (§12.4)."""

    def test_alias_collides_with_reserved_verb(self):
        registry = CommandRegistry()

        with pytest.raises(ValueError, match="reserved CLI verb"):

            @registry.register("export", cli="run")
            def export() -> None:
                pass

    def test_alias_collides_with_command_name(self):
        registry = CommandRegistry()

        @registry.register("validate", cli=True)
        def validate() -> None:
            pass

        with pytest.raises(ValueError, match="collides with command name"):

            @registry.register("export", cli="validate")
            def export() -> None:
                pass

    def test_alias_collides_with_other_alias(self):
        registry = CommandRegistry()

        @registry.register("validate", cli="v")
        def validate() -> None:
            pass

        with pytest.raises(ValueError, match="already used by command"):

            @registry.register("verify", cli="v")
            def verify() -> None:
                pass

    def test_duplicate_alias_in_same_command(self):
        registry = CommandRegistry()

        with pytest.raises(ValueError, match="duplicate alias"):

            @registry.register("export", cli=["e", "e"])
            def export() -> None:
                pass

    def test_reregister_same_command_allowed(self):
        """Re-registering a command's own name/alias is not a conflict."""
        registry = CommandRegistry()

        @registry.register("export", cli="exp")
        def export_v1() -> None:
            pass

        # Re-register same name + same alias: should not raise.
        @registry.register("export", cli="exp")
        def export_v2() -> None:
            pass

        assert registry.cli_meta("export").aliases == ["exp"]


class TestEnableCli:
    """Test batch CLI enablement (§14.2)."""

    def test_enable_cli_names_only(self):
        registry = CommandRegistry()

        @registry.register("export")
        def export() -> None:
            pass

        @registry.register("validate")
        def validate() -> None:
            pass

        registry.enable_cli("export", "validate")
        assert set(registry.list_cli_commands()) == {"export", "validate"}
        assert registry.cli_meta("export").aliases == []

    def test_enable_cli_with_aliases(self):
        registry = CommandRegistry()

        @registry.register("export")
        def export() -> None:
            pass

        registry.enable_cli({"export": "exp"})
        assert registry.cli_meta("export").aliases == ["exp"]

    def test_enable_cli_unknown_command_raises(self):
        registry = CommandRegistry()

        with pytest.raises(KeyError):
            registry.enable_cli("nope")

    def test_unregister_clears_cli_meta(self):
        registry = CommandRegistry()

        @registry.register("export", cli="exp")
        def export() -> None:
            pass

        assert registry.cli_meta("export") is not None
        registry.unregister("export")
        assert registry.cli_meta("export") is None


class TestCommandDecoratorMixin:
    """Test the public @webview.command decorator path (RFC 0018 §6.2)."""

    def _webview(self):
        from auroraview.core.mixins.commands import WebViewCommandsMixin

        class _FakeWebView(WebViewCommandsMixin):
            def __init__(self):
                self._commands = None

            def emit(self, *args, **kwargs):
                pass

        return _FakeWebView()

    def test_bare_decorator_legacy(self):
        wv = self._webview()

        @wv.command
        def greet(name: str) -> str:
            return name

        assert "greet" in wv.commands
        assert wv.commands.cli_meta("greet") is None

    def test_positional_name_legacy(self):
        wv = self._webview()

        @wv.command("add")
        def add(x: int, y: int) -> int:
            return x + y

        assert wv.commands.invoke("add", x=2, y=3) == 5

    def test_name_keyword_with_cli(self):
        wv = self._webview()

        @wv.command(name="export", cli="exp", help="Export data", args_help={"path": "out dir"})
        def export(path: str, dpi: int = 300) -> dict:
            return {"path": path}

        meta = wv.commands.cli_meta("export")
        assert meta is not None
        assert meta.aliases == ["exp"]
        assert meta.help == "Export data"
        assert "export" in wv.commands
        assert wv.commands.invoke("export", path="/tmp")["path"] == "/tmp"

    def test_cli_true_no_name(self):
        wv = self._webview()

        @wv.command(cli=True)
        def sync() -> None:
            """Sync it."""

        assert wv.commands.cli_meta("sync").help == "Sync it."
