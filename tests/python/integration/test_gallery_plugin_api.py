"""Test Gallery Plugin API - PluginManager integration tests.

This module tests the PluginManager and ProcessPlugin integration used by Gallery.
These are headless tests that don't require a display.
"""

from __future__ import annotations

import sys
import time

import pytest

# Mark all tests as integration tests
pytestmark = [
    pytest.mark.integration,
]


class TestPluginManagerBasics:
    """Test PluginManager basic functionality."""

    def test_plugin_manager_import(self):
        """Test that PluginManager can be imported."""
        from auroraview import PluginManager

        assert PluginManager is not None

    def test_plugin_manager_permissive(self):
        """Test creating a permissive PluginManager."""
        from auroraview import PluginManager

        plugins = PluginManager.permissive()
        assert plugins is not None

    def test_plugin_manager_handle_command_list(self):
        """Test plugin:process|list command."""
        from auroraview import PluginManager, json_loads

        plugins = PluginManager.permissive()
        result_json = plugins.handle_command("plugin:process|list", "{}")
        result = json_loads(result_json)

        assert result.get("success") is True
        assert "data" in result
        assert "processes" in result["data"]
        assert isinstance(result["data"]["processes"], list)


class TestProcessPluginSpawnKill:
    """Test ProcessPlugin spawn and kill functionality."""

    @pytest.fixture
    def plugins(self):
        """Create a permissive PluginManager."""
        from auroraview import PluginManager

        return PluginManager.permissive()

    def test_spawn_ipc_and_kill(self, plugins):
        """Test spawning a process and killing it."""
        from auroraview import json_dumps, json_loads

        # Spawn a simple Python process that sleeps
        spawn_args = json_dumps(
            {
                "command": sys.executable,
                "args": ["-c", "import time; time.sleep(60)"],
            }
        )
        result_json = plugins.handle_command("plugin:process|spawn_ipc", spawn_args)
        result = json_loads(result_json)

        assert result.get("success") is True
        assert "data" in result
        pid = result["data"].get("pid")
        assert pid is not None
        assert isinstance(pid, int)
        assert pid > 0

        # Verify process is in list
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        assert list_result.get("success") is True
        assert pid in list_result["data"]["processes"]

        # Kill the process - pass pid as object
        kill_args = json_dumps({"pid": pid})
        kill_result = json_loads(plugins.handle_command("plugin:process|kill", kill_args))
        assert kill_result.get("success") is True

        # Verify process is removed from list
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        assert pid not in list_result["data"]["processes"]

    def test_spawn_ipc_with_console(self, plugins):
        """Test spawning a process with console option."""
        from auroraview import json_dumps, json_loads

        spawn_args = json_dumps(
            {
                "command": sys.executable,
                "args": ["-c", "import time; time.sleep(1)"],
                "showConsole": True,
            }
        )
        result_json = plugins.handle_command("plugin:process|spawn_ipc", spawn_args)
        result = json_loads(result_json)

        assert result.get("success") is True
        pid = result["data"].get("pid")
        assert pid is not None

        # Clean up
        kill_args = json_dumps({"pid": pid})
        plugins.handle_command("plugin:process|kill", kill_args)

    def test_kill_nonexistent_process(self, plugins):
        """Test killing a process that doesn't exist."""
        from auroraview import json_dumps, json_loads

        # Kill a nonexistent process should succeed (already exited)
        kill_args = json_dumps({"pid": 999999})
        result = json_loads(plugins.handle_command("plugin:process|kill", kill_args))
        assert result.get("success") is True

    def test_kill_all(self, plugins):
        """Test kill_all command."""
        from auroraview import json_dumps, json_loads

        # Spawn multiple processes
        pids = []
        for i in range(3):
            spawn_args = json_dumps(
                {
                    "command": sys.executable,
                    "args": ["-c", "import time; time.sleep(60)"],
                }
            )
            result = json_loads(plugins.handle_command("plugin:process|spawn_ipc", spawn_args))
            if result.get("success"):
                pids.append(result["data"]["pid"])

        assert len(pids) == 3

        # Verify all processes are in list
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        for pid in pids:
            assert pid in list_result["data"]["processes"]

        # Kill all
        kill_all_result = json_loads(plugins.handle_command("plugin:process|kill_all", "{}"))
        assert kill_all_result.get("success") is True
        assert kill_all_result["data"]["killed"] == 3

        # Verify all processes are removed
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        assert len(list_result["data"]["processes"]) == 0


