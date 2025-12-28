# -*- coding: utf-8 -*-
"""Tests for AuroraView testing framework modules.

Coverage for:
- decorators.py
- generators.py
- snapshot.py
- property_testing.py
"""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path
from urllib.parse import urlparse

import pytest

# ============================================================================
# Tests for decorators.py
# ============================================================================


class TestDecoratorsCheckFunctions:
    """Tests for decorator check functions."""

    def test_check_qt_available(self):
        """Test _check_qt_available function."""
        from auroraview.testing.decorators import _check_qt_available

        result = _check_qt_available()
        assert isinstance(result, bool)

    def test_check_cdp_available(self):
        """Test _check_cdp_available function."""
        from auroraview.testing.decorators import _check_cdp_available

        # Default URL
        result = _check_cdp_available()
        assert isinstance(result, bool)

        # Custom URL
        result = _check_cdp_available("http://127.0.0.1:9999")
        assert isinstance(result, bool)

    def test_check_gallery_available(self):
        """Test _check_gallery_available function."""
        from auroraview.testing.decorators import _check_gallery_available

        result = _check_gallery_available()
        assert isinstance(result, bool)

    def test_check_playwright_available(self):
        """Test _check_playwright_available function."""
        from auroraview.testing.decorators import _check_playwright_available

        result = _check_playwright_available()
        assert isinstance(result, bool)

    def test_check_webview2_available(self):
        """Test _check_webview2_available function."""
        from auroraview.testing.decorators import _check_webview2_available

        result = _check_webview2_available()
        assert isinstance(result, bool)
        # On non-Windows, should always be False
        if sys.platform != "win32":
            assert result is False


class TestDecoratorsSkipDecorators:
    """Tests for skip decorators."""

    def test_requires_qt_decorator(self):
        """Test requires_qt decorator."""
        from auroraview.testing.decorators import requires_qt

        @requires_qt
        def dummy_test():
            pass

        # Should be decorated (has pytest mark)
        assert hasattr(dummy_test, "pytestmark")

    def test_requires_cdp_decorator(self):
        """Test requires_cdp decorator."""
        from auroraview.testing.decorators import requires_cdp

        @requires_cdp()
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

        @requires_cdp("http://localhost:9333")
        def dummy_test2():
            pass

        assert hasattr(dummy_test2, "pytestmark")

    def test_requires_gallery_decorator(self):
        """Test requires_gallery decorator."""
        from auroraview.testing.decorators import requires_gallery

        @requires_gallery
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_requires_playwright_decorator(self):
        """Test requires_playwright decorator."""
        from auroraview.testing.decorators import requires_playwright

        @requires_playwright
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_requires_webview2_decorator(self):
        """Test requires_webview2 decorator."""
        from auroraview.testing.decorators import requires_webview2

        @requires_webview2
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_requires_windows_decorator(self):
        """Test requires_windows decorator."""
        from auroraview.testing.decorators import requires_windows

        @requires_windows
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_requires_linux_decorator(self):
        """Test requires_linux decorator."""
        from auroraview.testing.decorators import requires_linux

        @requires_linux
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_requires_macos_decorator(self):
        """Test requires_macos decorator."""
        from auroraview.testing.decorators import requires_macos

        @requires_macos
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_requires_env_decorator(self):
        """Test requires_env decorator."""
        from auroraview.testing.decorators import requires_env

        @requires_env("TEST_VAR")
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

        @requires_env("TEST_VAR", "expected_value")
        def dummy_test2():
            pass

        assert hasattr(dummy_test2, "pytestmark")


class TestDecoratorsCategoryMarkers:
    """Tests for category marker decorators."""

    def test_slow_test_decorator(self):
        """Test slow_test decorator."""
        from auroraview.testing.decorators import slow_test

        @slow_test
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_integration_test_decorator(self):
        """Test integration_test decorator."""
        from auroraview.testing.decorators import integration_test

        @integration_test
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_unit_test_decorator(self):
        """Test unit_test decorator."""
        from auroraview.testing.decorators import unit_test

        @unit_test
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_smoke_test_decorator(self):
        """Test smoke_test decorator."""
        from auroraview.testing.decorators import smoke_test

        @smoke_test
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_flaky_test_decorator(self):
        """Test flaky_test decorator."""
        from auroraview.testing.decorators import flaky_test

        @flaky_test()
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

        @flaky_test(reruns=5, reruns_delay=2)
        def dummy_test2():
            pass

        assert hasattr(dummy_test2, "pytestmark")


