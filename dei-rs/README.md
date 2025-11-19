# Dei - Rust Edition

> **A blazingly fast, intelligent code analyzer for detecting god classes and architectural issues**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🚀 Major Improvements Over C# Version

### Performance
- **10-50x faster** thanks to Rust's zero-cost abstractions and better memory layout
- **Work-stealing parallelism** via Rayon (superior to .NET's Parallel.ForEach)
- **Arena allocation** for cache-friendly AST traversal
- **Zero-copy string handling** with Arc for minimal allocations

### Capabilities
- **Multi-language support** from day one (Rust, C#, with easy extensibility)
- **Tree-sitter parsing** for accurate, battle-tested AST analysis
- **DBSCAN clustering** (auto-determines cluster count vs K-means)
- **Graph-based coupling analysis** for architectural insights
- **Dependency cycle detection** (new capability)

### Design
- **Strong typing** with newtype patterns (Lines, Complexity, etc.)
- **Exhaustive pattern matching** eliminates null reference bugs
- **Trait-based extensibility** for custom analyzers
- **Immutable data structures** by default

## Installation

### via Homebrew (Recommended)

```bash
brew install griffincancode/dei/dei-rs
```

### From Source

```bash
git clone https://github.com/GriffinCanCode/Dei.git
cd Dei/dei-rs
cargo build --release
# Binary at target/release/dei
```

## Quick Start

```bash
# Analyze current directory
dei check .

# Check with custom thresholds
dei check ./src --max-lines 500 --max-methods 30

# Analyze architecture quality
dei arch ./src

# JSON output
dei check . --format json
```

## Architecture

### Crate Structure

```
dei-rs/
├── dei-core        # Domain models, traits, error types
├── dei-ast         # Arena-based AST with parallel traversal
├── dei-languages   # Tree-sitter parsers (Rust, C#, extensible)
├── dei-metrics     # Graph analysis, coupling detection
├── dei-clustering  # DBSCAN semantic clustering
└── dei-cli         # Beautiful CLI with clap
```

### Key Design Patterns

#### Arena Allocation
```rust
// Cache-friendly, fast traversal
let arena = SharedArena::new();
let node_id = arena.alloc(Node::new_file(path));
```

#### Newtype Pattern
```rust
// Type safety prevents mixing up metrics
pub struct Lines(pub usize);
pub struct Complexity(pub usize);
pub struct MethodCount(pub usize);
```

#### Work-Stealing Parallelism
```rust
// Rayon automatically balances work across cores
node.children
    .par_iter()
    .try_for_each(|&child_id| traverse_node(child_id))?;
```

### AST Traversal Pipeline

```
1. Build Filesystem AST
   ↓ (ignore crate for smart filtering)
2. Parallel Traversal
   ↓ (Rayon work-stealing)
3. Language-Specific Parsing
   ↓ (tree-sitter for accuracy)
4. Metrics Calculation
   ↓ (cyclomatic complexity, LOC)
5. Clustering Analysis
   ↓ (DBSCAN for semantic grouping)
6. Report Generation
   ↓ (colored terminal output)
```

## Algorithms

### Complexity Calculation

Uses tree-sitter AST for precise decision point counting:
- McCabe's cyclomatic complexity
- Handles nested conditions, loops, match arms
- Language-agnostic (works for any tree-sitter grammar)

### DBSCAN Clustering

Superior to K-means for method clustering:
- **Auto-determines cluster count** (no guessing K)
- **Handles outliers** (noise methods)
- **Finds arbitrary shapes** (not just spherical)

Algorithm:
1. Build TF-IDF feature matrix from method tokens
2. Add structural features (LOC, complexity, etc.)
3. Calculate pairwise distances
4. DBSCAN with auto-tuned epsilon
5. Generate semantic cluster names

### Dependency Graph Analysis

New capability using petgraph:
- **Afferent coupling** (incoming dependencies)
- **Efferent coupling** (outgoing dependencies)
- **Instability metric** (Ce / (Ca + Ce))
- **Cycle detection** (Kosaraju's algorithm)

## Configuration

Create `dei.toml` in your project root:

```toml
[thresholds]
max_class_lines = 300
max_methods = 20
max_class_complexity = 50
max_method_lines = 50
max_method_complexity = 10
max_parameters = 5
max_classes_per_file = 3

[clustering]
min_cluster_size = 3
tolerance = 0.5
```

## Example Output

```
╔════════════════════════════════════════════════════════════╗
║           DEI - CODE ANALYSIS (Rust Edition)               ║
╚════════════════════════════════════════════════════════════╝

📂 Analyzing: ./src

✓ AST built
✓ Analysis complete

SUMMARY:

  Total Classes: 45
  God Classes: 2
  Classes with God Methods: 3
  Healthy Classes: 40

⚠️  GOD CLASSES DETECTED:

  ❌ UserManager
     File: src/services/user_manager.rs
     Lines: 450 | Methods: 32 | Complexity: 78
     Suggested Extractions: 3
       → AuthenticationService (cohesion: 0.85)
         Methods: 8
       → ProfileService (cohesion: 0.72)
         Methods: 6

⚠️  GOD METHODS DETECTED:

  📝 PaymentProcessor
     File: src/business/payment.rs
     God Methods: 2
```

## Performance

### Benchmarks (10,000 LOC codebase)

| Tool | Time | Speedup |
|------|------|---------|
| dei-cs (C#) | 4.2s | 1x |
| dei-rs (Rust) | 0.4s | **10.5x** |

### Memory Usage

| Tool | Peak Memory |
|------|------------|
| dei-cs | ~200 MB |
| dei-rs | ~50 MB |

## Extending

### Add Language Support

```rust
// Implement Parser trait
impl Parser for PythonParser {
    fn parse_file(&self, path: &Path) -> Result<FileMetrics> {
        // Use tree-sitter-python
    }
}

// Register in MultiLanguageParser
```

### Custom Clustering

```rust
impl ClusterAnalyzer for MyAnalyzer {
    fn analyze(&self, class: &ClassMetrics, thresholds: &Thresholds) 
        -> Result<Vec<ResponsibilityCluster>> {
        // Custom algorithm
    }
}
```

## Contributing

This project follows Rust best practices:
- `cargo fmt` before committing
- `cargo clippy` must pass
- Add tests for new features
- Update docs

## License

MIT

## Credits

Inspired by:
- Original C# implementation
- Roslyn's API design
- tree-sitter's parser ecosystem
- Rust's ecosystem excellence

