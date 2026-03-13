from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    gallery_dir = Path(__file__).resolve().parent.parent / "gallery"
    dist_index = gallery_dir / "dist" / "index.html"

    if dist_index.exists():
        print(f"[OK] Reusing existing Gallery build: {dist_index}")
        return 0

    print("[INFO] gallery/dist not found; building frontend...")
    completed = subprocess.run(["vx", "bun", "run", "build"], cwd=gallery_dir)
    return completed.returncode


if __name__ == "__main__":
    raise SystemExit(main())