class TestProcessPluginSend:
    """Test ProcessPlugin send functionality."""

    @pytest.fixture
    def plugins(self):
        """Create a permissive PluginManager."""
        from auroraview import PluginManager

        return PluginManager.permissive()

    def test_send_to_process(self, plugins):
        """Test sending data to a process."""
        from auroraview import json_dumps, json_loads

        # Spawn a process that reads stdin
        spawn_args = json_dumps(
            {
                "command": sys.executable,
                "args": ["-c", "import sys; print(sys.stdin.readline())"],
            }
        )
        result = json_loads(plugins.handle_command("plugin:process|spawn_ipc", spawn_args))
        assert result.get("success") is True
        pid = result["data"]["pid"]

        # Send data to process
        send_args = json_dumps({"pid": pid, "data": "hello\n"})
        send_result = json_loads(plugins.handle_command("plugin:process|send", send_args))
        assert send_result.get("success") is True

        # Wait a bit for process to finish
        time.sleep(0.5)

        # Clean up (might already be exited)
        kill_args = json_dumps({"pid": pid})
        plugins.handle_command("plugin:process|kill", kill_args)

    def test_send_to_nonexistent_process(self, plugins):
        """Test sending data to a nonexistent process."""
        from auroraview import json_dumps, json_loads

        send_args = json_dumps({"pid": 999999, "data": "test"})
        result = json_loads(plugins.handle_command("plugin:process|send", send_args))
        assert result.get("success") is False
        assert "error" in result


class TestProcessPluginEvents:
    """Test ProcessPlugin event emission."""

    @pytest.fixture
    def plugins(self):
        """Create a permissive PluginManager with event callback."""
        from auroraview import PluginManager

        return PluginManager.permissive()

    def test_process_stdout_event(self, plugins):
        """Test that process stdout is captured."""
        from auroraview import json_dumps, json_loads

        events = []

        def on_event(event_name, data):
            events.append((event_name, data))

        plugins.set_emit_callback(on_event)

        # Spawn a process that prints to stdout
        spawn_args = json_dumps(
            {
                "command": sys.executable,
                "args": ["-c", "print('hello from stdout')"],
            }
        )
        result = json_loads(plugins.handle_command("plugin:process|spawn_ipc", spawn_args))
        assert result.get("success") is True
        pid = result["data"]["pid"]

        # Wait for output
        time.sleep(1.0)

        # Check events
        stdout_events = [e for e in events if e[0] == "process:stdout"]
        assert len(stdout_events) > 0

        # Verify event data
        event_data = stdout_events[0][1]
        assert event_data.get("pid") == pid
        assert "hello from stdout" in event_data.get("data", "")

        # Clean up
        kill_args = json_dumps({"pid": pid})
        plugins.handle_command("plugin:process|kill", kill_args)

    def test_process_exit_event(self, plugins):
        """Test that process exit event is emitted."""
        from auroraview import json_dumps, json_loads

        events = []

        def on_event(event_name, data):
            events.append((event_name, data))

        plugins.set_emit_callback(on_event)

        # Spawn a process that exits immediately
        spawn_args = json_dumps(
            {
                "command": sys.executable,
                "args": ["-c", "print('done')"],
            }
        )
        result = json_loads(plugins.handle_command("plugin:process|spawn_ipc", spawn_args))
        assert result.get("success") is True
        pid = result["data"]["pid"]

        # Wait for process to exit
        time.sleep(1.0)

        # Check for exit event
        exit_events = [e for e in events if e[0] == "process:exit"]
        assert len(exit_events) > 0

        # Verify event data
        event_data = exit_events[0][1]
        assert event_data.get("pid") == pid
        assert event_data.get("code") == 0


