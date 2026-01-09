#!/usr/bin/env python3
"""Test script for debugging dependency installation issues.

This script tests the dependency installation functionality with enhanced logging
to help diagnose where the installation process might be getting stuck.
"""

import sys
import time
from pathlib import Path

# Add project root to path
PROJECT_ROOT = Path(__file__).resolve().parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

def test_dependency_installation():
    """Test dependency installation with debug logging."""
    print("=" * 60)
    print("Testing Dependency Installation with Debug Logging")
    print("=" * 60)
    
    try:
        from gallery.backend.dependency_installer import (
            parse_requirements_from_docstring,
            get_missing_requirements,
            install_requirements,
        )
        print("✓ Successfully imported dependency installer modules")
    except ImportError as e:
        print(f"❌ Failed to import dependency installer: {e}")
        return False
    
    # Test docstring parsing
    test_docstring = '''AI Chat Assistant Demo.
    
    This example demonstrates a hybrid application.
    
    Requirements:
        - PySide6>=6.5.0
        - openai>=1.0.0
    '''
    
    print(f"\nTesting docstring parsing...")
    requirements = parse_requirements_from_docstring(test_docstring)
    print(f"✓ Parsed requirements: {requirements}")
    
    # Check missing requirements
    print(f"\nChecking missing requirements...")
    missing = get_missing_requirements(requirements)
    print(f"✓ Missing requirements: {missing}")
    
    if not missing:
        print("✓ All requirements are already satisfied!")
        return True
    
    # Test installation with debug logging
    print(f"\nTesting installation of missing requirements: {missing}")
    print("This will show detailed debug information...")
    
    def progress_callback(progress):
        """Enhanced progress callback with debug info."""
        event_type = progress.get("type", "unknown")
        package = progress.get("package", "unknown")
        message = progress.get("message", "")
        line = progress.get("line", "")
        
        timestamp = time.strftime("%H:%M:%S")
        
        if event_type == "start":
            print(f"\n[{timestamp}] 🚀 STARTING: {package}")
            if message:
                print(f"[{timestamp}] MESSAGE: {message}")
        elif event_type == "output":
            if line:
                print(f"[{timestamp}] OUTPUT: {line}")
        elif event_type == "complete":
            print(f"\n[{timestamp}] ✅ COMPLETED: {package}")
            if message:
                print(f"[{timestamp}] MESSAGE: {message}")
        elif event_type == "error":
            print(f"\n[{timestamp}] ❌ ERROR: {package}")
            if message:
                print(f"[{timestamp}] ERROR MESSAGE: {message}")
        else:
            print(f"[{timestamp}] DEBUG: {progress}")
    
    try:
        print(f"\nStarting installation...")
        result = install_requirements(missing, on_progress=progress_callback)
        
        print(f"\n" + "=" * 60)
        print("INSTALLATION RESULT:")
        print(f"Success: {result.get('success', False)}")
        print(f"Installed: {result.get('installed', [])}")
        print(f"Failed: {result.get('failed', [])}")
        print(f"Cancelled: {result.get('cancelled', False)}")
        if result.get('output'):
            print(f"Output length: {len(result.get('output', ''))}")
        print("=" * 60)
        
        return result.get('success', False)
        
    except Exception as e:
        print(f"\n❌ Exception during installation: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Main test function."""
    success = test_dependency_installation()
    
    if success:
        print(f"\n✅ Test completed successfully!")
        sys.exit(0)
    else:
        print(f"\n❌ Test failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()