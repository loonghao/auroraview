#!/usr/bin/env python3
"""Fail when a pytest suite is collected by pytest but never executed by CI.

Why this exists
---------------
A test suite can silently drop out of the CI run set. Nothing fails when that
happens: the suite simply stops running, and everybody keeps believing it is
covered. ``testpaths`` in ``pyproject.toml`` cannot catch this -- it is only
read by a *bare* ``pytest``, and every CI entry point passes explicit paths.

What this checks
----------------
1. Collected set -- every test file ``pytest --collect-only`` reports under the
   scope roots (default ``tests``).
2. Executed set -- every test file any CI workflow hands to a ``pytest``
   command, derived by parsing ``run:`` blocks of workflow files and composite
   actions and honouring ``--ignore`` / ``--ignore-glob`` / ``--deselect``.
3. Drift = collected - executed - allowlisted. Any drift fails, printing the
   drifted paths plus remediation advice.

The allowlist
-------------
``scripts/ci/drift_allowlist.txt`` holds suites deliberately not run by CI.
Every entry must carry a justification (inline ``# reason`` or a ``#`` comment
on the line above). Stale entries (CI runs them again) and dangling entries
(path no longer exists) also fail, so the allowlist cannot rot silently.

Usage
-----
    python scripts/ci/check_test_drift.py
    python scripts/ci/check_test_drift.py --verbose
    python scripts/ci/check_test_drift.py --json

Exit codes: 0 = clean, 1 = drift / allowlist problem, 2 = could not run.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Iterable, NamedTuple, Sequence

REPO_ROOT = Path(__file__).resolve().parents[2]

DEFAULT_SCOPE_ROOTS = ("tests",)
DEFAULT_WORKFLOW_GLOBS = (
    ".github/workflows/*.yml",
    ".github/workflows/*.yaml",
    ".github/actions/**/action.yml",
    ".github/actions/**/action.yaml",
)
DEFAULT_ALLOWLIST = "scripts/ci/drift_allowlist.txt"

# Tokens that start a pytest process. `python -m pytest` yields the bare token
# `pytest`; wrappers such as `xvfb-run` / `vx uv run` add tokens in front.
PYTEST_TOKENS = frozenset({"pytest", "pytest.exe", "py.test", "py.test.exe"})

EXCLUDE_OPTIONS = ("--ignore", "--ignore-glob", "--deselect")

# Characters stripped from both ends of a shell token. The Blender job embeds
# its pytest command as a Python list literal, so tokens arrive as `'pytest',`.
_TOKEN_EDGE_CHARS = ",;()[]{}'\""

_TOKEN_RE = re.compile(r'"[^"]*"|\'[^\']*\'|[^"\'\s]+')


def strip_comments(script: str) -> str:
    """Drop ``#``-to-end-of-line comments before tokenising."""
    return "\n".join(line.split("#", 1)[0] for line in script.splitlines())


def join_continuations(script: str) -> str:
    """Fold backslash-newline continuations into one logical line."""
    return re.sub(r"\\[ \t]*\n", " ", script)


def _clean_token(token: str) -> str:
    return token.strip().strip(_TOKEN_EDGE_CHARS).strip()


def tokenize(text: str) -> list[str]:
    """Split shell-ish text into cleaned tokens.

    A quoted span containing a pytest command (the Blender job passes Python
    source via ``--python-expr``) is re-tokenised so the embedded command is
    still understood.
    """
    tokens: list[str] = []
    for raw in _TOKEN_RE.findall(text):
        if len(raw) >= 2 and raw[0] == raw[-1] and raw[0] in "\"'":
            inner = raw[1:-1]
            if "pytest" in inner and re.search(r"\s", inner):
                tokens.extend(tokenize(inner))
                continue
            raw = inner
        tokens.append(_clean_token(raw))
    return tokens


def is_pytest_token(token: str) -> bool:
    return token in PYTEST_TOKENS or token.endswith(("/pytest", "/py.test"))


class Invocation(NamedTuple):
    """One pytest command line found in a workflow."""

    source: str  # "<workflow>::<job>::<step>"
    includes: tuple[str, ...]
    excludes: tuple[str, ...]


def _step_label(step: object, index: int) -> str:
    if isinstance(step, dict) and isinstance(step.get("name"), str):
        return step["name"]
    return f"step[{index}]"


