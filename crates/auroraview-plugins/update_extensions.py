#!/usr/bin/env python3
"""Update extensions.rs to use types module."""
import os

# Read extensions.rs
with open('src/extensions.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Find where "impl ExtensionsPlugin {" appears
impl_start = None
for i, line in enumerate(lines):
    if 'impl ExtensionsPlugin {' in line:
        impl_start = i
        break

if impl_start is None:
    print("Error: Could not find 'impl ExtensionsPlugin {'")
    exit(1)

print("Found 'impl ExtensionsPlugin {' at line {}".format(impl_start + 1))

# Build new content:
# 1. Keep lines 0-48 (module doc + imports, before "Callback for navigating...")
# 2. Add "mod types;"
# 3. Keep from impl_start to end
# 4. Skip lines 49 to impl_start-1 (type definitions)

new_lines = []
# Keep lines 0-48 (before "Callback for navigating...")
new_lines.extend(lines[0:49])
# Add mod types;
new_lines.append('\nmod types;\n')
# Skip lines 49 to impl_start-1 (type definitions)
# Keep from impl_start to end
new_lines.extend(lines[impl_start:])

# Write updated extensions.rs
with open('src/extensions.rs', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)

print("Updated src/extensions.rs:")
print("  - Kept lines 1-49 (module doc + imports)")
print("  - Added 'mod types;'")
print("  - Removed lines 50-{} (type definitions)".format(impl_start))
print("  - Kept lines {}+ (impl blocks)".format(impl_start + 1))
print("\nDone!")
