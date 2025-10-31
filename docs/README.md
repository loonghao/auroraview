# AuroraView Documentation

## [DOCS] Documentation Index

### Getting Started
- **[Project Status](PROJECT_STATUS.md)** - Current status, recent fixes, and roadmap
- **[DCC Integration Guide](DCC_INTEGRATION_GUIDE.md)** - How to integrate with Maya, 3ds Max, etc.
- **[Roadmap](ROADMAP.md)** - Future plans and milestones

### Technical Documentation
- **[Technical Design](TECHNICAL_DESIGN.md)** - Overall architecture and design decisions
- **[IPC Architecture](IPC_ARCHITECTURE.md)** - Inter-process communication system
- **[Testing Frameworks Guide](TESTING_FRAMEWORKS_GUIDE.md)** - Testing strategies

### Maya-Specific
- **[Maya Examples](../examples/maya/README.md)** - Maya integration examples and best practices
- **[Maya Fixes Summary](MAYA_FIXES_SUMMARY.md)** - Recent Maya-specific fixes
- **[Maya Outliner Implementation](MAYA_OUTLINER_IMPLEMENTATION.md)** - Outliner example details

### Comparisons
- **[Comparison with PyWebView](COMPARISON_WITH_PYWEBVIEW.md)** - Why AuroraView vs PyWebView
- **[Project Advantages](PROJECT_ADVANTAGES.md)** - Key advantages and features

## [GOAL] Quick Navigation

### I want to...

#### ...integrate AuroraView with Maya
1. Read [DCC Integration Guide](DCC_INTEGRATION_GUIDE.md)
2. Check [Maya Examples](../examples/maya/README.md)
3. Run `examples/maya/outliner_view.py`

#### ...understand the architecture
1. Read [Technical Design](TECHNICAL_DESIGN.md)
2. Read [IPC Architecture](IPC_ARCHITECTURE.md)
3. Check [Project Status](PROJECT_STATUS.md)

#### ...contribute to the project
1. Read [Roadmap](ROADMAP.md)
2. Check [Project Status](PROJECT_STATUS.md) for known issues
3. Read [Testing Frameworks Guide](TESTING_FRAMEWORKS_GUIDE.md)

#### ...compare with other solutions
1. Read [Comparison with PyWebView](COMPARISON_WITH_PYWEBVIEW.md)
2. Read [Project Advantages](PROJECT_ADVANTAGES.md)

##  Documentation Structure

```
docs/
├── README.md (this file)           # Documentation index
├── PROJECT_STATUS.md               # Current status and recent fixes
├── ROADMAP.md                      # Future plans
├── DCC_INTEGRATION_GUIDE.md        # Integration guide
├── TECHNICAL_DESIGN.md             # Architecture overview
├── IPC_ARCHITECTURE.md             # IPC system details
├── TESTING_FRAMEWORKS_GUIDE.md     # Testing strategies
├── MAYA_FIXES_SUMMARY.md           # Maya-specific fixes
├── MAYA_OUTLINER_IMPLEMENTATION.md # Outliner example
├── COMPARISON_WITH_PYWEBVIEW.md    # Comparison
└── PROJECT_ADVANTAGES.md           # Key features
```

## [SEARCH] Document Summaries

### PROJECT_STATUS.md
**What**: Current project status, completed features, known issues, and recent fixes

**When to read**: 
- Starting work on the project
- Checking what's been done
- Looking for known issues

**Key sections**:
- Completed features
- Recent fixes (DevTools, process leaks)
- Known issues (window close button)
- Architecture overview
- Code cleanup summary

---

### DCC_INTEGRATION_GUIDE.md
**What**: Complete guide for integrating AuroraView with DCC applications

**When to read**:
- Integrating with Maya, 3ds Max, Houdini, etc.
- Understanding thread-safe patterns
- Learning event processing loops

**Key sections**:
- Thread safety patterns
- Event processing loops
- Maya integration example
- Best practices
- Troubleshooting

---

### IPC_ARCHITECTURE.md
**What**: Detailed documentation of the IPC (Inter-Process Communication) system