def iter_workflow_scripts(workflow: Path) -> Iterable[tuple[str, str]]:
    """Yield ``(label, script)`` for every shell script in a workflow file."""
    import yaml  # imported lazily so --help works without the dependency

    document = yaml.safe_load(workflow.read_text(encoding="utf-8"))
    if not isinstance(document, dict):
        return

    jobs = document.get("jobs")
    if isinstance(jobs, dict):
        for job_name, job in jobs.items():
            steps = job.get("steps") if isinstance(job, dict) else None
            if isinstance(steps, list):
                for index, step in enumerate(steps):
                    if isinstance(step, dict) and isinstance(step.get("run"), str):
                        label = f"{workflow.name}::{job_name}::{_step_label(step, index)}"
                        yield label, step["run"]

    runs = document.get("runs")
    if isinstance(runs, dict) and isinstance(runs.get("steps"), list):
        for index, step in enumerate(runs["steps"]):
            if isinstance(step, dict) and isinstance(step.get("run"), str):
                label = f"{workflow.name}::runs::{_step_label(step, index)}"
                yield label, step["run"]


def _looks_like_test_path(token: str) -> bool:
    """True when a bare argument plausibly names a test file or directory.

    Requires the token to resolve to something real inside the repo, which
    filters out package names from wrappers such as ``--with playwright``.
    """
    if not token or "=" in token or "*" in token:
        return False
    candidate = (REPO_ROOT / token).resolve()
    try:
        candidate.relative_to(REPO_ROOT.resolve())
    except ValueError:
        return False
    return token.endswith(".py") or candidate.is_dir()


def extract_invocations(label: str, script: str) -> list[Invocation]:
    """Find the pytest commands inside one shell script."""
    tokens = tokenize(join_continuations(strip_comments(script)))
    invocations: list[Invocation] = []

    for index, token in enumerate(tokens):
        if not is_pytest_token(token):
            continue
        includes: list[str] = []
        excludes: list[str] = []
        position = index + 1
        while position < len(tokens) and not is_pytest_token(tokens[position]):
            argument = tokens[position]
            if argument in EXCLUDE_OPTIONS and position + 1 < len(tokens):
                excludes.append(tokens[position + 1])
                position += 2
                continue
            matched = next(
                (o for o in EXCLUDE_OPTIONS if argument.startswith(o + "=")), None
            )
            if matched is not None:
                excludes.append(argument[len(matched) + 1 :])
            elif not argument.startswith("-") and _looks_like_test_path(argument):
                includes.append(argument)
            position += 1
        if includes:
            invocations.append(Invocation(label, tuple(includes), tuple(excludes)))

    return invocations


def collect_test_files(scope_roots: Sequence[str]) -> tuple[set[str], list[str]]:
    """Return the files pytest collects under ``scope_roots`` plus any errors."""
    command = [
        sys.executable,
        "-m",
        "pytest",
        "--collect-only",
        "-q",
        "--continue-on-collection-errors",
        "-p",
        "no:cacheprovider",
        *scope_roots,
    ]
    completed = subprocess.run(
        command, cwd=REPO_ROOT, capture_output=True, text=True,
        encoding="utf-8", errors="replace",
    )

    files: set[str] = set()
    errors: list[str] = []
    prefixes = tuple(root.rstrip("/") + "/" for root in scope_roots)
    for line in completed.stdout.splitlines():
        line = line.strip().replace("\\", "/")
        if line.startswith(prefixes) and "::" in line:
            files.add(line.split("::", 1)[0])
        elif line.startswith("ERROR ") and line[6:].strip().endswith(".py"):
            errors.append(line[6:].strip().replace("\\", "/"))

    # A directory yielding nothing at all is a collection failure, not an empty
    # suite; fall back to a filesystem listing so the check still sees it.
    for root in scope_roots:
        if not any(p.startswith(root.rstrip("/") + "/") for p in files):
            fallback = {
                p.as_posix()
                for p in (REPO_ROOT / root).rglob("test_*.py")
                if ".git" not in p.parts
            }
            if fallback:
                files |= fallback
                errors.append(
                    f"{root}: collection produced no items, used a filesystem listing"
                )

    return files, errors


def _matches(pattern: str, path: str) -> bool:
    pattern = pattern.replace("\\", "/").rstrip("/")
    if any(c in pattern for c in "*?["):
        return fnmatch.fnmatch(path, pattern) or fnmatch.fnmatch(path, pattern + "/*")
    if pattern.endswith(".py"):
        return path == pattern
    return path == pattern or path.startswith(pattern + "/")


def build_coverage(
    invocations: Sequence[Invocation], collected: Iterable[str]
) -> dict[str, list[str]]:
    """Map each executed test file to the invocations that execute it."""
    coverage: dict[str, list[str]] = {}
    for invocation in invocations:
        for path in sorted(collected):
            if not any(_matches(p, path) for p in invocation.includes):
                continue
            if any(_matches(p, path) for p in invocation.excludes):
                continue
            coverage.setdefault(path, []).append(invocation.source)
    return coverage


class AllowlistEntry(NamedTuple):
    path: str
    reason: str


