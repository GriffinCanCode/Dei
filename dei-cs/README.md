# God Class Detector

A C# code analysis tool that identifies classes with excessive responsibilities and suggests targeted refactorings using semantic clustering and machine learning.

## Overview

This tool analyzes C# codebases to detect "god classes"—classes that violate the Single Responsibility Principle by having too many methods, lines of code, or cyclomatic complexity. It uses machine learning (K-means clustering) to group related methods and suggest meaningful class extractions.

## Features

- **AST-Based Architecture**: Builds complete filesystem and code ASTs for comprehensive analysis
- **Parallel Traversal**: Analyzes multiple files concurrently using all available CPU cores
- **God File Detection**: Identifies files with too many classes (violates single file responsibility)
- **God Class Detection**: Identifies classes exceeding configurable thresholds
- **God Method Detection**: Identifies overly complex functions/methods (too long, too complex, too many parameters)
- **Roslyn-Based Parsing**: Accurate C# syntax analysis with method-level granularity
- **Semantic Clustering**: ML-powered grouping of methods by responsibility
- **Actionable Suggestions**: Recommends specific class names and method groups for extraction
- **Rich Console Output**: Tree views and detailed reports with file structure visualization

## Architecture

### AST-Based Analysis Pipeline

The tool uses a two-phase approach:

1. **Filesystem AST Building**: Creates a tree representation of your project structure
2. **Parallel Traversal**: Walks the AST concurrently, building sub-ASTs for classes and methods

```
GodClassDetector/
├── GodClassDetector.Core/          # Domain models and interfaces
│   ├── Models/
│   │   ├── FileSystemNode.cs       # AST node for filesystem structure
│   │   └── ClassMetrics.cs         # Sub-AST for class/method analysis
│   └── Interfaces/
│       ├── IFileSystemASTBuilder.cs
│       └── IParallelASTTraverser.cs
├── GodClassDetector.Analysis/      # Roslyn parser and metrics
│   ├── Services/
│   │   ├── FileSystemASTBuilder.cs    # Builds filesystem AST
│   │   └── ParallelASTTraverser.cs    # Parallel AST traversal
│   └── Reporting/
│       └── ASTReportGenerator.cs      # AST-based report generation
├── GodClassDetector.Clustering/    # K-means semantic analysis
├── GodClassDetector.Console/       # CLI application
└── GodClassDetector.Tests/         # Unit tests
```

### How It Works

1. **Build Filesystem AST**: Scans directory structure, excluding build artifacts
2. **Parallel Traversal**: Processes C# files concurrently
3. **Sub-AST Creation**: For each file, builds AST of classes and their methods
4. **Analysis**: Applies metrics and detects god classes
5. **Report Generation**: Produces tree-view reports with problem highlights

## Quick Start

### Installation

#### Option 1: Homebrew (Recommended)

```bash
# Add the tap
brew tap GriffinCanCode/dei

# Install dei-cs
brew install dei-cs

# Use it anywhere
dei-cs check .
```

#### Option 2: Build from Source

Requires .NET 9.0 SDK or later.

```bash
cd src/GodClassDetector.Console
dotnet build
```

### Usage

#### If installed via Homebrew:

```bash
# Check current directory
dei-cs check .

# Check specific project
dei-cs check /path/to/your/project/src

# Legacy mode (single file)
dei-cs /path/to/YourClass.cs
```

#### If building from source:

**Recommended: Using the AST-based "check" command (parallel traversal):**

Check current directory:
```bash
cd src/GodClassDetector.Console
dotnet run check .
```

Check specific project:
```bash
cd src/GodClassDetector.Console
dotnet run check /path/to/your/project/src
```

**Legacy mode (single file or sequential directory scan):**
```bash
cd src/GodClassDetector.Console
dotnet run /path/to/YourClass.cs
```

## Configuration

Edit `appsettings.json` to customize detection thresholds:

```json
{
  "DetectionThresholds": {
    // Class-level thresholds
    "MaxLines": 300,
    "MaxMethods": 20,
    "MaxComplexity": 50,
    "MinClusterSize": 3,
    "ClusterThreshold": 0.7,
    
    // Method-level thresholds (god functions)
    "MaxMethodLines": 50,
    "MaxMethodComplexity": 10,
    "MaxMethodParameters": 5,
    
    // File-level thresholds (god files)
    "MaxClassesPerFile": 3,
    "MaxFileLinesOfCode": 500
  }
}
```

## Design Principles

