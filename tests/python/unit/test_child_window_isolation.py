# -*- coding: utf-8 -*-
"""RFC 0017 §4.3 / RFC 0015 §3.6: child-window isolation for ``capture_file_drop``.

Constructing ``WebViewConfig`` with both ``capture_file_drop=*`` and
``new_window_mode='child_webview'`` must remain a legal combination at
the Python layer:

    * The main window receives ``file_drop*`` IPC events normally
      (subject to the wry/WebView2 trade-off in RFC 0015 §2).
    * Child windows opened via ``window.open`` run on independent
      event loops and never register ``with_drag_drop_handler``,
      regardless of any setting on the parent. This is enforced by
      ``src/webview/child_window.rs`` and validated by Rust-side tests
      / CI grep, NOT by this Python suite.

These guards exist so a future PR cannot reintroduce an
over-eager validation rule that rejects this combination at construction
time.
"""

from __future__ import annotations

import pytest

from auroraview.core.config import (
    ContentConfig,
    NewWindowConfig,
    WebViewConfig,
)


@pytest.mark.parametrize(
    "capture_value",
    [
        pytest.param(True, id="capture_file_drop_true"),
        pytest.param(False, id="capture_file_drop_false"),
        pytest.param(None, id="capture_file_drop_omitted"),
    ],
)
def test_capture_file_drop_with_child_webview_mode_is_legal(capture_value):
    """All three tri-state values combine legally with ``child_webview`` mode."""
    cfg = WebViewConfig(
        content=ContentConfig(capture_file_drop=capture_value),
        new_window=NewWindowConfig(
            allow_new_window=True,
            new_window_mode="child_webview",
        ),
    )

    # Construction must succeed; tri-state must survive untouched.
    assert cfg.content.capture_file_drop is capture_value
    assert cfg.new_window.new_window_mode == "child_webview"
    assert cfg.new_window.allow_new_window is True


@pytest.mark.parametrize(
    "capture_value",
    [
        pytest.param(True, id="capture_file_drop_true"),
        pytest.param(False, id="capture_file_drop_false"),
        pytest.param(None, id="capture_file_drop_omitted"),
    ],
)
def test_from_kwargs_combines_capture_with_child_window(capture_value):
    """``from_kwargs`` flat form (the way users typically construct) is legal."""
    cfg = WebViewConfig.from_kwargs(
        capture_file_drop=capture_value,
        allow_new_window=True,
        new_window_mode="child_webview",
    )

    assert cfg.content.capture_file_drop is capture_value
    assert cfg.new_window.new_window_mode == "child_webview"


def test_to_kwargs_preserves_combination():
    """Round-trip: from_kwargs → to_kwargs must not flatten or drop fields."""
    cfg = WebViewConfig.from_kwargs(
        capture_file_drop=True,
        allow_new_window=True,
        new_window_mode="child_webview",
    )

    kwargs = cfg.to_kwargs()
    assert kwargs["capture_file_drop"] is True
    assert kwargs["new_window_mode"] == "child_webview"
    assert kwargs["allow_new_window"] is True
