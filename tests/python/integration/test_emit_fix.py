"""Test script to verify emit() event processing fix.

This script demonstrates that emit() now correctly processes events
in Qt/DCC environments by calling _post_eval_js_hook.

Usage:
    python examples/test_emit_fix.py
"""

import os

import pytest

from auroraview import WebView

# Skip in CI (requires display environment)
pytestmark = pytest.mark.skipif(
    os.environ.get("CI") == "true",
    reason="WebView tests require display environment, skipped in CI",
)


def test_emit_with_hook():
    """Test that emit() calls _post_eval_js_hook."""
    print("\n=== Testing emit() with _post_eval_js_hook ===")

    webview = WebView(title="Emit Test", width=800, height=600)

    # Track hook calls
    hook_calls = []

    def mock_hook():
        """Mock hook to track calls."""
        hook_calls.append(True)
        print(f"  ✓ Hook called! (total calls: {len(hook_calls)})")

    # Install hook (simulating Qt integration)
    webview._post_eval_js_hook = mock_hook
    print("✓ Hook installed")

    # Test emit()
    print("\nEmitting events...")
    webview.emit("test_event_1", {"data": "first"})
    webview.emit("test_event_2", {"data": "second"})
    webview.emit("test_event_3", {"data": "third"})

    # Verify hook was called for each emit
    assert len(hook_calls) == 3, f"Expected 3 hook calls, got {len(hook_calls)}"
    print(f"\n✅ SUCCESS: emit() called hook {len(hook_calls)} times")

    webview.close()


def test_eval_js_with_hook():
    """Test that eval_js() calls _post_eval_js_hook."""
    print("\n=== Testing eval_js() with _post_eval_js_hook ===")

    webview = WebView(title="Eval Test", width=800, height=600)

    # Track hook calls
    hook_calls = []

    def mock_hook():
        """Mock hook to track calls."""
        hook_calls.append(True)
        print(f"  ✓ Hook called! (total calls: {len(hook_calls)})")

    # Install hook
    webview._post_eval_js_hook = mock_hook
    print("✓ Hook installed")

    # Test eval_js()
    print("\nExecuting JavaScript...")
    webview.eval_js("console.log('test 1')")
    webview.eval_js("console.log('test 2')")

    # Verify hook was called for each eval_js
    assert len(hook_calls) == 2, f"Expected 2 hook calls, got {len(hook_calls)}"
    print(f"\n✅ SUCCESS: eval_js() called hook {len(hook_calls)} times")

    webview.close()


def test_without_hook():
    """Test that emit() and eval_js() work without hook."""
    print("\n=== Testing without _post_eval_js_hook ===")

    webview = WebView(title="No Hook Test", width=800, height=600)

    # No hook installed
    print("✓ No hook installed")

    # Should not crash
    print("\nEmitting events...")
    webview.emit("test_event", {"data": "test"})
    print("  ✓ emit() succeeded")

    print("\nExecuting JavaScript...")
    webview.eval_js("console.log('test')")
    print("  ✓ eval_js() succeeded")

    print("\n✅ SUCCESS: Both methods work without hook")

    webview.close()


def main():
    """Run all tests."""
    print("=" * 60)
    print("Testing emit() Event Processing Fix")
    print("=" * 60)

    try:
        test_emit_with_hook()
        test_eval_js_with_hook()
        test_without_hook()

        print("\n" + "=" * 60)
        print("✅ ALL TESTS PASSED")
        print("=" * 60)
        print("\nThe fix ensures that:")
        print("  1. emit() calls _post_eval_js_hook (like eval_js())")
        print("  2. Events are processed immediately in Qt/DCC environments")
        print("  3. Both methods work without hook (backward compatible)")

    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        return 1
    except Exception as e:
        print(f"\n❌ ERROR: {e}")
        import traceback

        traceback.print_exc()
        return 1

    return 0


if __name__ == "__main__":
    exit(main())
