#!/usr/bin/env python3
"""
Test Dependency Installation

This script tests the automatic dependency installation feature.

Requirements:
- requests>=2.25.0
- beautifulsoup4>=4.9.0

Usage:
    python test_dependency_install.py
"""

import sys
from pathlib import Path

# Add the project root to the path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root / "python"))

try:
    import requests
    import bs4
    print("✅ All dependencies are installed!")
    print(f"requests version: {requests.__version__}")
    print(f"beautifulsoup4 version: {bs4.__version__}")
    
    # Test a simple HTTP request
    response = requests.get("https://httpbin.org/json", timeout=5)
    print(f"✅ HTTP test successful: {response.status_code}")
    
except ImportError as e:
    print(f"❌ Missing dependency: {e}")
    sys.exit(1)
except Exception as e:
    print(f"❌ Error: {e}")
    sys.exit(1)

print("🎉 Test completed successfully!")