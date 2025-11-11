"""Nox sessions for testing and development."""

import nox

# Default sessions to run
nox.options.sessions = ["pytest", "lint"]

# Python versions to test
PYTHON_VERSIONS = ["3.7", "3.8", "3.9", "3.10", "3.11", "3.12"]


@nox.session(python=PYTHON_VERSIONS)
def pytest(session):
    """Run pytest tests without Qt dependencies.

    This tests the error handling when Qt is not installed.
    """
    session.install(".")
    session.install("pytest", "pytest-cov")
    session.run(
        "pytest",
        "tests/test_qt_import_error.py",
        "tests/test_testing_framework.py",
        "-v",
        "--cov=auroraview",
        "--cov-report=term-missing",
        *session.posargs,
    )


@nox.session(python=PYTHON_VERSIONS, name="pytest-qt")
def pytest_qt(session):
    """Run pytest tests WITH Qt dependencies.

    This tests the actual Qt backend functionality.
    """
    session.install(".[qt]")
    session.install("pytest", "pytest-cov", "pytest-qt")
    session.run(
        "pytest",
        "tests/test_qt_backend.py",
        "-v",
        "--cov=auroraview",
        "--cov-report=term-missing",
        *session.posargs,
    )


@nox.session(python=PYTHON_VERSIONS, name="pytest-all")
def pytest_all(session):
    """Run all pytest tests (both with and without Qt).

    This runs the complete test suite.
    """
    session.install(".[qt]")
    session.install("pytest", "pytest-cov", "pytest-qt")
    session.run(
        "pytest",
        "tests/",
        "-v",
        "--cov=auroraview",
        "--cov-report=term-missing",
        "--cov-report=html",
        *session.posargs,
    )


@nox.session(python="3.11")
def lint(session):
    """Run linting with ruff."""
    session.install("ruff")
    session.run("ruff", "check", "python/", "tests/")
    session.run("ruff", "format", "--check", "python/", "tests/")


@nox.session(python="3.11")
def format(session):
    """Format code with ruff."""
    session.install("ruff")
    session.run("ruff", "format", "python/", "tests/")
    session.run("ruff", "check", "--fix", "python/", "tests/")


@nox.session(python="3.11")
def mypy(session):
    """Run type checking with mypy."""
    session.install(".")
    session.install("mypy")
    session.run("mypy", "python/auroraview")


@nox.session(python="3.11", name="build-test")
def build_test(session):
    """Build the package and test the wheel."""
    session.install("maturin", "build")
    session.run("maturin", "build", "--release")
    session.run("python", "-m", "build", "--sdist")


@nox.session(python="3.11", name="docs-build")
def docs_build(session):
    """Build documentation."""
    session.install(".[qt]")
    session.install("sphinx", "sphinx-rtd-theme", "myst-parser")
    session.run("sphinx-build", "-b", "html", "docs/", "docs/_build/html")


@nox.session(python="3.11", name="docs-server")
def docs_server(session):
    """Build and serve documentation locally."""
    session.install(".[qt]")
    session.install("sphinx", "sphinx-rtd-theme", "myst-parser", "sphinx-autobuild")
    session.run(
        "sphinx-autobuild",
        "docs/",
        "docs/_build/html",
        "--open-browser",
        "--watch",
        "python/",
    )


@nox.session(python="3.11")
def coverage(session):
    """Generate coverage report."""
    session.install(".[qt]")
    session.install("pytest", "pytest-cov", "coverage[toml]")
    session.run(
        "pytest",
        "tests/",
        "-m",
        "not ui and not maya",
        "--cov=auroraview",
        "--cov-report=html",
        "--cov-report=term-missing",
    )
    session.run("coverage", "report")


@nox.session(python="3.11", name="test-maya")
def test_maya(session):
    """Test Maya integration (requires Maya to be installed).

    This is a special session for testing in Maya environment.
    """
    session.install(".")
    session.install("pytest")
    # Note: This would need to be run with mayapy instead of regular python
    session.run(
        "pytest",
        "tests/test_qt_import_error.py",
        "-v",
        "-k",
        "native",
        *session.posargs,
    )

