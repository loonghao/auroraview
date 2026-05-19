# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""Regression tests for ``_WebViewInitState.from_init_kwargs``.

This dataclass + classmethod pair is the single funnel that
``WebView.__init__`` and ``WebView.create_embedded`` use to build a
construction-state snapshot. It encodes a small but subtle state
machine:

  * required fields (no dataclass default) keep ``None`` as a real value,
  * optional fields treat ``None`` as "use the dataclass default",
  * ``forward_asset_root_to_core`` decides whether
    ``_init_runtime_state`` will call ``core.set_asset_root``,
  * ``strict=True`` (default) makes typos in kwargs fail loudly.

These tests pin every branch of that machine so a future refactor that
breaks one of them gets caught at PR time instead of at runtime in a
DCC host.
"""

from __future__ import annotations

import pytest

from auroraview.core.webview import _WebViewInitState as State

# Required (no-default) fields. Always passed in tests; ``None`` for
# the ``Optional[X]``-typed ones so we can verify they survive the
# None-sentinel filter.
_REQUIRED_KWARGS = {
    "title": "t",
    "width": 800,
    "height": 600,
    "url": None,
    "html": None,
    "debug": False,
    "resizable": True,
    "frame": True,
    "parent": None,
    "mode": None,
}


class TestRequiredFields:
    """Required fields: ``None`` is a real value, must survive."""

    def test_required_optional_none_preserved(self):
        s = State.from_init_kwargs(**_REQUIRED_KWARGS)
        assert s.url is None
        assert s.html is None
        assert s.parent is None
        assert s.mode is None

    def test_required_concrete_values_preserved(self):
        s = State.from_init_kwargs(
            **{**_REQUIRED_KWARGS, "url": "https://example.com", "parent": 12345, "mode": "child"}
        )
        assert s.url == "https://example.com"
        assert s.parent == 12345
        assert s.mode == "child"


class TestOptionalNoneSentinel:
    """Optional fields: ``None`` falls through to the dataclass default."""

    def test_optional_none_falls_to_default(self):
        s = State.from_init_kwargs(
            **_REQUIRED_KWARGS,
            always_on_top=None,
            transparent=None,
            auto_show=None,
            dcc_mode=None,
        )
        # Defaults from the dataclass definition.
        assert s.always_on_top is False
        assert s.transparent is False
        assert s.auto_show is True
        assert s.dcc_mode == "auto"

    def test_explicit_value_overrides_default(self):
        s = State.from_init_kwargs(
            **_REQUIRED_KWARGS,
            always_on_top=True,
            auto_show=False,
            dcc_mode="maya",
        )
        assert s.always_on_top is True
        assert s.auto_show is False
        assert s.dcc_mode == "maya"

    def test_bridge_false_is_legal_value_not_sentinel(self):
        """``bridge=False`` means "do not auto-create a bridge", which is
        a legal user choice — distinct from ``None`` ("use the dataclass
        default"). The funnel must not collapse them.
        """
        s = State.from_init_kwargs(**_REQUIRED_KWARGS, bridge=False)
        assert s.bridge is False


class TestAssetRootFork:
    """``forward_asset_root_to_core`` controls who calls ``set_asset_root``."""

    def test_default_does_not_forward(self):
        """Default (standard ``__init__`` non-packed) drops ``asset_root``
        because the Rust core constructor already received it.
        """
        s = State.from_init_kwargs(asset_root="/x", **_REQUIRED_KWARGS)
        assert s.asset_root is None

    def test_forward_flag_populates_field(self):
        """Set the flag when this path skipped the Rust core constructor —
        currently packed mode and ``create_embedded``.
        """
        s = State.from_init_kwargs(
            asset_root="/x",
            forward_asset_root_to_core=True,
            **_REQUIRED_KWARGS,
        )
        assert s.asset_root == "/x"

    def test_forward_flag_with_none_asset_root_still_uses_default(self):
        """Flag without a value is a no-op (we never write ``None`` over
        a default).
        """
        s = State.from_init_kwargs(
            asset_root=None,
            forward_asset_root_to_core=True,
            **_REQUIRED_KWARGS,
        )
        assert s.asset_root is None


class TestStrictMode:
    """Strict mode catches typos in kwargs at the funnel boundary."""

    def test_strict_default_raises_on_unknown_key(self):
        # Common typo: ``parent_hwnd`` instead of ``parent``. Without
        # strict mode this would silently leave ``state.parent=None``
        # and produce a hard-to-debug "embedded webview never reparented"
        # symptom downstream.
        with pytest.raises(TypeError, match="unexpected keyword argument"):
            State.from_init_kwargs(parent_hwnd=12345, **_REQUIRED_KWARGS)

    def test_strict_error_lists_offending_keys(self):
        with pytest.raises(TypeError) as exc_info:
            State.from_init_kwargs(typo_one="x", typo_two="y", **_REQUIRED_KWARGS)
        msg = str(exc_info.value)
        assert "typo_one" in msg
        assert "typo_two" in msg

    def test_non_strict_silently_drops_unknown(self):
        s = State.from_init_kwargs(
            strict=False,
            unknown_key="dropped",
            another_typo=42,
            **_REQUIRED_KWARGS,
        )
        # The valid required fields still made it through.
        assert s.title == "t"


class TestEmbeddedPath:
    """End-to-end shape used by ``WebView.create_embedded``."""

    def test_embedded_full_call(self):
        s = State.from_init_kwargs(
            asset_root="/embed",
            forward_asset_root_to_core=True,
            title="emb",
            width=400,
            height=300,
            url="https://x.com",
            html=None,
            debug=False,
            resizable=True,
            frame=False,
            parent=12345,
            mode="child",
            auto_show=False,
        )
        assert s.asset_root == "/embed"
        assert s.parent == 12345
        assert s.mode == "child"
        assert s.url == "https://x.com"
        assert s.auto_show is False
        # Optional fields not passed → dataclass defaults.
        assert s.always_on_top is False
        assert s.dcc_mode == "auto"


class TestRequiredFieldInvariant:
    """Pin the assumption that ``from_init_kwargs`` relies on.

    The None-sentinel rule only applies to fields with a dataclass
    default. If a future change adds a default to one of the required
    ``Optional[X]`` fields (e.g. ``debug = False``), the rule would
    silently change meaning for that field — ``debug=None`` would
    fall through to the default instead of staying ``None``.

    This test fails loudly when that assumption breaks so the author
    can decide whether the funnel logic still does the right thing.
    """

    def test_required_optional_fields_have_no_dataclass_default(self):
        from dataclasses import MISSING, fields

        # Fields whose type is ``Optional[X]`` and which the funnel
        # treats as required (None is a real value, not a sentinel).
        expected_required = {"url", "html", "parent", "mode"}
        defaults = {f.name: f.default for f in fields(State)}

        for name in expected_required:
            assert name in defaults, f"{name} disappeared from _WebViewInitState"
            assert defaults[name] is MISSING, (
                f"_WebViewInitState.{name} gained a dataclass default "
                f"({defaults[name]!r}). The None-sentinel rule in "
                f"from_init_kwargs treats fields-with-defaults specially; "
                f"adding a default to {name!r} silently changes the meaning "
                f"of `{name}=None` for every caller. Either revert the "
                f"default or update from_init_kwargs and this test."
            )
