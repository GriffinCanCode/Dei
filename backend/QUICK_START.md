# Quick Start Guide - DEI Check

## Installation

```bash
cd /Users/griffinstrier/projects/Dei/backend
dotnet build
```

## Basic Usage

### Option 1: Run from Console Directory

```bash
cd src/GodClassDetector.Console
dotnet run check .
```

### Option 2: Run from Project Root

```bash
cd /Users/griffinstrier/projects/Dei/backend
dotnet run --project src/GodClassDetector.Console check ./src
```

### Option 3: Check Any C# Project

```bash
cd /Users/griffinstrier/projects/Dei/backend
dotnet run --project src/GodClassDetector.Console check /path/to/any/csharp/project
```

## Command Syntax

```bash
dei check [path]
```

Where `[path]` can be:
- `.` - Current directory (default if omitted)
- `./relative/path` - Relative path from current directory
- `/absolute/path` - Absolute path to any directory

## What It Does

1. **Scans** your project directory
2. **Builds** an AST of the file structure
3. **Analyzes** all C# files in parallel
4. **Detects** god classes (classes with too many responsibilities)
5. **Reports** findings with a tree view

## Understanding the Output

### Summary Section
```
╭────────────────────────────┬───────╮
│ Total Files Analyzed       │    45 │
│ Total Classes              │    67 │
│ God Files Detected         │     3 │
│ God Classes Detected       │     2 │
│ God Methods Detected       │     5 │
│ Classes with God Methods   │     3 │
│ Healthy Classes            │    62 │
╰────────────────────────────┴───────╯
```

### God File Details
```
📄 Services.cs
   File: /src/Services/Services.cs
   Classes: 5, Total Lines: 420
   Classes in file: UserService, OrderService, ProductService, ShippingService, PaymentService
   • Classes (5) exceeds threshold (3)
```

### God Class Details
```
❌ UserManager
   File: /src/Services/UserManager.cs
   Lines: 450, Methods: 32, Complexity: 78
   Suggested Extractions: 3
   God Methods: 2
```

### God Method Details
```
📝 PaymentProcessor
   File: /src/Business/PaymentProcessor.cs
   God Methods: 3

   ⚠️  ProcessComplexPayment()
      Lines: 78, Complexity: 15, Parameters: 4
      • Lines (78) exceeds threshold (50)
      • Complexity (15) exceeds threshold (10)
```

### Tree View
```
└── src/
    ├── Services/
    │   ├── UserManager.cs ❌ [GOD CLASS]
    │   └── Services.cs 📄 [GOD FILE]
    └── Business/
        └── PaymentProcessor.cs ⚠️ [3 GOD METHOD(S)]
```

## Exit Codes

- **0** - No god files, classes, or methods found (success)
- **1** - God files, classes, or methods detected (failure)

Perfect for CI/CD pipelines!

## Configuration

Edit `src/GodClassDetector.Console/appsettings.json`:

```json
{
  "DetectionThresholds": {
    // Class-level thresholds
    "MaxLines": 300,              // Max lines per class
    "MaxMethods": 20,             // Max methods per class
    "MaxComplexity": 50,          // Max cyclomatic complexity
    "MinClusterSize": 3,          // Min methods for extraction suggestion
    "ClusterThreshold": 0.7,      // Cohesion threshold for clustering
    
    // Method-level thresholds (god functions)
    "MaxMethodLines": 50,         // Max lines per method
    "MaxMethodComplexity": 10,    // Max cyclomatic complexity per method
    "MaxMethodParameters": 5,     // Max parameters per method
    
    // File-level thresholds (god files)
    "MaxClassesPerFile": 3,       // Max classes per file
    "MaxFileLinesOfCode": 500     // Max total lines in a file
  }
}
```

## Common Scenarios

### CI/CD Integration

**.github/workflows/code-quality.yml**
```yaml
- name: Check for God Classes
  run: |
    cd backend
    dotnet run --project src/GodClassDetector.Console check ./src
```

### Pre-commit Hook

**.git/hooks/pre-commit**
```bash
#!/bin/bash
cd backend
dotnet run --project src/GodClassDetector.Console check ./src
if [ $? -ne 0 ]; then
  echo "God classes detected! Fix before committing."
  exit 1
fi
```

### Local Development

```bash
# Quick check before committing
alias dei-check='cd /Users/griffinstrier/projects/Dei/backend && dotnet run --project src/GodClassDetector.Console check'

# Then just run:
dei-check ./src
```

## Performance Tips

### For Large Projects

1. **Exclude test files** if not needed
2. **Run on specific directories** rather than entire repo
3. **Use SSD** for faster file I/O
4. **Multi-core CPU** benefits parallel processing

### Expected Performance

- Small (<50 files): ~1 second
- Medium (50-200 files): ~2-4 seconds  
- Large (>200 files): ~5-10 seconds

## Troubleshooting

### "Path not found"
- Verify the path exists
- Use absolute paths if relative paths cause issues
- Check you're in the correct directory

### "No classes found"
- Ensure path contains .cs files
- Check files aren't in excluded directories (bin/, obj/)
- Verify files are valid C# syntax

### Slow Performance
- Check CPU usage - should use all cores
- Verify not running on network drive
- Consider analyzing smaller directory scope

## Examples

### Analyze current project
```bash
cd /Users/griffinstrier/projects/Dei/backend
dotnet run --project src/GodClassDetector.Console check .
```

### Analyze specific module
```bash
dotnet run --project src/GodClassDetector.Console check ./src/GodClassDetector.Analysis
```

### Analyze example files
```bash
dotnet run --project src/GodClassDetector.Console check ./examples
```

### Legacy mode (single file)
```bash
dotnet run --project src/GodClassDetector.Console ./examples/GodClassExample.cs
```

## What Makes a God File, Class, or Method?

### God File

A file is considered a "god file" if it exceeds ANY threshold:

- **Classes**: > 3 classes per file
- **Total Lines**: > 500 lines in the file

### God Class

A class is considered a "god class" if it exceeds ANY threshold:

- **Lines**: > 300 lines
- **Methods**: > 20 methods
- **Complexity**: > 50 cyclomatic complexity

### God Method (God Function)

A method is considered a "god method" if it exceeds ANY threshold:

- **Lines**: > 50 lines
- **Complexity**: > 10 cyclomatic complexity
- **Parameters**: > 5 parameters

### Why These Thresholds?

Based on research and industry best practices:

**File-level:**
- Files with > 3 classes violate single file responsibility
- Files > 500 lines are hard to navigate and maintain
- Each class should typically have its own file

**Class-level:**
- Classes > 300 lines are hard to understand
- Classes > 20 methods likely have multiple responsibilities
- Complexity > 50 indicates high maintenance cost

**Method-level:**
- Methods > 50 lines are hard to understand and test
- Complexity > 10 indicates too many branches/decisions
- Methods > 5 parameters are hard to use and remember

### Customizing Thresholds

Adjust in `appsettings.json` based on your team's standards.

## Next Steps

1. **Run the tool** on your codebase
2. **Review findings** - are they legitimate concerns?
3. **Prioritize refactoring** - start with worst offenders
4. **Integrate into CI** - prevent new god classes
5. **Monitor over time** - track improvements

## Support

For issues or questions:
- Check `ARCHITECTURE.md` for technical details
- Review `README.md` for comprehensive documentation
- Run tests: `dotnet test`

