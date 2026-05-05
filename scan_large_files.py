#!/usr/bin/env python3
"""Scan for large files (>1000 lines) in the project."""
import os
import sys

def count_lines(filepath):
    """Count lines in a file."""
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            return sum(1 for _ in f)
    except Exception:
        return 0

def scan_directory(root_dir, extensions, min_lines=1000):
    """Scan directory for files with more than min_lines."""
    results = []
    
    for dirpath, dirnames, filenames in os.walk(root_dir):
        # Skip certain directories
        dirnames[:] = [d for d in dirnames if d not in {
            '.git', '__pycache__', '.pytest_cache', 'node_modules',
            'target', 'dist', 'build', '.venv', '.eggs'
        }]
        
        for filename in filenames:
            if any(filename.endswith(ext) for ext in extensions):
                filepath = os.path.join(dirpath, filename)
                line_count = count_lines(filepath)
                if line_count > min_lines:
                    results.append((filepath, line_count))
    
    return sorted(results, key=lambda x: -x[1])

if __name__ == '__main__':
    root = sys.argv[1] if len(sys.argv) > 1 else '.'
    extensions = ['.py', '.rs']
    
    print(f"Scanning {root} for files >1000 lines...")
    print("=" * 60)
    
    results = scan_directory(root, extensions)
    
    if results:
        print(f"\nFound {len(results)} files with >1000 lines:\n")
        for filepath, line_count in results:
            # Make path relative
            rel_path = os.path.relpath(filepath, root)
            print(f"  {rel_path}: {line_count} lines")
    else:
        print("\nNo files found with >1000 lines.")
    
    print("\nScan complete.")
