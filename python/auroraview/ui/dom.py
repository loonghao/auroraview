"""DOM Element manipulation module for AuroraView.

This module provides a Pythonic interface for DOM manipulation,
inspired by PyWebView's Element API and designed for future
Steel Browser integration.

Example:
    >>> # Basic usage
    >>> element = webview.dom("#my-button")
    >>> element.click()
    >>> element.set_text("Clicked!")

    >>> # Form handling
    >>> form = webview.dom("#login-form")
    >>> form.query("#username").set_value("admin")
    >>> form.query("#password").set_value("secret")
    >>> form.query("button[type=submit]").click()

    >>> # Batch operations (using Rust DomBatch for performance)
    >>> batch = DomBatch()
    >>> batch.add_class(".item", "processed")
    >>> batch.set_text("#status", "Done")
    >>> webview.eval_js(batch.to_js())
"""

from __future__ import annotations

import json
import logging
from typing import TYPE_CHECKING, Dict, Optional

if TYPE_CHECKING:
    from auroraview.core.webview import WebView

logger = logging.getLogger(__name__)

__all__ = ["Element", "ElementCollection"]


def _escape_selector(selector: str) -> str:
    """Escape selector for JavaScript string."""
    return selector.replace("\\", "\\\\").replace('"', '\\"')


def _make_element_script(selector: str, script: str) -> str:
    """Generate IIFE script for element operation."""
    return f'(function(){{var e=document.querySelector("{_escape_selector(selector)}");if(e){{{script}}}}})()'


def _make_result_script(selector: str, expr: str) -> str:
    """Generate script that stores result in global variable."""
    return _make_element_script(selector, f"window.__auroraview_result={expr}")


def _make_collection_script(selector: str, script: str) -> str:
    """Generate script for operating on all matching elements."""
    sel = json.dumps(selector)
    return f"(function(){{document.querySelectorAll({sel}).forEach(function(e){{{script}}})}})();"


