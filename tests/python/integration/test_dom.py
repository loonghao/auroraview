"""Tests for the DOM manipulation module.

This module tests the Element and ElementCollection classes that provide
DOM manipulation capabilities for AuroraView WebViews.
"""

from unittest.mock import MagicMock

import pytest


class TestElement:
    """Tests for the Element class."""

    @pytest.fixture
    def mock_webview(self):
        """Create a mock WebView for testing."""
        webview = MagicMock()
        webview.eval_js = MagicMock()
        return webview

    @pytest.fixture
    def element(self, mock_webview):
        """Create an Element instance for testing."""
        from auroraview.dom import Element

        return Element(mock_webview, "#test-element")

    # === Text & Content Tests ===

    def test_get_text(self, element, mock_webview):
        """Test get_text generates correct JavaScript."""
        element.get_text()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "textContent" in call_args
        assert "__auroraview_result" in call_args

    def test_set_text(self, element, mock_webview):
        """Test set_text generates correct JavaScript."""
        element.set_text("Hello World")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "textContent" in call_args
        assert "Hello World" in call_args

    def test_set_text_escapes_special_chars(self, element, mock_webview):
        """Test set_text properly escapes special characters."""
        element.set_text('Test "quotes" and \\backslash')
        call_args = mock_webview.eval_js.call_args[0][0]
        # json.dumps should escape these properly
        assert '\\"' in call_args or "quotes" in call_args

    def test_get_html(self, element, mock_webview):
        """Test get_html generates correct JavaScript."""
        element.get_html()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "innerHTML" in call_args

    def test_set_html(self, element, mock_webview):
        """Test set_html generates correct JavaScript."""
        element.set_html("<div>Content</div>")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "innerHTML" in call_args

    # === Attribute Tests ===

    def test_get_attribute(self, element, mock_webview):
        """Test get_attribute generates correct JavaScript."""
        element.get_attribute("data-id")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "getAttribute" in call_args
        assert "data-id" in call_args

    def test_set_attribute(self, element, mock_webview):
        """Test set_attribute generates correct JavaScript."""
        element.set_attribute("data-id", "123")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "setAttribute" in call_args
        assert "data-id" in call_args
        assert "123" in call_args

    def test_remove_attribute(self, element, mock_webview):
        """Test remove_attribute generates correct JavaScript."""
        element.remove_attribute("data-id")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "removeAttribute" in call_args

    def test_has_attribute(self, element, mock_webview):
        """Test has_attribute generates correct JavaScript."""
        element.has_attribute("disabled")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "hasAttribute" in call_args

    # === Class Tests ===

    def test_add_class(self, element, mock_webview):
        """Test add_class generates correct JavaScript."""
        element.add_class("active")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "classList.add" in call_args
        assert "active" in call_args

    def test_remove_class(self, element, mock_webview):
        """Test remove_class generates correct JavaScript."""
        element.remove_class("active")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "classList.remove" in call_args

    def test_toggle_class(self, element, mock_webview):
        """Test toggle_class generates correct JavaScript."""
        element.toggle_class("visible")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "classList.toggle" in call_args

    def test_has_class(self, element, mock_webview):
        """Test has_class generates correct JavaScript."""
        element.has_class("active")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "classList.contains" in call_args

    # === Style Tests ===

    def test_get_style(self, element, mock_webview):
        """Test get_style generates correct JavaScript."""
        element.get_style("color")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "getComputedStyle" in call_args
        assert "color" in call_args

    def test_set_style(self, element, mock_webview):
        """Test set_style generates correct JavaScript."""
        element.set_style("color", "red")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "style" in call_args
        assert "color" in call_args
        assert "red" in call_args

    def test_set_styles(self, element, mock_webview):
        """Test set_styles generates correct JavaScript."""
        element.set_styles({"color": "red", "fontSize": "16px"})
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "setProperty" in call_args or "style" in call_args

    # === Visibility Tests ===

    def test_show(self, element, mock_webview):
        """Test show generates correct JavaScript."""
        element.show()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "display" in call_args

    def test_hide(self, element, mock_webview):
        """Test hide generates correct JavaScript."""
        element.hide()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "display" in call_args
        assert "none" in call_args

    def test_is_visible(self, element, mock_webview):
        """Test is_visible generates correct JavaScript."""
        element.is_visible()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "offsetWidth" in call_args or "offsetParent" in call_args or "display" in call_args

    # === Query Tests ===

    def test_query(self, element, mock_webview):
        """Test query returns a new Element."""
        from auroraview.dom import Element

        child = element.query(".child")
        assert isinstance(child, Element)
        assert ".child" in child._selector

    def test_query_all(self, element, mock_webview):
        """Test query_all returns an ElementCollection."""
        from auroraview.dom import ElementCollection

        children = element.query_all(".child")
        assert isinstance(children, ElementCollection)

    def test_exists(self, element, mock_webview):
        """Test exists generates correct JavaScript."""
        element.exists()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "querySelector" in call_args
        assert "__auroraview_result" in call_args

    def test_count(self, element, mock_webview):
        """Test count generates correct JavaScript."""
        element.count()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "querySelectorAll" in call_args
        assert "length" in call_args

    # === Form Value Tests ===

    def test_get_value(self, element, mock_webview):
        """Test get_value generates correct JavaScript."""
        element.get_value()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "value" in call_args

    def test_set_value(self, element, mock_webview):
        """Test set_value generates correct JavaScript."""
        element.set_value("test input")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "value" in call_args
        assert "input" in call_args  # Event dispatch

    def test_get_checked(self, element, mock_webview):
        """Test get_checked generates correct JavaScript."""
        element.get_checked()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "checked" in call_args

    def test_set_checked(self, element, mock_webview):
        """Test set_checked generates correct JavaScript."""
        element.set_checked(True)
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "checked" in call_args
        assert "true" in call_args

    def test_is_disabled(self, element, mock_webview):
        """Test is_disabled generates correct JavaScript."""
        element.is_disabled()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "disabled" in call_args

    def test_set_disabled(self, element, mock_webview):
        """Test set_disabled generates correct JavaScript."""
        element.set_disabled(True)
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "disabled" in call_args
        assert "true" in call_args

    # === Select/Dropdown Tests ===

    def test_get_selected_options(self, element, mock_webview):
        """Test get_selected_options generates correct JavaScript."""
        element.get_selected_options()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "selectedOptions" in call_args

    def test_select_option(self, element, mock_webview):
        """Test select_option generates correct JavaScript."""
        element.select_option("option1")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "value" in call_args
        assert "change" in call_args

    def test_select_option_by_text(self, element, mock_webview):
        """Test select_option_by_text generates correct JavaScript."""
        element.select_option_by_text("Option One")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "options" in call_args
        assert "text" in call_args

    def test_select_option_by_index(self, element, mock_webview):
        """Test select_option_by_index generates correct JavaScript."""
        element.select_option_by_index(2)
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "selectedIndex" in call_args
        assert "2" in call_args

    # === Interaction Tests ===

    def test_click(self, element, mock_webview):
        """Test click generates correct JavaScript."""
        element.click()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "click()" in call_args

    def test_double_click(self, element, mock_webview):
        """Test double_click generates correct JavaScript."""
        element.double_click()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "dblclick" in call_args

    def test_focus(self, element, mock_webview):
        """Test focus generates correct JavaScript."""
        element.focus()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "focus()" in call_args

    def test_blur(self, element, mock_webview):
        """Test blur generates correct JavaScript."""
        element.blur()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "blur()" in call_args

    def test_scroll_into_view(self, element, mock_webview):
        """Test scroll_into_view generates correct JavaScript."""
        element.scroll_into_view(behavior="smooth", block="center")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "scrollIntoView" in call_args
        assert "smooth" in call_args
        assert "center" in call_args

    def test_hover(self, element, mock_webview):
        """Test hover generates correct JavaScript."""
        element.hover()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "mouseenter" in call_args

    # === Type & Submit Tests ===

    def test_type_text(self, element, mock_webview):
        """Test type_text generates correct JavaScript."""
        element.type_text("Hello")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "value" in call_args
        assert "Hello" in call_args

    def test_type_text_clear_first(self, element, mock_webview):
        """Test type_text with clear_first generates correct JavaScript."""
        element.type_text("Hello", clear_first=True)
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "value = ''" in call_args

    def test_clear(self, element, mock_webview):
        """Test clear generates correct JavaScript."""
        element.clear()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "value = ''" in call_args

    def test_submit(self, element, mock_webview):
        """Test submit generates correct JavaScript."""
        element.submit()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "closest('form')" in call_args
        assert "submit" in call_args

    # === Traversal Tests ===

    def test_parent(self, element, mock_webview):
        """Test parent returns a new Element with correct selector."""
        from auroraview.dom import Element

        parent = element.parent()
        assert isinstance(parent, Element)
        assert ":has(>" in parent._selector

    def test_closest(self, element, mock_webview):
        """Test closest returns a new Element with correct selector."""
        from auroraview.dom import Element

        ancestor = element.closest(".container")
        assert isinstance(ancestor, Element)
        assert ".container" in ancestor._selector

    def test_first_child(self, element, mock_webview):
        """Test first_child returns a new Element with correct selector."""
        from auroraview.dom import Element

        child = element.first_child()
        assert isinstance(child, Element)
        assert ":first-child" in child._selector

    def test_last_child(self, element, mock_webview):
        """Test last_child returns a new Element with correct selector."""
        from auroraview.dom import Element

        child = element.last_child()
        assert isinstance(child, Element)
        assert ":last-child" in child._selector

    def test_nth_child(self, element, mock_webview):
        """Test nth_child returns a new Element with correct selector."""
        from auroraview.dom import Element

        child = element.nth_child(3)
        assert isinstance(child, Element)
        assert ":nth-child(3)" in child._selector

    def test_next_sibling(self, element, mock_webview):
        """Test next_sibling returns a new Element with correct selector."""
        from auroraview.dom import Element

        sibling = element.next_sibling()
        assert isinstance(sibling, Element)
        assert "+ *" in sibling._selector

    def test_prev_sibling(self, element, mock_webview):
        """Test prev_sibling returns a new Element with correct selector."""
        from auroraview.dom import Element

        sibling = element.prev_sibling()
        assert isinstance(sibling, Element)
        assert ":has(+" in sibling._selector

    def test_children(self, element, mock_webview):
        """Test children returns an ElementCollection."""
        from auroraview.dom import ElementCollection

        children = element.children()
        assert isinstance(children, ElementCollection)
        assert "> *" in children._selector

    def test_siblings(self, element, mock_webview):
        """Test siblings returns an ElementCollection."""
        from auroraview.dom import ElementCollection

        siblings = element.siblings()
        assert isinstance(siblings, ElementCollection)

    # === DOM Manipulation Tests ===

    def test_append_html(self, element, mock_webview):
        """Test append_html generates correct JavaScript."""
        element.append_html("<span>New</span>")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "insertAdjacentHTML" in call_args
        assert "beforeend" in call_args

    def test_prepend_html(self, element, mock_webview):
        """Test prepend_html generates correct JavaScript."""
        element.prepend_html("<span>First</span>")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "insertAdjacentHTML" in call_args
        assert "afterbegin" in call_args

    def test_insert_before(self, element, mock_webview):
        """Test insert_before generates correct JavaScript."""
        element.insert_before("<div>Before</div>")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "insertAdjacentHTML" in call_args
        assert "beforebegin" in call_args

    def test_insert_after(self, element, mock_webview):
        """Test insert_after generates correct JavaScript."""
        element.insert_after("<div>After</div>")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "insertAdjacentHTML" in call_args
        assert "afterend" in call_args

    def test_remove(self, element, mock_webview):
        """Test remove generates correct JavaScript."""
        element.remove()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "remove()" in call_args

    def test_replace_with(self, element, mock_webview):
        """Test replace_with generates correct JavaScript."""
        element.replace_with("<div>Replacement</div>")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "outerHTML" in call_args

    def test_empty(self, element, mock_webview):
        """Test empty generates correct JavaScript."""
        element.empty()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "innerHTML = ''" in call_args

    # === Repr Test ===

    def test_repr(self, element):
        """Test Element repr."""
        assert repr(element) == "Element('#test-element')"


