"""Native file dialogs for AuroraView Qt integration.

This module provides cross-platform native file dialogs for opening, saving,
and selecting folders.

Example:
    >>> from auroraview.integration.qt import QtWebView
    >>> from auroraview.integration.qt.dialogs import FileDialog
    >>>
    >>> webview = QtWebView(parent=maya_main_window())
    >>>
    >>> # Open file dialog
    >>> files = webview.create_file_dialog(
    ...     FileDialog.OPEN,
    ...     allow_multiple=True,
    ...     file_types=('Image Files (*.png;*.jpg)', 'All files (*.*)')
    ... )
    >>> print(files)  # ['/path/to/file1.png', '/path/to/file2.jpg']
    >>>
    >>> # Save file dialog
    >>> save_path = webview.create_file_dialog(
    ...     FileDialog.SAVE,
    ...     directory='/home/user/documents',
    ...     save_filename='untitled.txt'
    ... )
    >>>
    >>> # Folder selection
    >>> folder = webview.create_file_dialog(FileDialog.FOLDER)
"""

from __future__ import annotations

import logging
from enum import IntEnum
from pathlib import Path
from typing import List, Optional, Tuple, Union

logger = logging.getLogger(__name__)

__all__ = ["FileDialog", "FileDialogMixin", "create_file_dialog"]


class FileDialog(IntEnum):
    """File dialog types.

    Attributes:
        OPEN: Open file(s) dialog
        SAVE: Save file dialog
        FOLDER: Folder selection dialog
    """

    OPEN = 0
    SAVE = 1
    FOLDER = 2


def create_file_dialog(
    dialog_type: FileDialog = FileDialog.OPEN,
    directory: str = "",
    allow_multiple: bool = False,
    save_filename: str = "",
    file_types: Tuple[str, ...] = (),
    parent: Optional["QWidget"] = None,  # noqa: F821
) -> Optional[Union[List[str], str]]:
    """Create a native file dialog.

    This is a standalone function that can be used without a WebView instance.

    Args:
        dialog_type: Type of dialog (OPEN, SAVE, or FOLDER)
        directory: Initial directory to open (default: current directory)
        allow_multiple: Allow selecting multiple files (OPEN only)
        save_filename: Default filename for SAVE dialogs
        file_types: Tuple of file type filters, e.g.:
            ('Image Files (*.png;*.jpg)', 'Text Files (*.txt)', 'All files (*.*)')
        parent: Parent Qt widget (optional)

    Returns:
        - OPEN (single): Path string or None if cancelled
        - OPEN (multiple): List of path strings or None if cancelled
        - SAVE: Path string or None if cancelled
        - FOLDER: Path string or None if cancelled

    Example:
        >>> from auroraview.integration.qt.dialogs import create_file_dialog, FileDialog
        >>>
        >>> # Open single file
        >>> path = create_file_dialog(FileDialog.OPEN)
        >>>
        >>> # Open multiple images
        >>> paths = create_file_dialog(
        ...     FileDialog.OPEN,
        ...     allow_multiple=True,
        ...     file_types=('Images (*.png *.jpg *.gif)',)
        ... )
    """
    try:
        from qtpy.QtWidgets import QFileDialog
    except ImportError as e:
        raise ImportError(
            "Qt dialogs require qtpy. Install with: pip install auroraview[qt]"
        ) from e

    # Convert file_types to Qt filter format
    # Input: "Image Files (*.png;*.jpg)" -> Qt format: "Image Files (*.png *.jpg)"
    qt_filters = []
    for ft in file_types:
        # Replace semicolons with spaces for Qt compatibility
        qt_filter = ft.replace(";", " ")
        qt_filters.append(qt_filter)
    filter_str = ";;".join(qt_filters) if qt_filters else ""

    # Ensure directory exists
    if directory and not Path(directory).exists():
        directory = ""

    if dialog_type == FileDialog.OPEN:
        if allow_multiple:
            result, _ = QFileDialog.getOpenFileNames(
                parent,
                "Open Files",
                directory,
                filter_str,
            )
            return result if result else None
        else:
            result, _ = QFileDialog.getOpenFileName(
                parent,
                "Open File",
                directory,
                filter_str,
            )
            return result if result else None

    elif dialog_type == FileDialog.SAVE:
        initial_path = directory
        if save_filename:
            initial_path = str(Path(directory) / save_filename) if directory else save_filename
        result, _ = QFileDialog.getSaveFileName(
            parent,
            "Save File",
            initial_path,
            filter_str,
        )
        return result if result else None

    elif dialog_type == FileDialog.FOLDER:
        result = QFileDialog.getExistingDirectory(
            parent,
            "Select Folder",
            directory,
        )
        return result if result else None

    else:
        raise ValueError(f"Unknown dialog type: {dialog_type}")