class TestShellPlugin:
    """Test ShellPlugin functionality."""

    @pytest.fixture
    def plugins(self):
        """Create a permissive PluginManager."""
        from auroraview import PluginManager

        return PluginManager.permissive()

    def test_shell_open_url(self, plugins):
        """Test shell open command (mock - don't actually open browser)."""
        from auroraview import json_dumps, json_loads

        # This would open a browser, so we just test the command parsing
        # In a real test, we'd mock the actual open call
        open_args = json_dumps({"path": "https://example.com"})
        result = json_loads(plugins.handle_command("plugin:shell|open", open_args))
        # Should succeed (even if browser doesn't open in CI)
        assert result.get("success") is True

    def test_shell_which(self, plugins):
        """Test shell which command."""
        from auroraview import json_dumps, json_loads

        which_args = json_dumps({"command": "python"})
        result = json_loads(plugins.handle_command("plugin:shell|which", which_args))
        assert result.get("success") is True

    def test_shell_get_env(self, plugins):
        """Test shell get_env command."""
        from auroraview import json_dumps, json_loads

        env_args = json_dumps({"name": "PATH"})
        result = json_loads(plugins.handle_command("plugin:shell|get_env", env_args))
        assert result.get("success") is True
        assert result["data"]["value"] is not None


class TestGalleryAPISimulation:
    """Simulate Gallery API calls to test the full flow."""

    @pytest.fixture
    def plugins(self):
        """Create a permissive PluginManager."""
        from auroraview import PluginManager

        return PluginManager.permissive()

    def test_gallery_run_sample_flow(self, plugins):
        """Test the full Gallery run_sample -> kill_process flow."""
        from auroraview import json_dumps, json_loads

        events = []

        def on_event(event_name, data):
            events.append((event_name, data))

        plugins.set_emit_callback(on_event)

        # Simulate run_sample API call
        spawn_args = json_dumps(
            {
                "command": sys.executable,
                "args": ["-c", "print('Sample running'); import time; time.sleep(10)"],
                "showConsole": False,
            }
        )
        result = json_loads(plugins.handle_command("plugin:process|spawn_ipc", spawn_args))

        assert result.get("success") is True
        pid = result["data"]["pid"]
        assert pid > 0

        # Wait for stdout
        time.sleep(0.5)

        # Verify stdout event
        stdout_events = [e for e in events if e[0] == "process:stdout"]
        assert len(stdout_events) > 0

        # Simulate kill_process API call - THIS IS THE KEY TEST
        # The frontend now sends { pid: number } instead of just number
        kill_args = json_dumps({"pid": pid})
        kill_result = json_loads(plugins.handle_command("plugin:process|kill", kill_args))

        assert kill_result.get("success") is True

        # Verify process is gone
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        assert pid not in list_result["data"]["processes"]

    def test_gallery_list_processes_flow(self, plugins):
        """Test the Gallery list_processes flow."""
        from auroraview import json_dumps, json_loads

        # Spawn a few processes
        pids = []
        for _ in range(2):
            spawn_args = json_dumps(
                {
                    "command": sys.executable,
                    "args": ["-c", "import time; time.sleep(30)"],
                }
            )
            result = json_loads(plugins.handle_command("plugin:process|spawn_ipc", spawn_args))
            if result.get("success"):
                pids.append(result["data"]["pid"])

        # List processes
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        assert list_result.get("success") is True
        assert len(list_result["data"]["processes"]) >= 2

        # Clean up
        plugins.handle_command("plugin:process|kill_all", "{}")

    def test_gallery_open_url_flow(self, plugins):
        """Test the Gallery open_url flow."""
        from auroraview import json_dumps, json_loads

        # Simulate open_url API call
        open_args = json_dumps({"path": "https://github.com"})
        result = json_loads(plugins.handle_command("plugin:shell|open", open_args))
        assert result.get("success") is True
