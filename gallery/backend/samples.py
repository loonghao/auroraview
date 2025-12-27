"""Sample scanning and management for AuroraView Gallery.

This module provides functionality for:
- Scanning example files from the examples directory
- Extracting and parsing docstrings
- Inferring categories, icons, and tags
- Managing sample metadata
"""

from __future__ import annotations

import ast
from pathlib import Path
from typing import Optional

from .config import CATEGORY_MAPPING, EXAMPLES_DIR, ICON_MAPPING, TAG_MAPPING


def extract_docstring(file_path: Path) -> Optional[str]:
    """Extract module docstring from a Python file."""
    try:
        source = file_path.read_text(encoding="utf-8")
        tree = ast.parse(source)
        return ast.get_docstring(tree)
    except (SyntaxError, UnicodeDecodeError):
        return None


def parse_docstring(docstring: str) -> dict:
    """Parse docstring to extract title, description, and features."""
    lines = docstring.strip().split("\n")
    result = {
        "title": "",
        "description": "",
        "features": [],
        "use_cases": [],
    }

    if not lines:
        return result

    # First line is usually the title
    first_line = lines[0].strip()
    # Remove trailing " - " suffix if present
    if " - " in first_line:
        result["title"] = first_line.split(" - ")[0].strip()
        # Rest of first line can be part of description
        rest = first_line.split(" - ", 1)[1].strip()
        if rest:
            result["description"] = rest
    else:
        result["title"] = first_line.rstrip(".")

    # Parse remaining content
    current_section = "description"
    description_lines = []

    for line in lines[1:]:
        stripped = line.strip()
        lower = stripped.lower()

        # Detect section headers
        if lower.startswith("features") or lower.startswith("key features"):
            current_section = "features"
            continue
        elif lower.startswith("use cases") or lower.startswith("use case"):
            current_section = "use_cases"
            continue
        elif lower.startswith("usage:") or lower.startswith("note:"):
            current_section = "skip"
            continue
        elif lower.startswith("recommended") or lower.startswith("supported"):
            current_section = "skip"
            continue

        if current_section == "skip":
            continue

        # Handle list items
        if stripped.startswith("- "):
            item = stripped[2:].strip()
            if current_section == "features":
                result["features"].append(item)
            elif current_section == "use_cases":
                result["use_cases"].append(item)
        elif stripped and current_section == "description":
            description_lines.append(stripped)

    # Build description if not already set
    if description_lines and not result["description"]:
        # Take first meaningful paragraph
        result["description"] = " ".join(description_lines[:2])

    return result


def infer_category(filename: str, docstring: str) -> str:
    """Infer category based on filename and docstring content."""
    text = (filename + " " + docstring).lower()

    for keyword, category in CATEGORY_MAPPING.items():
        if keyword in text:
            return category

    return "getting_started"  # Default


def infer_icon(filename: str, docstring: str) -> str:
    """Infer icon based on filename and docstring content."""
    text = (filename + " " + docstring).lower()

    for keyword, icon in ICON_MAPPING.items():
        if keyword in text:
            return icon

    return "code"  # Default


def infer_tags(filename: str, docstring: str) -> list:
    """Infer tags based on filename and docstring content."""
    text = (filename + " " + docstring).lower()
    tags = set()

    for tag, keywords in TAG_MAPPING.items():
        for keyword in keywords:
            if keyword in text:
                tags.add(tag)
                break

    return sorted(tags)


def filename_to_title(filename: str) -> str:
    """Convert filename to a readable title."""
    # Remove extension and common suffixes
    name = filename.replace(".py", "")
    for suffix in ["_demo", "_example", "_test"]:
        name = name.replace(suffix, "")

    # Convert to title case
    words = name.replace("_", " ").split()
    return " ".join(word.capitalize() for word in words)


def filename_to_id(filename: str) -> str:
    """Convert filename to a sample ID."""
    name = filename.replace(".py", "")
    for suffix in ["_demo", "_example", "_test"]:
        name = name.replace(suffix, "")
    return name


# Lazy-loaded samples cache
_SAMPLES_CACHE = None


def get_samples_list() -> list:
    """Get samples list with lazy loading."""
    global _SAMPLES_CACHE
    if _SAMPLES_CACHE is None:
        _SAMPLES_CACHE = scan_examples_from(EXAMPLES_DIR)
    return _SAMPLES_CACHE


def scan_examples_from(examples_dir: Path) -> list:
    """Scan examples from a specific directory."""
    samples = []

    if not examples_dir.exists():
        return samples

    for py_file in sorted(examples_dir.glob("*.py")):
        # Skip __init__.py and non-demo files
        if py_file.name.startswith("__"):
            continue

        docstring = extract_docstring(py_file)
        if not docstring:
            continue

        # Parse docstring
        parsed = parse_docstring(docstring)

        # Build sample entry
        sample_id = filename_to_id(py_file.name)
        title = parsed["title"] or filename_to_title(py_file.name)
        description = parsed["description"] or f"Demo: {title}"
        category = infer_category(py_file.name, docstring)
        icon = infer_icon(py_file.name, docstring)
        tags = infer_tags(py_file.name, docstring)

        # Truncate description if too long
        if len(description) > 100:
            description = description[:97] + "..."

        samples.append(
            {
                "id": sample_id,
                "title": title,
                "category": category,
                "description": description,
                "icon": icon,
                "source_file": py_file.name,
                "tags": tags,
            }
        )

    return samples


def get_sample_by_id(sample_id: str):
    # type: (str) -> dict | None
    """Get a sample by its ID."""
    for sample in get_samples_list():
        if sample["id"] == sample_id:
            return sample
    return None


def get_source_code(source_file: str) -> str:
    """Read source code from a sample file."""
    file_path = EXAMPLES_DIR / source_file
    if file_path.exists():
        return file_path.read_text(encoding="utf-8")
    return f"# Source file not found: {source_file}"
