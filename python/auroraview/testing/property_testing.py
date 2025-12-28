# -*- coding: utf-8 -*-
"""Property-based testing support for AuroraView.

This module provides Hypothesis strategies and utilities for property-based
testing of AuroraView components.

Property-based testing generates random test inputs to find edge cases that
manual test cases might miss.

Example:
    >>> from hypothesis import given
    >>> from auroraview.testing.property_testing import (
    ...     html_elements, js_values, event_names
    ... )
    >>>
    >>> @given(html=html_elements())
    >>> def test_html_parsing(html):
    ...     # Test with randomly generated HTML
    ...     assert "<" in html and ">" in html
    >>>
    >>> @given(value=js_values())
    >>> def test_json_roundtrip(value):
    ...     import json
    ...     assert json.loads(json.dumps(value)) == value

Requirements:
    pip install hypothesis
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

try:
    from hypothesis import HealthCheck, assume, given, settings
    from hypothesis import strategies as st

    HYPOTHESIS_AVAILABLE = True
except ImportError:
    HYPOTHESIS_AVAILABLE = False
    st = None
    assume = None
    given = None
    settings = None
    HealthCheck = None


def _check_hypothesis():
    """Check if hypothesis is available."""
    if not HYPOTHESIS_AVAILABLE:
        raise ImportError(
            "hypothesis is required for property-based testing. "
            "Install with: pip install hypothesis"
        )


# ─────────────────────────────────────────────────────────────────────────────
# Hypothesis Strategies
# ─────────────────────────────────────────────────────────────────────────────


def html_tags() -> "st.SearchStrategy[str]":
    """Strategy for valid HTML tag names.

    Returns:
        Hypothesis strategy generating HTML tag names
    """
    _check_hypothesis()
    return st.sampled_from(
        [
            "div",
            "span",
            "p",
            "a",
            "button",
            "input",
            "form",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "ul",
            "ol",
            "li",
            "table",
            "tr",
            "td",
            "th",
            "header",
            "footer",
            "nav",
            "main",
            "section",
            "article",
            "img",
            "video",
            "audio",
            "canvas",
            "svg",
        ]
    )


def css_classes() -> "st.SearchStrategy[str]":
    """Strategy for valid CSS class names.

    Returns:
        Hypothesis strategy generating CSS class names
    """
    _check_hypothesis()
    # CSS class names: start with letter, then letters/digits/hyphens/underscores
    return st.from_regex(r"[a-zA-Z][a-zA-Z0-9_-]{0,20}", fullmatch=True)


def css_ids() -> "st.SearchStrategy[str]":
    """Strategy for valid CSS ID names.

    Returns:
        Hypothesis strategy generating CSS ID names
    """
    _check_hypothesis()
    return st.from_regex(r"[a-zA-Z][a-zA-Z0-9_-]{0,20}", fullmatch=True)


def html_attributes() -> "st.SearchStrategy[Dict[str, str]]":
    """Strategy for HTML attributes dictionary.

    Returns:
        Hypothesis strategy generating attribute dictionaries
    """
    _check_hypothesis()
    attr_names = st.sampled_from(
        [
            "id",
            "class",
            "style",
            "title",
            "data-id",
            "data-test",
            "name",
            "type",
            "value",
            "placeholder",
            "href",
            "src",
        ]
    )
    attr_values = st.text(
        alphabet=st.characters(whitelist_categories=("L", "N", "P", "S")),
        min_size=0,
        max_size=50,
    )
    return st.dictionaries(attr_names, attr_values, max_size=5)


def html_elements(
    max_depth: int = 3,
    max_children: int = 5,
) -> "st.SearchStrategy[str]":
    """Strategy for generating HTML elements.

    Args:
        max_depth: Maximum nesting depth
        max_children: Maximum number of children per element

    Returns:
        Hypothesis strategy generating HTML strings
    """
    _check_hypothesis()

    @st.composite
    def _html_element(draw, depth=0):
        tag = draw(html_tags())
        attrs = draw(html_attributes())

        attr_str = " ".join(f'{k}="{v}"' for k, v in attrs.items())
        if attr_str:
            attr_str = " " + attr_str

        # Self-closing tags
        if tag in ("img", "input", "br", "hr"):
            return f"<{tag}{attr_str} />"

        # Generate content
        if depth >= max_depth:
            content = draw(st.text(min_size=0, max_size=50))
        else:
            # Either text content or child elements
            content_type = draw(st.sampled_from(["text", "children", "mixed"]))
            if content_type == "text":
                content = draw(st.text(min_size=0, max_size=50))
            elif content_type == "children":
                num_children = draw(st.integers(min_value=0, max_value=max_children))
                children = [draw(_html_element(depth + 1)) for _ in range(num_children)]
                content = "\n".join(children)
            else:
                # Mixed: text and children
                text = draw(st.text(min_size=0, max_size=20))
                num_children = draw(st.integers(min_value=0, max_value=2))
                children = [draw(_html_element(depth + 1)) for _ in range(num_children)]
                content = text + "\n".join(children)

        return f"<{tag}{attr_str}>{content}</{tag}>"

    return _html_element()


def js_primitives() -> "st.SearchStrategy[Any]":
    """Strategy for JavaScript primitive values.

    Returns:
        Hypothesis strategy generating JS primitives (string, number, bool, null)
    """
    _check_hypothesis()
    return st.one_of(
        st.text(max_size=100),
        st.integers(min_value=-(2**31), max_value=2**31 - 1),
        st.floats(allow_nan=False, allow_infinity=False),
        st.booleans(),
        st.none(),
    )


def js_values(max_depth: int = 3) -> "st.SearchStrategy[Any]":
    """Strategy for JSON-serializable JavaScript values.

    Args:
        max_depth: Maximum nesting depth for arrays/objects

    Returns:
        Hypothesis strategy generating JSON-serializable values
    """
    _check_hypothesis()

    if max_depth <= 0:
        return js_primitives()

    return st.recursive(
        js_primitives(),
        lambda children: st.one_of(
            st.lists(children, max_size=10),
            st.dictionaries(
                st.text(min_size=1, max_size=20),
                children,
                max_size=10,
            ),
        ),
        max_leaves=50,
    )


def event_names() -> "st.SearchStrategy[str]":
    """Strategy for event names.

    Returns:
        Hypothesis strategy generating valid event names
    """
    _check_hypothesis()
    prefixes = st.sampled_from(
        [
            "click",
            "change",
            "submit",
            "load",
            "error",
            "update",
            "create",
            "delete",
            "fetch",
            "sync",
            "user",
            "data",
            "state",
            "ui",
            "api",
        ]
    )
    suffixes = st.from_regex(r"[a-z0-9_]{0,10}", fullmatch=True)

    @st.composite
    def _event_name(draw):
        prefix = draw(prefixes)
        suffix = draw(suffixes)
        if suffix:
            return f"{prefix}_{suffix}"
        return prefix

    return _event_name()


def namespaced_events() -> "st.SearchStrategy[str]":
    """Strategy for namespaced event names (e.g., 'api:user_login').

    Returns:
        Hypothesis strategy generating namespaced event names
    """
    _check_hypothesis()
    namespaces = st.sampled_from(["api", "ui", "data", "system", "user", ""])

    @st.composite
    def _namespaced_event(draw):
        namespace = draw(namespaces)
        event = draw(event_names())
        if namespace:
            return f"{namespace}:{event}"
        return event

    return _namespaced_event()


def api_methods() -> "st.SearchStrategy[str]":
    """Strategy for API method names (e.g., 'api.get_user').

    Returns:
        Hypothesis strategy generating API method names
    """
    _check_hypothesis()
    namespaces = st.sampled_from(["api", "tool", "data", "ui", "system"])
    verbs = st.sampled_from(
        [
            "get",
            "set",
            "create",
            "update",
            "delete",
            "fetch",
            "sync",
            "validate",
            "process",
            "handle",
        ]
    )
    nouns = st.sampled_from(
        [
            "user",
            "data",
            "config",
            "state",
            "item",
            "list",
            "result",
            "info",
            "settings",
            "options",
        ]
    )

    @st.composite
    def _api_method(draw):
        namespace = draw(namespaces)
        verb = draw(verbs)
        noun = draw(nouns)
        return f"{namespace}.{verb}_{noun}"

    return _api_method()


def css_selectors() -> "st.SearchStrategy[str]":
    """Strategy for CSS selectors.

    Returns:
        Hypothesis strategy generating CSS selectors
    """
    _check_hypothesis()

    @st.composite
    def _css_selector(draw):
        selector_type = draw(st.sampled_from(["id", "class", "tag", "attr", "combined"]))

        if selector_type == "id":
            return f"#{draw(css_ids())}"
        elif selector_type == "class":
            return f".{draw(css_classes())}"
        elif selector_type == "tag":
            return draw(html_tags())
        elif selector_type == "attr":
            attr = draw(st.sampled_from(["data-id", "data-test", "name", "type"]))
            value = draw(
                st.text(min_size=1, max_size=10, alphabet="abcdefghijklmnopqrstuvwxyz0123456789")
            )
            return f'[{attr}="{value}"]'
        else:
            # Combined selector
            tag = draw(html_tags())
            cls = draw(css_classes())
            return f"{tag}.{cls}"

    return _css_selector()


def urls(
    schemes: Optional[List[str]] = None,
) -> "st.SearchStrategy[str]":
    """Strategy for URLs.

    Args:
        schemes: Allowed URL schemes (default: http, https)

    Returns:
        Hypothesis strategy generating URLs
    """
    _check_hypothesis()

    if schemes is None:
        schemes = ["http", "https"]

    @st.composite
    def _url(draw):
        scheme = draw(st.sampled_from(schemes))
        domain = draw(st.from_regex(r"[a-z]{3,10}\.(com|org|net|io)", fullmatch=True))
        path_parts = draw(
            st.lists(
                st.from_regex(r"[a-z0-9_-]{1,10}", fullmatch=True),
                max_size=3,
            )
        )
        path = "/" + "/".join(path_parts) if path_parts else ""
        return f"{scheme}://{domain}{path}"

    return _url()


def file_urls() -> "st.SearchStrategy[str]":
    """Strategy for file:// URLs.

    Returns:
        Hypothesis strategy generating file URLs
    """
    _check_hypothesis()

    @st.composite
    def _file_url(draw):
        path_parts = draw(
            st.lists(
                st.from_regex(r"[a-z0-9_-]{1,10}", fullmatch=True),
                min_size=1,
                max_size=5,
            )
        )
        extension = draw(st.sampled_from(["html", "htm", "js", "css", "json"]))
        path = "/".join(path_parts)
        filename = draw(st.from_regex(r"[a-z0-9_-]{1,10}", fullmatch=True))
        return f"file:///tmp/{path}/{filename}.{extension}"

    return _file_url()


# ─────────────────────────────────────────────────────────────────────────────
# Test Helpers
# ─────────────────────────────────────────────────────────────────────────────


def property_test(
    max_examples: int = 100,
    deadline: Optional[int] = None,
    suppress_health_check: Optional[List] = None,
):
    """Decorator for property-based tests with common settings.

    Args:
        max_examples: Maximum number of examples to generate
        deadline: Deadline in milliseconds (None to disable)
        suppress_health_check: List of health checks to suppress

    Returns:
        Test decorator

    Example:
        >>> @property_test(max_examples=50)
        >>> @given(value=js_values())
        >>> def test_json_roundtrip(value):
        ...     import json
        ...     assert json.loads(json.dumps(value)) == value
    """
    _check_hypothesis()

    if suppress_health_check is None:
        suppress_health_check = [HealthCheck.too_slow]

    return settings(
        max_examples=max_examples,
        deadline=deadline,
        suppress_health_check=suppress_health_check,
    )


# Re-export hypothesis utilities if available
if HYPOTHESIS_AVAILABLE:
    __all__ = [
        # Strategies
        "html_tags",
        "css_classes",
        "css_ids",
        "html_attributes",
        "html_elements",
        "js_primitives",
        "js_values",
        "event_names",
        "namespaced_events",
        "api_methods",
        "css_selectors",
        "urls",
        "file_urls",
        # Helpers
        "property_test",
        # Re-exports from hypothesis
        "st",
        "given",
        "assume",
        "settings",
        "HealthCheck",
        "HYPOTHESIS_AVAILABLE",
    ]
else:
    __all__ = ["HYPOTHESIS_AVAILABLE"]
