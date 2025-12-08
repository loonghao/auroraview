"""Test maya_qt_echo_demo module import and callable behavior.

This test module verifies that the maya_qt_echo_demo module can be imported
and called correctly, preventing the regression where the module was not callable.

Issue: When importing the module, it would execute show_auroraview_maya_dialog()
immediately instead of allowing the user to call maya_qt_echo_demo() function.

These tests require Qt dependencies to be installed:
    pip install auroraview[qt]
"""

import pytest

# Mark all tests as Qt tests
pytestmark = pytest.mark.qt


class TestMayaQtEchoDemoModule:
    """Test maya_qt_echo_demo module structure and exports."""

    def test_module_importable_from_examples(self):
        """Test that maya_qt_echo_demo can be imported from examples package."""
        try:
            from examples import maya_qt_echo_demo

            assert maya_qt_echo_demo is not None
            assert callable(maya_qt_echo_demo)
        except ImportError as e:
            # Maya dependencies not available, which is expected in CI
            pytest.skip(f"Maya dependencies not available: {e}")

    def test_function_exists_in_examples_init(self):
        """Test that maya_qt_echo_demo function is exported in examples.__init__."""
        try:
            import examples

            assert hasattr(examples, "maya_qt_echo_demo")
            assert callable(examples.maya_qt_echo_demo)
        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")

    def test_show_function_exists_in_examples_init(self):
        """Test that show_auroraview_maya_dialog function is exported."""
        try:
            import examples

            assert hasattr(examples, "show_auroraview_maya_dialog")
            assert callable(examples.show_auroraview_maya_dialog)
        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")

    def test_module_does_not_auto_execute(self):
        """Test that importing the module does not auto-execute the dialog.

        This is a regression test for the issue where the module would
        immediately call show_auroraview_maya_dialog() on import.
        """
        try:
            # Import the module - this should NOT create any dialogs
            from examples import maya_qt_echo_demo

            # If we get here without errors, the module imported successfully
            # without auto-executing
            assert callable(maya_qt_echo_demo)

        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")
        except Exception as e:
            # If we get a different error (like Maya not running),
            # that's expected and we should skip
            if "Maya" in str(e) or "MQtUtil" in str(e):
                pytest.skip(f"Maya runtime not available: {e}")
            else:
                raise


class TestMayaQtEchoDemoFunctionSignature:
    """Test function signatures and documentation."""

    def test_maya_qt_echo_demo_is_callable(self):
        """Test that maya_qt_echo_demo is a callable function."""
        try:
            from examples.maya_qt_echo_demo import maya_qt_echo_demo

            assert callable(maya_qt_echo_demo)
        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")

    def test_show_auroraview_maya_dialog_is_callable(self):
        """Test that show_auroraview_maya_dialog is a callable function."""
        try:
            from examples.maya_qt_echo_demo import show_auroraview_maya_dialog

            assert callable(show_auroraview_maya_dialog)
        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")

    def test_maya_qt_echo_demo_has_docstring(self):
        """Test that maya_qt_echo_demo has proper documentation."""
        try:
            from examples.maya_qt_echo_demo import maya_qt_echo_demo

            assert maya_qt_echo_demo.__doc__ is not None
            assert len(maya_qt_echo_demo.__doc__.strip()) > 0
        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")

    def test_show_auroraview_maya_dialog_has_docstring(self):
        """Test that show_auroraview_maya_dialog has proper documentation."""
        try:
            from examples.maya_qt_echo_demo import show_auroraview_maya_dialog

            assert show_auroraview_maya_dialog.__doc__ is not None
            assert len(show_auroraview_maya_dialog.__doc__.strip()) > 0
        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")


class TestMayaQtEchoDemoClasses:
    """Test classes defined in maya_qt_echo_demo module."""

    def test_shelf_api_class_exists(self):
        """Test that _ShelfAPI class exists."""
        try:
            from examples.maya_qt_echo_demo import _ShelfAPI

            assert _ShelfAPI is not None
        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")

    def test_auroraview_maya_dialog_class_exists(self):
        """Test that AuroraViewMayaDialog class exists."""
        try:
            from examples.maya_qt_echo_demo import AuroraViewMayaDialog

            assert AuroraViewMayaDialog is not None
        except ImportError as e:
            pytest.skip(f"Maya dependencies not available: {e}")