class Element:
    """DOM Element wrapper with Steel-compatible API.

    Provides a high-level interface for DOM manipulation using
    JavaScript evaluation under the hood.
    """

    __slots__ = ("_webview", "_selector")

    def __init__(self, webview: "WebView", selector: str) -> None:
        self._webview = webview
        self._selector = selector

    @property
    def selector(self) -> str:
        """Get the CSS selector for this element."""
        return self._selector

    def _eval(self, script: str) -> None:
        """Execute JavaScript on this element."""
        self._webview.eval_js(_make_element_script(self._selector, script))

    def _get(self, expr: str) -> None:
        """Get value and store in __auroraview_result."""
        self._webview.eval_js(_make_result_script(self._selector, expr))

    # === Text & Content ===
    def get_text(self) -> None:
        self._get("e.textContent")

    def set_text(self, text: str) -> None:
        self._eval(f"e.textContent={json.dumps(text)}")

    def get_html(self) -> None:
        self._get("e.innerHTML")

    def set_html(self, html: str) -> None:
        self._eval(f"e.innerHTML={json.dumps(html)}")

    def get_outer_html(self) -> None:
        self._get("e.outerHTML")

    # === Attributes ===
    def get_attribute(self, name: str) -> None:
        self._get(f"e.getAttribute({json.dumps(name)})")

    def set_attribute(self, name: str, value: str) -> None:
        self._eval(f"e.setAttribute({json.dumps(name)},{json.dumps(value)})")

    def remove_attribute(self, name: str) -> None:
        self._eval(f"e.removeAttribute({json.dumps(name)})")

    def has_attribute(self, name: str) -> None:
        self._get(f"e.hasAttribute({json.dumps(name)})")

    # === Classes ===
    def add_class(self, *class_names: str) -> None:
        args = ",".join(json.dumps(c) for c in class_names)
        self._eval(f"e.classList.add({args})")

    def remove_class(self, *class_names: str) -> None:
        args = ",".join(json.dumps(c) for c in class_names)
        self._eval(f"e.classList.remove({args})")

    def toggle_class(self, class_name: str, force: Optional[bool] = None) -> None:
        if force is None:
            self._eval(f"e.classList.toggle({json.dumps(class_name)})")
        else:
            self._eval(f"e.classList.toggle({json.dumps(class_name)},{str(force).lower()})")

    def has_class(self, class_name: str) -> None:
        self._get(f"e.classList.contains({json.dumps(class_name)})")

    # === Styles ===
    def get_style(self, prop: str) -> None:
        self._get(f"getComputedStyle(e).getPropertyValue({json.dumps(prop)})")

    def set_style(self, prop: str, value: str) -> None:
        self._eval(f"e.style.setProperty({json.dumps(prop)},{json.dumps(value)})")

    def set_styles(self, styles: Dict[str, str]) -> None:
        parts = ";".join(f"e.style[{json.dumps(k)}]={json.dumps(v)}" for k, v in styles.items())
        self._eval(parts)

    # === Visibility ===
    def show(self, display: str = "block") -> None:
        self._eval(f"e.style.display={json.dumps(display)}")

    def hide(self) -> None:
        self._eval("e.style.display='none'")

    def is_visible(self) -> None:
        self._get("!!(e.offsetWidth||e.offsetHeight||e.getClientRects().length)")

    # === Query ===
    def query(self, selector: str) -> "Element":
        return Element(self._webview, f"{self._selector} {selector}")

    def query_all(self, selector: str) -> "ElementCollection":
        return ElementCollection(self._webview, f"{self._selector} {selector}")

    def exists(self) -> None:
        self._get("true")

    def count(self) -> None:
        sel = json.dumps(self._selector)
        self._webview.eval_js(f"window.__auroraview_result=document.querySelectorAll({sel}).length")

    # === Element Info ===
    def get_tag_name(self) -> None:
        self._get("e.tagName.toLowerCase()")

    def get_bounding_rect(self) -> None:
        self._get(
            "(function(r){{return{{x:r.x,y:r.y,width:r.width,height:r.height,top:r.top,right:r.right,bottom:r.bottom,left:r.left}}}})(e.getBoundingClientRect())"
        )

    # === Form Values ===
    def get_value(self) -> None:
        self._get("e.value")

    def set_value(self, value: str) -> None:
        self._eval(
            f"e.value={json.dumps(value)};e.dispatchEvent(new Event('input',{{bubbles:true}}))"
        )

    def get_checked(self) -> None:
        self._get("e.checked")

    def set_checked(self, checked: bool) -> None:
        self._eval(
            f"e.checked={str(checked).lower()};e.dispatchEvent(new Event('change',{{bubbles:true}}))"
        )

    def is_disabled(self) -> None:
        self._get("e.disabled")

    def set_disabled(self, disabled: bool) -> None:
        self._eval(f"e.disabled={str(disabled).lower()}")

    # === Select/Dropdown ===
    def get_selected_options(self) -> None:
        self._get(
            "Array.from(e.selectedOptions).map(function(o){{return{{value:o.value,text:o.text,index:o.index}}}})"
        )

    def select_option(self, value: str) -> None:
        self._eval(
            f"e.value={json.dumps(value)};e.dispatchEvent(new Event('change',{{bubbles:true}}))"
        )

    def select_option_by_text(self, text: str) -> None:
        self._eval(
            f"var o=Array.from(e.options).find(function(x){{return x.text==={json.dumps(text)}}});if(o){{e.value=o.value;e.dispatchEvent(new Event('change',{{bubbles:true}}))}}"
        )

    def select_option_by_index(self, index: int) -> None:
        self._eval(f"e.selectedIndex={index};e.dispatchEvent(new Event('change',{{bubbles:true}}))")

    # === User Interactions ===
    def click(self) -> None:
        self._eval("e.click()")

    def double_click(self) -> None:
        self._eval("e.dispatchEvent(new MouseEvent('dblclick',{bubbles:true,cancelable:true}))")

    def focus(self) -> None:
        self._eval("e.focus()")

    def blur(self) -> None:
        self._eval("e.blur()")

    def scroll_into_view(self, behavior: str = "smooth", block: str = "center") -> None:
        self._eval(
            f"e.scrollIntoView({{behavior:{json.dumps(behavior)},block:{json.dumps(block)}}})"
        )

    def hover(self) -> None:
        self._eval("e.dispatchEvent(new MouseEvent('mouseenter',{bubbles:true}))")

    # === Type & Submit ===
    def type_text(self, text: str, clear_first: bool = False) -> None:
        t = json.dumps(text)
        if clear_first:
            self._eval(
                f"e.value='';e.value={t};e.dispatchEvent(new Event('input',{{bubbles:true}}))"
            )
        else:
            self._eval(f"e.value+={t};e.dispatchEvent(new Event('input',{{bubbles:true}}))")

    def clear(self) -> None:
        self._eval("e.value='';e.dispatchEvent(new Event('input',{bubbles:true}))")

    def submit(self) -> None:
        self._eval(
            "var f=e.closest('form');if(f)f.dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}))"
        )

    # === Traversal ===
    def parent(self) -> "Element":
        return Element(self._webview, f":has(> {self._selector})")

    def closest(self, ancestor_selector: str) -> "Element":
        return Element(self._webview, f"{ancestor_selector}:has({self._selector})")

    def first_child(self) -> "Element":
        return Element(self._webview, f"{self._selector} > :first-child")

    def last_child(self) -> "Element":
        return Element(self._webview, f"{self._selector} > :last-child")

    def nth_child(self, n: int) -> "Element":
        return Element(self._webview, f"{self._selector} > :nth-child({n})")

    def next_sibling(self) -> "Element":
        return Element(self._webview, f"{self._selector} + *")

    def prev_sibling(self) -> "Element":
        return Element(self._webview, f":has(+ {self._selector})")

    def children(self) -> "ElementCollection":
        return ElementCollection(self._webview, f"{self._selector} > *")

    def siblings(self) -> "ElementCollection":
        return ElementCollection(self._webview, f":has(> {self._selector}) > *")

    # === DOM Manipulation ===
    def append_html(self, html: str) -> None:
        self._eval(f"e.insertAdjacentHTML('beforeend',{json.dumps(html)})")

    def prepend_html(self, html: str) -> None:
        self._eval(f"e.insertAdjacentHTML('afterbegin',{json.dumps(html)})")

    def insert_before(self, html: str) -> None:
        self._eval(f"e.insertAdjacentHTML('beforebegin',{json.dumps(html)})")

    def insert_after(self, html: str) -> None:
        self._eval(f"e.insertAdjacentHTML('afterend',{json.dumps(html)})")

    def remove(self) -> None:
        self._eval("e.remove()")

    def replace_with(self, html: str) -> None:
        self._eval(f"e.outerHTML={json.dumps(html)}")

    def empty(self) -> None:
        self._eval("e.innerHTML=''")

    def __repr__(self) -> str:
        return f"Element({self._selector!r})"