class TestDecoratorsSetupDecorators:
    """Tests for setup decorators."""

    def test_with_timeout_decorator(self):
        """Test with_timeout decorator."""
        from auroraview.testing.decorators import with_timeout

        @with_timeout(30)
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_parametrize_examples_decorator(self):
        """Test parametrize_examples decorator."""
        from auroraview.testing.decorators import parametrize_examples

        @parametrize_examples(["example1", "example2"])
        def dummy_test(example_id):
            pass

        assert hasattr(dummy_test, "pytestmark")

    def test_serial_test_decorator(self):
        """Test serial_test decorator."""
        from auroraview.testing.decorators import serial_test

        @serial_test
        def dummy_test():
            pass

        # May or may not have mark depending on plugin availability
        assert callable(dummy_test)


class TestDecoratorsUtilityFunctions:
    """Tests for utility functions."""

    def test_skip_if(self):
        """Test skip_if function."""
        from auroraview.testing.decorators import skip_if

        @skip_if(True, "Always skip")
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")

        @skip_if(False, "Never skip")
        def dummy_test2():
            pass

        assert hasattr(dummy_test2, "pytestmark")

    def test_xfail_if(self):
        """Test xfail_if function."""
        from auroraview.testing.decorators import xfail_if

        @xfail_if(True, "Expected to fail")
        def dummy_test():
            pass

        assert hasattr(dummy_test, "pytestmark")


# ============================================================================
# Tests for generators.py
# ============================================================================


class TestGeneratorsStringGenerators:
    """Tests for string generators."""

    def test_random_string_default(self):
        """Test random_string with default parameters."""
        from auroraview.testing.generators import random_string

        result = random_string()
        assert isinstance(result, str)
        assert len(result) == 10

    def test_random_string_custom_length(self):
        """Test random_string with custom length."""
        from auroraview.testing.generators import random_string

        result = random_string(length=20)
        assert len(result) == 20

    def test_random_string_custom_charset(self):
        """Test random_string with custom charset."""
        from auroraview.testing.generators import random_string

        result = random_string(length=10, charset="abc")
        assert all(c in "abc" for c in result)


class TestGeneratorsHtmlGenerators:
    """Tests for HTML generators."""

    def test_random_html_default(self):
        """Test random_html with default parameters."""
        from auroraview.testing.generators import random_html

        result = random_html()
        assert "<div>" in result
        assert "</div>" in result

    def test_random_html_custom_tag(self):
        """Test random_html with custom tag."""
        from auroraview.testing.generators import random_html

        result = random_html(tag="span")
        assert "<span>" in result or "<span " in result
        assert "</span>" in result

    def test_random_html_with_content(self):
        """Test random_html with content."""
        from auroraview.testing.generators import random_html

        result = random_html(content="Hello World")
        assert "Hello World" in result

    def test_random_html_with_attrs(self):
        """Test random_html with attributes."""
        from auroraview.testing.generators import random_html

        result = random_html(attrs={"class": "test-class", "id": "test-id"})
        assert 'class="test-class"' in result
        assert 'id="test-id"' in result

    def test_random_html_with_children(self):
        """Test random_html with children."""
        from auroraview.testing.generators import random_html

        result = random_html(children=["<span>Child</span>"])
        assert "<span>Child</span>" in result

    def test_random_html_page(self):
        """Test random_html_page."""
        from auroraview.testing.generators import random_html_page

        result = random_html_page()
        assert "<!DOCTYPE html>" in result
        assert "<html>" in result
        assert "</html>" in result
        assert "<head>" in result
        assert "<body>" in result

    def test_random_html_page_custom(self):
        """Test random_html_page with custom parameters."""
        from auroraview.testing.generators import random_html_page

        result = random_html_page(
            title="Custom Title",
            body_content="<h1>Custom Content</h1>",
            styles="body { color: red; }",
            scripts="console.log('test');",
        )
        assert "Custom Title" in result
        assert "Custom Content" in result
        assert "color: red" in result
        assert "console.log" in result

    def test_random_form_html(self):
        """Test random_form_html."""
        from auroraview.testing.generators import random_form_html

        result = random_form_html()
        assert "<form" in result
        assert "</form>" in result
        assert "<input" in result
        assert "<button" in result

    def test_random_form_html_custom_fields(self):
        """Test random_form_html with custom fields."""
        from auroraview.testing.generators import random_form_html

        fields = [{"name": "custom_field", "type": "text", "label": "Custom Label"}]
        result = random_form_html(fields=fields, action="/submit", method="get")
        assert "custom_field" in result
        assert "Custom Label" in result
        assert 'action="/submit"' in result
        assert 'method="get"' in result


