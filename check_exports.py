from auroraview.core import _core
print("Exported names in _core:")
for name in sorted(dir(_core)):
    if not name.startswith('_'):
        print(f"  {name}")
print()
print("SidecarBridge" in dir(_core))
