# AuroraView CI/CD Quick Reference

## Workflow Triggers

### CI Workflow (ci.yml)
- **Automatic**: Push to `main` branch
- **Manual**: Add `full-ci` or `test-wheels` label to PR
- **Manual**: Workflow dispatch from GitHub Actions

### PR Checks Workflow (pr-checks.yml)
- **Automatic**: PR opened, synchronized, reopened, or ready for review
- **Manual**: Workflow dispatch from GitHub Actions

### Build Wheels Workflow (build-wheels.yml)
- **Automatic**: Push to `main` branch or tags
- **Automatic**: PR with changes to source files
- **Manual**: Workflow dispatch from GitHub Actions

## Local Development

### Setup Development Environment
```bash
just dev
```

### Run Tests Locally
```bash
# All tests
just test

# Fast tests only
just test-fast

# Specific test file
just test-file tests/test_basic.py

# With coverage
just test-cov
```

### Code Quality
```bash
# Format code
just format

# Lint code
just lint

# Fix issues
just fix

# All checks
just check
```

### Build
```bash
# Development build
just build

# Release build
just build-release

# Build wheels
just release
```

## CI Commands Used in Workflows

These commands are defined in `justfile` and used by CI workflows:

```bash
# Install dependencies
just ci-install

# Build extension
just ci-build

# Run tests
just ci-test-rust      # Rust tests only
just ci-test-python    # Python tests only
just ci-test-basic     # Basic import tests

# Linting
just ci-lint
```

## GitHub Actions Labels

Add labels to PRs to trigger specific workflows:

| Label | Effect |
|-------|--------|
| `full-ci` | Run full test suite on all platforms |
| `test-wheels` | Build and test wheels on all platforms |

## System Dependencies

### Linux (Ubuntu)
The CI automatically installs:
- `libwebkit2gtk-4.1-dev`
- `libgtk-3-dev`
- `libglib2.0-dev`
- `libpango1.0-dev`
- `libcairo2-dev`

### macOS
- Xcode Command Line Tools (usually pre-installed)

### Windows
- Visual Studio Build Tools (usually pre-installed)

## Troubleshooting

### Build Fails with "glib-sys" Error
**Cause**: Missing system dependencies on Linux
**Solution**: The CI automatically installs these. For local development:
```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libglib2.0-dev libpango1.0-dev libcairo2-dev
```

### Cache Issues
If you suspect cache corruption:
1. Go to GitHub Actions
2. Click "Clear all caches" in the workflow settings
3. Re-run the workflow

### Wheel Build Fails
1. Check that all system dependencies are installed
2. Ensure Rust is up to date: `rustup update`
3. Clean build artifacts: `just clean`
4. Try building locally first: `just release`

## Performance Tips

1. **Use `just test-fast`** for quick feedback during development
2. **Run `just lint`** before pushing to catch issues early
3. **Use `just format`** to auto-fix formatting issues
4. **Cache is automatically managed** - no manual intervention needed

## File Structure

```
.github/
├── actions/
│   ├── setup-auroraview/      # Environment setup
│   ├── cache-setup/           # Caching strategy
│   ├── build-and-test/        # Build and test logic
│   └── build-wheel/           # Wheel building
├── workflows/
│   ├── ci.yml                 # Main CI workflow
│   ├── pr-checks.yml          # PR validation
│   └── build-wheels.yml       # Wheel building
├── CI_STRUCTURE.md            # Detailed documentation
└── QUICK_REFERENCE.md         # This file
```

## Key Differences from PyRustor

While following PyRustor's structure, AuroraView has these customizations:

1. **Package Name**: `auroraview` instead of `pyrustor`
2. **System Dependencies**: WebKitGTK instead of other libraries
3. **Test Markers**: Adapted for AuroraView's test structure
4. **Wheel Targets**: Includes universal2 for macOS

## Next Steps

1. Read [CI_STRUCTURE.md](./CI_STRUCTURE.md) for detailed documentation
2. Check [justfile](../justfile) for all available commands
3. Review individual action files in `.github/actions/` for implementation details

## Support

For issues or questions:
1. Check the CI logs in GitHub Actions
2. Review the error messages in the workflow run
3. Consult [CI_STRUCTURE.md](./CI_STRUCTURE.md) for detailed information
4. Check the PyRustor project for reference: https://github.com/loonghao/PyRustor

