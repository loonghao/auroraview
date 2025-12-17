"""Debug script for packed gallery executable."""

import os
import subprocess
import sys
import time


def main():
    exe_path = os.path.join(os.path.dirname(__file__), "pack-output", "pack-output.exe")

    if not os.path.exists(exe_path):
        print(f"ERROR: {exe_path} not found")
        return 1

    print(f"Running: {exe_path}")
    print("=" * 60)

    # Run with full output capture
    proc = subprocess.Popen(
        [exe_path],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        cwd=os.path.dirname(exe_path),
    )

    start_time = time.time()
    lines = []

    try:
        while True:
            line = proc.stdout.readline()
            if not line:
                if proc.poll() is not None:
                    break
                continue

            elapsed = time.time() - start_time
            print(f"[{elapsed:6.2f}s] {line.rstrip()}")
            lines.append(line)

            # Check for common errors
            if "Error" in line or "error" in line or "panic" in line.lower():
                print("\n!!! ERROR DETECTED !!!")

            # Stop after 30 seconds
            if elapsed > 30:
                print("\n[Timeout - killing process]")
                proc.terminate()
                break

    except KeyboardInterrupt:
        print("\n[Interrupted]")
        proc.terminate()

    proc.wait(timeout=5)

    print("=" * 60)
    print(f"Exit code: {proc.returncode}")
    print(f"Total lines: {len(lines)}")

    return proc.returncode


if __name__ == "__main__":
    sys.exit(main() or 0)
