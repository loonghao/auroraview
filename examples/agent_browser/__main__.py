# -*- coding: utf-8 -*-
"""Entry point for running agent_browser as a module.

Usage:
    python -m examples.agent_browser
"""

import sys
from pathlib import Path

# Add examples directory to path
examples_dir = Path(__file__).parent.parent
if str(examples_dir) not in sys.path:
    sys.path.insert(0, str(examples_dir))

# Add python directory to path
project_root = examples_dir.parent
python_dir = project_root / "python"
if str(python_dir) not in sys.path:
    sys.path.insert(0, str(python_dir))

from agent_browser import main

if __name__ == "__main__":
    main()
