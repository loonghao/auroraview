#!/usr/bin/env python
"""
Test show() method directly
"""

import sys
import logging
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from dcc_webview._core import WebView

# Configure logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def main():
    """Main function."""
    logger.info("Creating WebView...")
    w = WebView('test', 400, 300)
    logger.info(f"WebView created: {w}")
    logger.info(f"show method: {w.show}")
    logger.info(f"show method type: {type(w.show)}")
    
    logger.info("Loading HTML...")
    w.load_html("<h1>Test</h1>")
    
    logger.info("Calling show()...")
    try:
        w.show()
        logger.info("show() returned")
    except Exception as e:
        logger.error(f"Error: {e}", exc_info=True)


if __name__ == "__main__":
    main()

