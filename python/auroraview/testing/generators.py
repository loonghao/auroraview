# -*- coding: utf-8 -*-
"""Test data generators for AuroraView testing framework.

This module provides data generators for property-based testing and
randomized test data generation.

Example:
    >>> from auroraview.testing.generators import (
    ...     random_html, random_js_value, random_event_name
    ... )
    >>>
    >>> html = random_html(tag="div", content="Hello")
    >>> js_val = random_js_value()
    >>> event = random_event_name()
"""

from __future__ import annotations

import random
import string
from typing import Any, Dict, List, Optional, Union

# ─────────────────────────────────────────────────────────────────────────────
# HTML Generators
# ─────────────────────────────────────────────────────────────────────────────


def random_string(
    length: int = 10,
    charset: str = string.ascii_letters + string.digits,
) -> str:
    """Generate a random string.

    Args:
        length: Length of the string
        charset: Characters to choose from

    Returns:
        Random string
    """
    return "".join(random.choice(charset) for _ in range(length))


def random_html(
    tag: str = "div",
    content: Optional[str] = None,
    attrs: Optional[Dict[str, str]] = None,
    children: Optional[List[str]] = None,
) -> str:
    """Generate random HTML element.

    Args:
        tag: HTML tag name
        content: Text content
        attrs: HTML attributes
        children: Child HTML elements

    Returns:
        HTML string

    Example:
        >>> html = random_html("div", content="Hello", attrs={"class": "test"})
        >>> print(html)
        <div class="test">Hello</div>
    """
    attrs = attrs or {}
    attr_str = " ".join(f'{k}="{v}"' for k, v in attrs.items())
    if attr_str:
        attr_str = " " + attr_str

    if children:
        inner = "\n".join(children)
    elif content:
        inner = content
    else:
        inner = random_string(20)

    return f"<{tag}{attr_str}>{inner}</{tag}>"


def random_html_page(
    title: Optional[str] = None,
    body_content: Optional[str] = None,
    styles: Optional[str] = None,
    scripts: Optional[str] = None,
) -> str:
    """Generate a complete HTML page.

    Args:
        title: Page title
        body_content: Body HTML content
        styles: CSS styles
        scripts: JavaScript code

    Returns:
        Complete HTML page string
    """
    title = title or f"Test Page {random_string(5)}"
    body_content = body_content or random_html("h1", "Test Page")
    styles = styles or ""
    scripts = scripts or ""

    return f"""<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{title}</title>
    <style>{styles}</style>
</head>
<body>
    {body_content}
    <script>{scripts}</script>
</body>
</html>"""


def random_form_html(
    fields: Optional[List[Dict[str, str]]] = None,
    action: str = "#",
    method: str = "post",
) -> str:
    """Generate a random HTML form.

    Args:
        fields: List of field definitions with 'name', 'type', 'label'
        action: Form action URL
        method: Form method

    Returns:
        HTML form string
    """
    if fields is None:
        fields = [
            {"name": "username", "type": "text", "label": "Username"},
            {"name": "password", "type": "password", "label": "Password"},
            {"name": "email", "type": "email", "label": "Email"},
        ]

    field_html = []
    for field in fields:
        field_id = f"field_{field['name']}"
        field_html.append(f"""
        <div class="form-group">
            <label for="{field_id}">{field.get("label", field["name"])}</label>
            <input type="{field["type"]}" id="{field_id}" name="{field["name"]}">
        </div>
        """)

    return f"""
    <form action="{action}" method="{method}">
        {"".join(field_html)}
        <button type="submit">Submit</button>
    </form>
    """


# ─────────────────────────────────────────────────────────────────────────────
# JavaScript Value Generators
# ─────────────────────────────────────────────────────────────────────────────