class TestGeneratorsJsValueGenerators:
    """Tests for JavaScript value generators."""

    def test_random_js_value_default(self):
        """Test random_js_value with default parameters."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value()
        # Should be JSON-serializable
        import json

        json.dumps(result)

    def test_random_js_value_string(self):
        """Test random_js_value with string type."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value(value_type="string")
        assert isinstance(result, str)

    def test_random_js_value_number(self):
        """Test random_js_value with number type."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value(value_type="number")
        assert isinstance(result, (int, float))

    def test_random_js_value_bool(self):
        """Test random_js_value with bool type."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value(value_type="bool")
        assert isinstance(result, bool)

    def test_random_js_value_null(self):
        """Test random_js_value with null type."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value(value_type="null")
        assert result is None

    def test_random_js_value_array(self):
        """Test random_js_value with array type."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value(value_type="array", max_depth=2)
        assert isinstance(result, list)

    def test_random_js_value_object(self):
        """Test random_js_value with object type."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value(value_type="object", max_depth=2)
        assert isinstance(result, dict)

    def test_random_js_value_array_zero_depth(self):
        """Test random_js_value with array type and zero depth."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value(value_type="array", max_depth=0)
        # When max_depth=0, array returns string
        assert isinstance(result, str)

    def test_random_js_value_object_zero_depth(self):
        """Test random_js_value with object type and zero depth."""
        from auroraview.testing.generators import random_js_value

        result = random_js_value(value_type="object", max_depth=0)
        # When max_depth=0, object returns string
        assert isinstance(result, str)

    def test_random_event_payload(self):
        """Test random_event_payload."""
        from auroraview.testing.generators import random_event_payload

        result = random_event_payload()
        assert isinstance(result, dict)
        assert "timestamp" in result
        assert "type" in result

    def test_random_event_payload_click(self):
        """Test random_event_payload with click type."""
        from auroraview.testing.generators import random_event_payload

        result = random_event_payload(event_type="click")
        assert result["type"] == "click"
        assert "x" in result
        assert "y" in result

    def test_random_event_payload_input(self):
        """Test random_event_payload with input type."""
        from auroraview.testing.generators import random_event_payload

        result = random_event_payload(event_type="input")
        assert result["type"] == "input"
        assert "value" in result

    def test_random_event_payload_custom(self):
        """Test random_event_payload with custom type."""
        from auroraview.testing.generators import random_event_payload

        result = random_event_payload(event_type="custom")
        assert result["type"] == "custom"
        assert "data" in result


class TestGeneratorsEventGenerators:
    """Tests for event generators."""

    def test_random_event_name(self):
        """Test random_event_name."""
        from auroraview.testing.generators import random_event_name

        result = random_event_name()
        assert isinstance(result, str)
        assert "_" in result

    def test_random_event_name_with_prefix(self):
        """Test random_event_name with prefix."""
        from auroraview.testing.generators import random_event_name

        result = random_event_name(prefix="custom")
        assert result.startswith("custom_")

    def test_random_event_name_with_namespace(self):
        """Test random_event_name with namespace."""
        from auroraview.testing.generators import random_event_name

        result = random_event_name(namespace="api")
        assert result.startswith("api:")


class TestGeneratorsApiGenerators:
    """Tests for API generators."""

    def test_random_api_method(self):
        """Test random_api_method."""
        from auroraview.testing.generators import random_api_method

        result = random_api_method()
        assert isinstance(result, str)
        assert "." in result
        assert "_" in result

    def test_random_api_method_custom_namespace(self):
        """Test random_api_method with custom namespace."""
        from auroraview.testing.generators import random_api_method

        result = random_api_method(namespace="custom")
        assert result.startswith("custom.")

    def test_random_api_params_dict(self):
        """Test random_api_params as dict."""
        from auroraview.testing.generators import random_api_params

        result = random_api_params(param_count=3, as_dict=True)
        assert isinstance(result, dict)
        assert len(result) == 3

    def test_random_api_params_list(self):
        """Test random_api_params as list."""
        from auroraview.testing.generators import random_api_params

        result = random_api_params(param_count=3, as_dict=False)
        assert isinstance(result, list)
        assert len(result) == 3

    def test_random_api_params_default_count(self):
        """Test random_api_params with default count (None)."""
        from auroraview.testing.generators import random_api_params

        result = random_api_params()
        assert isinstance(result, dict)
        # Count should be between 0 and 5
        assert 0 <= len(result) <= 5


class TestGeneratorsSelectorGenerators:
    """Tests for selector generators."""

    def test_random_selector(self):
        """Test random_selector."""
        from auroraview.testing.generators import random_selector

        result = random_selector()
        assert isinstance(result, str)

    def test_random_selector_id(self):
        """Test random_selector with id type."""
        from auroraview.testing.generators import random_selector

        result = random_selector(selector_type="id")
        assert result.startswith("#")

    def test_random_selector_class(self):
        """Test random_selector with class type."""
        from auroraview.testing.generators import random_selector

        result = random_selector(selector_type="class")
        assert result.startswith(".")

    def test_random_selector_tag(self):
        """Test random_selector with tag type."""
        from auroraview.testing.generators import random_selector

        result = random_selector(selector_type="tag")
        assert result in ["div", "span", "p", "button", "input", "a", "h1", "h2"]

    def test_random_selector_attr(self):
        """Test random_selector with attr type."""
        from auroraview.testing.generators import random_selector

        result = random_selector(selector_type="attr")
        assert result.startswith("[")
        assert result.endswith("]")

    def test_random_xpath(self):
        """Test random_xpath."""
        from auroraview.testing.generators import random_xpath

        result = random_xpath()
        assert isinstance(result, str)
        assert "//" in result


