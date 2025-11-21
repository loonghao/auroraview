"""Tests for the CLI main entry point.

This module tests the Python CLI entry point that delegates to the Rust binary.
"""

import sys
from unittest.mock import MagicMock, patch

import pytest


def test_main_success():
    """Test successful CLI execution."""
    from auroraview.__main__ import main

    # Mock subprocess.run to simulate successful execution
    mock_result = MagicMock()
    mock_result.returncode = 0

    # Mock sys.argv to have only the program name
    with patch("subprocess.run", return_value=mock_result) as mock_run:
        with patch.object(sys, "argv", ["auroraview"]):
            with pytest.raises(SystemExit) as exc_info:
                main()

            # Verify subprocess.run was called correctly
            mock_run.assert_called_once_with(
                ["auroraview"],
                check=False,
            )
            # Verify exit code
            assert exc_info.value.code == 0


def test_main_with_arguments():
    """Test CLI execution with arguments."""
    from auroraview.__main__ import main

    # Mock subprocess.run to simulate successful execution
    mock_result = MagicMock()
    mock_result.returncode = 0

    # Mock sys.argv to include arguments
    test_args = ["auroraview", "--html", "test.html", "--debug"]

    with patch("subprocess.run", return_value=mock_result) as mock_run:
        with patch.object(sys, "argv", test_args):
            with pytest.raises(SystemExit) as exc_info:
                main()

            # Verify subprocess.run was called with arguments
            mock_run.assert_called_once_with(
                ["auroraview", "--html", "test.html", "--debug"],
                check=False,
            )
            assert exc_info.value.code == 0


def test_main_non_zero_exit_code():
    """Test CLI execution with non-zero exit code."""
    from auroraview.__main__ import main

    # Mock subprocess.run to simulate failure
    mock_result = MagicMock()
    mock_result.returncode = 1

    with patch("subprocess.run", return_value=mock_result) as mock_run:
        with patch.object(sys, "argv", ["auroraview"]):
            with pytest.raises(SystemExit) as exc_info:
                main()

            mock_run.assert_called_once()
            assert exc_info.value.code == 1


def test_main_binary_not_found():
    """Test CLI execution when binary is not found."""
    from auroraview.__main__ import main

    # Mock subprocess.run to raise FileNotFoundError
    with patch("subprocess.run", side_effect=FileNotFoundError("Binary not found")):
        with patch.object(sys, "argv", ["auroraview"]):
            with patch("builtins.print") as mock_print:
                with pytest.raises(SystemExit) as exc_info:
                    main()

                # Verify error messages were printed
                assert mock_print.called
                assert exc_info.value.code == 1


def test_main_generic_exception():
    """Test CLI execution with generic exception."""
    from auroraview.__main__ import main

    # Mock subprocess.run to raise a generic exception
    with patch("subprocess.run", side_effect=RuntimeError("Unexpected error")):
        with patch.object(sys, "argv", ["auroraview"]):
            with patch("builtins.print") as mock_print:
                with pytest.raises(SystemExit) as exc_info:
                    main()

                # Verify error message was printed
                assert mock_print.called
                assert exc_info.value.code == 1


def test_main_module_execution():
    """Test that __main__ module can be executed."""
    # This tests the if __name__ == "__main__": block
    import importlib.util
    import os

    # Get the path to __main__.py
    main_path = os.path.join(os.path.dirname(__file__), "..", "python", "auroraview", "__main__.py")
    main_path = os.path.abspath(main_path)

    # Load the module
    spec = importlib.util.spec_from_file_location("__main__", main_path)
    if spec and spec.loader:
        module = importlib.util.module_from_spec(spec)

        # Mock subprocess.run to avoid actual execution
        mock_result = MagicMock()
        mock_result.returncode = 0

        with patch("subprocess.run", return_value=mock_result):
            with pytest.raises(SystemExit):
                # Execute the module
                spec.loader.exec_module(module)