### SOLID Compliance
- Single Responsibility Principle
- Open/Closed Principle
- Liskov Substitution Principle
- Interface Segregation Principle
- Dependency Inversion Principle

### Modern C# Patterns
- Records for immutable DTOs
- Result pattern for explicit error handling
- Nullable reference types
- Dependency injection throughout
- Options pattern for configuration
- Async/await for I/O operations

## Example Output

```
╔════════════════════════════════════════════════════════════╗
║           DEI CHECK - PROJECT ANALYSIS REPORT              ║
╚════════════════════════════════════════════════════════════╝

Root Path: /path/to/your/project
Analyzed: 2025-11-19 05:32:21 UTC

╭────────────────────────────┬───────╮
│ Total Files Analyzed       │    45 │
│ Total Classes              │    67 │
│ God Files Detected         │     3 │
│ God Classes Detected       │     2 │
│ God Methods Detected       │     5 │
│ Classes with God Methods   │     3 │
│ Healthy Classes            │    62 │
╰────────────────────────────┴───────╯

⚠️  GOD FILES DETECTED:

  📄 Services.cs
     File: /src/Services/Services.cs
     Classes: 5, Total Lines: 420
     Classes in file: UserService, OrderService, ProductService, ShippingService, PaymentService
     • Classes (5) exceeds threshold (3)

⚠️  GOD CLASSES DETECTED:

  ❌ UserManager
     File: /src/Services/UserManager.cs
     Lines: 450, Methods: 32, Complexity: 78
     Suggested Extractions: 3
     God Methods: 2

  ❌ OrderProcessor
     File: /src/Business/OrderProcessor.cs
     Lines: 389, Methods: 28, Complexity: 65
     Suggested Extractions: 2

⚠️  GOD METHODS DETECTED:

  📝 PaymentProcessor
     File: /src/Business/PaymentProcessor.cs
     God Methods: 3

     ⚠️  ProcessComplexPayment()
        Lines: 78, Complexity: 15, Parameters: 4
        • Lines (78) exceeds threshold (50)
        • Complexity (15) exceeds threshold (10)

     ⚠️  ValidatePaymentDetails()
        Lines: 52, Complexity: 12, Parameters: 7
        • Lines (52) exceeds threshold (50)
        • Complexity (12) exceeds threshold (10)
        • Parameters (7) exceeds threshold (5)


Problem Files in Project Structure:

PROJECT STRUCTURE:

└── src/
    ├── Services/
    │   ├── UserManager.cs ❌ [GOD CLASS]
    │   └── Services.cs 📄 [GOD FILE]
    └── Business/
        ├── OrderProcessor.cs ❌ [GOD CLASS]
        └── PaymentProcessor.cs ⚠️ [3 GOD METHOD(S)]
```

## Testing

Run the test suite:
```bash
dotnet test
```

All tests use xUnit, FluentAssertions, and Moq.

## Extension Points

### Custom Metrics Calculator

```csharp
public class CustomMetricsCalculator : IMetricsCalculator
{
    public int CalculateCyclomaticComplexity(string methodBody)
    {
        // Your implementation
    }
}
```

Register in `Program.cs`:
```csharp
services.AddSingleton<IMetricsCalculator, CustomMetricsCalculator>();
```

## CI/CD Integration

### Exit Codes
- `0`: No god classes detected
- `1`: God classes found or error occurred

### GitHub Actions Example
```yaml
- name: Detect God Classes
  run: dotnet run --project src/GodClassDetector.Console -- ./src
```

## Performance

### AST-Based Architecture Benefits

- **Parallel Processing**: Leverages all CPU cores for file analysis
- **Efficient Memory**: Streams file processing, doesn't load entire codebase at once
- **Smart Filtering**: Excludes build artifacts (bin/, obj/, etc.) at AST-build time
- **Scalable**: Sub-linear performance improvement with core count

### Complexity Analysis

- **Filesystem AST Building**: O(n) where n = number of files
- **Parallel Traversal**: O(n/p) where p = processor count
- **Per-File Parsing**: O(m) where m = file size
- **Clustering**: O(k × m × i) where k=clusters, m=methods, i=iterations

### Typical Performance

- Small projects (<50 files): ~0.5-1 second
- Medium projects (50-200 files): ~2-4 seconds
- Large projects (>200 files): ~5-10 seconds
- 4-8x speedup on multi-core systems vs sequential processing

## License

MIT License

## Contributing

Contributions are welcome. Please ensure:
- SOLID principles are followed
- Unit test coverage for new features
- Modern C# idioms
- XML documentation comments
