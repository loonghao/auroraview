#!/usr/bin/env python
"""
Update WebView API parameter names in all example files

Changes:
- dev_tools -> debug
- parent_hwnd -> parent
- parent_mode -> mode
"""

import re
from pathlib import Path


def update_file(file_path: Path) -> bool:
    """Update a single file. Returns True if changes were made."""
    try:
        content = file_path.read_text(encoding='utf-8')
        original_content = content
        
        # Replace parameter names in WebView() calls
        # Pattern: dev_tools=True/False
        content = re.sub(r'\bdev_tools\s*=', 'debug=', content)
        
        # Pattern: parent_hwnd=...
        content = re.sub(r'\bparent_hwnd\s*=', 'parent=', content)
        
        # Pattern: parent_mode=...
        content = re.sub(r'\bparent_mode\s*=', 'mode=', content)
        
        # Also update attribute access: ._parent_hwnd -> ._parent
        content = re.sub(r'\._parent_hwnd\b', '._parent', content)
        
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            return True
        return False
    except Exception as e:
        print(f"Error updating {file_path}: {e}")
        return False


def main():
    """Update all Python files in examples directory."""
    repo_root = Path(__file__).parent.parent
    examples_dir = repo_root / "examples"
    
    if not examples_dir.exists():
        print(f"Examples directory not found: {examples_dir}")
        return
    
    print("Updating WebView API parameter names...")
    print(f"Scanning: {examples_dir}")
    print()
    
    updated_files = []
    skipped_files = []
    
    # Find all Python files
    for py_file in examples_dir.rglob("*.py"):
        # Skip __pycache__ and node_modules
        if "__pycache__" in str(py_file) or "node_modules" in str(py_file):
            continue
        
        if update_file(py_file):
            updated_files.append(py_file)
            print(f"✓ Updated: {py_file.relative_to(repo_root)}")
        else:
            skipped_files.append(py_file)
    
    print()
    print("=" * 60)
    print(f"Updated: {len(updated_files)} files")
    print(f"Skipped: {len(skipped_files)} files (no changes needed)")
    print("=" * 60)
    
    if updated_files:
        print("\nUpdated files:")
        for f in updated_files:
            print(f"  - {f.relative_to(repo_root)}")


if __name__ == "__main__":
    main()

