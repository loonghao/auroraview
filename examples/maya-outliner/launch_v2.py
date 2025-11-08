"""
Quick launch script for Maya Outliner V2 with lifecycle management.

This script provides a simple way to launch the new version in Maya.

Usage in Maya:
    1. Make sure dev server is running: npm run dev
    2. Open Maya Script Editor
    3. Run: exec(open("launch_v2.py", encoding='utf-8').read())
    4. Try closing the window with X button
    5. Observe improved close detection!

Alternative usage:
    python launch_v2.py  # Standalone mode (for testing without Maya)
"""

import os
import sys

# Add necessary directories to path
project_root = r"C:\Users\hallo\Documents\augment-projects\dcc_webview"
python_dir = os.path.join(project_root, "python")
maya_integration_dir = os.path.join(project_root, "examples", "maya-outliner", "maya_integration")

for path in [python_dir, maya_integration_dir]:
    if path not in sys.path:
        sys.path.insert(0, path)


def main():
    """Launch Maya Outliner V2"""
    print("\n" + "=" * 80)
    print("[LAUNCH] Maya Outliner V2 with Lifecycle Management")
    print("=" * 80 + "\n")

    # Check if running in Maya
    try:
        import maya.cmds as cmds

        in_maya = True
        print("[OK] Running in Maya")
    except ImportError:
        in_maya = False
        print("[WARNING] Running in standalone mode (Maya not detected)")

    print()

    # Import Maya Outliner V2
    try:
        from maya_integration import maya_outliner_v2

        print("[OK] Maya Outliner V2 imported")
    except ImportError as e:
        print("[ERROR] Failed to import Maya Outliner V2: %s" % e)
        print("\nMake sure:")
        print("1. AuroraView is built: cargo build --release")
        print("2. PYTHONPATH includes the python directory")
        return None

    print()

    # Launch outliner
    try:
        print("[INFO] Creating Maya Outliner V2...")
        print("       URL: http://localhost:5173 (dev server)")
        print("       Mode: Singleton (only one instance allowed)")
        print()

        outliner = maya_outliner_v2.main(url="http://localhost:5173", singleton=True)

        print()
        print("=" * 80)
        print("[SUCCESS] Maya Outliner V2 launched successfully!")
        print("=" * 80)
        print()

        # Show lifecycle status
        if outliner._lifecycle_supported:
            print("[INFO] NEW FEATURES ACTIVE:")
            print("       [+] Event-driven close detection")
            print("       [+] Guaranteed resource cleanup")
            print("       [+] Cross-platform support")
            print("       [+] Lifecycle state tracking")
        else:
            print("[WARNING] Lifecycle not available, using fallback mode")

        print()
        print("[INFO] TESTING INSTRUCTIONS:")
        print()
        print("1. Click the X button to close the window")
        print("   -> Should close immediately")
        print("   -> Check console for lifecycle messages")
        print()
        print("2. Run: outliner.close()")
        print("   -> Programmatic close")
        print()
        print("3. Run: maya_outliner_v2.main()")
        print("   -> Should return existing instance (singleton)")
        print()
        print("4. Check lifecycle state:")
        print("   -> Run: outliner._webview._core.get_lifecycle_state()")
        print()

        # Store in global namespace for easy access
        if in_maya:
            globals()["outliner"] = outliner
            print("[TIP] Access outliner with: outliner.close()")

        print()
        print("=" * 80)

        return outliner

    except Exception as e:
        print("\n[ERROR] Failed to launch Maya Outliner V2: %s" % e)
        import traceback

        traceback.print_exc()
        return None


if __name__ == "__main__":
    # Check if dev server is running
    print("\n[INFO] Checking dev server...")
    try:
        import urllib.request

        try:
            urllib.request.urlopen("http://localhost:5173", timeout=1)
            print("[OK] Dev server is running at http://localhost:5173")
        except Exception:
            print("[WARNING] Dev server not detected at http://localhost:5173")
            print("\nPlease start the dev server:")
            print("   cd examples/maya-outliner")
            print("   npm run dev")
            print()
            response = input("Continue anyway? (y/n): ")
            if response.lower() != "y":
                print("Aborted.")
                sys.exit(0)
    except ImportError:
        print("[WARNING] urllib not available, skipping server check")

    print()

    # Launch
    outliner = main()

    # Keep reference
    if outliner:
        print("\n[OK] Outliner reference stored in 'outliner' variable")
        print("     Use: outliner.close() to close programmatically")

        # In standalone mode, wait for user input
        if "maya" not in sys.modules:
            print("\n[INFO] Press Enter to close and exit...")
            input()
            outliner.close()
            print("[OK] Closed successfully")