**When to read**:
- Understanding JavaScript ↔ Python communication
- Debugging event flow issues
- Optimizing performance

**Key sections**:
- Architecture diagram
- Message flow (JS → Python, Python → JS)
- Performance optimizations
- Thread safety
- Error handling

---

### TECHNICAL_DESIGN.md
**What**: Overall technical architecture and design decisions

**When to read**:
- Understanding the big picture
- Making architectural decisions
- Contributing to core features

**Key sections**:
- System architecture
- Technology stack
- Design patterns
- Performance considerations

---

### ROADMAP.md
**What**: Future plans, milestones, and feature priorities

**When to read**:
- Planning contributions
- Understanding project direction
- Checking feature status

**Key sections**:
- Short-term goals (2 weeks)
- Medium-term goals (1 month)
- Long-term goals (3 months)
- Feature priorities

---

### MAYA_FIXES_SUMMARY.md
**What**: Summary of Maya-specific fixes and improvements

**When to read**:
- Debugging Maya integration issues
- Understanding recent changes
- Learning from past fixes

**Key sections**:
- DevTools support fix
- Process leak fix
- Window close button investigation

---

### MAYA_OUTLINER_IMPLEMENTATION.md
**What**: Detailed implementation of the Maya Outliner example

**When to read**:
- Building similar tools
- Understanding event-driven UI
- Learning Maya integration patterns

**Key sections**:
- Feature overview
- Implementation details
- Event flow
- Best practices

---

### COMPARISON_WITH_PYWEBVIEW.md
**What**: Comparison between AuroraView and PyWebView

**When to read**:
- Choosing between solutions
- Understanding advantages
- Justifying technology choice

**Key sections**:
- Feature comparison
- Performance comparison
- Use case recommendations

---

### PROJECT_ADVANTAGES.md
**What**: Key advantages and unique features of AuroraView

**When to read**:
- Presenting the project
- Understanding value proposition
- Marketing/documentation

**Key sections**:
- Performance advantages
- DCC integration features
- Developer experience
- Production readiness

---

### TESTING_FRAMEWORKS_GUIDE.md
**What**: Testing strategies and frameworks

**When to read**:
- Setting up tests
- Contributing test coverage
- Debugging test failures

**Key sections**:
- Unit testing
- Integration testing
- Manual testing procedures

## 🆕 Recent Changes

### 2025-01-29: Documentation Cleanup
**Removed**:
- 7 test scripts from `examples/maya/`
- 8 debug/temporary docs from `docs/`
- 8 IPC-related docs (consolidated into `IPC_ARCHITECTURE.md`)

**Added**:
- `PROJECT_STATUS.md` - Comprehensive status document
- `IPC_ARCHITECTURE.md` - Consolidated IPC documentation
- `docs/README.md` - This documentation index

**Result**:
- [OK] Cleaner repository structure
- [OK] Easier to find information
- [OK] No duplicate content
- [OK] Clear documentation hierarchy

## [NOTE] Documentation Guidelines

### When adding new documentation:

1. **Check existing docs first** - Avoid duplication
2. **Use clear titles** - Make it easy to find
3. **Add to this index** - Keep the index up-to-date
4. **Follow structure** - Use consistent formatting
5. **Include examples** - Show, don't just tell

### Documentation format:

```markdown
# Title

## Overview
Brief description of what this document covers

## Key Concepts
Main concepts explained

## Examples
Practical examples

## Related Documentation
Links to related docs
```

## [LINK] External Resources

- **GitHub Repository**: https://github.com/loonghao/auroraview
- **Wry Documentation**: https://docs.rs/wry/
- **PyO3 Guide**: https://pyo3.rs/
- **Maya Python API**: https://help.autodesk.com/view/MAYAUL/2024/ENU/?guid=Maya_SDK_py_ref_index_html

##  Getting Help

- **Issues**: Report bugs or request features on [GitHub Issues](https://github.com/loonghao/auroraview/issues)
- **Discussions**: Ask questions on [GitHub Discussions](https://github.com/loonghao/auroraview/discussions)
- **Email**: Contact maintainer at hal.long@outlook.com

