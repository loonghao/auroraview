#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
Script to update _core.pyd with the latest compiled version.

This script handles the file locking issue by:
1. Removing the old _core.pyd
2. Renaming _core_new.pyd to _core.pyd
"""

import os
import sys
import shutil
import time

def update_core_pyd():
    """Update _core.pyd with _core_new.pyd."""
    
    # Get the directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_dir = os.path.dirname(script_dir)
    auroraview_dir = os.path.join(project_dir, 'python', 'auroraview')
    
    old_pyd = os.path.join(auroraview_dir, '_core.pyd')
    new_pyd = os.path.join(auroraview_dir, '_core_new.pyd')
    
    print(f"Updating _core.pyd...")
    print(f"  Old: {old_pyd}")
    print(f"  New: {new_pyd}")
    
    if not os.path.exists(new_pyd):
        print(f"[ERROR] Error: {new_pyd} not found")
        print("Please run 'just rebuild-core' first")
        return False
    
    # Try to remove old file
    max_retries = 5
    for attempt in range(max_retries):
        try:
            if os.path.exists(old_pyd):
                os.remove(old_pyd)
                print(f"[OK] Removed old _core.pyd")
            break
        except PermissionError as e:
            if attempt < max_retries - 1:
                print(f"[WARNING]  File locked, retrying in 1 second... (attempt {attempt + 1}/{max_retries})")
                time.sleep(1)
            else:
                print(f"[ERROR] Error: Could not remove old _core.pyd after {max_retries} attempts")
                print(f"   {e}")
                print("   Please close any applications using this file (IDE, Python, etc.)")
                return False
    
    # Rename new file
    try:
        shutil.move(new_pyd, old_pyd)
        print(f"[OK] Successfully updated _core.pyd")
        return True
    except Exception as e:
        print(f"[ERROR] Error: Failed to rename _core_new.pyd to _core.pyd")
        print(f"   {e}")
        return False

if __name__ == '__main__':
    success = update_core_pyd()
    sys.exit(0 if success else 1)