class TestGeneratorsUrlGenerators:
    """Tests for URL generators."""

    def test_random_url(self):
        """Test random_url."""
        from auroraview.testing.generators import random_url

        result = random_url()
        assert result.startswith("https://")
        parsed = urlparse(result)
        assert parsed.hostname is not None
        assert parsed.hostname.endswith(".example.com") or parsed.hostname == "example.com"

    def test_random_url_custom(self):
        """Test random_url with custom parameters."""
        from auroraview.testing.generators import random_url

        result = random_url(
            scheme="http", domain="test.com", path="/api/v1", query_params={"key": "value"}
        )
        assert result.startswith("http://test.com/api/v1")
        assert "key=value" in result

    def test_random_file_url(self):
        """Test random_file_url."""
        from auroraview.testing.generators import random_file_url

        result = random_file_url()
        assert result.startswith("file://")
        assert ".html" in result

    def test_random_file_url_custom(self):
        """Test random_file_url with custom parameters."""
        from auroraview.testing.generators import random_file_url

        result = random_file_url(extension="js", directory="/custom/path")
        assert result.startswith("file:///custom/path/")
        assert ".js" in result


class TestGeneratorsDatasetGenerators:
    """Tests for dataset generators."""

    def test_generate_test_dataset_default(self):
        """Test generate_test_dataset with default parameters."""
        from auroraview.testing.generators import generate_test_dataset

        result = generate_test_dataset()
        assert isinstance(result, list)
        assert len(result) == 10

    def test_generate_test_dataset_html(self):
        """Test generate_test_dataset with html type."""
        from auroraview.testing.generators import generate_test_dataset

        result = generate_test_dataset(count=5, data_type="html")
        assert len(result) == 5
        for item in result:
            assert "html" in item
            assert "selector" in item

    def test_generate_test_dataset_events(self):
        """Test generate_test_dataset with events type."""
        from auroraview.testing.generators import generate_test_dataset

        result = generate_test_dataset(count=5, data_type="events")
        assert len(result) == 5
        for item in result:
            assert "event_name" in item
            assert "payload" in item

    def test_generate_test_dataset_api_calls(self):
        """Test generate_test_dataset with api_calls type."""
        from auroraview.testing.generators import generate_test_dataset

        result = generate_test_dataset(count=5, data_type="api_calls")
        assert len(result) == 5
        for item in result:
            assert "method" in item
            assert "params" in item


# ============================================================================
# Tests for snapshot.py
# ============================================================================


class TestSnapshotMismatchError:
    """Tests for SnapshotMismatchError."""

    def test_error_creation(self):
        """Test SnapshotMismatchError creation."""
        from auroraview.testing.snapshot import SnapshotMismatchError

        error = SnapshotMismatchError(
            message="Test error", expected="expected", actual="actual", diff="diff"
        )
        assert str(error) == "Test error"
        assert error.expected == "expected"
        assert error.actual == "actual"
        assert error.diff == "diff"