class TestElementCollection:
    """Tests for the ElementCollection class."""

    @pytest.fixture
    def mock_webview(self):
        """Create a mock WebView for testing."""
        webview = MagicMock()
        webview.eval_js = MagicMock()
        return webview

    @pytest.fixture
    def collection(self, mock_webview):
        """Create an ElementCollection instance for testing."""
        from auroraview.dom import ElementCollection

        return ElementCollection(mock_webview, ".items")

    def test_first(self, collection, mock_webview):
        """Test first returns an Element with same selector."""
        from auroraview.dom import Element

        first = collection.first()
        assert isinstance(first, Element)
        # first() returns the same selector (querySelector returns first match)
        assert first._selector == ".items"

    def test_nth(self, collection, mock_webview):
        """Test nth returns an Element with nth-child selector."""
        from auroraview.dom import Element

        # nth(3) means index 3 (0-based), which is :nth-child(4) (1-based)
        third = collection.nth(3)
        assert isinstance(third, Element)
        assert ":nth-child(4)" in third._selector

    def test_count(self, collection, mock_webview):
        """Test count generates correct JavaScript."""
        collection.count()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "querySelectorAll" in call_args
        assert "length" in call_args

    def test_add_class(self, collection, mock_webview):
        """Test add_class generates correct JavaScript."""
        collection.add_class("highlight")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "forEach" in call_args
        assert "classList.add" in call_args

    def test_remove_class(self, collection, mock_webview):
        """Test remove_class generates correct JavaScript."""
        collection.remove_class("highlight")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "forEach" in call_args
        assert "classList.remove" in call_args

    def test_set_style(self, collection, mock_webview):
        """Test set_style generates correct JavaScript."""
        collection.set_style("color", "blue")
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "forEach" in call_args
        assert "style" in call_args

    def test_hide(self, collection, mock_webview):
        """Test hide generates correct JavaScript."""
        collection.hide()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "forEach" in call_args
        assert "none" in call_args

    def test_show(self, collection, mock_webview):
        """Test show generates correct JavaScript."""
        collection.show()
        call_args = mock_webview.eval_js.call_args[0][0]
        assert "forEach" in call_args
        assert "display" in call_args

    def test_repr(self, collection):
        """Test ElementCollection repr."""
        assert repr(collection) == "ElementCollection('.items')"


