"""AuroraView CLI entry point.

This module provides a Python entry point that delegates to the Rust CLI binary.
When installed via pip/uv, the binary is automatically available in PATH.
"""

import subprocess
import sys


def main():
    """Main entry point for the CLI.

    This function simply delegates to the auroraview binary that is installed
    alongside the Python package by maturin.
    """
    try:
        # When installed via maturin, the binary is in PATH
        result = subprocess.run(
            ["auroraview"] + sys.argv[1:],
            check=False,
        )
        sys.exit(result.returncode)
    except FileNotFoundError:
        print(
            "Error: auroraview CLI binary not found.",
            file=sys.stderr,
        )
        print(
            "Please ensure the package is properly installed with: pip install auroraview",
            file=sys.stderr,
        )
        print(
            "For development: cargo build --release --features cli --bin auroraview",
            file=sys.stderr,
        )
        sys.exit(1)
    except Exception as e:
        print(f"Error executing CLI: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
