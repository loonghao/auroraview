"""Unit tests for the State class."""

from __future__ import annotations

from auroraview.core.state import State


class TestStateBasic:
    """Test basic State functionality."""

    def test_init_empty(self):
        """Test State initialization without webview."""
        state = State()
        assert len(state) == 0
        assert state.to_dict() == {}

    def test_setitem_getitem(self):
        """Test setting and getting items."""
        state = State()
        state["key1"] = "value1"
        state["key2"] = {"nested": "data"}

        assert state["key1"] == "value1"
        assert state["key2"] == {"nested": "data"}

    def test_delitem(self):
        """Test deleting items."""
        state = State()
        state["key"] = "value"
        del state["key"]

        assert "key" not in state

    def test_contains(self):
        """Test 'in' operator."""
        state = State()
        state["exists"] = True

        assert "exists" in state
        assert "missing" not in state

    def test_len(self):
        """Test length."""
        state = State()
        assert len(state) == 0

        state["a"] = 1
        state["b"] = 2
        assert len(state) == 2

    def test_iter(self):
        """Test iteration over keys."""
        state = State()
        state["a"] = 1
        state["b"] = 2

        keys = list(state)
        assert set(keys) == {"a", "b"}

    def test_get_with_default(self):
        """Test get with default value."""
        state = State()
        state["exists"] = "value"

        assert state.get("exists") == "value"
        assert state.get("missing") is None
        assert state.get("missing", "default") == "default"

    def test_keys_values_items(self):
        """Test dict-like methods."""
        state = State()
        state["a"] = 1
        state["b"] = 2

        assert set(state.keys()) == {"a", "b"}
        assert set(state.values()) == {1, 2}
        assert set(state.items()) == {("a", 1), ("b", 2)}

    def test_update(self):
        """Test batch update."""
        state = State()
        state.update({"a": 1, "b": 2, "c": 3})

        assert state["a"] == 1
        assert state["b"] == 2
        assert state["c"] == 3

    def test_clear(self):
        """Test clearing all state."""
        state = State()
        state["a"] = 1
        state["b"] = 2
        state.clear()

        assert len(state) == 0

    def test_to_dict(self):
        """Test converting to dict."""
        state = State()
        state["a"] = 1
        state["b"] = {"nested": True}

        d = state.to_dict()
        assert d == {"a": 1, "b": {"nested": True}}
        # Ensure it's a copy
        d["a"] = 999
        assert state["a"] == 1

    def test_repr(self):
        """Test string representation."""
        state = State()
        state["key"] = "value"

        assert "State" in repr(state)
        assert "key" in repr(state)


class TestStateChangeHandlers:
    """Test State change handler functionality."""

    def test_on_change_decorator(self):
        """Test on_change as decorator."""
        state = State()
        changes = []

        @state.on_change
        def handler(key, value, source):
            changes.append((key, value, source))

        state["test"] = "value"

        assert len(changes) == 1
        assert changes[0] == ("test", "value", "python")

    def test_off_change(self):
        """Test removing change handler."""
        state = State()
        changes = []

        def handler(key, value, source):
            changes.append((key, value, source))

        state.on_change(handler)
        state["a"] = 1
        assert len(changes) == 1

        state.off_change(handler)
        state["b"] = 2
        assert len(changes) == 1  # No new changes

    def test_multiple_handlers(self):
        """Test multiple change handlers."""
        state = State()
        results1 = []
        results2 = []

        @state.on_change
        def handler1(key, value, source):
            results1.append(key)

        @state.on_change
        def handler2(key, value, source):
            results2.append(key)

        state["test"] = "value"

        assert results1 == ["test"]
        assert results2 == ["test"]