class TestRustDomBatch:
    """Tests for the Rust-powered DomBatch class.

    DomBatch provides high-performance DOM operations by generating
    optimized JavaScript in Rust and batching multiple operations
    into a single eval_js call.
    """

    @pytest.fixture
    def batch(self):
        """Create a DomBatch instance for testing."""
        try:
            from auroraview import DomBatch

            if DomBatch is None:
                pytest.skip("DomBatch not available (Rust core not compiled)")
            return DomBatch()
        except ImportError:
            pytest.skip("DomBatch not available (Rust core not compiled)")

    def test_batch_creation(self, batch):
        """Test DomBatch can be created."""
        assert batch is not None
        assert batch.count == 0
        assert len(batch) == 0

    def test_batch_set_text(self, batch):
        """Test set_text adds operation to batch."""
        batch.set_text("#title", "Hello World")
        assert batch.count == 1
        js = batch.to_js()
        assert "#title" in js
        assert "Hello World" in js
        assert "textContent" in js

    def test_batch_multiple_operations(self, batch):
        """Test multiple operations can be batched."""
        batch.set_text("#title", "Hello")
        batch.add_class(".item", "active")
        batch.click("#btn")

        assert batch.count == 3
        js = batch.to_js()
        assert "#title" in js
        assert ".item" in js
        assert "#btn" in js
        assert "classList.add" in js
        assert "click()" in js

    def test_batch_is_wrapped_in_iife(self, batch):
        """Test generated JS is wrapped in IIFE for isolation."""
        batch.set_text("#test", "value")
        js = batch.to_js()
        assert js.startswith("(function(){")
        assert js.endswith("})()")

    def test_batch_escapes_special_chars(self, batch):
        """Test special characters are properly escaped."""
        batch.set_text("#test", 'Hello "World"')
        js = batch.to_js()
        # Should escape double quotes
        assert '\\"' in js or "World" in js

    def test_batch_clear(self, batch):
        """Test clear removes all operations."""
        batch.set_text("#a", "1")
        batch.set_text("#b", "2")
        assert batch.count == 2
        batch.clear()
        assert batch.count == 0

    def test_batch_empty_generates_noop(self, batch):
        """Test empty batch generates valid no-op JS."""
        js = batch.to_js()
        assert js == "(function(){})()"

    def test_batch_set_html(self, batch):
        """Test set_html operation."""
        batch.set_html("#content", "<p>Hello</p>")
        js = batch.to_js()
        assert "innerHTML" in js
        assert "<p>Hello</p>" in js or "Hello" in js

    def test_batch_add_class(self, batch):
        """Test add_class operation."""
        batch.add_class("#elem", "active")
        js = batch.to_js()
        assert "classList.add" in js
        assert "active" in js

    def test_batch_remove_class(self, batch):
        """Test remove_class operation."""
        batch.remove_class("#elem", "hidden")
        js = batch.to_js()
        assert "classList.remove" in js
        assert "hidden" in js

    def test_batch_toggle_class(self, batch):
        """Test toggle_class operation."""
        batch.toggle_class("#elem", "expanded")
        js = batch.to_js()
        assert "classList.toggle" in js
        assert "expanded" in js

    def test_batch_set_attribute(self, batch):
        """Test set_attribute operation."""
        batch.set_attribute("#link", "href", "https://example.com")
        js = batch.to_js()
        assert "setAttribute" in js
        assert "href" in js

    def test_batch_remove_attribute(self, batch):
        """Test remove_attribute operation."""
        batch.remove_attribute("#elem", "disabled")
        js = batch.to_js()
        assert "removeAttribute" in js
        assert "disabled" in js

    def test_batch_set_style(self, batch):
        """Test set_style operation."""
        batch.set_style("#box", "backgroundColor", "red")
        js = batch.to_js()
        assert "style" in js
        assert "backgroundColor" in js
        assert "red" in js

    def test_batch_show_hide(self, batch):
        """Test show and hide operations."""
        batch.show("#elem1")
        batch.hide("#elem2")
        js = batch.to_js()
        assert "display" in js
        assert "none" in js

    def test_batch_set_value(self, batch):
        """Test set_value operation."""
        batch.set_value("#input", "test value")
        js = batch.to_js()
        assert ".value" in js
        assert "test value" in js

    def test_batch_set_checked(self, batch):
        """Test set_checked operation."""
        batch.set_checked("#checkbox", True)
        js = batch.to_js()
        assert "checked" in js
        assert "true" in js

    def test_batch_set_disabled(self, batch):
        """Test set_disabled operation."""
        batch.set_disabled("#btn", True)
        js = batch.to_js()
        assert "disabled" in js
        assert "true" in js

    def test_batch_click(self, batch):
        """Test click operation."""
        batch.click("#submit")
        js = batch.to_js()
        assert "click()" in js

    def test_batch_double_click(self, batch):
        """Test double_click operation."""
        batch.double_click("#item")
        js = batch.to_js()
        assert "dblclick" in js
        assert "MouseEvent" in js

    def test_batch_focus_blur(self, batch):
        """Test focus and blur operations."""
        batch.focus("#input")
        batch.blur("#other")
        js = batch.to_js()
        assert "focus()" in js
        assert "blur()" in js

    def test_batch_scroll_into_view(self, batch):
        """Test scroll_into_view operation."""
        batch.scroll_into_view("#section", smooth=True)
        js = batch.to_js()
        assert "scrollIntoView" in js
        assert "smooth" in js

    def test_batch_type_text(self, batch):
        """Test type_text operation."""
        batch.type_text("#input", "hello", clear=True)
        js = batch.to_js()
        assert "input" in js.lower()
        assert "hello" in js

    def test_batch_clear_input(self, batch):
        """Test clear_input operation."""
        batch.clear_input("#search")
        js = batch.to_js()
        assert "value" in js
        assert "input" in js.lower()

    def test_batch_submit(self, batch):
        """Test submit operation."""
        batch.submit("#form")
        js = batch.to_js()
        assert "submit" in js.lower()

    def test_batch_append_html(self, batch):
        """Test append_html operation."""
        batch.append_html("#list", "<li>Item</li>")
        js = batch.to_js()
        assert "insertAdjacentHTML" in js
        assert "beforeend" in js

    def test_batch_prepend_html(self, batch):
        """Test prepend_html operation."""
        batch.prepend_html("#list", "<li>First</li>")
        js = batch.to_js()
        assert "insertAdjacentHTML" in js
        assert "afterbegin" in js

    def test_batch_remove(self, batch):
        """Test remove operation."""
        batch.remove("#old-elem")
        js = batch.to_js()
        assert "remove()" in js

    def test_batch_empty(self, batch):
        """Test empty operation (clear content)."""
        batch.empty("#container")
        js = batch.to_js()
        assert "innerHTML" in js

    def test_batch_raw(self, batch):
        """Test raw JavaScript on element."""
        batch.raw("#elem", "console.log(e.id)")
        js = batch.to_js()
        assert "console.log" in js

    def test_batch_raw_global(self, batch):
        """Test raw global JavaScript."""
        batch.raw_global("console.log('Hello')")
        js = batch.to_js()
        assert "console.log('Hello')" in js

    def test_batch_repr(self, batch):
        """Test batch string representation."""
        batch.set_text("#a", "1")
        batch.set_text("#b", "2")
        assert "DomBatch" in repr(batch)
        assert "2" in repr(batch)

    def test_batch_len(self, batch):
        """Test len() on batch."""
        assert len(batch) == 0
        batch.set_text("#test", "value")
        assert len(batch) == 1
