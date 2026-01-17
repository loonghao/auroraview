# -*- coding: utf-8 -*-
"""Qt WebEngine Multi-Tab Browser.

A true multi-tab browser built with PySide6/Qt WebEngine.
Unlike the AuroraView-based agent_browser which has threading issues,
this implementation uses Qt's native event loop and WebEngine.

Features:
- True multi-tab browsing with QTabWidget
- No threading issues - all WebViews share Qt's event loop
- Full browser functionality (back, forward, reload, home)
- Smart URL bar (auto-detect search vs URL)
- Keyboard shortcuts (Ctrl+T, Ctrl+W, Ctrl+L, F5)
- Tab management (drag, close, new tab)

Requirements:
    pip install PySide6 PySide6-WebEngine

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import sys
from typing import Optional
from urllib.parse import urlparse

try:
    from PySide6.QtCore import QUrl, Qt, Signal, QSize
    from PySide6.QtGui import QAction, QIcon, QKeySequence, QShortcut
    from PySide6.QtWebEngineWidgets import QWebEngineView
    from PySide6.QtWebEngineCore import QWebEnginePage, QWebEngineProfile
    from PySide6.QtWidgets import (
        QApplication,
        QHBoxLayout,
        QLineEdit,
        QMainWindow,
        QProgressBar,
        QPushButton,
        QTabWidget,
        QToolBar,
        QVBoxLayout,
        QWidget,
        QStyle,
        QTabBar,
    )
except ImportError:
    print("Error: PySide6 and PySide6-WebEngine are required.")
    print("Install with: pip install PySide6 PySide6-WebEngine")
    sys.exit(1)


class BrowserTab(QWebEngineView):
    """A single browser tab with its own WebEngine view."""

    title_changed = Signal(str)
    icon_changed = Signal(QIcon)
    url_changed = Signal(QUrl)

    def __init__(self, parent: Optional[QWidget] = None):
        super().__init__(parent)

        # Connect signals
        self.titleChanged.connect(self._on_title_changed)
        self.iconChanged.connect(self._on_icon_changed)
        self.urlChanged.connect(self._on_url_changed)

        # Set a custom page for handling new window requests
        page = BrowserPage(self)
        self.setPage(page)

    def _on_title_changed(self, title: str) -> None:
        self.title_changed.emit(title or "New Tab")

    def _on_icon_changed(self, icon: QIcon) -> None:
        self.icon_changed.emit(icon)

    def _on_url_changed(self, url: QUrl) -> None:
        self.url_changed.emit(url)


class BrowserPage(QWebEnginePage):
    """Custom page to handle new window requests."""

    def __init__(self, view: BrowserTab):
        super().__init__(view)
        self._view = view

    def createWindow(self, window_type: QWebEnginePage.WebWindowType) -> Optional[QWebEnginePage]:
        """Handle window.open() and target="_blank" links."""
        # Find the main window
        main_window = self._view.window()
        if isinstance(main_window, QtBrowser):
            new_tab = main_window.add_tab()
            return new_tab.page()
        return None


class QtBrowser(QMainWindow):
    """Multi-tab browser using Qt WebEngine."""

    HOME_URL = "https://www.google.com"
    SEARCH_ENGINE = "https://www.google.com/search?q={}"

    def __init__(self):
        super().__init__()

        self.setWindowTitle("Qt Browser")
        self.setMinimumSize(1024, 768)

        # Central widget
        central = QWidget()
        self.setCentralWidget(central)
        layout = QVBoxLayout(central)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(0)

        # Navigation toolbar
        self._create_toolbar()
        layout.addWidget(self.toolbar)

        # Tab widget
        self.tabs = QTabWidget()
        self.tabs.setTabsClosable(True)
        self.tabs.setMovable(True)
        self.tabs.setDocumentMode(True)
        self.tabs.tabCloseRequested.connect(self._close_tab)
        self.tabs.currentChanged.connect(self._on_tab_changed)

        # Add "new tab" button to tab bar
        self.tabs.setCornerWidget(self._create_new_tab_button(), Qt.TopRightCorner)

        layout.addWidget(self.tabs)

        # Progress bar
        self.progress = QProgressBar()
        self.progress.setMaximumHeight(3)
        self.progress.setTextVisible(False)
        self.progress.hide()
        layout.addWidget(self.progress)

        # Setup keyboard shortcuts
        self._setup_shortcuts()

        # Create initial tab
        self.add_tab(self.HOME_URL)

    def _create_toolbar(self) -> None:
        """Create the navigation toolbar."""
        self.toolbar = QToolBar()
        self.toolbar.setMovable(False)
        self.toolbar.setIconSize(QSize(20, 20))

        # Style the toolbar
        self.toolbar.setStyleSheet("""
            QToolBar {
                background: #1a1a2e;
                border: none;
                padding: 4px 8px;
                spacing: 4px;
            }
            QToolButton {
                background: transparent;
                border: none;
                border-radius: 4px;
                padding: 6px;
                color: #e4e4e4;
            }
            QToolButton:hover {
                background: #2a2a4a;
            }
            QToolButton:pressed {
                background: #3a3a5a;
            }
            QToolButton:disabled {
                color: #666;
            }
        """)

        # Navigation buttons
        style = self.style()

        self.back_btn = QPushButton()
        self.back_btn.setIcon(style.standardIcon(QStyle.SP_ArrowBack))
        self.back_btn.setToolTip("Back (Alt+Left)")
        self.back_btn.clicked.connect(self._go_back)
        self.back_btn.setEnabled(False)
        self.toolbar.addWidget(self.back_btn)

        self.forward_btn = QPushButton()
        self.forward_btn.setIcon(style.standardIcon(QStyle.SP_ArrowForward))
        self.forward_btn.setToolTip("Forward (Alt+Right)")
        self.forward_btn.clicked.connect(self._go_forward)
        self.forward_btn.setEnabled(False)
        self.toolbar.addWidget(self.forward_btn)

        self.reload_btn = QPushButton()
        self.reload_btn.setIcon(style.standardIcon(QStyle.SP_BrowserReload))
        self.reload_btn.setToolTip("Reload (F5)")
        self.reload_btn.clicked.connect(self._reload)
        self.toolbar.addWidget(self.reload_btn)

        self.home_btn = QPushButton()
        self.home_btn.setIcon(style.standardIcon(QStyle.SP_DirHomeIcon))
        self.home_btn.setToolTip("Home")
        self.home_btn.clicked.connect(self._go_home)
        self.toolbar.addWidget(self.home_btn)

        # URL bar
        self.url_bar = QLineEdit()
        self.url_bar.setPlaceholderText("Search or enter URL...")
        self.url_bar.returnPressed.connect(self._navigate)
        self.url_bar.setStyleSheet("""
            QLineEdit {
                background: #0f0f1a;
                border: 1px solid #2a2a4a;
                border-radius: 16px;
                padding: 8px 16px;
                color: #e4e4e4;
                font-size: 14px;
                selection-background-color: #4facfe;
            }
            QLineEdit:focus {
                border-color: #4facfe;
            }
        """)
        self.toolbar.addWidget(self.url_bar)

        # Button styling
        btn_style = """
            QPushButton {
                background: transparent;
                border: none;
                border-radius: 4px;
                padding: 6px;
                min-width: 32px;
                min-height: 32px;
            }
            QPushButton:hover {
                background: #2a2a4a;
            }
            QPushButton:pressed {
                background: #3a3a5a;
            }
            QPushButton:disabled {
                color: #666;
            }
        """
        for btn in [self.back_btn, self.forward_btn, self.reload_btn, self.home_btn]:
            btn.setStyleSheet(btn_style)

    def _create_new_tab_button(self) -> QPushButton:
        """Create the new tab button."""
        btn = QPushButton("+")
        btn.setToolTip("New Tab (Ctrl+T)")
        btn.setFixedSize(28, 28)
        btn.setStyleSheet("""
            QPushButton {
                background: transparent;
                border: none;
                border-radius: 4px;
                font-size: 18px;
                font-weight: bold;
                color: #888;
            }
            QPushButton:hover {
                background: #2a2a4a;
                color: #e4e4e4;
            }
        """)
        btn.clicked.connect(lambda: self.add_tab())
        return btn

    def _setup_shortcuts(self) -> None:
        """Setup keyboard shortcuts."""
        # New tab
        QShortcut(QKeySequence("Ctrl+T"), self, self.add_tab)
        # Close tab
        QShortcut(QKeySequence("Ctrl+W"), self, self._close_current_tab)
        # Focus URL bar
        QShortcut(QKeySequence("Ctrl+L"), self, self._focus_url_bar)
        QShortcut(QKeySequence("Alt+D"), self, self._focus_url_bar)
        # Reload
        QShortcut(QKeySequence("F5"), self, self._reload)
        QShortcut(QKeySequence("Ctrl+R"), self, self._reload)
        # Navigation
        QShortcut(QKeySequence("Alt+Left"), self, self._go_back)
        QShortcut(QKeySequence("Alt+Right"), self, self._go_forward)
        # Tab switching
        for i in range(9):
            QShortcut(QKeySequence(f"Ctrl+{i + 1}"), self, lambda idx=i: self._switch_to_tab(idx))

    def add_tab(self, url: Optional[str] = None) -> BrowserTab:
        """Add a new browser tab."""
        tab = BrowserTab()

        # Connect signals
        tab.title_changed.connect(lambda title: self._update_tab_title(tab, title))
        tab.icon_changed.connect(lambda icon: self._update_tab_icon(tab, icon))
        tab.url_changed.connect(lambda qurl: self._update_url_bar(tab, qurl))
        tab.loadStarted.connect(lambda: self._on_load_started(tab))
        tab.loadProgress.connect(self._on_load_progress)
        tab.loadFinished.connect(lambda ok: self._on_load_finished(tab, ok))

        # Add to tab widget
        index = self.tabs.addTab(tab, "New Tab")
        self.tabs.setCurrentIndex(index)

        # Navigate to URL
        if url:
            tab.setUrl(QUrl(url))
        else:
            self._focus_url_bar()

        return tab

    def _close_tab(self, index: int) -> None:
        """Close a tab by index."""
        if self.tabs.count() > 1:
            widget = self.tabs.widget(index)
            self.tabs.removeTab(index)
            widget.deleteLater()
        else:
            # Last tab - close the browser
            self.close()

    def _close_current_tab(self) -> None:
        """Close the current tab."""
        self._close_tab(self.tabs.currentIndex())

    def _on_tab_changed(self, index: int) -> None:
        """Handle tab change."""
        tab = self.tabs.widget(index)
        if isinstance(tab, BrowserTab):
            self._update_url_bar(tab, tab.url())
            self._update_nav_buttons(tab)

    def _update_tab_title(self, tab: BrowserTab, title: str) -> None:
        """Update tab title."""
        index = self.tabs.indexOf(tab)
        if index >= 0:
            # Truncate long titles
            display_title = title[:25] + "..." if len(title) > 25 else title
            self.tabs.setTabText(index, display_title)
            self.tabs.setTabToolTip(index, title)

            # Update window title if this is the current tab
            if self.tabs.currentWidget() == tab:
                self.setWindowTitle(f"{title} - Qt Browser")

    def _update_tab_icon(self, tab: BrowserTab, icon: QIcon) -> None:
        """Update tab icon."""
        index = self.tabs.indexOf(tab)
        if index >= 0:
            self.tabs.setTabIcon(index, icon)

    def _update_url_bar(self, tab: BrowserTab, url: QUrl) -> None:
        """Update URL bar if this is the current tab."""
        if self.tabs.currentWidget() == tab:
            self.url_bar.setText(url.toString())
            self._update_nav_buttons(tab)

    def _update_nav_buttons(self, tab: BrowserTab) -> None:
        """Update navigation button states."""
        self.back_btn.setEnabled(tab.history().canGoBack())
        self.forward_btn.setEnabled(tab.history().canGoForward())

    def _on_load_started(self, tab: BrowserTab) -> None:
        """Handle load started."""
        if self.tabs.currentWidget() == tab:
            self.progress.show()
            self.progress.setValue(0)

    def _on_load_progress(self, progress: int) -> None:
        """Handle load progress."""
        self.progress.setValue(progress)

    def _on_load_finished(self, tab: BrowserTab, ok: bool) -> None:
        """Handle load finished."""
        if self.tabs.currentWidget() == tab:
            self.progress.hide()
            self._update_nav_buttons(tab)

    def _navigate(self) -> None:
        """Navigate to URL or search."""
        text = self.url_bar.text().strip()
        if not text:
            return

        # Determine if it's a URL or search query
        url = self._text_to_url(text)

        tab = self.tabs.currentWidget()
        if isinstance(tab, BrowserTab):
            tab.setUrl(QUrl(url))

    def _text_to_url(self, text: str) -> str:
        """Convert text to URL (detect URL vs search query)."""
        # Check if it looks like a URL
        if text.startswith(("http://", "https://", "file://")):
            return text

        # Check if it's a domain-like pattern
        if "." in text and " " not in text:
            parsed = urlparse(f"https://{text}")
            if parsed.netloc:
                return f"https://{text}"

        # Treat as search query
        return self.SEARCH_ENGINE.format(text)

    def _go_back(self) -> None:
        """Go back in history."""
        tab = self.tabs.currentWidget()
        if isinstance(tab, BrowserTab):
            tab.back()

    def _go_forward(self) -> None:
        """Go forward in history."""
        tab = self.tabs.currentWidget()
        if isinstance(tab, BrowserTab):
            tab.forward()

    def _reload(self) -> None:
        """Reload current page."""
        tab = self.tabs.currentWidget()
        if isinstance(tab, BrowserTab):
            tab.reload()

    def _go_home(self) -> None:
        """Go to home page."""
        tab = self.tabs.currentWidget()
        if isinstance(tab, BrowserTab):
            tab.setUrl(QUrl(self.HOME_URL))

    def _focus_url_bar(self) -> None:
        """Focus the URL bar and select all text."""
        self.url_bar.setFocus()
        self.url_bar.selectAll()

    def _switch_to_tab(self, index: int) -> None:
        """Switch to tab by index (0-8 for Ctrl+1 to Ctrl+9)."""
        if index == 8:
            # Ctrl+9 goes to last tab
            index = self.tabs.count() - 1
        if 0 <= index < self.tabs.count():
            self.tabs.setCurrentIndex(index)


def main():
    """Run the Qt Browser."""
    print("=" * 60)
    print("Qt Browser - Multi-Tab Browser")
    print("=" * 60)
    print()
    print("Built with PySide6 + Qt WebEngine")
    print("No threading issues - all tabs share Qt's event loop")
    print()
    print("Keyboard shortcuts:")
    print("  Ctrl+T: New tab")
    print("  Ctrl+W: Close tab")
    print("  Ctrl+L: Focus URL bar")
    print("  Ctrl+1-9: Switch to tab")
    print("  F5: Reload")
    print("  Alt+Left/Right: Back/Forward")
    print()

    app = QApplication(sys.argv)

    # Apply dark theme
    app.setStyleSheet("""
        QMainWindow {
            background: #1a1a2e;
        }
        QTabWidget::pane {
            border: none;
            background: #0f0f1a;
        }
        QTabBar::tab {
            background: #1a1a2e;
            color: #888;
            padding: 8px 16px;
            border: none;
            border-top-left-radius: 4px;
            border-top-right-radius: 4px;
            margin-right: 2px;
        }
        QTabBar::tab:selected {
            background: #2a2a4a;
            color: #e4e4e4;
        }
        QTabBar::tab:hover:!selected {
            background: #252540;
        }
        QTabBar::close-button {
            image: none;
            subcontrol-position: right;
        }
        QTabBar::close-button:hover {
            background: #e74c3c;
            border-radius: 2px;
        }
        QProgressBar {
            background: transparent;
            border: none;
        }
        QProgressBar::chunk {
            background: #4facfe;
        }
    """)

    browser = QtBrowser()
    browser.show()

    sys.exit(app.exec())


if __name__ == "__main__":
    main()
