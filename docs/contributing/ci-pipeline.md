# CI Pipeline Architecture

This document describes the CI/CD pipeline architecture for AuroraView, optimized for package isolation and efficient builds.

## Overview

AuroraView uses a **package-isolated CI strategy** where each package (Rust crates, SDK, MCP, Gallery, Docs) has its own CI workflow. This approach:

- **Reduces CI time**: Only affected packages are built and tested
- **Improves feedback**: Faster feedback for focused changes
- **Respects dependencies**: Dependency chains trigger downstream tests automatically

## Package Structure

```
AuroraView
├── Rust Crates
│   ├── aurora-signals (standalone)
│   ├── aurora-protect (standalone)
│   ├── auroraview-plugin-core (standalone)
│   ├── auroraview-plugin-fs → plugin-core
│   ├── auroraview-extensions (standalone)
│   ├── auroraview-plugins → plugin-core, plugin-fs, extensions
│   ├── auroraview-core → signals, plugins
│   ├── auroraview-pack → protect (optional)
│   ├── auroraview-cli → core, pack
│   └── auroraview (root) → core, signals
├── Frontend Packages
│   ├── @auroraview/sdk (TypeScript)
│   └── auroraview-gallery → SDK
├── Python Packages
│   ├── auroraview (Python bindings)
│   └── auroraview-mcp (MCP server)
└── Documentation
    └── docs (VitePress)
```

## Workflow Files

| Workflow | Purpose | Trigger |
|----------|---------|---------|
| `pr-checks.yml` | PR validation | Pull requests |
| `rust-crates-ci.yml` | Rust crate testing | Crate changes |
| `python-ci.yml` | Python testing | Python changes |
| `sdk-ci.yml` | SDK build & test | SDK changes |
| `mcp-ci.yml` | MCP server CI | MCP changes |
| `docs.yml` | Documentation | Docs changes |
| `build-gallery.yml` | Gallery packaging | Release |

## Dependency Chain Detection

When a file changes, the CI automatically detects which packages need to be tested based on the dependency graph.

### Example: `aurora-signals` Change

```
aurora-signals changed
    └── triggers: auroraview-core (depends on signals)
        └── triggers: auroraview-cli (depends on core)
            └── triggers: auroraview (root, depends on core)
```

### Example: `auroraview-plugin-core` Change

```
auroraview-plugin-core changed
    ├── triggers: auroraview-plugin-fs (depends on plugin-core)
    └── triggers: auroraview-plugins (depends on plugin-core)
        └── triggers: auroraview-core (depends on plugins)
            └── triggers: auroraview-cli, auroraview (root)
```

## Local Development Commands

Use `just` commands for package-level testing:

```bash
# Test individual crates
just test-signals          # aurora-signals
just test-protect          # aurora-protect
just test-plugin-core      # auroraview-plugin-core
just test-plugin-fs        # auroraview-plugin-fs
just test-extensions       # auroraview-extensions
just test-plugins          # auroraview-plugins
just test-core             # auroraview-core
just test-pack             # auroraview-pack
just test-cli              # auroraview-cli

# Test groups
just test-standalone       # All standalone crates
just test-python           # Python tests only
just test-python-unit      # Python unit tests
just test-python-integration  # Python integration tests

# SDK and Gallery
just sdk-test              # SDK unit tests
just sdk-ci                # Full SDK CI
just gallery-test          # Gallery E2E tests

# MCP
just mcp-test              # MCP tests
just mcp-ci                # Full MCP CI
```

## Path Filters

The CI uses path filters to determine which workflows to run:

| Category | Paths | Triggers |
|----------|-------|----------|
| `rust` | `src/**`, `crates/**`, `Cargo.*` | Rust builds, wheel builds |
| `python` | `python/**`, `tests/python/**` | Python tests |
| `sdk` | `packages/auroraview-sdk/**` | SDK build |
| `mcp` | `packages/auroraview-mcp/**` | MCP build |
| `gallery` | `gallery/**` | Gallery E2E |
| `docs` | `docs/**`, `*.md` | Docs build |
| `ci` | `.github/**`, `justfile` | All checks |

## Artifact Reuse

To avoid duplicate builds, artifacts are shared between jobs:

1. **SDK Assets**: Built once, used by wheel builds and Gallery
2. **Wheels**: Built once per platform, used by Python tests and Gallery pack
3. **CLI**: Built once per platform, used by Gallery pack

## Best Practices

### For Contributors

1. **Focus changes**: Keep PRs focused on specific packages
2. **Run local tests**: Use `just test-<package>` before pushing
3. **Check CI summary**: Review the "Detected Changes" summary in PR checks

### For Maintainers

1. **Monitor CI times**: Track build times per package
2. **Update dependencies**: Keep the dependency graph in sync with `Cargo.toml`
3. **Cache optimization**: Ensure cache keys are package-specific

## Troubleshooting

### CI runs all checks unexpectedly

- Check if `.github/**` or `justfile` was modified (triggers all checks)
- Verify path filters are correctly configured

### Dependency not detected

- Ensure the dependency is listed in the workflow's dependency chain computation
- Check `rust-crates-ci.yml` for the dependency graph logic

### Cache misses

- Cache keys are based on `Cargo.lock` hash
- Different packages may have different cache keys
