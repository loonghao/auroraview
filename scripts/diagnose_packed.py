#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Diagnose packed mode startup issues.

This script analyzes the environment and configuration to identify
why a packed AuroraView application might fail to start properly.

Usage:
    python scripts/diagnose_packed.py [path_to_packed_exe]
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def print_header(title: str) -> None:
    """Print a formatted header."""
    print(f"\n{'='*60}")
    print(f" {title}")
    print(f"{'='*60}")


def print_status(name: str, value: str, ok: bool = True) -> None:
    """Print a status line with OK/FAIL indicator."""
    status = "✓" if ok else "✗"
    color = "\033[92m" if ok else "\033[91m"
    reset = "\033[0m"
    print(f"  {color}{status}{reset} {name}: {value}")


def check_environment() -> dict:
    """Check environment variables."""
    print_header("Environment Variables")
    
    env_vars = {
        "AURORAVIEW_PACKED": os.environ.get("AURORAVIEW_PACKED"),
        "AURORAVIEW_PYTHON_PATH": os.environ.get("AURORAVIEW_PYTHON_PATH"),
        "AURORAVIEW_PYTHON_EXE": os.environ.get("AURORAVIEW_PYTHON_EXE"),
        "AURORAVIEW_RESOURCES_DIR": os.environ.get("AURORAVIEW_RESOURCES_DIR"),
        "PYTHONPATH": os.environ.get("PYTHONPATH"),
        "PYTHONUNBUFFERED": os.environ.get("PYTHONUNBUFFERED"),
    }
    
    issues = []
    
    for name, value in env_vars.items():
        if value:
            print_status(name, value)
        else:
            print_status(name, "(not set)", ok=False)
            if name in ["AURORAVIEW_PYTHON_PATH", "AURORAVIEW_RESOURCES_DIR"]:
                issues.append(f"{name} is not set - this is required in packed mode")
    
    return {"env_vars": env_vars, "issues": issues}


def check_paths() -> dict:
    """Check if important paths exist."""
    print_header("Path Validation")
    
    issues = []
    paths_checked = {}
    
    # Check AURORAVIEW_PYTHON_PATH directories
    python_path = os.environ.get("AURORAVIEW_PYTHON_PATH", "")
    if python_path:
        separator = ";" if sys.platform == "win32" else ":"
        for path in python_path.split(separator):
            if path:
                exists = Path(path).exists()
                paths_checked[path] = exists
                print_status(f"PYTHONPATH entry", path, ok=exists)
                if not exists:
                    issues.append(f"PYTHONPATH entry does not exist: {path}")
    
    # Check AURORAVIEW_RESOURCES_DIR
    resources_dir = os.environ.get("AURORAVIEW_RESOURCES_DIR")
    if resources_dir:
        exists = Path(resources_dir).exists()
        paths_checked[resources_dir] = exists
        print_status("RESOURCES_DIR", resources_dir, ok=exists)
        if not exists:
            issues.append(f"RESOURCES_DIR does not exist: {resources_dir}")
        else:
            # Check for examples subdirectory
            examples_dir = Path(resources_dir) / "examples"
            exists = examples_dir.exists()
            paths_checked[str(examples_dir)] = exists
            print_status("examples directory", str(examples_dir), ok=exists)
            if not exists:
                issues.append(f"Examples directory does not exist: {examples_dir}")
    
    # Check AURORAVIEW_PYTHON_EXE
    python_exe = os.environ.get("AURORAVIEW_PYTHON_EXE")
    if python_exe:
        exists = Path(python_exe).exists()
        paths_checked[python_exe] = exists
        print_status("Python executable", python_exe, ok=exists)
        if not exists:
            issues.append(f"Python executable does not exist: {python_exe}")
    
    return {"paths": paths_checked, "issues": issues}


def check_imports() -> dict:
    """Check if required modules can be imported."""
    print_header("Module Imports")
    
    issues = []
    modules_checked = {}
    
    required_modules = [
        "auroraview",
        "auroraview.core.webview",
        "auroraview.core.packed",
        "auroraview.plugins",
    ]
    
    for module in required_modules:
        try:
            __import__(module)
            modules_checked[module] = True
            print_status(module, "imported successfully")
        except ImportError as e:
            modules_checked[module] = False
            print_status(module, f"FAILED: {e}", ok=False)
            issues.append(f"Cannot import {module}: {e}")
    
    return {"modules": modules_checked, "issues": issues}


def check_cache_directory() -> dict:
    """Check the cache directory structure."""
    print_header("Cache Directory")
    
    issues = []
    
    # Determine cache directory
    if sys.platform == "win32":
        cache_base = Path(os.environ.get("LOCALAPPDATA", "")) / "AuroraView"
    else:
        cache_base = Path.home() / ".cache" / "AuroraView"
    
    print_status("Cache base", str(cache_base), ok=cache_base.exists())
    
    if cache_base.exists():
        # List subdirectories
        for subdir in ["python", "runtime"]:
            subpath = cache_base / subdir
            if subpath.exists():
                print_status(f"  {subdir}/", str(subpath))
                # List contents
                for item in sorted(subpath.iterdir())[:5]:
                    print(f"      - {item.name}")
                remaining = len(list(subpath.iterdir())) - 5
                if remaining > 0:
                    print(f"      ... and {remaining} more")
            else:
                print_status(f"  {subdir}/", "(not found)", ok=False)
    
    return {"cache_base": str(cache_base), "issues": issues}


