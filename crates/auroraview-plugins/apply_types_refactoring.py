#!/usr/bin/env python3
"""Update extensions.rs to use types module."""
# Read extensions.rs
with open('src/extensions.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Find where "impl ExtensionsPlugin {" starts
impl_idx = None
for i, line in enumerate(lines):
    if 'impl ExtensionsPlugin {' in line:
        impl_idx = i
        break

if impl_idx is None:
    print("Error: Could not find 'impl ExtensionsPlugin {'")
    exit(1)

print("Found 'impl ExtensionsPlugin {' at line {}".format(impl_idx + 1))

# New content:
# - Keep lines 0-48 (module doc + imports)
# - Add "mod types;\nuse types::*;\n"
# - Skip lines 49 to impl_idx-1 (type definitions)
# - Keep lines impl_idx to end (impl blocks)
new_lines = lines[0:49]
new_lines.append('\nmod types;\nuse types::*;\n')
new_lines.extend(lines[impl_idx:])

# Write updated extensions.rs
with open('src/extensions.rs', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)

print("Updated src/extensions.rs:")
print("  - Kept lines 1-49 (module doc + imports)")
print("  - Added 'mod types;' and 'use types::*;'")
print("  - Removed lines 50-{} (type definitions)".format(impl_idx))
print("  - Kept lines {}+ (impl blocks)".format(impl_idx + 1))
