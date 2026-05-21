#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""RFC 0016 §5: static guard for Browser-mode ``capture_file_drop`` isolation.

In multi-tab Browser mode (controller + business tabs), wry/WebView2's
multi-webview overlay model cannot maintain a coherent drag-drop state
machine across pixel boundaries. RFC 0016 §3 therefore mandates:

    1. ``BrowserConfig`` / ``TabManagerConfig`` MUST NOT expose a
       ``capture_file_drop`` field. This is enforced at compile time by
       the Rust type system, but the field could be reintroduced
       silently — this guard fails on any textual reference to the
       identifier inside the relevant config files.

    2. Every ``attach_drag_drop_handler`` call in Browser-mode code must
       pass the literal ``false`` as its second argument. Dynamic values
       (``cfg.capture_file_drop``, function calls, etc.) imply that some
       configuration entry point has been added in violation of (1).

    3. ``src/webview/child_window.rs`` MUST NOT reference
       ``with_drag_drop_handler`` at all. RFC 0015 §3.6 / §6.1 specify
       that child windows opened via ``window.open`` run on independent
       event loops without an IPC channel back to the parent and never
       register the drag-drop proxy.

The companion Python guard for tri-state preservation lives in
``check_capture_file_drop_defaults.py``.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


# Files where the Rust type ``BrowserConfig`` / ``TabManagerConfig`` /
# ``BrowserConfigBuilder`` is defined. Any textual occurrence of
# ``capture_file_drop`` here is a regression.
CONFIG_FILES = [
    REPO_ROOT / "crates" / "auroraview-browser" / "src" / "config.rs",
    REPO_ROOT / "src" / "webview" / "tab_manager.rs",
]


# Files / directories where every ``attach_drag_drop_handler`` second
# argument must be the literal ``false``.
BROWSER_CALL_SITES = [
    REPO_ROOT / "crates" / "auroraview-browser" / "src",
    REPO_ROOT / "src" / "webview" / "tab_manager.rs",
]


# RFC 0015 §3.6: child windows must NEVER reference ``with_drag_drop_handler``.
# A non-comment occurrence here means someone reintroduced the drag-drop
# proxy on a webview that has no IPC channel back to the parent.
CHILD_WINDOW_FILE = REPO_ROOT / "src" / "webview" / "child_window.rs"


# Match: attach_drag_drop_handler(<arg1>, <whatever-not-`false`>...)
# Use multiline DOTALL because the call may span lines.
ATTACH_CALL_RE = re.compile(
    r"attach_drag_drop_handler\s*\(\s*"  # function name + "("
    r"[^,()]+(?:\([^()]*\))?[^,()]*"  # arg1 — builder; tolerate one level of nested ()
    r",\s*"  # comma between arg1 and arg2
    r"(?P<second>[^,()]+(?:\([^()]*\))?)"  # arg2 — the capture flag we audit
    r"\s*,",
    re.DOTALL,
)


def iter_rust_files(target: Path):
    if target.is_file():
        yield target
        return
    for path in target.rglob("*.rs"):
        # Skip generated tests if any.
        yield path


def check_no_capture_field() -> list[tuple[Path, int, str]]:
    """Files in CONFIG_FILES must NOT mention ``capture_file_drop`` at all."""
    violations: list[tuple[Path, int, str]] = []
    for path in CONFIG_FILES:
        if not path.exists():
            print(f"[check] config file missing (skipped): {path}", file=sys.stderr)
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as exc:
            print(f"[check] cannot read {path}: {exc}", file=sys.stderr)
            continue

        # Allow the identifier inside `// RFC ...` comments (we're guarding
        # against new code, not against existing reference comments). We
        # detect this by skipping lines whose stripped form starts with `//`.
        # `tab_manager.rs` itself uses `NoopDragDropSink`, NOT a config field;
        # we still flag any non-comment occurrence as a regression.
        for line_no, raw_line in enumerate(text.splitlines(), start=1):
            stripped = raw_line.lstrip()
            if stripped.startswith("//") or stripped.startswith("///"):
                continue
            if "capture_file_drop" in raw_line:
                violations.append((path, line_no, raw_line.rstrip()))
    return violations