def simulate_packed_startup() -> dict:
    """Simulate what happens during packed mode startup."""
    print_header("Packed Mode Simulation")
    
    issues = []
    
    # Check if we're in packed mode
    is_packed = os.environ.get("AURORAVIEW_PACKED", "0") == "1"
    print_status("Packed mode", "YES" if is_packed else "NO", ok=True)
    
    if not is_packed:
        print("\n  Note: Not running in packed mode. Set AURORAVIEW_PACKED=1 to simulate.")
        return {"is_packed": False, "issues": issues}
    
    # Try to import and check packed module
    try:
        from auroraview.core.packed import is_packed_mode, PACKED_MODE
        print_status("Packed module", "loaded")
        print_status("PACKED_MODE constant", str(PACKED_MODE))
    except ImportError as e:
        print_status("Packed module", f"FAILED: {e}", ok=False)
        issues.append(f"Cannot import packed module: {e}")
        return {"is_packed": True, "issues": issues}
    
    # Check WebView import
    try:
        from auroraview import WebView
        print_status("WebView", "imported")
    except ImportError as e:
        print_status("WebView", f"FAILED: {e}", ok=False)
        issues.append(f"Cannot import WebView: {e}")
    
    return {"is_packed": True, "issues": issues}


def analyze_log_output(log_text: str) -> dict:
    """Analyze log output for common issues."""
    print_header("Log Analysis")
    
    issues = []
    
    # Check for common error patterns
    error_patterns = [
        ("ModuleNotFoundError", "Python module import failed"),
        ("FileNotFoundError", "Required file is missing"),
        ("PermissionError", "Permission denied accessing file/directory"),
        ("OSError", "Operating system error"),
        ("Python process closed stdout", "Python crashed before sending ready signal"),
        ("Failed to spawn", "Process spawn failed"),
        ("No overlay data found", "Packed executable is corrupted or not packed"),
    ]
    
    for pattern, description in error_patterns:
        if pattern in log_text:
            print_status(pattern, description, ok=False)
            issues.append(f"{pattern}: {description}")
    
    # Check for success indicators
    success_patterns = [
        ("Python backend ready", "Python started successfully"),
        ("Ready signal sent", "Python sent ready signal"),
        ("Overlay read completed", "Overlay data read successfully"),
    ]
    
    for pattern, description in success_patterns:
        if pattern in log_text:
            print_status(pattern, description, ok=True)
    
    return {"issues": issues}


def print_summary(all_issues: list) -> None:
    """Print a summary of all issues found."""
    print_header("Diagnosis Summary")
    
    if not all_issues:
        print("\n  ✓ No issues detected!")
        print("\n  If the application still fails to start, check:")
        print("    1. Run the packed exe from command line to see stderr output")
        print("    2. Enable show_console=true in auroraview.pack.toml for debugging")
        print("    3. Check Windows Event Viewer for crash reports")
    else:
        print(f"\n  Found {len(all_issues)} issue(s):\n")
        for i, issue in enumerate(all_issues, 1):
            print(f"  {i}. {issue}")
        
        print("\n  Suggested fixes:")
        print("    1. Ensure all paths in PYTHONPATH exist")
        print("    2. Verify auroraview module is included in include_paths")
        print("    3. Check that resources are correctly extracted")
        print("    4. Re-run pack command with --verbose for more details")


def main() -> None:
    """Main entry point."""
    print("\n" + "="*60)
    print(" AuroraView Packed Mode Diagnostics")
    print("="*60)
    print(f"\n  Python: {sys.executable}")
    print(f"  Version: {sys.version}")
    print(f"  Platform: {sys.platform}")
    
    all_issues = []
    
    # Run all checks
    result = check_environment()
    all_issues.extend(result.get("issues", []))
    
    result = check_paths()
    all_issues.extend(result.get("issues", []))
    
    result = check_imports()
    all_issues.extend(result.get("issues", []))
    
    result = check_cache_directory()
    all_issues.extend(result.get("issues", []))
    
    result = simulate_packed_startup()
    all_issues.extend(result.get("issues", []))
    
    # If log file provided, analyze it
    if len(sys.argv) > 1:
        log_file = sys.argv[1]
        if Path(log_file).exists():
            with open(log_file) as f:
                log_text = f.read()
            result = analyze_log_output(log_text)
            all_issues.extend(result.get("issues", []))
    
    # Print summary
    print_summary(all_issues)


if __name__ == "__main__":
    main()
