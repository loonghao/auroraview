#!/usr/bin/env python3
"""Audit Rust unsafe usage against an explicit allowlist.

AuroraView needs some unsafe code at Win32, COM, and WebView2 FFI boundaries.
This audit keeps that unsafe surface intentional: new unsafe constructs must be
added to the allowlist with a count and rationale instead of slipping in silently.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


UNSAFE_CONSTRUCT_RE = re.compile(r"\bunsafe\s*(?:\{|impl\b|fn\b|extern\b|trait\b)")
EXCLUDED_DIRS = {
    ".git",
    ".venv",
    "dist",
    "gallery/dist",
    "node_modules",
    "packages/auroraview-sdk/coverage",
    "submodules",
    "target",
}


@dataclass(frozen=True)
class AllowlistEntry:
    count: int
    reason: str
    line_number: int


def repo_relative(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def is_excluded(path: Path, root: Path) -> bool:
    rel = repo_relative(path, root)
    rel_parts = Path(rel).parts

    for excluded in EXCLUDED_DIRS:
        excluded_parts = Path(excluded).parts
        if len(excluded_parts) == 1 and excluded_parts[0] in rel_parts:
            return True
        if rel_parts[: len(excluded_parts)] == excluded_parts:
            return True

    return False


def load_allowlist(path: Path) -> tuple[dict[str, AllowlistEntry], list[str]]:
    entries: dict[str, AllowlistEntry] = {}
    errors: list[str] = []

    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue

        parts = [part.strip() for part in line.split("|", 2)]
        if len(parts) != 3:
            errors.append(f"{path}:{line_number}: expected 'path | count | reason'")
            continue

        rel_path, count_text, reason = parts
        if not rel_path.endswith(".rs"):
            errors.append(f"{path}:{line_number}: allowlist path must be a Rust source file")
            continue
        if not reason:
            errors.append(f"{path}:{line_number}: unsafe allowlist entry needs a rationale")
            continue
        if rel_path in entries:
            errors.append(f"{path}:{line_number}: duplicate allowlist entry for {rel_path}")
            continue

        try:
            count = int(count_text)
        except ValueError:
            errors.append(f"{path}:{line_number}: unsafe count must be an integer")
            continue
        if count <= 0:
            errors.append(f"{path}:{line_number}: unsafe count must be positive")
            continue

        entries[rel_path] = AllowlistEntry(count=count, reason=reason, line_number=line_number)

    return entries, errors


def scan_unsafe(root: Path) -> dict[str, int]:
    counts: dict[str, int] = {}

    for path in root.rglob("*.rs"):
        if is_excluded(path, root):
            continue

        count = 0
        for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
            stripped = line.strip()
            if stripped.startswith("//"):
                continue
            count += len(UNSAFE_CONSTRUCT_RE.findall(line))

        if count:
            counts[repo_relative(path, root)] = count

    return counts


def audit(root: Path, allowlist_path: Path) -> int:
    allowlist, errors = load_allowlist(allowlist_path)
    actual = scan_unsafe(root)

    for rel_path, actual_count in sorted(actual.items()):
        entry = allowlist.get(rel_path)
        if entry is None:
            errors.append(
                f"{rel_path}: found {actual_count} unsafe construct(s), but the file is not allowlisted"
            )
            continue
        if actual_count != entry.count:
            errors.append(
                f"{rel_path}: expected {entry.count} unsafe construct(s) from allowlist line "
                f"{entry.line_number}, found {actual_count}"
            )

    for rel_path, entry in sorted(allowlist.items()):
        if rel_path not in actual:
            errors.append(
                f"{allowlist_path}:{entry.line_number}: stale allowlist entry for {rel_path}; "
                "the file has no unsafe constructs"
            )

    total = sum(actual.values())
    print(
        f"Unsafe audit: {total} construct(s) across {len(actual)} Rust file(s); "
        f"{len(allowlist)} allowlist entr{'y' if len(allowlist) == 1 else 'ies'}."
    )

    if errors:
        print("\nUnsafe audit failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        print(
            "\nIf the unsafe usage is genuinely required, update "
            f"{repo_relative(allowlist_path, root)} with the new count and rationale.",
            file=sys.stderr,
        )
        return 1

    print("Unsafe audit passed.")
    return 0


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--allowlist",
        type=Path,
        default=root / "scripts" / "unsafe_allowlist.txt",
        help="Path to the unsafe allowlist file.",
    )
    args = parser.parse_args()

    allowlist_path = args.allowlist
    if not allowlist_path.is_absolute():
        allowlist_path = root / allowlist_path

    return audit(root, allowlist_path)


if __name__ == "__main__":
    raise SystemExit(main())
