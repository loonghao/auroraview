#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Screenshot generator for AuroraView examples.

This script runs each example in the examples/ directory and captures
a screenshot using Chrome DevTools Protocol (CDP).

Usage:
    python scripts/screenshot_examples.py                    # All examples
    python scripts/screenshot_examples.py --example window_effects_demo
    python scripts/screenshot_examples.py --list             # List available examples

Requirements:
    pip install playwright
    playwright install chromium
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional

PROJECT_ROOT = Path(__file__).parent.parent
EXAMPLES_DIR = PROJECT_ROOT / "examples"
SCREENSHOTS_DIR = PROJECT_ROOT / "docs" / "public" / "examples"
CDP_PORT = 9222

# Examples configuration: (filename, description, wait_seconds, skip)
# wait_seconds: how long to wait before taking screenshot
# skip: True if example requires special setup (Qt, DCC, etc.)
EXAMPLE_CONFIGS = {
    "simple_decorator": {
        "description": "Simple Decorator Pattern",
        "wait": 3,
        "skip": False,
    },
    "dynamic_binding": {
        "description": "Dynamic API Binding",
        "wait": 3,
        "skip": False,
    },
    "window_effects_demo": {
        "description": "Window Effects (Blur, Mica)",
        "wait": 4,
        "skip": False,
    },
    "window_events_demo": {
        "description": "Window Events",
        "wait": 3,
        "skip": False,
    },
    "desktop_app_demo": {
        "description": "Desktop Application",
        "wait": 4,
        "skip": False,
    },
    "desktop_events_demo": {
        "description": "Desktop Events",
        "wait": 3,
        "skip": False,
    },
    "floating_panel_demo": {
        "description": "Floating Panel",
        "wait": 3,
        "skip": False,
    },
    "multi_window_demo": {
        "description": "Multi-Window",
        "wait": 4,
        "skip": False,
    },
    "child_window_demo": {
        "description": "Child Windows",
        "wait": 4,
        "skip": False,
    },
    "child_aware_demo": {
        "description": "Child-Aware Context",
        "wait": 3,
        "skip": False,
    },
    "native_menu_demo": {
        "description": "Native Menu",
        "wait": 3,
        "skip": False,
    },
    "custom_context_menu_demo": {
        "description": "Custom Context Menu",
        "wait": 3,
        "skip": False,
    },
    "cookie_management_demo": {
        "description": "Cookie Management",
        "wait": 3,
        "skip": False,
    },
    "dom_manipulation_demo": {
        "description": "DOM Manipulation",
        "wait": 3,
        "skip": False,
    },
    "ipc_channel_demo": {
        "description": "IPC Channel",
        "wait": 3,
        "skip": False,
    },
    "local_assets_example": {
        "description": "Local Assets Loading",
        "wait": 3,
        "skip": False,
    },
    "logo_button_demo": {
        "description": "Logo Button",
        "wait": 3,
        "skip": False,
    },
    "signals_advanced_demo": {
        "description": "Advanced Signals",
        "wait": 4,
        "skip": False,
    },
    "system_tray_demo": {
        "description": "System Tray",
        "wait": 3,
        "skip": False,
    },
    # Qt-based examples (require Qt environment)
    "qt_style_tool": {
        "description": "Qt Style Tool",
        "wait": 3,
        "skip": True,
        "skip_reason": "Requires Qt/PySide",
    },
    "qt_custom_menu_demo": {
        "description": "Qt Custom Menu",
        "wait": 3,
        "skip": True,
        "skip_reason": "Requires Qt/PySide",
    },
    # DCC-specific examples
    "maya_qt_echo_demo": {
        "description": "Maya Qt Echo",
        "wait": 3,
        "skip": True,
        "skip_reason": "Requires Maya",
    },
    "dcc_integration_example": {
        "description": "DCC Integration",
        "wait": 3,
        "skip": True,
        "skip_reason": "Requires DCC application",
    },
}


def wait_for_cdp(port: int, timeout: float = 30.0) -> bool:
    """Wait for CDP port to become available."""
    import socket

    start = time.time()
    while time.time() - start < timeout:
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(1)
            result = sock.connect_ex(("127.0.0.1", port))
            sock.close()
            if result == 0:
                return True
        except Exception:
            pass
        time.sleep(0.5)
    return False


