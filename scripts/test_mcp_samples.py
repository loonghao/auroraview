#!/usr/bin/env python
"""Test MCP samples discovery in Gallery.

This script checks if MCP-related samples are correctly discovered
and categorized by the Gallery backend.
"""

import sys
sys.path.insert(0, str(__file__).rsplit("scripts", 1)[0])

from gallery.backend.samples import get_samples_list
from gallery.backend.config import CATEGORIES


def main():
    """Check MCP samples."""
    print("=" * 60)
    print("MCP Samples Discovery Test")
    print("=" * 60)
    
    # Get all samples
    samples = get_samples_list()
    print(f"\nTotal samples found: {len(samples)}")
    
    # Check categories
    print(f"\nCategories defined: {list(CATEGORIES.keys())}")
    
    # Find MCP-related samples
    mcp_samples = [s for s in samples if "mcp" in s["id"].lower()]
    print(f"\nMCP samples found: {len(mcp_samples)}")
    
    for s in mcp_samples:
        print(f"  - {s['id']}")
        print(f"    Title: {s['title']}")
        print(f"    Category: {s['category']}")
        print(f"    Tags: {s.get('tags', [])}")
        print()
    
    # Check if mcp_integration category has samples
    mcp_category_samples = [s for s in samples if s["category"] == "mcp_integration"]
    print(f"Samples in 'mcp_integration' category: {len(mcp_category_samples)}")
    for s in mcp_category_samples:
        print(f"  - {s['id']}: {s['title']}")
    
    print("\n" + "=" * 60)
    print("Test completed!")
    print("=" * 60)


if __name__ == "__main__":
    main()

