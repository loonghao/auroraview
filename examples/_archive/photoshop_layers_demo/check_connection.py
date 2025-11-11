"""Quick connection check script"""
import socket
import sys

def check_port(port):
    """Check if a port is available or in use."""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(1)
    
    try:
        result = sock.connect_ex(('localhost', port))
        if result == 0:
            print(f"✅ Port {port} is OPEN (something is listening)")
            return True
        else:
            print(f"❌ Port {port} is CLOSED (nothing is listening)")
            return False
    except Exception as e:
        print(f"❌ Error checking port {port}: {e}")
        return False
    finally:
        sock.close()

def main():
    print("=" * 60)
    print("AuroraView Connection Check")
    print("=" * 60)
    print()
    
    # Check Bridge port
    print("Checking Bridge WebSocket port...")
    bridge_ok = check_port(9001)
    print()
    
    # Check Discovery port
    print("Checking HTTP Discovery port...")
    discovery_ok = check_port(9000)
    print()
    
    print("=" * 60)
    if bridge_ok:
        print("✅ Python backend is running!")
        print("   You can connect from UXP plugin.")
    else:
        print("❌ Python backend is NOT running!")
        print()
        print("📝 To start the backend, run:")
        print("   python examples/photoshop_layers_demo/photoshop_layers_tool.py")
    print("=" * 60)
    
    return 0 if bridge_ok else 1

if __name__ == '__main__':
    sys.exit(main())

