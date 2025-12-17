"""Test running a sample from packed mode."""

import os
import subprocess
import time


def main():
    exe_path = os.path.join(os.path.dirname(__file__), "pack-output", "pack-output.exe")

    print("Starting packed exe...")
    proc = subprocess.Popen(
        [exe_path],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        cwd=os.path.dirname(exe_path),
    )

    start_time = time.time()

    try:
        while True:
            line = proc.stdout.readline()
            if not line:
                if proc.poll() is not None:
                    break
                continue

            elapsed = time.time() - start_time

            # Filter for relevant logs
            if any(
                x in line
                for x in ["ProcessPlugin", "run_sample", "PYTHONPATH", "spawn", "ERROR", "error"]
            ):
                print(f"[{elapsed:6.2f}s] {line.rstrip()}")

            # Stop after 60 seconds
            if elapsed > 60:
                print("\n[Timeout - killing process]")
                proc.terminate()
                break

    except KeyboardInterrupt:
        print("\n[Interrupted]")
        proc.terminate()

    proc.wait(timeout=5)
    print(f"Exit code: {proc.returncode}")


if __name__ == "__main__":
    main()
