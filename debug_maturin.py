import subprocess
import sys
import os

os.chdir(r"c:\Users\hallo\Documents\augment-projects\dcc_webview")

# Check pyproject.toml features
import tomllib
with open("pyproject.toml", "rb") as f:
    config = tomllib.load(f)

maturin_config = config.get("tool", {}).get("maturin", {})
print("Maturin config from pyproject.toml:")
print(f"  features: {maturin_config.get('features', 'NOT SET')}")
print()

# Check if maturin sees mcp-sidecar
result = subprocess.run(
    ["uv", "run", "maturin", "develop", "-v", "--dry-run"],
    capture_output=True,
    text=True,
)
print("STDOUT:", result.stdout[:2000] if result.stdout else "None")
print("STDERR:", result.stderr[:2000] if result.stderr else "None")