class ElementCollection:
    """Collection of DOM Elements for batch operations."""

    __slots__ = ("_webview", "_selector")

    def __init__(self, webview: "WebView", selector: str) -> None:
        self._webview = webview
        self._selector = selector

    @property
    def selector(self) -> str:
        return self._selector

    def _batch(self, script: str) -> None:
        """Execute script on all matching elements."""
        self._webview.eval_js(_make_collection_script(self._selector, script))

    def first(self) -> Element:
        return Element(self._webview, self._selector)

    def nth(self, index: int) -> Element:
        return Element(self._webview, f"{self._selector}:nth-child({index + 1})")

    def count(self) -> None:
        sel = json.dumps(self._selector)
        self._webview.eval_js(f"window.__auroraview_result=document.querySelectorAll({sel}).length")

    # === Batch Operations ===
    def add_class(self, *class_names: str) -> None:
        args = ",".join(json.dumps(c) for c in class_names)
        self._batch(f"e.classList.add({args})")

    def remove_class(self, *class_names: str) -> None:
        args = ",".join(json.dumps(c) for c in class_names)
        self._batch(f"e.classList.remove({args})")

    def set_style(self, prop: str, value: str) -> None:
        self._batch(f"e.style.setProperty({json.dumps(prop)},{json.dumps(value)})")

    def hide(self) -> None:
        self._batch("e.style.display='none'")

    def show(self, display: str = "block") -> None:
        self._batch(f"e.style.display={json.dumps(display)}")

    def __repr__(self) -> str:
        return f"ElementCollection({self._selector!r})"
