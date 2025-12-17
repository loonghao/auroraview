"""Test spawn_ipc directly."""

import os
import sys
import time

# Setup paths
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, PROJECT_ROOT)
sys.path.insert(0, os.path.join(PROJECT_ROOT, "python"))

from auroraview import PluginManager, json_dumps, json_loads


def main():
    print("Testing spawn_ipc...")

    # Create plugin manager
    plugins = PluginManager.permissive()

    # Get Python executable
    python_exe = os.environ.get("AURORAVIEW_PYTHON_EXE", sys.executable)
    print(f"Python executable: {python_exe}")

    # Test script
    test_script = os.path.join(PROJECT_ROOT, "examples", "simple_decorator.py")
    print(f"Test script: {test_script}")

    if not os.path.exists(test_script):
        print("ERROR: Test script not found!")
        return

    # Test with show_console=False (IPC mode)
    print("\n=== Test 1: show_console=False (IPC mode) ===")
    args_json = json_dumps(
        {
            "command": python_exe,
            "args": [test_script],
            "cwd": os.path.dirname(test_script),
            "showConsole": False,
        }
    )
    print(f"Args: {args_json}")

    result_json = plugins.handle_command("plugin:process|spawn_ipc", args_json)
    result = json_loads(result_json)
    print(f"Result: {result}")

    if result.get("success"):
        pid = result.get("data", {}).get("pid")
        print(f"SUCCESS: Started with PID {pid}")

        # Wait a bit for process output
        print("Waiting for process output...")
        time.sleep(3)

        # List processes
        list_result = json_loads(plugins.handle_command("plugin:process|list", "{}"))
        print(f"Running processes: {list_result}")
    else:
        print(f"FAILED: {result.get('error')}")

    # Test with show_console=True
    print("\n=== Test 2: show_console=True (Console mode) ===")
    args_json = json_dumps(
        {
            "command": python_exe,
            "args": [test_script],
            "cwd": os.path.dirname(test_script),
            "showConsole": True,
        }
    )
    print(f"Args: {args_json}")

    result_json = plugins.handle_command("plugin:process|spawn_ipc", args_json)
    result = json_loads(result_json)
    print(f"Result: {result}")

    if result.get("success"):
        pid = result.get("data", {}).get("pid")
        print(f"SUCCESS: Started with PID {pid}")
        print("A console window should appear...")
        time.sleep(5)
    else:
        print(f"FAILED: {result.get('error')}")

    # Kill all processes
    print("\n=== Cleanup ===")
    plugins.handle_command("plugin:process|kill_all", "{}")
    print("Done.")


if __name__ == "__main__":
    main()
