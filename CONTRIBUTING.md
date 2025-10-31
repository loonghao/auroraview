# Contributing to DCC WebView

Thank you for your interest in contributing to DCC WebView! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions with the community.

## Development Setup

### Prerequisites

- Rust 1.75 or higher
- Python 3.7 or higher
- Git

### Setting Up Your Development Environment

1. Fork and clone the repository:
```bash
git clone https://github.com/YOUR_USERNAME/dcc_webview.git
cd dcc_webview
```

2. Install Rust dependencies:
```bash
cargo build
```

3. Install Python development dependencies:
```bash
pip install -e ".[dev]"
```

## Development Workflow

### Code Style

#### Rust
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Follow Rust naming conventions

#### Python
- Use `ruff` for formatting and linting
- Follow PEP 8 guidelines
- Type hints are required for all public APIs

### Testing

#### Rust Tests
```bash
cargo test
```

#### Python Tests
```bash
pytest tests/
```

### Commit Messages

Follow the Conventional Commits specification:

- `feat: add new feature`
- `fix: fix bug`
- `docs: update documentation`
- `test: add tests`
- `refactor: refactor code`
- `chore: update dependencies`

All commits must include DCO sign-off:
```
Signed-off-by: Your Name <your.email@example.com>
```

Use `git commit -s` to automatically add the sign-off.

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes following the code style guidelines
3. Add tests for new functionality
4. Update documentation as needed
5. Run all tests and ensure they pass
6. Run `cargo fmt` and `cargo clippy`
7. Commit your changes with descriptive messages
8. Push to your fork and create a pull request

### PR Description

- Use English for PR title and description
- Clearly describe what the PR does
- Reference any related issues
- Include screenshots/videos for UI changes

## Reporting Issues

When reporting issues, please include:

- DCC software name and version
- Python version
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Error messages or logs

## Questions?

Feel free to open an issue for questions or discussions.

Thank you for contributing! [SUCCESS]

