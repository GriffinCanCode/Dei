# God Class Detector

A sophisticated C# code analysis tool that identifies "god classes" (classes with too many responsibilities) and suggests refactorings using semantic clustering powered by machine learning.

## Features

✨ **Intelligent Detection**
- Identifies classes exceeding configurable thresholds (lines, methods, complexity)
- Uses Roslyn for accurate C# parsing and analysis
- Calculates cyclomatic complexity and other code metrics

🤖 **AI-Powered Clustering**
- Semantic analysis using machine learning (K-means clustering)
- Groups methods by responsibility based on:
  - Method naming patterns
  - Shared dependencies
  - Structural similarities
  - Token frequency analysis (TF-IDF inspired)

🎯 **Actionable Suggestions**
- Recommends specific class extractions
- Suggests meaningful class names based on method groups
- Provides cohesion scores and justifications
- Lists methods to extract for each suggestion

📊 **Multiple Output Formats**
- Beautiful console output with Spectre.Console
- JSON format for CI/CD integration
- Markdown reports for documentation

## Architecture

The solution follows clean architecture principles with clear separation of concerns:

```
GodClassDetector/
├── GodClassDetector.Core/          # Domain models and interfaces
│   ├── Models/                     # Domain entities (ClassMetrics, AnalysisResult, etc.)
│   └── Interfaces/                 # Abstractions (IClassParser, ISemanticAnalyzer, etc.)
│
├── GodClassDetector.Analysis/      # Analysis implementation
│   ├── Parsers/                    # Roslyn-based C# parser
│   ├── Metrics/                    # Complexity calculators
│   ├── Services/                   # Core detection logic
│   └── Reporting/                  # Report generators
│
├── GodClassDetector.Clustering/    # ML-based semantic analysis
│   └── Analyzers/                  # K-means clustering implementation
│
├── GodClassDetector.Console/       # CLI application
│   ├── Configuration/              # Options pattern configuration
│   └── Services/                   # Application orchestration
│
└── GodClassDetector.Tests/         # Unit tests
    ├── Metrics/
    └── Models/
```

## Design Patterns & Best Practices

### SOLID Principles

- **Single Responsibility**: Each class has one clear purpose
- **Open/Closed**: Extensible through interfaces without modification
- **Liskov Substitution**: Interfaces properly abstracted
- **Interface Segregation**: Small, focused interfaces
- **Dependency Inversion**: Depends on abstractions, not concretions

### Modern C# Features

- **Records**: Immutable DTOs with value semantics
- **Result Pattern**: Explicit error handling without exceptions
- **Nullable Reference Types**: Compile-time null safety
- **Pattern Matching**: Expressive control flow
- **Async/Await**: Non-blocking I/O operations
- **Required Properties**: Enforced initialization

### Architectural Patterns

- **Dependency Injection**: Constructor injection throughout
- **Options Pattern**: Type-safe configuration
- **Strategy Pattern**: Pluggable analyzers and parsers
- **Repository Pattern**: Abstracted data access (via interfaces)

## Configuration

Edit `appsettings.json` to customize detection thresholds:

```json
{
  "DetectionThresholds": {
    "MaxLines": 300,           // Maximum lines in a class
    "MaxMethods": 20,          // Maximum method count
    "MaxComplexity": 50,       // Maximum cyclomatic complexity
    "MinClusterSize": 3,       // Minimum methods per cluster
    "ClusterThreshold": 0.7    // Similarity threshold for clustering
  }
}
```

## Usage

### Analyze a Single File

```bash
cd src/GodClassDetector.Console
dotnet run /path/to/YourClass.cs
```

### Analyze an Entire Project

```bash
dotnet run /path/to/your/project/src
```

### Example Output

```
╔════════════════════════════════════════════════════════════════╗
║            GOD CLASS DETECTION REPORT                          ║
╚════════════════════════════════════════════════════════════════╝

┌─────────────────────────┬───────┐
│ Metric                  │ Value │
├─────────────────────────┼───────┤
│ Total Classes Analyzed  │ 15    │
│ God Classes Detected    │ 2     │
│ Healthy Classes         │ 13    │
└─────────────────────────┴───────┘

⚠️  God Classes Detected:

╔══════════════════════════════════════════════════════════════════╗
║ UserManager                                                      ║
╚══════════════════════════════════════════════════════════════════╝
  File: /src/Services/UserManager.cs

  Metrics:
    • Lines:      450
    • Methods:    32
    • Complexity: 78

  💡 Suggested Refactorings (3):

    → AuthenticationService
      Cohesion Score: 0.85
      Methods (8):
        • Login
        • Logout
        • ValidateCredentials
        • GenerateToken
        • RefreshToken
        • RevokeToken
        • VerifyTwoFactor
        • SendPasswordReset
      Reason: Cohesive group of 8 method(s) sharing dependencies on _authProvider, _tokenService

    → UserValidationService
      Cohesion Score: 0.72
      Methods (6):
        • ValidateEmail
        • ValidatePassword
        • CheckPasswordStrength
        • ValidatePhoneNumber
        • CheckEmailUnique
        • CheckUsernameUnique
      Reason: Cohesive group of 6 method(s) sharing dependencies on _validator

    → NotificationService
      Cohesion Score: 0.68
      Methods (5):
        • SendWelcomeEmail
        • SendVerificationEmail
        • SendPasswordResetEmail
        • NotifyAccountLocked
        • SendSecurityAlert
      Reason: Cohesive group of 5 method(s) sharing dependencies on _emailService
```

## Building

### Prerequisites

- .NET 8.0 SDK or later
- C# 12 or later

### Build the Solution

```bash
dotnet restore
dotnet build
```

### Run Tests

```bash
dotnet test
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
- name: Analyze for God Classes
  run: |
    dotnet run --project src/GodClassDetector.Console -- ./src > analysis.txt
    cat analysis.txt
```

### Exit Codes

- `0`: No god classes detected
- `1`: God classes found or error occurred

## Extension Points

### Custom Metrics Calculator

```csharp
public class CustomMetricsCalculator : IMetricsCalculator
{
    public int CalculateCyclomaticComplexity(string methodBody)
    {
        // Your custom logic
    }
}
```

### Custom Semantic Analyzer

```csharp
public class CustomSemanticAnalyzer : ISemanticAnalyzer
{
    public Task<Result<IReadOnlyList<ResponsibilityCluster>>> AnalyzeAsync(
        ClassMetrics classMetrics,
        DetectionThresholds thresholds,
        CancellationToken cancellationToken = default)
    {
        // Your custom clustering logic
    }
}
```

Register in `Program.cs`:

```csharp
services.AddSingleton<IMetricsCalculator, CustomMetricsCalculator>();
services.AddSingleton<ISemanticAnalyzer, CustomSemanticAnalyzer>();
```

## Performance

- **Parser**: O(n) where n is file size
- **Clustering**: O(k * m * i) where k=clusters, m=methods, i=iterations
- **Memory**: Efficient streaming for large codebases
- **Typical Analysis Time**: ~100ms per file

## License

MIT License - Feel free to use, modify, and distribute.

## Contributing

Contributions welcome! Please follow:
1. SOLID principles
2. Unit test coverage
3. Modern C# idioms
4. XML documentation

## Roadmap

- [ ] Support for VB.NET
- [ ] Integration with Visual Studio extension
- [ ] ML model training on real codebases
- [ ] Automated refactoring suggestions
- [ ] Code fix providers
- [ ] Real-time analysis in IDE

