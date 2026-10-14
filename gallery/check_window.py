"""Check window position and visibility."""

import ctypes
import subprocess
import time
from ctypes import wintypes

# Windows API
user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32

# Window constants
SW_SHOW = 5
SW_RESTORE = 9
HWND_TOPMOST = -1
SWP_NOMOVE = 0x0002
SWP_NOSIZE = 0x0001

EnumWindows = user32.EnumWindows
EnumWindowsProc = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
GetWindowTextW = user32.GetWindowTextW
GetWindowTextLengthW = user32.GetWindowTextLengthW
IsWindowVisible = user32.IsWindowVisible
GetWindowRect = user32.GetWindowRect
SetWindowPos = user32.SetWindowPos
ShowWindow = user32.ShowWindow
GetWindowThreadProcessId = user32.GetWindowThreadProcessId


class RECT(ctypes.Structure):
    _fields_ = [
        ("left", ctypes.c_long),
        ("top", ctypes.c_long),
        ("right", ctypes.c_long),
        ("bottom", ctypes.c_long),
    ]


# Configure Win32 function signatures. Without argtypes, ctypes marshals
# Python int as a 32-bit C int, which overflows on 64-bit Windows when an
# HWND/handle is passed ("OverflowError: int too long to convert").
EnumWindows.argtypes = [EnumWindowsProc, wintypes.LPARAM]
EnumWindows.restype = wintypes.BOOL
GetWindowTextW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
GetWindowTextW.restype = ctypes.c_int
GetWindowTextLengthW.argtypes = [wintypes.HWND]
GetWindowTextLengthW.restype = ctypes.c_int
IsWindowVisible.argtypes = [wintypes.HWND]
IsWindowVisible.restype = wintypes.BOOL
GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(RECT)]
GetWindowRect.restype = wintypes.BOOL
# hWndInsertAfter accepts negative sentinels (HWND_TOPMOST=-1, HWND_NOTOPMOST=-2),
# so declare it as c_ssize_t to carry both real handles and the negatives.
SetWindowPos.argtypes = [
    wintypes.HWND,
    ctypes.c_ssize_t,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.c_int,
    wintypes.UINT,
]
SetWindowPos.restype = wintypes.BOOL
ShowWindow.argtypes = [wintypes.HWND, ctypes.c_int]
ShowWindow.restype = wintypes.BOOL
GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
GetWindowThreadProcessId.restype = wintypes.DWORD
user32.GetSystemMetrics.argtypes = [ctypes.c_int]
user32.GetSystemMetrics.restype = ctypes.c_int
user32.MoveWindow.argtypes = [
    wintypes.HWND,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.c_int,
    wintypes.BOOL,
]
user32.MoveWindow.restype = wintypes.BOOL


def get_window_title(hwnd):
    length = GetWindowTextLengthW(hwnd)
    if length == 0:
        return ""
    buf = ctypes.create_unicode_buffer(length + 1)
    GetWindowTextW(hwnd, buf, length + 1)
    return buf.value


def get_window_rect(hwnd):
    rect = RECT()
    GetWindowRect(hwnd, ctypes.byref(rect))
    return rect.left, rect.top, rect.right, rect.bottom


def find_windows_by_title(title_contains):
    """Find all windows containing the given title."""
    windows = []

    def callback(hwnd, lparam):
        title = get_window_title(hwnd)
        if title_contains.lower() in title.lower():
            pid = ctypes.c_ulong()
            GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
            visible = IsWindowVisible(hwnd)
            rect = get_window_rect(hwnd)
            windows.append(
                {
                    "hwnd": hwnd,
                    "title": title,
                    "pid": pid.value,
                    "visible": visible,
                    "rect": rect,
                }
            )
        return True

    EnumWindows(EnumWindowsProc(callback), 0)
    return windows


def bring_to_front(hwnd):
    """Bring window to front and make visible."""
    ShowWindow(hwnd, SW_RESTORE)
    ShowWindow(hwnd, SW_SHOW)
    # Set as topmost then remove topmost to bring to front
    SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
    SetWindowPos(hwnd, -2, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)  # HWND_NOTOPMOST


def move_to_center(hwnd, width=1200, height=800):
    """Move window to screen center."""
    screen_width = user32.GetSystemMetrics(0)
    screen_height = user32.GetSystemMetrics(1)
    x = (screen_width - width) // 2
    y = (screen_height - height) // 2
    user32.MoveWindow(hwnd, x, y, width, height, True)


def main():
    import os

    exe_path = os.path.join(os.path.dirname(__file__), "pack-output", "pack-output.exe")

    print("Starting packed exe...")
    proc = subprocess.Popen(
        [exe_path],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        cwd=os.path.dirname(exe_path),
    )

    print(f"Process started: PID {proc.pid}")

    # Wait for window to be created
    time.sleep(5)

    # Find AuroraView windows
    print("\nSearching for AuroraView windows...")
    windows = find_windows_by_title("AuroraView")

    if not windows:
        windows = find_windows_by_title("Gallery")

    if not windows:
        # Try to find by process
        print("No windows found by title, searching all windows...")
        all_windows = []

        def callback(hwnd, lparam):
            pid = ctypes.c_ulong()
            GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
            if pid.value == proc.pid:
                title = get_window_title(hwnd)
                visible = IsWindowVisible(hwnd)
                rect = get_window_rect(hwnd)
                all_windows.append(
                    {
                        "hwnd": hwnd,
                        "title": title,
                        "pid": pid.value,
                        "visible": visible,
                        "rect": rect,
                    }
                )
            return True

        EnumWindows(EnumWindowsProc(callback), 0)
        windows = all_windows

    print(f"\nFound {len(windows)} windows:")
    for w in windows:
        x, y, r, b = w["rect"]
        width = r - x
        height = b - y
        print(f"  HWND: {w['hwnd']}")
        print(f"  Title: '{w['title']}'")
        print(f"  PID: {w['pid']}")
        print(f"  Visible: {w['visible']}")
        print(f"  Position: ({x}, {y})")
        print(f"  Size: {width}x{height}")
        print()

        # Check if window is off-screen
        screen_width = user32.GetSystemMetrics(0)
        screen_height = user32.GetSystemMetrics(1)

        if x < -width or x > screen_width or y < -height or y > screen_height:
            print("  ⚠️ Window is OFF-SCREEN! Moving to center...")
            move_to_center(w["hwnd"])
            bring_to_front(w["hwnd"])
            print("  ✓ Window moved to center")
        elif not w["visible"]:
            print("  ⚠️ Window is not visible! Showing...")
            bring_to_front(w["hwnd"])
            print("  ✓ Window shown")
        else:
            print("  ✓ Window should be visible")
            bring_to_front(w["hwnd"])

    print("\nWaiting 10 seconds... Check if window is visible now.")
    time.sleep(10)

    print("\nTerminating process...")
    proc.terminate()
    proc.wait(timeout=5)
    print("Done.")


if __name__ == "__main__":
    main()
