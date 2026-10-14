#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""CI grep guard: keep the Rust and Python render-backend contracts in sync.

The host-adapter / render-backend contract is declared twice -- once in Rust
(``crates/auroraview-contract``) and once in Python (``python/auroraview/adapter``)
-- because hosts on both sides must report the same capabilities. The capability
bit values and each backend's declared capability set are therefore duplicated by
design, which makes them easy to drift apart silently.

A drift here is not cosmetic: if the Rust native backend declares
``MAIN_THREAD_DISPATCH`` and the Python one does not, the same logical backend
reports different capabilities depending on which language you ask. That is
exactly the bug this guard exists to catch, because no unit test on either side
can see the other.

Checks:
  1. every ``Feature`` bit constant has the same value in both languages;
  2. ``NativeWebviewBackend::capabilities()`` and
     ``NativeWebviewBackend.capabilities()`` declare the same set;
  3. same for ``ChromiumBackend``.

Exit code 0 on parity, 1 with a diff on mismatch.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]

RUST_CAPABILITY = REPO_ROOT / "crates" / "auroraview-contract" / "src" / "capability.rs"
RUST_BACKEND = REPO_ROOT / "crates" / "auroraview-contract" / "src" / "backend.rs"
PY_CAPABILITY = REPO_ROOT / "python" / "auroraview" / "adapter" / "capability.py"
PY_BACKEND = REPO_ROOT / "python" / "auroraview" / "adapter" / "backends.py"


def rust_bits(text: str) -> dict:
    """Parse `pub const NAME: Self = Self(1 << N);` from capability.rs."""
    pattern = re.compile(r"pub const ([A-Z_]+): Self = Self\(1 << (\d+)\);")
    return {name: int(shift) for name, shift in pattern.findall(text)}


def python_bits(text: str) -> dict:
    """Parse `NAME = 1 << N` from the Feature class in capability.py."""
    # Restrict to the Feature class body so unrelated module constants are ignored.
    start = text.index("class Feature:")
    body = text[start:]
    pattern = re.compile(r"^    ([A-Z_]+) = 1 << (\d+)$", re.MULTILINE)
    return {name: int(shift) for name, shift in pattern.findall(body)}


def rust_backend_caps(text: str, backend: str) -> set:
    """Parse the capability set of a Rust backend impl."""
    marker = "impl RenderBackend for {}".format(backend)
    if marker not in text:
        raise SystemExit("guard is stale: no `{}` in backend.rs".format(marker))
    body = text[text.index(marker) :]
    block = body[body.index("fn capabilities(&self) -> Features {") :]
    block = block[: block.index("}")]
    return set(re.findall(r"Features::([A-Z_]+)", block))


def python_backend_caps(text: str, class_name: str) -> set:
    """Parse the capability set of a Python backend class."""
    marker = "class {}(RenderBackend):".format(class_name)
    if marker not in text:
        raise SystemExit("guard is stale: no `{}` in backends.py".format(marker))
    body = text[text.index(marker) :]
    block = body[body.index("def capabilities(self) -> int:") :]
    block = block[: block.index("\n    def ")]
    return set(re.findall(r"Feature\.([A-Z_]+)", block))


def main() -> int:
    failures = []

    rust = rust_bits(RUST_CAPABILITY.read_text(encoding="utf-8"))
    py = python_bits(PY_CAPABILITY.read_text(encoding="utf-8"))

    if not rust or not py:
        print("[contract-parity] FAIL: could not parse Feature constants")
        return 1

    for name in sorted(set(rust) | set(py)):
        if rust.get(name) != py.get(name):
            failures.append(
                "capability bit {}: rust={!r} python={!r}".format(
                    name, rust.get(name), py.get(name)
                )
            )

    rust_backend_text = RUST_BACKEND.read_text(encoding="utf-8")
    py_backend_text = PY_BACKEND.read_text(encoding="utf-8")

    for backend in ("NativeWebviewBackend", "ChromiumBackend"):
        r_caps = rust_backend_caps(rust_backend_text, backend)
        p_caps = python_backend_caps(py_backend_text, backend)
        if r_caps != p_caps:
            failures.append(
                "{} capabilities differ: only-rust={} only-python={}".format(
                    backend, sorted(r_caps - p_caps), sorted(p_caps - r_caps)
                )
            )

    if failures:
        print("[contract-parity] FAIL")
        for line in failures:
            print("  - {}".format(line))
        print(
            "\nThe Rust and Python render-backend contracts must declare identical\n"
            "capabilities; see docs/design/adapter-contract.md."
        )
        return 1

    print("[contract-parity] OK: {} capability bits and 2 backends in sync".format(len(rust)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
