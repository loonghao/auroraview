# AuroraView Documentation

## Documentation Index

### Getting Started
- **[DCC Integration Guide](DCC_INTEGRATION_GUIDE.md)** - How to integrate with Maya, 3ds Max, etc.
- **[Roadmap](ROADMAP.md)** - Future plans and milestones

### Technical Documentation
- **[Architecture](ARCHITECTURE.md)** - Modular backend architecture
- **[Technical Design](TECHNICAL_DESIGN.md)** - Overall architecture and design decisions
- **[IPC Architecture](IPC_ARCHITECTURE.md)** - Inter-process communication system
- **[Packing Guide](PACKING.md)** - Bundle applications into standalone executables

### CLI & Tools
- **[CLI Documentation](CLI.md)** - Command-line interface usage

### Maya-Specific
- **[Maya Solution](MAYA_SOLUTION.md)** - Maya integration guide
- **[Maya Integration Issues](MAYA_INTEGRATION_ISSUES.md)** - Technical details and issues

## Quick Navigation

### I want to...

#### ...integrate AuroraView with Maya
1. Read [DCC Integration Guide](DCC_INTEGRATION_GUIDE.md)
2. Check [Maya Solution](MAYA_SOLUTION.md)

#### ...understand the architecture
1. Read [Architecture](ARCHITECTURE.md)
2. Read [Technical Design](TECHNICAL_DESIGN.md)
3. Read [IPC Architecture](IPC_ARCHITECTURE.md)

#### ...pack my application
1. Read [Packing Guide](PACKING.md)
2. Check [CLI Documentation](CLI.md)

#### ...contribute to the project
1. Read [Roadmap](ROADMAP.md)
2. Check [Current Status](CURRENT_STATUS.md) for known issues

## Documentation Structure

```
docs/
├── README.md                       # Documentation index
├── ARCHITECTURE.md                 # Backend architecture
├── PACKING.md                      # Packing and distribution
├── CLI.md                          # CLI usage
├── ROADMAP.md                      # Future plans
├── DCC_INTEGRATION_GUIDE.md        # Integration guide
├── TECHNICAL_DESIGN.md             # Architecture overview
├── IPC_ARCHITECTURE.md             # IPC system details
├── MAYA_SOLUTION.md                # Maya integration
└── CURRENT_STATUS.md               # Current status
```

## Document Summaries

### PACKING.md
**What**: Complete guide for packing AuroraView applications

**When to read**:
- Building standalone executables
- Understanding packed vs development mode
- Configuring Python dependencies and resources
- Troubleshooting packing issues

---

### ARCHITECTURE.md
**What**: Modular backend architecture and design

**When to read**: 
- Understanding the backend system
- Contributing to core features

---

### DCC_INTEGRATION_GUIDE.md
**What**: Complete guide for integrating AuroraView with DCC applications

**When to read**:
- Integrating with Maya, 3ds Max, Houdini, etc.
- Understanding thread-safe patterns
- Learning event processing loops

---

### IPC_ARCHITECTURE.md
**What**: Detailed documentation of the IPC (Inter-Process Communication) system

**When to read**:
- Understanding JavaScript ↔ Python communication
- Debugging event flow issues
- Optimizing performance

---

### TECHNICAL_DESIGN.md
**What**: Overall technical architecture and design decisions

**When to read**:
- Understanding the big picture
- Making architectural decisions
- Contributing to core features

---

### ROADMAP.md
**What**: Future plans, milestones, and feature priorities

**When to read**:
- Planning contributions
- Understanding project direction
- Checking feature status

---

### MAYA_SOLUTION.md
**What**: Maya integration guide and solutions

**When to read**:
- Integrating with Maya
- Debugging Maya issues
- Understanding Maya-specific patterns

## External Resources

- **GitHub Repository**: https://github.com/loonghao/auroraview
- **Wry Documentation**: https://docs.rs/wry/
- **PyO3 Guide**: https://pyo3.rs/
- **Maya Python API**: https://help.autodesk.com/view/MAYAUL/2024/ENU/?guid=Maya_SDK_py_ref_index_html

## Getting Help

- **Issues**: Report bugs or request features on [GitHub Issues](https://github.com/loonghao/auroraview/issues)
- **Discussions**: Ask questions on [GitHub Discussions](https://github.com/loonghao/auroraview/discussions)
- **Email**: Contact maintainer at hal.long@outlook.com

