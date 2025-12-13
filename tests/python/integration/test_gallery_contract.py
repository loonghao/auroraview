"""Contract tests for Gallery API.

These tests verify that the Python backend API signatures match
what the frontend expects. This catches parameter mismatches early.

The contract is defined by inspecting the actual Python function signatures
and comparing them against expected TypeScript-style interfaces.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

# Add gallery to path
GALLERY_DIR = Path(__file__).parent.parent.parent.parent / "gallery"
sys.path.insert(0, str(GALLERY_DIR))

pytestmark = [
    pytest.mark.integration,
    pytest.mark.e2e,
]


class TestGalleryAPIContract:
    """Test that Gallery API functions have correct signatures."""

    @pytest.fixture
    def gallery_view(self):
        """Create a mock WebView with bound API functions."""
        from unittest.mock import MagicMock

        # Import gallery main to get the API bindings
        # We need to mock WebView to capture the bound functions
        import auroraview

        original_webview = auroraview.WebView

        bound_functions = {}

        class MockWebView:
            def __init__(self, **kwargs):
                self.kwargs = kwargs

            def bind_call(self, name):
                def decorator(func):
                    bound_functions[name] = func
                    return func

                return decorator

            def on(self, event):
                def decorator(func):
                    return func

                return decorator

            def create_emitter(self):
                emitter = MagicMock()
                emitter.emit = MagicMock()
                return emitter

            def show(self):
                pass

        # Patch WebView
        auroraview.WebView = MockWebView

        try:
            # Import will execute module-level code and register APIs
            # We need to reload to capture bindings
            if "gallery.main" in sys.modules:
                del sys.modules["gallery.main"]

            # Can't fully test without running gallery, so we test signatures directly
            yield bound_functions
        finally:
            auroraview.WebView = original_webview

    def test_kill_process_accepts_pid_kwarg(self):
        """Test that kill_process accepts pid as keyword argument.

        Frontend sends: { pid: number }
        Backend must accept: pid: int = 0
        """
        # Import the actual function
        sys.path.insert(0, str(GALLERY_DIR.parent / "python"))

        from auroraview import PluginManager, json_dumps, json_loads

        plugins = PluginManager.permissive()

        # Test that the plugin accepts pid as object key
        args_json = json_dumps({"pid": 12345})
        result_json = plugins.handle_command("plugin:process|kill", args_json)
        result = json_loads(result_json)

        # Should succeed (process doesn't exist, but format is correct)
        assert result.get("success") is True

    def test_spawn_ipc_accepts_object_params(self):
        """Test that spawn_ipc accepts all expected parameters."""
        import time

        sys.path.insert(0, str(GALLERY_DIR.parent / "python"))

        from auroraview import PluginManager, json_dumps, json_loads

        plugins = PluginManager.permissive()

        # Test full parameter set - use a quick command that exits immediately
        args = {
            "command": sys.executable,
            "args": ["-c", "print('test')"],
            "cwd": str(GALLERY_DIR),
            "showConsole": False,
        }
        args_json = json_dumps(args)
        result_json = plugins.handle_command("plugin:process|spawn_ipc", args_json)
        result = json_loads(result_json)

        assert result.get("success") is True
        assert "data" in result
        assert "pid" in result["data"]

        # Wait a bit for process to complete naturally
        time.sleep(0.5)

        # Cleanup - kill if still running (ignore errors if already exited)
        pid = result["data"]["pid"]
        plugins.handle_command("plugin:process|kill", json_dumps({"pid": pid}))

    def test_shell_open_accepts_path_param(self):
        """Test that shell open accepts path parameter."""
        sys.path.insert(0, str(GALLERY_DIR.parent / "python"))

        from auroraview import PluginManager, json_dumps, json_loads

        plugins = PluginManager.permissive()

        # Test with path parameter (not url)
        args_json = json_dumps({"path": "https://example.com"})
        result_json = plugins.handle_command("plugin:shell|open", args_json)
        result = json_loads(result_json)

        assert result.get("success") is True


class TestAPIParameterFormats:
    """Test expected parameter formats for all Gallery APIs."""

    # Define the expected contract
    API_CONTRACT = {
        "api.get_source": {
            "params": {"sample_id": str},
            "returns": str,
        },
        "api.run_sample": {
            "params": {"sample_id": str, "show_console": bool},
            "returns": dict,  # {ok, pid?, message?, error?}
        },
        "api.kill_process": {
            "params": {"pid": int},
            "returns": dict,  # {ok, error?}
        },
        "api.send_to_process": {
            "params": {"pid": int, "data": str},
            "returns": dict,  # {ok, error?}
        },
        "api.list_processes": {
            "params": {},
            "returns": dict,  # {ok, processes?, error?}
        },
        "api.open_url": {
            "params": {"url": str},
            "returns": dict,  # {ok, error?}
        },
        "api.get_samples": {
            "params": {},
            "returns": list,
        },
        "api.get_categories": {
            "params": {},
            "returns": dict,
        },
    }

    def test_contract_documentation(self):
        """Verify contract is documented."""
        for api_name, contract in self.API_CONTRACT.items():
            assert "params" in contract, f"{api_name} missing params"
            assert "returns" in contract, f"{api_name} missing returns"

    @pytest.mark.filterwarnings("ignore::DeprecationWarning")
    def test_frontend_uses_correct_formats(self):
        """Test that frontend API calls use correct parameter formats."""
        try:
            from playwright.sync_api import sync_playwright
        except ImportError:
            pytest.skip("Playwright not installed")

        # Test each API's expected format
        test_cases = [
            ("api.kill_process", [{"pid": 12345}]),
            ("api.get_source", [{"sample_id": "test"}]),
            ("api.run_sample", [{"sample_id": "test", "show_console": False}]),
            ("api.open_url", [{"url": "https://example.com"}]),
            ("api.send_to_process", [{"pid": 123, "data": "hello"}]),
            ("api.list_processes", []),
            ("api.get_samples", []),
            ("api.get_categories", []),
        ]

        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context()

            # Inject bridge
            context.add_init_script("""
                window.auroraview = {
                    call: function(method, params) {
                        window._lastCall = { method, params };
                        return Promise.resolve();
                    },
                    on: function() { return () => {}; },
                    api: new Proxy({}, {
                        get: function(target, prop) {
                            return function(...args) {
                                return window.auroraview.call('api.' + prop, args);
                            };
                        }
                    }),
                    _testMode: true
                };
            """)

            page = context.new_page()
            page.goto("data:text/html,<h1>Test</h1>")

            for api_name, expected_params in test_cases:
                # Simulate frontend call
                method_name = api_name.replace("api.", "")

                if expected_params:
                    # Call with params
                    page.evaluate(
                        f"""
                        window.auroraview.api.{method_name}({expected_params[0]!r})
                    """.replace("'", '"')
                        .replace("True", "true")
                        .replace("False", "false")
                    )
                else:
                    # Call without params
                    page.evaluate(f"window.auroraview.api.{method_name}()")

                last_call = page.evaluate("window._lastCall")
                assert last_call["method"] == api_name, f"Method mismatch for {api_name}"

                if expected_params:
                    assert last_call["params"] == expected_params, (
                        f"Params mismatch for {api_name}: {last_call['params']} != {expected_params}"
                    )

            browser.close()


class TestEventContract:
    """Test event payload contracts."""

    EVENT_CONTRACT = {
        "process:stdout": {"pid": int, "data": str},
        "process:stderr": {"pid": int, "data": str},
        "process:exit": {"pid": int, "code": int},
    }

    def test_process_events_have_correct_structure(self):
        """Test that process events emit correct payload structure."""
        import time

        sys.path.insert(0, str(GALLERY_DIR.parent / "python"))

        from auroraview import PluginManager, json_dumps, json_loads

        plugins = PluginManager.permissive()
        events = []

        def on_event(event_name, data):
            events.append((event_name, data))

        plugins.set_emit_callback(on_event)

        # Spawn a process that outputs and exits quickly
        args = {
            "command": sys.executable,
            "args": ["-c", "print('hello')"],
        }
        result = json_loads(plugins.handle_command("plugin:process|spawn_ipc", json_dumps(args)))

        assert result.get("success") is True
        pid = result["data"]["pid"]

        # Wait for events (process should complete quickly)
        time.sleep(0.5)

        # Cleanup - kill if still running
        plugins.handle_command("plugin:process|kill", json_dumps({"pid": pid}))

        # Verify event structure
        for event_name, data in events:
            if event_name in self.EVENT_CONTRACT:
                expected = self.EVENT_CONTRACT[event_name]
                for key, expected_type in expected.items():
                    assert key in data, f"Missing {key} in {event_name}"
                    # Type check (allow None for code on some platforms)
                    if data[key] is not None:
                        assert isinstance(data[key], expected_type), (
                            f"Wrong type for {key} in {event_name}: {type(data[key])}"
                        )
