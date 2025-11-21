# DEI - Detect Everything Immediately

A high-performance static analysis tool for detecting god classes, god methods, and architectural issues in your codebase.

## Features

- 🚀 **Fast parallel analysis** using Rayon's work-stealing scheduler
- 🔍 **Multi-language support** (Rust, C#, with more coming)
- 📊 **Architecture metrics** including coupling analysis and circular dependency detection
- 🎯 **Configurable thresholds** for lines, methods, and complexity
- 💾 **JSON output** for CI/CD integration
- 🎨 **Beautiful terminal output** with progress indicators

## Installation

```bash
cargo install dei
```

## Quick Start

Analyze a file or directory:

```bash
dei check src/
```

Check with custom thresholds:

```bash
dei check src/ --max-lines 200 --max-methods 15 --max-complexity 40
```

Output as JSON for CI integration:

```bash
dei check src/ --format json
```

Analyze architecture quality:

```bash
dei arch src/
```

## What it Detects

### God Classes
Classes that do too much - violating single responsibility principle:
- Too many lines of code
- Too many methods
- High cyclomatic complexity

### God Methods
Methods that are too complex:
- Too many lines
- Too high cyclomatic complexity
- Too many parameters

### Architecture Issues
- High coupling between components
- Circular dependencies
- Poor maintainability metrics

## Supported Languages

- ✅ Rust
- ✅ C#
- 🔜 Python
- 🔜 JavaScript/TypeScript
- 🔜 Java
- 🔜 Go

## Configuration

Default thresholds:
- Max class lines: 300
- Max methods per class: 20
- Max class complexity: 50
- Max method lines: 50
- Max method complexity: 10
- Max parameters: 5

## Exit Codes

- `0` - No issues detected
- `1` - God classes or methods found

## Performance

DEI is built for speed:
- Arena allocation for zero-copy AST traversal
- Lock-free concurrent data structures (DashMap)
- Rayon's work-stealing parallelism
- Efficient tree-sitter parsing

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