def random_js_value(
    value_type: Optional[str] = None,
    max_depth: int = 3,
) -> Any:
    """Generate a random JSON-serializable value.

    Args:
        value_type: Specific type ('string', 'number', 'bool', 'array', 'object', 'null')
        max_depth: Maximum nesting depth for arrays/objects

    Returns:
        Random JSON-serializable value
    """
    if value_type is None:
        value_type = random.choice(["string", "number", "bool", "array", "object", "null"])

    if value_type == "string":
        return random_string(random.randint(1, 50))
    elif value_type == "number":
        return random.choice(
            [
                random.randint(-1000, 1000),
                round(random.uniform(-1000, 1000), 2),
            ]
        )
    elif value_type == "bool":
        return random.choice([True, False])
    elif value_type == "null":
        return None
    elif value_type == "array" and max_depth > 0:
        length = random.randint(0, 5)
        return [random_js_value(max_depth=max_depth - 1) for _ in range(length)]
    elif value_type == "object" and max_depth > 0:
        length = random.randint(0, 5)
        return {random_string(8): random_js_value(max_depth=max_depth - 1) for _ in range(length)}
    else:
        return random_string(10)


def random_event_payload(
    event_type: Optional[str] = None,
) -> Dict[str, Any]:
    """Generate a random event payload.

    Args:
        event_type: Type of event ('click', 'input', 'custom')

    Returns:
        Event payload dictionary
    """
    if event_type is None:
        event_type = random.choice(["click", "input", "custom"])

    base = {
        "timestamp": random.randint(1000000000000, 9999999999999),
        "type": event_type,
    }

    if event_type == "click":
        base.update(
            {
                "x": random.randint(0, 1920),
                "y": random.randint(0, 1080),
                "button": random.choice([0, 1, 2]),
                "target": f"#{random_string(8)}",
            }
        )
    elif event_type == "input":
        base.update(
            {
                "value": random_string(20),
                "target": f"#{random_string(8)}",
            }
        )
    else:
        base.update(
            {
                "data": random_js_value(max_depth=2),
            }
        )

    return base


# ─────────────────────────────────────────────────────────────────────────────
# Event Name Generators
# ─────────────────────────────────────────────────────────────────────────────


def random_event_name(
    prefix: Optional[str] = None,
    namespace: Optional[str] = None,
) -> str:
    """Generate a random event name.

    Args:
        prefix: Event name prefix
        namespace: Event namespace

    Returns:
        Event name string

    Example:
        >>> event = random_event_name(prefix="user", namespace="auth")
        >>> print(event)  # e.g., "auth:user_abc123"
    """
    name = prefix or random.choice(
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
        ]
    )

    suffix = random_string(6, string.ascii_lowercase + string.digits)
    event_name = f"{name}_{suffix}"

    if namespace:
        event_name = f"{namespace}:{event_name}"

    return event_name


# ─────────────────────────────────────────────────────────────────────────────
# API Call Generators
# ─────────────────────────────────────────────────────────────────────────────


def random_api_method(
    namespace: str = "api",
) -> str:
    """Generate a random API method name.

    Args:
        namespace: API namespace

    Returns:
        API method string (e.g., "api.get_user")
    """
    verbs = ["get", "set", "create", "update", "delete", "fetch", "sync", "validate"]
    nouns = ["user", "data", "config", "state", "item", "list", "result", "info"]

    verb = random.choice(verbs)
    noun = random.choice(nouns)

    return f"{namespace}.{verb}_{noun}"


def random_api_params(
    param_count: Optional[int] = None,
    as_dict: bool = True,
) -> Union[Dict[str, Any], List[Any]]:
    """Generate random API parameters.

    Args:
        param_count: Number of parameters (random if None)
        as_dict: Return as dict (kwargs) or list (args)

    Returns:
        Parameters as dict or list
    """
    if param_count is None:
        param_count = random.randint(0, 5)

    if as_dict:
        return {
            random_string(8, string.ascii_lowercase): random_js_value(max_depth=2)
            for _ in range(param_count)
        }
    else:
        return [random_js_value(max_depth=2) for _ in range(param_count)]


# ─────────────────────────────────────────────────────────────────────────────
# DOM Selector Generators
# ─────────────────────────────────────────────────────────────────────────────


def random_selector(
    selector_type: Optional[str] = None,
) -> str:
    """Generate a random CSS selector.

    Args:
        selector_type: Type of selector ('id', 'class', 'tag', 'attr')

    Returns:
        CSS selector string
    """
    if selector_type is None:
        selector_type = random.choice(["id", "class", "tag", "attr"])

    if selector_type == "id":
        return f"#{random_string(8, string.ascii_lowercase)}"
    elif selector_type == "class":
        return f".{random_string(8, string.ascii_lowercase)}"
    elif selector_type == "tag":
        return random.choice(["div", "span", "p", "button", "input", "a", "h1", "h2"])
    elif selector_type == "attr":
        attr = random.choice(["data-id", "data-test", "name", "type"])
        return f'[{attr}="{random_string(8)}"]'
    else:
        return f"#{random_string(8, string.ascii_lowercase)}"


