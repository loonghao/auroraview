#!/usr/bin/env python
"""
Run AuroraTest integration tests.

Usage:
    python run_tests.py                    # Run all tests
    python run_tests.py -k webview         # Run webview tests only
    python run_tests.py -k dom             # Run DOM tests only
    python run_tests.py -k events          # Run event tests only
    python run_tests.py -k api             # Run API tests only
    python run_tests.py -v                 # Verbose output
    python run_tests.py --headed           # Run with visible browser
"""

import sys
from pathlib import Path

# Add project paths
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root / "python"))


def main():
    import pytest

    # Default arguments
    args = [
        str(Path(__file__).parent),  # Test directory
        "-v",  # Verbose
        "-s",  # Show print statements
        "--tb=short",  # Short traceback
    ]

    # Add any command line arguments
    args.extend(sys.argv[1:])

    # Run pytest
    exit_code = pytest.main(args)
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