class FileDialogMixin:
    """Mixin providing file dialog methods for QtWebView.

    This mixin adds native file dialog methods to QtWebView.
    """

    def create_file_dialog(
        self,
        dialog_type: FileDialog = FileDialog.OPEN,
        directory: str = "",
        allow_multiple: bool = False,
        save_filename: str = "",
        file_types: Tuple[str, ...] = (),
    ) -> Optional[Union[List[str], str]]:
        """Create a native file dialog.

        Args:
            dialog_type: Type of dialog (FileDialog.OPEN, SAVE, or FOLDER)
            directory: Initial directory to open
            allow_multiple: Allow selecting multiple files (OPEN only)
            save_filename: Default filename for SAVE dialogs
            file_types: Tuple of file type filters, e.g.:
                ('Image Files (*.png;*.jpg)', 'All files (*.*)')

        Returns:
            - OPEN (single): Path string or None if cancelled
            - OPEN (multiple): List of path strings or None if cancelled
            - SAVE: Path string or None if cancelled
            - FOLDER: Path string or None if cancelled

        Example:
            >>> # Open file with filters
            >>> files = webview.create_file_dialog(
            ...     FileDialog.OPEN,
            ...     allow_multiple=True,
            ...     file_types=('Images (*.png *.jpg)', 'All files (*.*)')
            ... )
            >>>
            >>> # Save file
            >>> path = webview.create_file_dialog(
            ...     FileDialog.SAVE,
            ...     directory='/home/user/documents',
            ...     save_filename='output.txt'
            ... )
            >>>
            >>> # Select folder
            >>> folder = webview.create_file_dialog(FileDialog.FOLDER)
        """
        # Use self as parent widget for proper dialog centering
        parent = self if hasattr(self, "winId") else None
        return create_file_dialog(
            dialog_type=dialog_type,
            directory=directory,
            allow_multiple=allow_multiple,
            save_filename=save_filename,
            file_types=file_types,
            parent=parent,
        )

    def create_confirmation_dialog(
        self,
        title: str,
        message: str,
    ) -> bool:
        """Create a confirmation dialog with Yes/No buttons.

        Args:
            title: Dialog title
            message: Dialog message

        Returns:
            True if user clicked Yes, False otherwise

        Example:
            >>> if webview.create_confirmation_dialog("Confirm", "Are you sure?"):
            ...     print("User confirmed")
        """
        try:
            from qtpy.QtWidgets import QMessageBox
        except ImportError as e:
            raise ImportError(
                "Qt dialogs require qtpy. Install with: pip install auroraview[qt]"
            ) from e

        parent = self if hasattr(self, "winId") else None
        result = QMessageBox.question(
            parent,
            title,
            message,
            QMessageBox.Yes | QMessageBox.No,
            QMessageBox.No,  # Default button
        )
        return result == QMessageBox.Yes

    def create_alert_dialog(
        self,
        title: str,
        message: str,
    ) -> None:
        """Create an alert dialog with OK button.

        Args:
            title: Dialog title
            message: Dialog message

        Example:
            >>> webview.create_alert_dialog("Info", "Operation completed!")
        """
        try:
            from qtpy.QtWidgets import QMessageBox
        except ImportError as e:
            raise ImportError(
                "Qt dialogs require qtpy. Install with: pip install auroraview[qt]"
            ) from e

        parent = self if hasattr(self, "winId") else None
        QMessageBox.information(parent, title, message)
