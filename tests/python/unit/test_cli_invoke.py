# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Unit tests for headless CLI invoke (RFC 0018 §7 / §15.2).

Covers argument parsing (positional, ``--key value``, mixed, bool flags, type
coercion, JSON), command lookup across both registries, and the §4.4 exit
codes.
"""

from __future__ import annotations

import json
from io import StringIO
from unittest.mock import patch

from auroraview.core.cli_invoke import (
    EXIT_COMMAND_ERROR,
    EXIT_OK,
    EXIT_USAGE,
    _parse_args,
    _UsageError,
    run_cli_invoke,
)
from auroraview.core.commands import CommandError, CommandErrorCode, CommandRegistry


class _FakeWebView:
    """Minimal stand-in exposing the two registries used by §15.2 lookup."""

    def __init__(self, registry=None, bound=None):
        self._commands = registry
        self._bound_functions = bound or {}


def _registry_webview():
    wv = _FakeWebView(registry=CommandRegistry())
    return wv


class TestArgParsing:
    """Test _parse_args (§6.3)."""

    def test_keyword_args(self):
        def export(path: str, dpi: int = 300) -> dict:
            return {}

        assert _parse_args(export, ["--path", "./out", "--dpi", "600"]) == {
            "path": "./out",
            "dpi": 600,
        }

    def test_positional_args(self):
        def export(path: str, dpi: int = 300) -> dict:
            return {}

        assert _parse_args(export, ["./out", "600"]) == {"path": "./out", "dpi": 600}

    def test_mixed_positional_then_keyword(self):
        def export(path: str, dpi: int = 300) -> dict:
            return {}

        assert _parse_args(export, ["./out", "--dpi", "600"]) == {
            "path": "./out",
            "dpi": 600,
        }

    def test_positional_after_keyword_is_error(self):
        def export(path: str, dpi: int = 300) -> dict:
            return {}

        try:
            _parse_args(export, ["--dpi", "600", "./out"])
        except _UsageError as exc:
            assert "must come before" in str(exc)
        else:
            raise AssertionError("expected _UsageError")

    def test_double_assignment_is_error(self):
        def export(path: str) -> dict:
            return {}

        try:
            _parse_args(export, ["./out", "--path", "./other"])
        except _UsageError as exc:
            assert "both positionally" in str(exc)
        else:
            raise AssertionError("expected _UsageError")

    def test_unknown_option_is_error(self):
        def export(path: str) -> dict:
            return {}

        try:
            _parse_args(export, ["--unknown", "x"])
        except _UsageError as exc:
            assert "unknown option" in str(exc)
        else:
            raise AssertionError("expected _UsageError")

    def test_missing_required_is_error(self):
        def export(path: str) -> dict:
            return {}

        try:
            _parse_args(export, [])
        except _UsageError as exc:
            assert "missing required" in str(exc)
        else:
            raise AssertionError("expected _UsageError")

    def test_too_many_positionals_is_error(self):
        def export(path: str) -> dict:
            return {}

        try:
            _parse_args(export, ["a", "b"])
        except _UsageError as exc:
            assert "too many positional" in str(exc)
        else:
            raise AssertionError("expected _UsageError")

    def test_bool_flag_true(self):
        def sync(force: bool = False) -> dict:
            return {}

        assert _parse_args(sync, ["--force"]) == {"force": True}

    def test_bool_flag_no_prefix_false(self):
        def sync(force: bool = True) -> dict:
            return {}

        assert _parse_args(sync, ["--no-force"]) == {"force": False}

    def test_bool_positional_parses_truthy(self):
        def sync(force: bool = False) -> dict:
            return {}

        assert _parse_args(sync, ["true"]) == {"force": True}
        assert _parse_args(sync, ["0"]) == {"force": False}

    def test_float_coercion(self):
        def scale(factor: float) -> dict:
            return {}

        assert _parse_args(scale, ["--factor", "1.5"]) == {"factor": 1.5}

    def test_int_coercion_error(self):
        def export(dpi: int) -> dict:
            return {}

        try:
            _parse_args(export, ["--dpi", "abc"])
        except _UsageError as exc:
            assert "expected int" in str(exc)
        else:
            raise AssertionError("expected _UsageError")

    def test_json_for_complex_annotation(self):
        def configure(config: dict) -> dict:
            return {}

        assert _parse_args(configure, ["--config", '{"a": 1}']) == {"config": {"a": 1}}

    def test_unannotated_stays_string(self):
        def echo(value) -> dict:
            return {}

        assert _parse_args(echo, ["--value", "hello"]) == {"value": "hello"}


class TestRunCliInvoke:
    """Test run_cli_invoke end-to-end (lookup + invoke + exit code)."""

    def test_invokes_registry_command(self):
        wv = _registry_webview()

        @wv._commands.register("export", cli=True)
        def export(path: str) -> dict:
            return {"written": path}

        out = StringIO()
        with patch("sys.stdout", out):
            code = run_cli_invoke(wv, "export", ["--path", "./out"])

        assert code == EXIT_OK
        assert json.loads(out.getvalue()) == {"written": "./out"}

    def test_falls_back_to_bound_functions(self):
        def handler(name: str) -> dict:
            return {"hello": name}

        wv = _FakeWebView(bound={"greet": handler})

        out = StringIO()
        with patch("sys.stdout", out):
            code = run_cli_invoke(wv, "greet", ["--name", "world"])

        assert code == EXIT_OK
        assert json.loads(out.getvalue()) == {"hello": "world"}

    def test_command_not_found_exit_2(self):
        wv = _registry_webview()

        err = StringIO()
        with patch("sys.stderr", err):
            code = run_cli_invoke(wv, "missing", [])

        assert code == EXIT_USAGE
        assert "command not found" in err.getvalue()

    def test_usage_error_exit_2(self):
        wv = _registry_webview()

        @wv._commands.register("export", cli=True)
        def export(path: str) -> dict:
            return {}

        err = StringIO()
        with patch("sys.stderr", err):
            code = run_cli_invoke(wv, "export", [])  # missing required

        assert code == EXIT_USAGE
        assert "missing required" in err.getvalue()

    def test_command_exception_exit_1(self):
        wv = _registry_webview()

        @wv._commands.register("boom", cli=True)
        def boom() -> dict:
            raise ValueError("kaboom")

        err = StringIO()
        with patch("sys.stderr", err):
            code = run_cli_invoke(wv, "boom", [])

        assert code == EXIT_COMMAND_ERROR
        payload = json.loads(err.getvalue())
        assert payload["error"]["name"] == "ValueError"
        assert "kaboom" in payload["error"]["message"]

    def test_command_error_reuses_code(self):
        wv = _registry_webview()

        @wv._commands.register("denied", cli=True)
        def denied() -> dict:
            raise CommandError(CommandErrorCode.PERMISSION_DENIED, "nope")

        err = StringIO()
        with patch("sys.stderr", err):
            code = run_cli_invoke(wv, "denied", [])

        assert code == EXIT_COMMAND_ERROR
        payload = json.loads(err.getvalue())
        assert payload["error"]["name"] == "PERMISSION_DENIED"
        assert payload["error"]["message"] == "nope"
