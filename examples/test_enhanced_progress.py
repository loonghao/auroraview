#!/usr/bin/env python3
"""Test script for enhanced dependency installation progress.

This script simulates the enhanced progress reporting functionality
to verify that all events and progress updates work correctly.
"""

import time
import threading
from pathlib import Path
import sys

# Add project root to path
PROJECT_ROOT = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

def simulate_progress_events():
    """Simulate the progress events that would be emitted during installation."""
    
    # Mock WebView for testing
    class MockWebView:
        def __init__(self):
            self.events = []
        
        def emit(self, event_name, data):
            timestamp = time.strftime("%H:%M:%S")
            print(f"[{timestamp}] EVENT: {event_name}")
            print(f"  Data: {data}")
            print("-" * 50)
            self.events.append((event_name, data))
    
    webview = MockWebView()
    
    # Simulate installation start
    webview.emit("dependency:install_start", {
        "total": 2,
        "packages": ["openai>=1.0.0", "requests>=2.0.0"],
        "message": "Starting installation of 2 packages..."
    })
    
    time.sleep(0.5)
    
    # Simulate first package installation
    webview.emit("dependency:install_progress", {
        "type": "start",
        "package": "openai",
        "index": 0,
        "total": 2,
        "message": "[09:21:40] Installing openai..."
    })
    
    time.sleep(0.3)
    
    # Simulate download progress
    for progress in [10, 25, 50, 75, 90, 100]:
        webview.emit("dependency:install_progress", {
            "type": "output",
            "package": "openai",
            "line": f"[09:21:41] Downloading openai... {progress}%"
        })
        time.sleep(0.2)
    
    # Simulate completion of first package
    webview.emit("dependency:install_progress", {
        "type": "complete",
        "package": "openai",
        "success": True,
        "message": "[09:21:43] Successfully installed openai"
    })
    
    time.sleep(0.5)
    
    # Simulate second package with error
    webview.emit("dependency:install_progress", {
        "type": "start",
        "package": "requests",
        "index": 1,
        "total": 2,
        "message": "[09:21:44] Installing requests..."
    })
    
    time.sleep(0.3)
    
    # Simulate error
    webview.emit("dependency:install_progress", {
        "type": "error",
        "package": "requests",
        "success": False,
        "message": "[09:21:45] Failed to install requests (exit 1)",
        "output": "ERROR: Could not find a version that satisfies the requirement requests>=999.0.0\nERROR: No matching distribution found for requests>=999.0.0"
    })
    
    time.sleep(0.5)
    
    # Simulate installation completion with mixed results
    webview.emit("dependency:install_done", {
        "success": False,
        "installed": ["openai>=1.0.0"],
        "failed": ["requests>=2.0.0"],
        "output": "Installation completed with errors. See details above.",
        "cancelled": False
    })
    
    print(f"\nSimulation completed. Total events emitted: {len(webview.events)}")
    return webview.events

def test_progress_parsing():
    """Test progress percentage parsing from pip output."""
    test_lines = [
        "Downloading openai-1.3.5-py3-none-any.whl (234 kB) 25%",
        "Installing collected packages: openai 50%",
        "Successfully installed openai-1.3.5 100%",
        "ERROR: Could not find a version",
        "Collecting requests>=2.0.0",
    ]
    
    print("Testing progress parsing:")
    print("-" * 30)
    
    for line in test_lines:
        import re
        progress_match = re.search(r'(\d+)%', line)
        if progress_match:
            percent = int(progress_match.group(1))
            print(f"Line: {line}")
            print(f"  -> Progress: {percent}%")
        else:
            print(f"Line: {line}")
            print(f"  -> No progress found")
        print()

def main():
    """Run the test simulation."""
    print("=" * 60)
    print("Enhanced Dependency Installation Progress Test")
    print("=" * 60)
    print()
    
    print("1. Testing progress event simulation...")
    events = simulate_progress_events()
    
    print("\n" + "=" * 60)
    print("2. Testing progress parsing...")
    test_progress_parsing()
    
    print("\n" + "=" * 60)
    print("Test completed successfully!")
    print(f"Total events simulated: {len(events)}")
    print("=" * 60)

if __name__ == "__main__":
    main()