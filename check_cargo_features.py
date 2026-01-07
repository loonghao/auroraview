import subprocess
import sys

# Run maturin develop with verbose to see actual cargo command
result = subprocess.run(
    [sys.executable, "-m", "maturin", "develop", "-v"],
    capture_output=True,
    text=True,
    cwd=r"c:\Users\hallo\Documents\augment-projects\dcc_webview"
)

print("=== STDOUT ===")
for line in result.stdout.split('\n'):
    if 'feature' in line.lower() or 'cargo' in line.lower():
        print(line)

print("\n=== STDERR ===")
for line in result.stderr.split('\n'):
    if 'feature' in line.lower() or 'cargo' in line.lower():
        print(line)
