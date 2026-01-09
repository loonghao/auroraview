#!/usr/bin/env python3
"""
Test script to verify the cancel button fix.
This simulates the installation and cancellation process.
"""

import time
import threading
from threading import Event

def simulate_installation_with_cancel():
    """Simulate installation process with cancel capability."""
    
    class MockWebView:
        def __init__(self):
            self.events = []
            
        def emit(self, event_name, data):
            timestamp = time.strftime("%H:%M:%S")
            print(f"[{timestamp}] EVENT: {event_name} -> {data}")
            self.events.append((event_name, data))
    
    class MockInstaller:
        def __init__(self, webview):
            self.webview = webview
            self._install_cancel_event = None
            
        def install_dependencies(self):
            """Start installation process."""
            self._install_cancel_event = Event()
            
            # Start installation
            self.webview.emit("dependency:install_start", {
                "total": 3,
                "packages": ["openai", "requests", "pyside6"],
                "message": "Starting installation of 3 packages..."
            })
            
            def install_worker():
                packages = ["openai", "requests", "pyside6"]
                
                for i, package in enumerate(packages):
                    if self._install_cancel_event.is_set():
                        print(f"Installation cancelled during {package}")
                        self.webview.emit("dependency:install_done", {
                            "success": False,
                            "cancelled": True,
                            "installed": packages[:i],
                            "failed": packages[i:],
                        })
                        return
                    
                    # Start package installation
                    self.webview.emit("dependency:install_progress", {
                        "type": "start",
                        "package": package,
                        "total": len(packages),
                        "index": i
                    })
                    
                    # Simulate installation progress
                    for progress in [25, 50, 75, 100]:
                        if self._install_cancel_event.is_set():
                            print(f"Installation cancelled during {package} at {progress}%")
                            self.webview.emit("dependency:install_done", {
                                "success": False,
                                "cancelled": True,
                                "installed": packages[:i],
                                "failed": packages[i:],
                            })
                            return
                        
                        self.webview.emit("dependency:install_progress", {
                            "type": "output",
                            "line": f"Downloading {package}... {progress}%",
                            "package": package
                        })
                        time.sleep(0.5)  # Simulate download time
                    
                    # Complete package
                    self.webview.emit("dependency:install_progress", {
                        "type": "complete",
                        "package": package,
                        "total": len(packages),
                        "index": i
                    })
                
                # All packages completed
                self.webview.emit("dependency:install_done", {
                    "success": True,
                    "installed": packages,
                    "failed": [],
                    "cancelled": False
                })
            
            thread = threading.Thread(target=install_worker, daemon=True)
            thread.start()
            return {"started": True}
        
        def cancel_installation(self):
            """Cancel ongoing installation."""
            if self._install_cancel_event is None:
                return {"success": False, "error": "No installation in progress"}
            
            try:
                # Send cancel progress events
                self.webview.emit("dependency:cancel_progress", {
                    "type": "cancelling",
                    "message": "Requesting cancellation..."
                })
                
                # Set cancel event
                self._install_cancel_event.set()
                
                # Send cancel confirmation
                self.webview.emit("dependency:cancel_progress", {
                    "type": "cancelled", 
                    "message": "Cancellation signal sent"
                })
                
                return {"success": True, "message": "Cancellation requested"}
            except Exception as e:
                return {"success": False, "error": f"Failed to cancel: {str(e)}"}

    # Test scenario
    print("🧪 Testing Cancel Button Fix")
    print("=" * 50)
    
    webview = MockWebView()
    installer = MockInstaller(webview)
    
    print("\n1. Starting installation...")
    installer.install_dependencies()
    
    # Let it run for a bit
    time.sleep(2)
    
    print("\n2. User clicks cancel button...")
    result = installer.cancel_installation()
    print(f"Cancel result: {result}")
    
    # Wait for cancellation to complete
    time.sleep(1)
    
    print("\n3. Final events summary:")
    for event_name, data in webview.events[-5:]:  # Show last 5 events
        print(f"   {event_name}: {data}")
    
    print("\n✅ Test completed!")
    print("\nExpected behavior:")
    print("- Progress bar should remain visible during cancellation")
    print("- Cancel button should be disabled after clicking")
    print("- Progress text should show 'Cancelling installation...'")
    print("- Final event should be 'dependency:install_done' with cancelled=True")

if __name__ == "__main__":
    simulate_installation_with_cancel()