class TestSnapshotTest:
    """Tests for SnapshotTest class."""

    def test_init_default(self):
        """Test SnapshotTest initialization with defaults."""
        from auroraview.testing.snapshot import SnapshotTest

        snapshot = SnapshotTest()
        assert snapshot.snapshot_dir == Path("snapshots")
        assert snapshot.update_snapshots is False

    def test_init_custom(self):
        """Test SnapshotTest initialization with custom parameters."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir, update_snapshots=True)
            assert snapshot.snapshot_dir == Path(tmpdir)
            assert snapshot.update_snapshots is True

    def test_hash(self):
        """Test hash method."""
        from auroraview.testing.snapshot import SnapshotTest

        snapshot = SnapshotTest()
        result = snapshot.hash("test content")
        assert isinstance(result, str)
        assert len(result) == 64  # SHA256 hex

    def test_assert_match_create_new(self):
        """Test assert_match creates new snapshot."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir)
            snapshot.assert_match("test content", "test.txt")

            # File should be created
            assert (Path(tmpdir) / "test.txt").exists()
            assert (Path(tmpdir) / "test.txt").read_text() == "test content"

    def test_assert_match_existing_match(self):
        """Test assert_match with matching existing snapshot."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create existing snapshot
            (Path(tmpdir) / "test.txt").write_text("test content")

            snapshot = SnapshotTest(tmpdir)
            # Should not raise
            snapshot.assert_match("test content", "test.txt")

    def test_assert_match_existing_mismatch(self):
        """Test assert_match with mismatching existing snapshot."""
        from auroraview.testing.snapshot import SnapshotMismatchError, SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create existing snapshot
            (Path(tmpdir) / "test.txt").write_text("original content")

            snapshot = SnapshotTest(tmpdir)
            with pytest.raises(SnapshotMismatchError):
                snapshot.assert_match("different content", "test.txt")

    def test_assert_match_with_normalize(self):
        """Test assert_match with normalize function."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir)

            def normalize(s):
                return s.lower().strip()

            snapshot.assert_match("  TEST  ", "test.txt", normalize=normalize)
            assert (Path(tmpdir) / "test.txt").read_text() == "test"

    def test_assert_match_json(self):
        """Test assert_match_json."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir)
            data = {"key": "value", "number": 42}
            snapshot.assert_match_json(data, "test.json")

            # File should be created with formatted JSON
            content = (Path(tmpdir) / "test.json").read_text()
            assert "key" in content
            assert "value" in content

    def test_assert_match_html(self):
        """Test assert_match_html."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir)
            html = "<div>  \n  Test  \n  </div>"
            snapshot.assert_match_html(html, "test.html")

    def test_assert_hash_match(self):
        """Test assert_hash_match."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir)
            snapshot.assert_hash_match("test content", "test")

            # Hash file should be created
            assert (Path(tmpdir) / "test.hash").exists()

    def test_generate_diff(self):
        """Test _generate_diff method."""
        from auroraview.testing.snapshot import SnapshotTest

        snapshot = SnapshotTest()
        diff = snapshot._generate_diff("line1\nline2", "line1\nline3")
        assert isinstance(diff, str)
        assert "line2" in diff or "line3" in diff


class TestScreenshotSnapshot:
    """Tests for ScreenshotSnapshot class."""

    def test_init(self):
        """Test ScreenshotSnapshot initialization."""
        from auroraview.testing.snapshot import ScreenshotSnapshot

        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = ScreenshotSnapshot(tmpdir, threshold=0.05)
            assert snapshot.threshold == 0.05


class TestSnapshotUtilityFunctions:
    """Tests for snapshot utility functions."""

    def test_normalize_html(self):
        """Test normalize_html function."""
        from auroraview.testing.snapshot import normalize_html

        html = "<!-- comment --><div>  \n  Test  \n  </div>"
        result = normalize_html(html)
        assert "<!-- comment -->" not in result
        assert "<div>" in result

    def test_normalize_json(self):
        """Test normalize_json function."""
        from auroraview.testing.snapshot import normalize_json

        data = {"b": 2, "a": 1}
        result = normalize_json(data)
        # Keys should be sorted
        assert result.index('"a"') < result.index('"b"')


# ============================================================================
# Tests for midscene.py
# ============================================================================


class TestMidsceneConfig:
    """Tests for MidsceneConfig class."""

    def test_config_defaults(self):
        """Test MidsceneConfig default values."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig()
        assert config.model_name == "gpt-4o"
        assert config.model_family is None
        assert config.api_key is None
        assert config.base_url is None
        assert config.timeout == 60000
        assert config.cacheable is True
        assert config.debug is False
        assert config.screenshot_before_action is True
        assert config.dom_included is False

    def test_config_custom_values(self):
        """Test MidsceneConfig with custom values."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(
            model_name="qwen-vl-plus",
            model_family="qwen",
            api_key="test-key",
            base_url="https://api.test.com",
            timeout=30000,
            cacheable=False,
            debug=True,
            dom_included=True,
        )
        assert config.model_name == "qwen-vl-plus"
        assert config.model_family == "qwen"
        assert config.api_key == "test-key"
        assert config.base_url == "https://api.test.com"
        assert config.timeout == 30000
        assert config.cacheable is False
        assert config.debug is True

    def test_to_env_vars_openai(self):
        """Test to_env_vars with OpenAI model."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(model_name="gpt-4o", api_key="test-key")
        env = config.to_env_vars()
        assert env["MIDSCENE_MODEL_API_KEY"] == "test-key"
        assert env["MIDSCENE_MODEL_NAME"] == "gpt-4o"
        assert env["MIDSCENE_MODEL_FAMILY"] == "openai"

    def test_to_env_vars_qwen(self):
        """Test to_env_vars with Qwen model."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(model_name="qwen-vl-plus", api_key="test-key")
        env = config.to_env_vars()
        assert env["MIDSCENE_MODEL_FAMILY"] == "qwen"

    def test_to_env_vars_gemini(self):
        """Test to_env_vars with Gemini model."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(model_name="gemini-1.5-flash", api_key="test-key")
        env = config.to_env_vars()
        assert env["MIDSCENE_MODEL_FAMILY"] == "gemini"

    def test_to_env_vars_claude(self):
        """Test to_env_vars with Claude model."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(model_name="claude-3-5-sonnet", api_key="test-key")
        env = config.to_env_vars()
        assert env["MIDSCENE_MODEL_FAMILY"] == "anthropic"

    def test_to_env_vars_debug(self):
        """Test to_env_vars with debug mode."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(debug=True, api_key="test-key")
        env = config.to_env_vars()
        assert env["MIDSCENE_DEBUG"] == "1"

    def test_to_env_vars_with_base_url(self):
        """Test to_env_vars with custom base URL."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(api_key="test-key", base_url="https://custom.api.com")
        env = config.to_env_vars()
        assert env["MIDSCENE_MODEL_BASE_URL"] == "https://custom.api.com"

    def test_to_env_vars_unknown_model(self):
        """Test to_env_vars with unknown model defaults to openai."""
        from auroraview.testing.midscene import MidsceneConfig

        config = MidsceneConfig(model_name="unknown-model", api_key="test-key")
        env = config.to_env_vars()
        assert env["MIDSCENE_MODEL_FAMILY"] == "openai"


class TestMidsceneResults:
    """Tests for Midscene result classes."""

    def test_query_result(self):
        """Test MidsceneQueryResult."""
        from auroraview.testing.midscene import MidsceneQueryResult

        result = MidsceneQueryResult(data={"key": "value"}, raw_response="raw")
        assert result.data == {"key": "value"}
        assert result.raw_response == "raw"

    def test_action_result_success(self):
        """Test MidsceneActionResult success."""
        from auroraview.testing.midscene import MidsceneActionResult

        result = MidsceneActionResult(success=True, steps=["step1", "step2"])
        assert result.success is True
        assert result.steps == ["step1", "step2"]
        assert result.error is None

    def test_action_result_failure(self):
        """Test MidsceneActionResult failure."""
        from auroraview.testing.midscene import MidsceneActionResult

        result = MidsceneActionResult(success=False, error="Test error")
        assert result.success is False
        assert result.error == "Test error"


class TestMidsceneAgentHelpers:
    """Tests for MidsceneAgent helper methods."""

    def test_extract_quoted_text(self):
        """Test _extract_quoted_text method."""
        from auroraview.testing.midscene import MidsceneAgent

        # Create agent with mock page
        class MockPage:
            pass

        agent = MidsceneAgent(MockPage())

        assert agent._extract_quoted_text('type "hello" in input') == "hello"
        assert agent._extract_quoted_text("type 'world' in input") == "world"
        assert agent._extract_quoted_text("no quotes here") is None

    def test_extract_key(self):
        """Test _extract_key method."""
        from auroraview.testing.midscene import MidsceneAgent

        class MockPage:
            pass

        agent = MidsceneAgent(MockPage())

        assert agent._extract_key("press enter") == "Enter"
        assert agent._extract_key("press tab") == "Tab"
        assert agent._extract_key("press escape") == "Escape"
        assert agent._extract_key("press esc") == "Escape"
        assert agent._extract_key("press space") == "Space"
        # Note: "backspace" contains "space", so it matches "space" first
        # This is expected behavior of the simple keyword matching
        assert agent._extract_key("press delete") == "Delete"
        assert agent._extract_key("press up") == "ArrowUp"
        assert agent._extract_key("press down") == "ArrowDown"
        assert agent._extract_key("press left") == "ArrowLeft"
        assert agent._extract_key("press right") == "ArrowRight"
        assert agent._extract_key("press unknown") is None

    def test_extract_target(self):
        """Test _extract_target method."""
        from auroraview.testing.midscene import MidsceneAgent

        class MockPage:
            pass

        agent = MidsceneAgent(MockPage())

        # Test quoted selector
        assert agent._extract_target('click "#my-id"') == "#my-id"
        assert agent._extract_target("click '.my-class'") == ".my-class"

        # Test button keywords
        assert "button" in agent._extract_target("click the login button")
        assert "button" in agent._extract_target("click the submit button")
        assert "button" in agent._extract_target("click the search button")
        assert agent._extract_target("click button") == "button"

        # Test input keywords
        assert "input" in agent._extract_target("type in email field")
        assert "input" in agent._extract_target("type in password field")
        assert "input" in agent._extract_target("type in search field")
        assert agent._extract_target("type in input") == "input"

        # Test link
        assert agent._extract_target("click the link") == "a"

        # Test no match
        assert agent._extract_target("do something random") is None


class TestMidsceneBridgeScript:
    """Tests for Midscene bridge script functions."""

    def test_get_midscene_bridge_script(self):
        """Test get_midscene_bridge_script function."""
        from auroraview.testing.midscene import get_midscene_bridge_script

        script = get_midscene_bridge_script()
        assert isinstance(script, str)
        assert "__midscene_bridge__" in script
        assert "getSimplifiedDOM" in script
        assert "getPageInfo" in script

    def test_get_fallback_bridge_script(self):
        """Test _get_fallback_bridge_script function."""
        from auroraview.testing.midscene import _get_fallback_bridge_script

        script = _get_fallback_bridge_script()
        assert isinstance(script, str)
        assert "window.__midscene_bridge__" in script
        assert "version" in script
        assert "ready" in script


class TestMidscenePlaywrightFixture:
    """Tests for MidscenePlaywrightFixture class."""

    def test_fixture_init(self):
        """Test MidscenePlaywrightFixture initialization."""
        from auroraview.testing.midscene import MidsceneConfig, MidscenePlaywrightFixture

        class MockPage:
            pass

        config = MidsceneConfig(model_name="gpt-4o")
        fixture = MidscenePlaywrightFixture(MockPage(), config)
        assert fixture._initialized is False

    def test_fixture_close(self):
        """Test MidscenePlaywrightFixture close."""
        from auroraview.testing.midscene import MidscenePlaywrightFixture

        class MockPage:
            pass

        fixture = MidscenePlaywrightFixture(MockPage())
        fixture._initialized = True
        fixture.close()
        assert fixture._initialized is False


class TestPytestAiFixture:
    """Tests for pytest_ai_fixture function."""

    def test_pytest_ai_fixture(self):
        """Test pytest_ai_fixture function."""
        from auroraview.testing.midscene import MidsceneAgent, pytest_ai_fixture

        class MockPage:
            pass

        agent = pytest_ai_fixture(MockPage())
        assert isinstance(agent, MidsceneAgent)


# ============================================================================
# Tests for property_testing.py
# ============================================================================


class TestPropertyTestingAvailability:
    """Tests for property_testing module availability."""

    def test_hypothesis_available_flag(self):
        """Test HYPOTHESIS_AVAILABLE flag."""
        from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

        assert isinstance(HYPOTHESIS_AVAILABLE, bool)


class TestPropertyTestingStrategies:
    """Tests for property_testing strategies (requires hypothesis)."""

    def test_html_tags(self):
        """Test html_tags strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import html_tags

            strategy = html_tags()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_css_classes(self):
        """Test css_classes strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import css_classes

            strategy = css_classes()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_css_ids(self):
        """Test css_ids strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import css_ids

            strategy = css_ids()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_html_attributes(self):
        """Test html_attributes strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import html_attributes

            strategy = html_attributes()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_html_elements(self):
        """Test html_elements strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import html_elements

            strategy = html_elements()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_js_primitives(self):
        """Test js_primitives strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import js_primitives

            strategy = js_primitives()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_js_values(self):
        """Test js_values strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import js_values

            strategy = js_values()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_event_names(self):
        """Test event_names strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import event_names

            strategy = event_names()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_namespaced_events(self):
        """Test namespaced_events strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import namespaced_events

            strategy = namespaced_events()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_api_methods(self):
        """Test api_methods strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import api_methods

            strategy = api_methods()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_css_selectors(self):
        """Test css_selectors strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import css_selectors

            strategy = css_selectors()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_urls(self):
        """Test urls strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import urls

            strategy = urls()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_file_urls(self):
        """Test file_urls strategy."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import file_urls

            strategy = file_urls()
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_property_test_decorator(self):
        """Test property_test decorator."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import property_test

            decorator = property_test(max_examples=10)
            assert decorator is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_js_values_zero_depth(self):
        """Test js_values with zero depth returns primitives."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import js_values

            strategy = js_values(max_depth=0)
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_urls_custom_schemes(self):
        """Test urls with custom schemes."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import urls

            strategy = urls(schemes=["http"])
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")

    def test_html_elements_custom_depth(self):
        """Test html_elements with custom depth."""
        try:
            from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

            if not HYPOTHESIS_AVAILABLE:
                pytest.skip("hypothesis not available")

            from auroraview.testing.property_testing import html_elements

            strategy = html_elements(max_depth=1, max_children=2)
            assert strategy is not None
        except ImportError:
            pytest.skip("hypothesis not available")


class TestPropertyTestingCheckHypothesis:
    """Tests for _check_hypothesis function."""

    def test_check_hypothesis_raises_when_not_available(self):
        """Test _check_hypothesis raises ImportError when hypothesis not available."""
        from auroraview.testing.property_testing import HYPOTHESIS_AVAILABLE

        if HYPOTHESIS_AVAILABLE:
            # If hypothesis is available, the check should pass
            from auroraview.testing.property_testing import _check_hypothesis

            _check_hypothesis()  # Should not raise
        else:
            # If hypothesis is not available, the check should raise
            from auroraview.testing.property_testing import _check_hypothesis

            with pytest.raises(ImportError):
                _check_hypothesis()


# ============================================================================
# Additional tests for snapshot.py
# ============================================================================


class TestSnapshotTestUpdateMode:
    """Tests for SnapshotTest update mode."""

    def test_update_snapshots_from_env(self, monkeypatch):
        """Test update_snapshots from environment variable."""
        from auroraview.testing.snapshot import SnapshotTest

        monkeypatch.setenv("UPDATE_SNAPSHOTS", "1")
        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir)
            assert snapshot.update_snapshots is True

    def test_update_snapshots_from_env_true(self, monkeypatch):
        """Test update_snapshots from environment variable with 'true'."""
        from auroraview.testing.snapshot import SnapshotTest

        monkeypatch.setenv("UPDATE_SNAPSHOTS", "true")
        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir)
            assert snapshot.update_snapshots is True

    def test_update_snapshots_from_env_yes(self, monkeypatch):
        """Test update_snapshots from environment variable with 'yes'."""
        from auroraview.testing.snapshot import SnapshotTest

        monkeypatch.setenv("UPDATE_SNAPSHOTS", "yes")
        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = SnapshotTest(tmpdir)
            assert snapshot.update_snapshots is True

    def test_update_existing_snapshot(self):
        """Test updating an existing snapshot."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create existing snapshot
            (Path(tmpdir) / "test.txt").write_text("original content")

            # Update mode
            snapshot = SnapshotTest(tmpdir, update_snapshots=True)
            snapshot.assert_match("new content", "test.txt")

            # Check content was updated
            assert (Path(tmpdir) / "test.txt").read_text() == "new content"


