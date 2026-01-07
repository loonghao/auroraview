from auroraview.core import SidecarBridge
print("SidecarBridge:", SidecarBridge)
if SidecarBridge is not None:
    b = SidecarBridge()
    print("Channel:", b.channel_name)
    print("Token:", b.auth_token[:10] + "...")
else:
    print("SidecarBridge is None - mcp-sidecar feature not enabled in build")
