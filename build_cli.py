"""Build script for AuroraView CLI binary.

This script builds the Rust CLI binary and copies it to the Python package directory
for distribution with the wheel.
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path


def build_cli():
    """Build the CLI binary using cargo."""
    print("Building AuroraView CLI binary...")

    # Build the CLI binary
    result = subprocess.run(
        [
            "cargo",
            "build",
            "--release",
            "--features",
            "cli",
            "--bin",
            "auroraview",
        ],
        check=False,
    )

    if result.returncode != 0:
        print("Error: Failed to build CLI binary", file=sys.stderr)
        sys.exit(1)

    print("CLI binary built successfully")


def copy_binary():
    """Copy the built binary to the Python package directory."""
    print("Copying CLI binary to package directory...")

    # Determine binary name based on platform
    if sys.platform == "win32":
        binary_name = "auroraview.exe"
    else:
        binary_name = "auroraview"

    # Source path
    source = Path("target") / "release" / binary_name

    if not source.exists():
        print(f"Error: Binary not found at {source}", file=sys.stderr)
        sys.exit(1)

    # Destination path
    dest_dir = Path("python") / "auroraview" / "bin"
    dest_dir.mkdir(parents=True, exist_ok=True)
    dest = dest_dir / binary_name

    # Copy binary
    shutil.copy2(source, dest)

    # Make executable on Unix-like systems
    if sys.platform != "win32":
        os.chmod(dest, 0o755)

    print(f"Binary copied to {dest}")


def main():
    """Main entry point."""
    build_cli()
    copy_binary()
    print("CLI build complete!")


if __name__ == "__main__":
    main()

