# -*- coding: utf-8 -*-
"""RFC 0017 §4.2: end-to-end passthrough of ``capture_file_drop`` to Rust.

The full contract:

    Python kwarg (Optional[bool])
        → ContentConfig.capture_file_drop : Optional[bool]
        → WebViewConfig.to_kwargs()['capture_file_drop'] : Optional[bool]
        → PyO3 boundary (src/bindings/desktop_runner.rs)
        → Rust WebViewConfig.capture_file_drop : bool   (unwrap_or(false))

The PyO3 layer is the SOLE permitted flatten point. These tests use the
``_dump_capture_file_drop`` test hook to observe the Rust-side bool with
no other side effects (no event loop, no window).

If a future PR adds ``setdefault('capture_file_drop', False)`` somewhere
in the Python passthrough chain, ``test_passthrough_omitted_falls_to_default_false``
will still pass *only because* the value collapsed early — but the
companion ``test_unit_*`` guarantees in
``tests/python/unit/test_capture_file_drop_tristate.py`` will fail.
The two layers together are the "static + runtime" defense from §4.
"""

from __future__ import annotations

import importlib

import pytest

# The PyO3 _core extension is built by maturin. Skip cleanly if running
# in an environment where the wheel hasn't been compiled yet (e.g. pure
# Python lint pass).
_core = pytest.importorskip(
    "auroraview._core",
    reason="auroraview._core PyO3 extension not built; run `vx just build` first",
)

if not hasattr(_core, "_dump_capture_file_drop"):
    pytest.skip(
        "auroraview._core._dump_capture_file_drop missing — rebuild PyO3 extension",
        allow_module_level=True,
    )


def _dump(value):
    """Invoke the Rust-side flatten point and return the resulting bool."""
    return _core._dump_capture_file_drop(value)


def test_passthrough_explicit_true():
    """``True`` survives the Python → Rust boundary."""
    assert _dump(True) is True


def test_passthrough_explicit_false():
    """``False`` is NOT silently dropped; Rust receives False, not the default."""
    assert _dump(False) is False


def test_passthrough_omitted_falls_to_default_false():
    """RFC 0017 §4.2 core defense.

    Omitted kwarg → ``None`` all the way through the Python layer →
    Rust ``unwrap_or(false)`` produces ``False``. If any middle layer
    silently substitutes ``False`` for ``None``, this test still passes,
    so it pairs with the unit test that proves the dataclass keeps
    ``None`` intact.
    """
    assert _dump(None) is False


def test_passthrough_via_webview_config_round_trip():
    """``WebViewConfig.to_kwargs()`` value must traverse PyO3 without change.

    Mirrors the real call site in ``factory.py`` / ``run_desktop``:
    take the dict produced by ``to_kwargs`` and feed exactly the
    ``capture_file_drop`` field into the binding.
    """
    config_module = importlib.import_module("auroraview.core.config")

    # Case 1: omitted
    kwargs = config_module.WebViewConfig.from_kwargs().to_kwargs()
    assert _dump(kwargs["capture_file_drop"]) is False

    # Case 2: explicit True
    kwargs = config_module.WebViewConfig.from_kwargs(capture_file_drop=True).to_kwargs()
    assert _dump(kwargs["capture_file_drop"]) is True

    # Case 3: explicit False
    kwargs = config_module.WebViewConfig.from_kwargs(capture_file_drop=False).to_kwargs()
    assert _dump(kwargs["capture_file_drop"]) is False