def take_screenshot_cdp(port: int, output_path: Path, wait_ms: int = 2000) -> bool:
    """Take screenshot using CDP via Playwright."""
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("[ERROR] Playwright not installed. Run: pip install playwright && playwright install chromium")
        return False

    try:
        with sync_playwright() as p:
            browser = p.chromium.connect_over_cdp(f"http://127.0.0.1:{port}")
            contexts = browser.contexts
            if not contexts:
                print("    [WARN] No browser contexts found")
                return False

            context = contexts[0]
            pages = context.pages
            if not pages:
                print("    [WARN] No pages found")
                return False

            page = pages[0]
            page.wait_for_timeout(wait_ms)

            output_path.parent.mkdir(parents=True, exist_ok=True)
            page.screenshot(path=str(output_path))
            return True

    except Exception as e:
        print(f"    [WARN] CDP screenshot failed: {e}")
        return False


def run_example_and_screenshot(
    example_name: str,
    config: dict,
    output_dir: Path,
    cdp_port: int = CDP_PORT,
) -> bool:
    """Run an example and take a screenshot.

    Returns True if successful.
    """
    example_file = EXAMPLES_DIR / f"{example_name}.py"
    if not example_file.exists():
        print(f"  [SKIP] File not found: {example_file.name}")
        return False

    if config.get("skip", False):
        reason = config.get("skip_reason", "Skipped")
        print(f"  [SKIP] {config['description']}: {reason}")
        return False

    print(f"  [RUN] {config['description']}...")

    # Set environment for CDP
    env = os.environ.copy()
    # WebView2 CDP argument
    env["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS"] = f"--remote-debugging-port={cdp_port}"

    # Start the example process
    process = subprocess.Popen(
        [sys.executable, str(example_file)],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=str(PROJECT_ROOT),
    )

    try:
        # Wait for CDP to be ready
        print(f"    Waiting for CDP on port {cdp_port}...")
        if not wait_for_cdp(cdp_port, timeout=15.0):
            print(f"    [WARN] CDP not available after 15s")
            return False

        # Additional wait for app to render
        wait_time = config.get("wait", 3)
        print(f"    Waiting {wait_time}s for app to render...")
        time.sleep(wait_time)

        # Take screenshot
        output_path = output_dir / f"{example_name}.png"
        if take_screenshot_cdp(cdp_port, output_path, wait_ms=1000):
            print(f"    [OK] Screenshot: {output_path.name}")
            return True
        else:
            print(f"    [FAIL] Could not capture screenshot")
            return False

    finally:
        # Terminate the example
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
        # Small delay to release port
        time.sleep(0.5)


def list_examples():
    """List all available examples."""
    print("\nAvailable examples:\n")
    print(f"{'Name':<30} {'Description':<30} {'Status'}")
    print("-" * 80)

    for name, config in sorted(EXAMPLE_CONFIGS.items()):
        status = "Skip: " + config.get("skip_reason", "") if config.get("skip") else "OK"
        print(f"{name:<30} {config['description']:<30} {status}")

    # Also list examples not in config
    print("\n\nExamples not configured (will use defaults):\n")
    for py_file in sorted(EXAMPLES_DIR.glob("*.py")):
        name = py_file.stem
        if name not in EXAMPLE_CONFIGS and name != "__init__":
            print(f"  {name}")


def main():
    parser = argparse.ArgumentParser(description="Screenshot generator for AuroraView examples")
    parser.add_argument("--example", "-e", help="Run specific example only")
    parser.add_argument("--list", "-l", action="store_true", help="List available examples")
    parser.add_argument("--output", "-o", default=str(SCREENSHOTS_DIR), help="Output directory")
    parser.add_argument("--cdp-port", type=int, default=CDP_PORT, help="CDP port")
    args = parser.parse_args()

    if args.list:
        list_examples()
        return 0

    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"\n[INFO] Screenshot output: {output_dir}")
    print(f"[INFO] Examples directory: {EXAMPLES_DIR}")
    print(f"[INFO] CDP port: {args.cdp_port}\n")

    # Determine which examples to run
    if args.example:
        examples = {args.example: EXAMPLE_CONFIGS.get(args.example, {"description": args.example, "wait": 3, "skip": False})}
    else:
        examples = EXAMPLE_CONFIGS

    # Run examples and take screenshots
    results = {"success": 0, "failed": 0, "skipped": 0}

    for name, config in examples.items():
        if config.get("skip", False):
            results["skipped"] += 1
            reason = config.get("skip_reason", "Skipped")
            print(f"  [SKIP] {name}: {reason}")
            continue

        success = run_example_and_screenshot(name, config, output_dir, args.cdp_port)

        if success:
            results["success"] += 1
        else:
            results["failed"] += 1

    # Summary
    print(f"\n{'=' * 50}")
    print(f"Results: {results['success']} success, {results['failed']} failed, {results['skipped']} skipped")
    print(f"Screenshots saved to: {output_dir}")
    print(f"{'=' * 50}\n")

    return 0 if results["failed"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