def random_xpath(
    element_type: Optional[str] = None,
) -> str:
    """Generate a random XPath expression.

    Args:
        element_type: Type of element to select

    Returns:
        XPath expression string
    """
    if element_type is None:
        element_type = random.choice(["div", "span", "button", "input", "a"])

    patterns = [
        f"//{element_type}",
        f"//{element_type}[@id='{random_string(8)}']",
        f"//{element_type}[@class='{random_string(8)}']",
        f"//{element_type}[contains(text(), '{random_string(5)}')]",
        f"//div/{element_type}",
    ]

    return random.choice(patterns)


# ─────────────────────────────────────────────────────────────────────────────
# URL Generators
# ─────────────────────────────────────────────────────────────────────────────


def random_url(
    scheme: str = "https",
    domain: Optional[str] = None,
    path: Optional[str] = None,
    query_params: Optional[Dict[str, str]] = None,
) -> str:
    """Generate a random URL.

    Args:
        scheme: URL scheme (http, https, file)
        domain: Domain name
        path: URL path
        query_params: Query parameters

    Returns:
        URL string
    """
    if domain is None:
        domain = f"{random_string(8, string.ascii_lowercase)}.example.com"

    if path is None:
        path_parts = [random_string(5, string.ascii_lowercase) for _ in range(random.randint(0, 3))]
        path = "/" + "/".join(path_parts) if path_parts else ""

    url = f"{scheme}://{domain}{path}"

    if query_params:
        query = "&".join(f"{k}={v}" for k, v in query_params.items())
        url = f"{url}?{query}"

    return url


def random_file_url(
    extension: str = "html",
    directory: Optional[str] = None,
) -> str:
    """Generate a random file:// URL.

    Args:
        extension: File extension
        directory: Base directory

    Returns:
        File URL string
    """
    if directory is None:
        directory = "/tmp/test"

    filename = f"{random_string(10, string.ascii_lowercase)}.{extension}"
    return f"file://{directory}/{filename}"


# ─────────────────────────────────────────────────────────────────────────────
# Test Data Sets
# ─────────────────────────────────────────────────────────────────────────────


def generate_test_dataset(
    count: int = 10,
    data_type: str = "mixed",
) -> List[Dict[str, Any]]:
    """Generate a dataset for testing.

    Args:
        count: Number of items to generate
        data_type: Type of data ('html', 'events', 'api_calls', 'mixed')

    Returns:
        List of test data items
    """
    items = []

    for i in range(count):
        if data_type == "html":
            items.append(
                {
                    "id": i,
                    "html": random_html_page(),
                    "selector": random_selector(),
                }
            )
        elif data_type == "events":
            items.append(
                {
                    "id": i,
                    "event_name": random_event_name(),
                    "payload": random_event_payload(),
                }
            )
        elif data_type == "api_calls":
            items.append(
                {
                    "id": i,
                    "method": random_api_method(),
                    "params": random_api_params(),
                }
            )
        else:  # mixed
            item_type = random.choice(["html", "events", "api_calls"])
            if item_type == "html":
                items.append(
                    {
                        "id": i,
                        "type": "html",
                        "data": random_html(),
                    }
                )
            elif item_type == "events":
                items.append(
                    {
                        "id": i,
                        "type": "event",
                        "name": random_event_name(),
                        "payload": random_event_payload(),
                    }
                )
            else:
                items.append(
                    {
                        "id": i,
                        "type": "api_call",
                        "method": random_api_method(),
                        "params": random_api_params(),
                    }
                )

    return items


__all__ = [
    # String generators
    "random_string",
    # HTML generators
    "random_html",
    "random_html_page",
    "random_form_html",
    # JavaScript value generators
    "random_js_value",
    "random_event_payload",
    # Event generators
    "random_event_name",
    # API generators
    "random_api_method",
    "random_api_params",
    # DOM selector generators
    "random_selector",
    "random_xpath",
    # URL generators
    "random_url",
    "random_file_url",
    # Dataset generators
    "generate_test_dataset",
]