def load_allowlist(path: Path) -> tuple[list[AllowlistEntry], list[str]]:
    """Parse the allowlist, enforcing that every entry carries a reason."""
    if not path.exists():
        return [], []

    entries: list[AllowlistEntry] = []
    problems: list[str] = []
    previous_comment: str | None = None

    for number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        stripped = raw_line.strip()
        if not stripped:
            previous_comment = None
            continue
        if stripped.startswith("#"):
            previous_comment = stripped.lstrip("#").strip()
            continue

        inline_reason = ""
        if "#" in stripped:
            stripped, inline_reason = stripped.split("#", 1)
            stripped = stripped.strip()
            inline_reason = inline_reason.strip()

        reason = inline_reason or previous_comment or ""
        if not reason:
            problems.append(
                f"{path.as_posix()}:{number}: entry '{stripped}' has no justification; "
                "add an inline '# reason' comment"
            )
        entries.append(AllowlistEntry(stripped.replace("\\", "/"), reason))
        previous_comment = None

    return entries, problems


def _resolve_workflows(globs: Sequence[str]) -> list[Path]:
    paths: list[Path] = []
    for pattern in globs:
        paths.extend(sorted(REPO_ROOT.glob(pattern)))
    return [p for p in dict.fromkeys(paths) if p.is_file()]


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--scope", nargs="+", default=list(DEFAULT_SCOPE_ROOTS),
        help="directories whose collected tests must be executed by CI",
    )
    parser.add_argument(
        "--workflow-glob", nargs="+", default=list(DEFAULT_WORKFLOW_GLOBS),
        help="workflow / composite-action files scanned for pytest commands",
    )
    parser.add_argument("--allowlist", default=DEFAULT_ALLOWLIST)
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    workflows = _resolve_workflows(args.workflow_glob)
    if not workflows:
        print(f"error: no workflow files matched {args.workflow_glob}", file=sys.stderr)
        return 2

    invocations: list[Invocation] = []
    for workflow in workflows:
        try:
            for label, script in iter_workflow_scripts(workflow):
                invocations.extend(extract_invocations(label, script))
        except ImportError:
            print("error: PyYAML is required (pip install pyyaml)", file=sys.stderr)
            return 2

    collected, collection_errors = collect_test_files(args.scope)
    coverage = build_coverage(invocations, collected)

    allowlist_path = REPO_ROOT / args.allowlist
    allowlist, allowlist_problems = load_allowlist(allowlist_path)
    allowed = {entry.path for entry in allowlist}

    drift = sorted(collected - set(coverage) - allowed)
    stale = sorted(e.path for e in allowlist if e.path in coverage)
    dangling = sorted(
        e.path for e in allowlist
        if e.path not in collected and not (REPO_ROOT / e.path).exists()
    )

    payload = {
        "collected": len(collected),
        "executed": len(coverage),
        "allowlisted": len(allowed),
        "drift": drift,
        "stale_allowlist": stale,
        "dangling_allowlist": dangling,
        "allowlist_problems": allowlist_problems,
        "collection_errors": collection_errors,
    }

    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0 if not (drift or stale or dangling or allowlist_problems) else 1

    print("pytest suite drift check")
    print(f"  scope              : {' '.join(args.scope)}")
    print(f"  workflows scanned  : {len(workflows)}")
    print(f"  pytest invocations : {len(invocations)}")
    print(f"  tests collected    : {len(collected)}")
    print(f"  executed by CI     : {len(coverage)}")
    print(f"  allowlisted        : {len(allowed)}")
    for message in collection_errors:
        print(f"  ! {message}")
    print()

    if args.verbose:
        print("coverage map")
        for path in sorted(collected):
            sources = coverage.get(path)
            if sources:
                print(f"  {path}  <- {sources[0]}")
            elif path in allowed:
                print(f"  {path}  <- allowlisted")
            else:
                print(f"  {path}  <- NOT EXECUTED")
        print()

    failed = False

    if drift:
        failed = True
        print(f"FAIL: {len(drift)} collected test file(s) are never executed by CI:")
        for path in drift:
            print(f"  - {path}")
        print()
        print("Fix by adding the path to the pytest invocation of an appropriate CI job")
        print("(python-ci.yml / pr-checks.yml / dcc-integration.yml), or -- if the suite")
        print(f"genuinely cannot run in CI -- add it to {args.allowlist} with a reason.")
        print()

    if allowlist_problems:
        failed = True
        print(f"FAIL: {args.allowlist} is not reviewable:")
        for problem in allowlist_problems:
            print(f"  - {problem}")
        print()

    if stale:
        failed = True
        print("FAIL: allowlist entries that CI executes again (delete them):")
        for path in stale:
            print(f"  - {path}")
        print()

    if dangling:
        failed = True
        print("FAIL: allowlist entries that no longer exist (delete them):")
        for path in dangling:
            print(f"  - {path}")
        print()

    if failed:
        return 1

    print("OK: every collected test file is executed by CI or explicitly allowlisted.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