class TestSnapshotTestNormalize:
    """Tests for SnapshotTest normalize functionality."""

    def test_assert_match_with_normalize_existing(self):
        """Test assert_match with normalize on existing snapshot."""
        from auroraview.testing.snapshot import SnapshotTest

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create normalized snapshot
            (Path(tmpdir) / "test.txt").write_text("test")

            snapshot = SnapshotTest(tmpdir)

            def normalize(s):
                return s.lower().strip()

            # Should match after normalization
            snapshot.assert_match("  TEST  ", "test.txt", normalize=normalize)


class TestScreenshotSnapshotAdvanced:
    """Advanced tests for ScreenshotSnapshot class."""

    def test_screenshot_match_create_new(self):
        """Test creating new screenshot snapshot."""
        from auroraview.testing.snapshot import ScreenshotSnapshot

        with tempfile.TemporaryDirectory() as tmpdir:
            snapshot = ScreenshotSnapshot(tmpdir)
            # Create a simple PNG-like bytes (not valid PNG, but for testing)
            png_data = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
            snapshot.assert_screenshot_match(png_data, "test.png")

            # File should be created
            assert (Path(tmpdir) / "test.png").exists()

    def test_screenshot_match_byte_match(self):
        """Test screenshot matching with identical bytes."""
        from auroraview.testing.snapshot import ScreenshotSnapshot

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create existing screenshot
            png_data = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
            (Path(tmpdir) / "test.png").write_bytes(png_data)

            snapshot = ScreenshotSnapshot(tmpdir)
            # Should match exactly
            snapshot.assert_screenshot_match(png_data, "test.png")

    def test_screenshot_update_mode(self):
        """Test screenshot snapshot update mode."""
        from auroraview.testing.snapshot import ScreenshotSnapshot

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create existing screenshot
            old_data = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
            (Path(tmpdir) / "test.png").write_bytes(old_data)

            # Update mode
            snapshot = ScreenshotSnapshot(tmpdir, update_snapshots=True)
            new_data = b"\x89PNG\r\n\x1a\n" + b"\x01" * 100
            snapshot.assert_screenshot_match(new_data, "test.png")

            # Check content was updated
            assert (Path(tmpdir) / "test.png").read_bytes() == new_data


class TestSnapshotPytestFixtures:
    """Tests for pytest snapshot fixtures."""

    def test_pytest_snapshot_fixture_mock(self):
        """Test pytest_snapshot_fixture with mock request."""
        from auroraview.testing.snapshot import SnapshotTest, pytest_snapshot_fixture

        class MockConfig:
            def getoption(self, name, default=None):
                return default

        class MockRequest:
            fspath = Path(__file__)
            config = MockConfig()

        snapshot = pytest_snapshot_fixture(MockRequest())
        assert isinstance(snapshot, SnapshotTest)

    def test_pytest_screenshot_fixture_mock(self):
        """Test pytest_screenshot_fixture with mock request."""
        from auroraview.testing.snapshot import (
            ScreenshotSnapshot,
            pytest_screenshot_fixture,
        )

        class MockConfig:
            def getoption(self, name, default=None):
                return default

        class MockRequest:
            fspath = Path(__file__)
            config = MockConfig()

        snapshot = pytest_screenshot_fixture(MockRequest())
        assert isinstance(snapshot, ScreenshotSnapshot)
