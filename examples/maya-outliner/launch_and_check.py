"""Launch Maya Outliner and immediately check window style."""

import ctypes
import sys
from ctypes import wintypes

# Add parent directory to path
parent_dir = r"C:\Users\hallo\Documents\augment-projects\dcc_webview\python"
if parent_dir not in sys.path:
    sys.path.insert(0, parent_dir)

from auroraview import WebView

# Windows constants
GWL_STYLE = -16
WS_CHILD = 0x40000000
WS_POPUP = 0x80000000
WS_SYSMENU = 0x00080000
WS_CAPTION = 0x00C00000

# Windows API
user32 = ctypes.windll.user32
GetWindowLongW = user32.GetWindowLongW
GetWindowLongW.argtypes = [wintypes.HWND, ctypes.c_int]
GetWindowLongW.restype = wintypes.LONG


def check_window_style(hwnd):
    """Check and display window style information."""
    print("\n" + "=" * 80)
    print(f"🔍 Checking window style for HWND: 0x{hwnd:08X}")
    print("=" * 80)

    style = GetWindowLongW(hwnd, GWL_STYLE)

    if style == 0:
        print("❌ Failed to get window style")
        return False

    print(f"\n📋 Window Style: 0x{style:08X}")
    print()

    # Check key flags
    is_child = bool(style & WS_CHILD)
    is_popup = bool(style & WS_POPUP)
    has_sysmenu = bool(style & WS_SYSMENU)
    has_caption = bool(style & WS_CAPTION)

    print("🎯 Key Flags:")
    print(f"   WS_CHILD:    {'✅ YES' if is_child else '❌ NO'}")
    print(f"   WS_POPUP:    {'✅ YES' if is_popup else '❌ NO'}")
    print(f"   WS_SYSMENU:  {'✅ YES' if has_sysmenu else '❌ NO'}")
    print(f"   WS_CAPTION:  {'✅ YES' if has_caption else '❌ NO'}")
    print()

    print("=" * 80)
    print("💡 Diagnosis:")
    print("=" * 80)

    if is_child:
        print("❌ PROBLEM FOUND: Window is WS_CHILD")
        print("   Child windows don't receive WM_CLOSE from X button!")
        print("   Solution: Use Owner mode instead of Child mode")
        return False
    elif not has_sysmenu:
        print("❌ PROBLEM FOUND: Window missing WS_SYSMENU")
        print("   Without system menu, X button won't work!")
        print("   Solution: Add WS_SYSMENU to window style")
        return False
    else:
        print("✅ Window style looks correct!")
        return True

    print("=" * 80)


# Get Maya main window
try:
    import maya.OpenMayaUI as omui
    import shiboken2
    from PySide2 import QtWidgets

    maya_main_window_ptr = omui.MQtUtil.mainWindow()
    maya_main_window = shiboken2.wrapInstance(int(maya_main_window_ptr), QtWidgets.QWidget)
    parent_hwnd = int(maya_main_window.winId())

    print(f"✅ Maya main window HWND: 0x{parent_hwnd:08X}")
except Exception as e:
    print(f"❌ Failed to get Maya main window: {e}")
    parent_hwnd = None

# Create WebView window
print("\n" + "=" * 80)
print("🚀 Creating WebView window...")
print("=" * 80)

view = WebView.create(
    title="Maya Outliner",
    url="https://www.example.com",
    width=400,
    height=600,
    parent=parent_hwnd,
    auto_show=False,  # Don't auto-show, we'll do it manually
)

# Show the window asynchronously (non-blocking)
print("\n📺 Showing window (non-blocking)...")
view.show(wait=False)

# Give it a moment to initialize
print("⏳ Waiting for window to initialize...")
import time

time.sleep(1.0)

# Get the window handle
try:
    hwnd = view._core.get_hwnd()
    if hwnd is None:
        print("\n❌ Failed to get window handle!")
        print("   Window may not be created yet")
    else:
        print("\n✅ WebView window created!")
        print(f"   HWND: 0x{hwnd:08X}")

        # Check window style immediately
        check_window_style(hwnd)
except Exception as e:
    print(f"\n❌ Error getting window handle: {e}")

print("\n" + "=" * 80)
print("📝 Instructions:")
print("=" * 80)
print("1. The window should now be visible")
print("2. Try clicking the X button")
print("3. Check if the window closes properly")
print("4. If it doesn't close, we've confirmed the problem!")
print("=" * 80)
