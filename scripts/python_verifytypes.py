#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Verify Python type exports using pyright --verifytypes.

This script runs pyright --verifytypes on the auroraview package to ensure
that all public API types are properly exported and documented.

Usage:
    python scripts/python_verifytypes.py [--warn-only]

Options:
    --warn-only    Report issues as warnings instead of errors (exit 0)
"""
from __future__ import annotations

import subprocess
import sys


def main():
    warn_only = "--warn-only" in sys.argv

    print("=" * 60)
    print("Running pyright --verifytypes auroraview")
    print("=" * 60)

    cmd = [sys.executable, "-m", "pyright", "--verifytypes", "auroraview"]

    result = subprocess.run(cmd, capture_output=False)

    if result.returncode != 0:
        if warn_only:
            print()
            print("WARNING: pyright --verifytypes reported issues (warn-only mode)")
            print("Fix these to improve type safety for downstream users.")
            sys.exit(0)
        else:
            print()
            print("ERROR: pyright --verifytypes failed.")
            print("Run 'python scripts/python_verifytypes.py --warn-only' to see warnings.")
            sys.exit(1)
    else:
        print()
        print("All type exports verified successfully!")


if __name__ == "__main__":
    main()