def check_browser_attach_calls_use_false() -> list[tuple[Path, int, str]]:
    """Every Browser-mode attach call must pass the literal ``false``."""
    violations: list[tuple[Path, int, str]] = []
    for target in BROWSER_CALL_SITES:
        for path in iter_rust_files(target):
            if not path.exists():
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError) as exc:
                print(f"[check] cannot read {path}: {exc}", file=sys.stderr)
                continue

            # Strip line comments to avoid matching example snippets in
            # `///` doc-comments / `//` rationale comments.
            stripped_lines = []
            for raw_line in text.splitlines():
                without_comment = re.sub(r"//.*$", "", raw_line)
                stripped_lines.append(without_comment)
            stripped_text = "\n".join(stripped_lines)

            for match in ATTACH_CALL_RE.finditer(stripped_text):
                second_arg = match.group("second").strip()
                if second_arg == "false":
                    continue
                # Compute approximate line number from the match start.
                line_no = stripped_text.count("\n", 0, match.start()) + 1
                violations.append(
                    (path, line_no, f"second arg = `{second_arg}` (must be `false`)")
                )
    return violations


def check_child_window_no_drag_drop() -> list[tuple[Path, int, str]]:
    """``src/webview/child_window.rs`` must not register drag-drop handlers.

    RFC 0015 §3.6 / §6.1: child windows opened via ``window.open`` run on
    independent event loops without an IPC channel back to the parent and
    must never call ``with_drag_drop_handler`` (directly or via the
    ``attach_drag_drop_handler`` helper).

    Comment lines (``//`` / ``///``) are tolerated so the module-level
    docstring referencing the design rationale stays intact.
    """
    violations: list[tuple[Path, int, str]] = []
    if not CHILD_WINDOW_FILE.exists():
        print(
            f"[check] child window file missing (skipped): {CHILD_WINDOW_FILE}",
            file=sys.stderr,
        )
        return violations

    try:
        text = CHILD_WINDOW_FILE.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        print(f"[check] cannot read {CHILD_WINDOW_FILE}: {exc}", file=sys.stderr)
        return violations

    forbidden_idents = ("with_drag_drop_handler", "attach_drag_drop_handler")
    for line_no, raw_line in enumerate(text.splitlines(), start=1):
        stripped = raw_line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        for ident in forbidden_idents:
            if ident in raw_line:
                violations.append(
                    (CHILD_WINDOW_FILE, line_no, f"`{ident}` in {raw_line.rstrip()}")
                )
                break
    return violations


def main() -> int:
    failed = False

    field_violations = check_no_capture_field()
    if field_violations:
        failed = True
        print(
            "ERROR: BrowserConfig / TabManagerConfig must not expose "
            "`capture_file_drop` (RFC 0016 §3.1)."
        )
        for path, line_no, line in field_violations:
            rel = path.relative_to(REPO_ROOT)
            print(f"  {rel}:{line_no}: {line}")
        print()

    attach_violations = check_browser_attach_calls_use_false()
    if attach_violations:
        failed = True
        print(
            "ERROR: Browser-mode `attach_drag_drop_handler` second argument "
            "must be the literal `false` (RFC 0016 §3.2)."
        )
        for path, line_no, detail in attach_violations:
            rel = path.relative_to(REPO_ROOT)
            print(f"  {rel}:{line_no}: {detail}")
        print()

    child_violations = check_child_window_no_drag_drop()
    if child_violations:
        failed = True
        print(
            "ERROR: child_window.rs must not register drag-drop handlers "
            "(RFC 0015 §3.6)."
        )
        for path, line_no, detail in child_violations:
            rel = path.relative_to(REPO_ROOT)
            print(f"  {rel}:{line_no}: {detail}")
        print()

    if failed:
        return 1

    print(
        "OK: Browser mode never attaches with_drag_drop_handler dynamically; "
        "child_window.rs stays drag-drop-free."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
