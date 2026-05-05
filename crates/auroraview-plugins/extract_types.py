#!/usr/bin/env python3
"""Extract types from extensions.rs to types.rs."""
import re

# Read extensions.rs
with open('src/extensions.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Find where types end (just before "impl ExtensionsPlugin {")
# The types are from line 50 to line 349 (before "impl ExtensionsPlugin {")
lines = content.split('\n')

# Find the line number where "impl ExtensionsPlugin {" appears
impl_start = None
for i, line in enumerate(lines):
    if 'impl ExtensionsPlugin {' in line:
        impl_start = i
        break

if impl_start is None:
    print("Error: Could not find 'impl ExtensionsPlugin {'")
    exit(1)

print("'impl ExtensionsPlugin {' starts at line {}".format(impl_start + 1))

# Keep:
# - Lines 0-48 (module doc + imports)
# - From impl_start to end (impl blocks)
# Remove: Lines 49 to impl_start-1 (type definitions)

module_doc = '\n'.join(lines[0:49])
impl_blocks = '\n'.join(lines[impl_start:])

# Add `mod types;` after the imports
# Find the last `use` statement in the imports
updated_content = module_doc + '\nmod types;\nuse types::*;\n\n' + impl_blocks

# Write updated extensions.rs
with open('src/extensions.rs', 'w', encoding='utf-8') as f:
    f.write(updated_content)

print("Updated extensions.rs: extracted types to types.rs")
print(f"  - Kept lines 1-49 (module doc + imports)")
print(f"  - Added 'mod types;' and 'use types::*;'")
print(f"  - Kept lines {impl_start + 1}+ (impl blocks)")
print(f"  - Removed lines 50-{impl_start} (type definitions)")
